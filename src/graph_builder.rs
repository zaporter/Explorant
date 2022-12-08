use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
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
    pub fn update_raw_nodes(&mut self,mut nodes: HashMap<usize,GraphNode>) -> anyhow::Result<()>{
        self.is_prepared = false;
        for mut node in &mut nodes.values_mut() {
            // Update for empty FQN and
            // also update in case module changed. 
            node.FQN = file_parsing::name_to_fqn(&format!("{}::{}", &node.module, &node.name), &self.modules)?;
        }
        self.nodes = nodes;
        Ok(())
    }
    pub fn update_raw_modules(&mut self,modules: HashMap<String,GraphModule>)->anyhow::Result<()>{
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
    //
    pub fn prepare(&mut self, bin_interface: &mut BinaryInterface) -> anyhow::Result<()> {
        log::warn!("a");

        // bin_interface.pin_mut().restart_from_event(0);

        log::warn!("a2");
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
            dbg!(&node);
            bin_interface.pin_mut().set_sw_breakpoint(node.address, 1);
        }
        dbg!(self.nodes.len());

        log::warn!("a3");
        let mut opened_frame_time: i64 = bin_interface.current_frame_time();
        self.address_recorder
            .reset_ft_for_writing(opened_frame_time as usize);

        let mut signal = 5;
        while signal == 5 {
            let rip = bin_interface
                .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
                .to_usize();
            dbg!(&rip);
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

        log::warn!("a4");
        self.address_recorder.finished_writing_ft();

        // Record locations in test.log, run synoptic, then read the output file
        {
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
                dbg!(&name);
                writeln!(output, "{}", name)?;
            }
            let out = Command::new(format!("{}/synoptic/run.sh", &base)).output()?;
            dbg!(out);
            let gml_data = std::fs::read_to_string(format!("{}/synoptic/shared/out.gml", &base))
                .unwrap_or("graph [\n]".into());
            let root = GMLObject::from_str(&gml_data)?;
            let graph = gml_parser::Graph::from_gml(root)?;

            self.build_synoptic_nodes(&graph);
            self.gml_graph = Some(graph);
        }
        log::warn!("a5");
        // Remove all set swbreakpoints to not alter internal state of machine
        for node in self.nodes.values() {
            bin_interface.pin_mut().remove_sw_breakpoint(node.address, 1);
        }

        log::warn!("a6");

        self.is_prepared = true;
        Ok(())
    }
    // TODO ensure this fits into the graph rather than just grabbing the 
    // address
    pub fn get_addr_occurrences(&self, synoptic_id: usize) -> Vec<TimeStamp>{
        let addr = &self.synoptic_nodes[&synoptic_id].address;
        self.address_recorder.get_addr_occurrences(*addr)

    }
    fn build_synoptic_nodes(&mut self, gml_graph: &gml_parser::Graph) {
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
    fn create_node_recursive(
        &self,
        parent_name: Option<&str>,
        parent_scope: &mut Scope,
        nodes: &Vec<gml_parser::Node>,
        settings: &Settings,
    ) {
        if parent_name.is_some() {
            parent_scope.set_label(parent_name.unwrap());
        }
        for node in nodes {
            let label = &node.label.clone().unwrap();
            if label == "INITIAL" || label == "TERMINAL" {
                continue;
            }
            let (p_name, s_name) = Self::get_direct_module_parent(&label);
            if p_name == parent_name {
                let is_selected = Some(node.id as usize) == settings.selected_node_id;
                let color: dot_writer::Color = if is_selected {
                    dot_writer::Color::Red
                } else {
                    dot_writer::Color::Black
                };
                parent_scope
                    .node_named(format!("N{}", node.id))
                    .set_color(color)
                    .set("root", "true", false)
                    .set_label(&label);
            }
        }
        for (mod_name, module) in &self.modules {
            if module.parent.as_deref() == parent_name {
                let mut child_scope = parent_scope.cluster();
                self.create_node_recursive(Some(&mod_name), &mut child_scope, nodes, settings);
            }
        }
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

            self.create_node_recursive(None, &mut digraph, &gml_graph.nodes, settings);
            // let mut cluster_map : HashMap<Option<String>, Rc<RefCell<Scope>>>  = HashMap::new();
            // cluster_map.insert(None, Rc::new(RefCell::new(digraph)));
            // let mut to_resolve = VecDeque::new();
            // to_resolve.push_back(None);
            // while let Some(resolving) = to_resolve.pop_front(){
            //         let parent_scope = cluster_map.get(&resolving).unwrap();
            //     for (key, module) in &self.modules {
            //         if module.parent == resolving {
            //             let to_insert = Some(key.clone());
            //             to_resolve.push_back(to_insert.clone());

            //             let inner_scope = parent_scope.borrow_mut().cluster();
            //             cluster_map.insert(to_insert.clone(), Rc::new(RefCell::new(inner_scope)));
            //         }

            //     }

            // }
            // for node in gml_graph.nodes {
            //     digraph.node_named(format!("N{}",node.id))
            //         .set_label(&node.label.unwrap_or("[unnamed]".into()))
            //         .set_color(dot_writer::Color::Red);
            // }
            for edge in &gml_graph.edges {
                let mut attribs = digraph
                    .edge(format!("N{}", edge.source), format!("N{}", edge.target))
                    .attributes();
                if let Some(label) = edge.label.clone() {
                    let val = &label[3..];
                    // dbg!(&val);
                    let val = val.parse::<f32>();
                    if let Ok(val) = val {
                        attribs.set_pen_width(val * 6.);
                    }
                }
                // if Some(edge.source as usize) == settings.selected_node_id {
                //     attribs.set_label(&edge.label.clone().unwrap_or("".into()));
                // } else if Some(edge.target as usize) == settings.selected_node_id {
                //     attribs.set_label(&edge.label.clone().unwrap_or("".into()));
                // }
            }
        }
        Ok(String::from_utf8(output_bytes)?)
    }
}

