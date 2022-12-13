use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::process::Command;
use std::rc::Rc;
use std::{collections::HashSet, io::BufWriter};

use dot_writer::{Attributes, DotWriter, Scope};
use gml_parser::GMLObject;
use itertools::Itertools;
use itertools::TupleWindows;
use librr_rs::BinaryInterface;
use std::fs::{self, File, OpenOptions};
use std::ops::Range;

use crate::address_recorder::{self, AddrIter};
use crate::erebor::Erebor;
use crate::file_parsing;
use crate::shared_structs::{GraphModule, Settings};
use crate::{
    address_recorder::AddressRecorder,
    query::node::TimeRange,
    shared_structs::{GraphNode, TimeStamp},
};
use librr_rs::*;

// TODO
// Implement a clever prepared scheme
// that saves prior work.
//
// This implementation is comically inefficient
//
pub struct GraphBuilder {
    address_recorder: AddressRecorder,
    //graph_nodes : HashMap<usize,GraphNode>,
    ranges: Vec<TimeRange>,
    breakpoints: HashSet<usize>,
    is_prepared: bool,
    gml_graph: Option<gml_parser::Graph>,
    pub modules: HashMap<String, GraphModule>,
    pub synoptic_nodes: HashMap<usize, GraphNode>,
    pub nodes: HashMap<usize, GraphNode>,
}

// Stablize negative f------ trait impls
// impl !Send for GraphBuilder

impl GraphBuilder {
    pub fn new(max_ft: usize) -> Self {
        Self {
            address_recorder: AddressRecorder::new(max_ft),
            nodes: HashMap::new(),
            synoptic_nodes: HashMap::new(),
            ranges: Vec::new(),
            breakpoints: HashSet::new(),
            is_prepared: false,
            gml_graph: None,
            modules: HashMap::new(),
        }
    }
    // pub fn get_parent_tree_nodes_syn() -> Option<Vec<usize>>{

    // }
    pub fn update_raw_nodes(
        &mut self,
        mut nodes: HashMap<usize, GraphNode>,
        erebor: &Erebor,
    ) -> anyhow::Result<()> {
        self.is_prepared = false;

        for mut node in &mut nodes.values_mut() {
            // Update for empty FQN and
            // also update in case module changed.
            node.FQN = file_parsing::name_to_fqn(
                &format!("{}::{}", &node.module, &node.name),
                &self.modules,
            )?;

            // Should only impose minor perf pentalty on future runs
            // as the naive_line_num will be accurate on future executions
            // (final_offset will be 0)
            let naive_line_num = node.location.line_num;
            let file_info = erebor.files.get(&node.location.file).ok_or_else(|| {
                anyhow::anyhow!(
                    "File name for {} was not inside of the DWARF data for the binary. ",
                    node.FQN
                )
            })?;
            let mut event_addr = None;
            let mut final_offset = 0;
            'addr_search: for offset in 0..1000 {
                let addrs = file_info.lines.get(&((naive_line_num + offset) as u32));
                if let Some(addrs) = addrs {
                    if addrs.len() > 0 {
                        event_addr = Some(addrs.first().unwrap().clone());
                        final_offset = offset;
                        break 'addr_search;
                    }
                }
            }
            let Some(event_addr) = event_addr else {
                return Err(anyhow::anyhow!("Unable to find an address for the {} event annotation", node.FQN));
            };
            node.address = event_addr;
            node.location.line_num = (final_offset + naive_line_num) as u32;
        }
        self.nodes.clear();
        for (_, node) in nodes {
            self.nodes.insert(node.address, node);
        }
        Ok(())
    }
    pub fn update_raw_modules(
        &mut self,
        modules: HashMap<String, GraphModule>,
    ) -> anyhow::Result<()> {
        self.is_prepared = false;
        self.modules = modules;
        Ok(())
    }
    pub fn insert_time_range(&mut self, ts: TimeRange) {
        self.ranges.push(ts);
        self.is_prepared = false;
    }
    pub fn insert_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.address, node);
        self.is_prepared = false;
    }

    pub fn get_graph_as_dot(&mut self, settings: &Settings) -> anyhow::Result<Option<String>> {
        if !self.is_prepared {
            return Ok(None);
        }

        let data = self.gml_to_dot_str(
            self.gml_graph
                .as_ref()
                .expect("gml graph was None during get_graph_as_dot"),
            settings,
        )?;
        Ok(Some(data))
    }
    //TODO: This code is heavily flawed and was written hastily in order to get something
    //written
    // TODO : This code is also very fragile and /requires/ tests
    // run_level:
    // 0 => rerun all
    // 1 => rerun synoptic but not program
    // 2 => Dont rerun
    pub fn prepare(
        &mut self,
        bin_interface: &mut BinaryInterface,
        run_level: u32,
    ) -> anyhow::Result<()> {
        // bin_interface.pin_mut().restart_from_event(0);
        if run_level == 0 {
            let cont = GdbContAction {
                type_: GdbActionType::ACTION_CONTINUE,
                target: bin_interface.get_current_thread(),
                signal_to_deliver: 0,
            };
            let step = GdbContAction {
                type_: GdbActionType::ACTION_STEP,
                target: bin_interface.get_current_thread(),
                signal_to_deliver: 0,
            };

            for node in self.nodes.values() {
                // dbg!(&node);
                if node.address == 0 {
                    anyhow::bail!(
                        "Node address for {} is 0. This should never happen.",
                        &node.FQN
                    );
                }
                bin_interface.pin_mut().set_sw_breakpoint(node.address, 1);
            }
            dbg!(self.nodes.len());

            let mut opened_frame_time: i64 = bin_interface.current_frame_time();
            self.address_recorder.clear();
            self.address_recorder
                .reset_ft_for_writing(opened_frame_time as usize);

            let mut signal = 5;
            while signal == 5 {
                let rip = bin_interface
                    .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
                    .to_usize();
                let current_ft = bin_interface.current_frame_time();
                if current_ft != opened_frame_time {
                    self.address_recorder.finished_writing_ft();
                    opened_frame_time = current_ft;
                    self.address_recorder
                        .reset_ft_for_writing(opened_frame_time as usize);
                }
                // serious problems about efficiently telling if this is an address of a node
                // or of a timestamp
                if self.nodes.contains_key(&rip) {
                    self.address_recorder.insert_address(rip);

                    // meaning there is a breakpoint at rip
                    // so we have to step over it when there is no
                    // breakpoint
                    bin_interface.pin_mut().remove_sw_breakpoint(rip, 1);
                    signal = bin_interface.pin_mut().continue_forward(step).unwrap();
                    if signal != 5 {
                        break;
                    }
                    bin_interface.pin_mut().set_sw_breakpoint(rip, 1);
                }

                signal = bin_interface.pin_mut().continue_forward(cont).unwrap();
            }
            self.address_recorder.finished_writing_ft();

            // Remove all set swbreakpoints to not alter internal state of machine
            for node in self.nodes.values() {
                bin_interface
                    .pin_mut()
                    .remove_sw_breakpoint(node.address, 1);
            }
        }
        // Record locations in test.log, run synoptic, then read the output file
        if run_level == 0 || run_level == 1 {
            let base = "/home/zack/Tools/MQP/code_slicer";
            // make sure to delete the out.gml file so we don't use stale data
            // intentionally dont fail if the file doesn't exist
            std::fs::remove_file(format!("{}/synoptic/shared/out.gml", &base));

            // let addresses_2 : Vec<usize> = self.address_recorder.get_all_addresses().unwrap().collect();
            // dbg!(addresses_2);
            let addresses = self.address_recorder.get_all_addresses().unwrap();
            //let it: TupleWindows<AddrIter, (usize,usize)> = addresses.tuple_windows();
            let node_names = addresses.map(|addr| &self.nodes.get(&addr).unwrap().FQN);
            let mut output = File::create(format!("{}/synoptic/shared/test.log", &base))?;
            for name in node_names {
                writeln!(output, "{}", name)?;
            }
            let out = Command::new(format!("{}/synoptic/run.sh", &base)).output()?;
            let gml_data = std::fs::read_to_string(format!("{}/synoptic/shared/out.gml", &base))
                .unwrap_or("graph [\n]".into());
            let root = GMLObject::from_str(&gml_data)?;
            let graph = gml_parser::Graph::from_gml(root)?;

            self.build_synoptic_nodes(&graph);
            self.gml_graph = Some(graph);
        }

        self.is_prepared = true;
        Ok(())
    }
    // TODO ensure this fits into the graph rather than just grabbing the
    // address
    pub fn get_addr_occurrences(&self, synoptic_id: usize) -> Vec<TimeStamp> {
        let addr = &self.synoptic_nodes[&synoptic_id].address;
        self.address_recorder.get_addr_occurrences(*addr)
    }
    fn build_synoptic_nodes(&mut self, gml_graph: &gml_parser::Graph) {
        self.synoptic_nodes.clear();
        'outer: for gml_node in &gml_graph.nodes {
            for (_, my_node) in &self.nodes {
                if my_node.FQN == gml_node.label.clone().unwrap() {
                    self.synoptic_nodes
                        .insert(gml_node.id as usize, my_node.clone());
                    continue 'outer;
                }
            }
        }
    }
    fn get_synoptic_node_groups<'a>(
        module_nodes: &'a Vec<&'a gml_parser::Node>,
        edges: &Vec<gml_parser::Edge>,
    ) -> Vec<Vec<&'a gml_parser::Node>> {
        let mut groups: Vec<Vec<&gml_parser::Node>> = Vec::new();
        let mut outgoing_pairs = Vec::new(); // Node -> id
        let mut incoming_pairs = Vec::new(); // id -> Node
        'node_loop: for node in module_nodes {
            let mut incoming_pair = None;
            let mut outgoing_pair = None;

            'outgoing_search: for edge in edges {
                if edge.source == node.id {
                    if outgoing_pair.is_some() {
                        outgoing_pair = None;
                        break 'outgoing_search;
                    }
                    outgoing_pair = Some((node, edge.target));
                }
            }
            'incoming_search: for edge in edges {
                if edge.target == node.id {
                    if incoming_pair.is_some() {
                        incoming_pair = None;
                        break 'incoming_search;
                    }
                    incoming_pair = Some((edge.source, node));
                }
            }
            if let Some(outgoing_pair) = outgoing_pair {
                outgoing_pairs.push(outgoing_pair)
            }
            if let Some(incoming_pair) = incoming_pair {
                incoming_pairs.push(incoming_pair)
            }
        }
        let mut strict_pairs = Vec::new();
        for outgoing_pair in &outgoing_pairs {
            for incoming_pair in &incoming_pairs {
                if outgoing_pair.0.id == incoming_pair.0 && outgoing_pair.1 == incoming_pair.1.id {
                    strict_pairs.push((&outgoing_pair.0, &incoming_pair.1));
                }
            }
        }
        'pair: for strict_pair in strict_pairs {
            let source = strict_pair.0;
            let dest = strict_pair.1;
            let mut src_group = None;
            let mut dest_group = None;
            for (index,group) in groups.iter().enumerate() {
                if group.contains(&source) {
                    if src_group.is_some() {
                        panic!("Duplicate src group entries. This is not allowed.");
                    }
                    src_group = Some(index);
                }
                if group.contains(&dest) {
                    if dest_group.is_some() {
                        panic!("Duplicate dest group entries. This is not allowed.")
                    }
                    dest_group = Some(index);
                }
            }
            if src_group.is_none() && dest_group.is_none() {
                groups.push(vec![source, dest]);
            } else if src_group.is_some() && dest_group.is_none() {
                groups[src_group.unwrap()].push(dest);
            } else if src_group.is_none() && dest_group.is_some() {
                groups[dest_group.unwrap()].push(source);
            } else { //both are some
                let dest_group_vals = groups[dest_group.unwrap()].clone();
                groups[src_group.unwrap()].extend(dest_group_vals);
                groups.swap_remove(dest_group.unwrap());
            }
            // for mut group in &mut groups {
            //     if group.contains(&source) {
            //         if !group.contains(&dest) {
            //             group.push(dest);
            //         }
            //         continue 'pair;
            //     } else if group.contains(&dest) {
            //         if !group.contains(&source) {
            //             group.push(source);
            //         }
            //         continue 'pair;
            //     }
            // }
            // Not in any group, create a new group
            // groups.push(vec![source, dest]);
        }
        // Add all of the solo nodes to their own
        // single groups
        'node: for node in module_nodes {
            for group in &groups {
                if group.contains(&node) {
                    continue 'node;
                }
            }
            groups.push(vec![node]);
        }
        groups
    }
    fn create_node_recursive(
        &self,
        parent_name: Option<&str>,
        parent_scope: &mut Scope,
        nodes: &Vec<gml_parser::Node>,
        edges: &Vec<gml_parser::Edge>,
        settings: &Settings,
        collapsed_module_map: &Vec<(Vec<i64>, i64)>,
    ) {
        if parent_name.is_some() {
            parent_scope
                .set_font_size(20.)
                .set_pen_width(2.)
                .set_style(dot_writer::Style::Rounded)
                .set_label(parent_name.unwrap());
        }
        let mut module_nodes = Vec::new();
        let mut module_fqns = HashSet::new();
        for node in nodes {
            let label = &node.label.clone().unwrap();
            if label == "INITIAL" || label == "TERMINAL" {
                if parent_name == None {
                    parent_scope
                        .node_named(format!("N{}", node.id))
                        .set_label(&label);
                }
                continue;
            }
            let (p_name, s_name) = Self::get_direct_module_parent(&label);
            if p_name == parent_name {
                module_nodes.push(node);
                module_fqns.insert(label.clone());
                // Check if node is collapsed and add the collapsed node
                for (collapsed_group, new_id) in collapsed_module_map {
                    if collapsed_group.contains(&node.id) {
                        parent_scope
                            .node_named(format!("C{}", new_id))
                            .set_label("...");
                        // Do not continue adding nodes or children
                        return;
                    }
                }
            }
        }

        // ADD ALL OF THE UN-RUN EVENTS TO THE MODULE
        if settings.show_unreachable_nodes {
            let mut unruncluster = parent_scope.cluster();
            unruncluster.set_label("Unreachable")
                        .set_pen_width(1.)
                        .set_style(dot_writer::Style::Dashed);
            for event in self.nodes.values() {
                let (p_name, s_name) = Self::get_direct_module_parent(&event.FQN);
                if p_name == parent_name {
                    if !module_fqns.contains(&event.FQN) {
                        // PURE LUCK THAT THIS WORKS
                        let is_selected = Some(event.address as usize) == settings.selected_node_id;
                        let color: dot_writer::Color = if is_selected {
                            dot_writer::Color::Red
                        } else {
                            dot_writer::Color::Black
                        };
                        unruncluster
                            .node_named(format!("U{}", event.address))
                            .set_color(color)
                            .set_pen_width(1.)
                            .set_style(dot_writer::Style::Dashed)
                            .set_label(s_name);
                    }
                }
            }
        }
        //DO ALL OF THE GROUPS
        let groups = Self::get_synoptic_node_groups(&module_nodes, edges);
        for group in groups {
            let group_len = group.len();

            let mut cluster = if group_len == 1 {
                parent_scope.subgraph()
            } else {
                parent_scope.cluster()
            };
            cluster.set_label("")
                    .set_pen_width(1.)
                    .set_style(dot_writer::Style::Dashed);

            for node in group {
                let label = &node.label.clone().unwrap();
                let is_selected = Some(node.id as usize) == settings.selected_node_id;
                let color: dot_writer::Color = if is_selected {
                    dot_writer::Color::Red
                } else {
                    dot_writer::Color::Black
                };
                let mut shape = dot_writer::Shape::Rectangle;
                let mut name = None;
                'inner: for real_node in self.nodes.values() {
                    if real_node.FQN == *label {
                        if real_node.node_type == "Flow" {
                            shape = dot_writer::Shape::Mdiamond;
                        }
                        name = Some(format!("{}", &real_node.name));
                        break 'inner;
                    }
                }
                cluster
                    .node_named(format!("N{}", node.id))
                    .set_color(color)
                    .set_shape(shape)
                    .set_label(&name.unwrap());
            }
        }

        for (mod_name, module) in &self.modules {
            if module.parent.as_deref() == parent_name {
                let mut child_scope = if parent_name.is_none() {
                    parent_scope.subgraph()
                } else {
                    parent_scope.cluster()
                };
                self.create_node_recursive(
                    Some(&mod_name),
                    &mut child_scope,
                    nodes,
                    edges,
                    settings,
                    collapsed_module_map,
                );
            }
        }
    }
    fn calculate_hash(t: &str) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish() / 10000000000
    }
    fn get_collapsed_children_recursive(
        &self,
        mut is_collapsed: bool,
        current_module_str: &str,
    ) -> Vec<(Vec<i64>, i64)> {
        let mut collapsed_module_ignore: (Vec<i64>, i64) = (Vec::new(), (Self::calculate_hash(current_module_str) as i64).abs());
        if !is_collapsed {
            let this_module = self.modules.get(current_module_str).unwrap();
            if this_module.module_attributes.get("collapsed") == Some(&"true".to_string()) {
                is_collapsed = true;
            }
        }
        // Loop through all of the nodes... again
        // I need to pick better data structures
        if is_collapsed {
            for (id, node) in &self.synoptic_nodes {
                if node.module == current_module_str {
                    collapsed_module_ignore.0.push(*id as i64);
                }
            }
        }

        let mut to_ret = Vec::new();
        if is_collapsed {
            to_ret.push(collapsed_module_ignore);
        }
        // find children
        for (mod_name, module) in &self.modules {
            if module.parent.as_deref() == Some(current_module_str) {
                // if this is a collapsed module, we will only be returning 1
                // giant vec
                if is_collapsed {
                    let child_children = self.get_collapsed_children_recursive(true, mod_name);
                    for elem in &child_children[0].0 {
                        to_ret[0].0.push(*elem);
                    }
                }
                // if this is not a collapsed module, then each of the children who return vectors
                // need to have their vectors flat-inserted into this one
                else {
                    let child_children = self.get_collapsed_children_recursive(false, mod_name);
                    for list in child_children {
                        to_ret.push(list);
                    }
                }
            }
        }
        to_ret
    }
    fn get_direct_module_parent(name: &str) -> (Option<&str>, &str) {
        let elems: Vec<&str> = name.split("::").collect();
        let m_name = elems[elems.len() - 2];
        let self_name = elems[elems.len() - 1];
        if m_name.len() == 0 {
            (None, self_name)
        } else {
            (Some(m_name), self_name)
        }
    }

    fn gml_to_dot_str(
        &self,
        gml_graph: &gml_parser::Graph,
        settings: &Settings,
    ) -> anyhow::Result<String> {
        let mut output_bytes = Vec::new();
        {
            let mut writer = DotWriter::from(&mut output_bytes);
            writer.set_pretty_print(false);
            let mut digraph = writer.digraph();
            let mut collapsed_module_map: Vec<(Vec<i64>, i64)> =
                self.get_collapsed_children_recursive(false, "");

            self.create_node_recursive(
                None,
                &mut digraph,
                &gml_graph.nodes,
                &gml_graph.edges,
                settings,
                &collapsed_module_map,
            );
            let mut edge_vec = HashSet::new();
            'edge: for edge in &gml_graph.edges {
                // for node in &gml_graph.nodes{
                //     if node.id == edge.source && node.label == Some("INITIAL".into()) {
                //         continue 'edge;
                //     }
                //     if node.id == edge.target && node.label == Some("TERMINAL".into()) {
                //         continue 'edge;
                //     }
                // }

                let mut source = edge.source;
                let mut target = edge.target;
                let mut src_prefix = "N";
                let mut target_prefix = "N";

                for collapsed_module in &collapsed_module_map {
                    if collapsed_module.0.contains(&source) {
                        source = collapsed_module.1;
                        src_prefix = "C";
                        break;
                    }
                }
                for collapsed_module in &collapsed_module_map {
                    if collapsed_module.0.contains(&target) {
                        target = collapsed_module.1;
                        target_prefix = "C";
                        break;
                    }
                }
                if edge_vec.contains(&(source,target)) {
                    continue 'edge;
                }
                edge_vec.insert((source,target));

                let mut attribs = digraph
                    .edge(format!("{}{}",src_prefix, source), format!("{}{}", target_prefix, target))
                    .attributes();
                if let Some(label) = edge.label.clone() {
                    let val = &label[3..];
                    // dbg!(&val);
                    let val = val.parse::<f32>();
                    if let Ok(val) = val {
                        attribs.set_pen_width(val * 5. + 1.5);
                    }
                }
                if Some(source as usize) == settings.selected_node_id {
                    attribs.set_color(dot_writer::Color::Red);
                    // attribs.set_rank(dot_writer::Rank::Max);
                } else if Some(target as usize) == settings.selected_node_id {
                    attribs.set_color(dot_writer::Color::Blue);
                    // attribs.set_rank(dot_writer::Rank::Max);
                }
            }
        }
        Ok(String::from_utf8(output_bytes)?)
    }
}
