use std::collections::HashMap;
use std::{collections::HashSet, io::BufWriter};
use std::borrow::Cow;
use std::io::Write;

use librr_rs::BinaryInterface;
use itertools::Itertools;
use itertools::TupleWindows;
use std::ops::Range;
use std::fs::File;

use librr_rs::*;
use crate::address_recorder::AddrIter;
use crate::{address_recorder::AddressRecorder, shared_structs::{GraphNode, TimeStamp}, query::node::TimeRange};

// TODO
// Implement a clever prepared scheme 
// that saves prior work.
//
// This implementation is comically inefficient
//
pub struct GraphBuilder {
    address_recorder : AddressRecorder,
    graph_nodes : HashMap<usize,GraphNode>,
    ranges : Vec<TimeRange>,
    breakpoints : HashSet<usize>,
    is_prepared: bool,
}

// Stablize negative f------ trait impls
// impl !Send for GraphBuilder 


impl GraphBuilder {
    pub fn new(max_ft : usize) -> Self {
        Self {
            address_recorder: AddressRecorder::new(max_ft),
            graph_nodes: HashMap::new(),
            ranges: Vec::new(),
            breakpoints: HashSet::new(),
            is_prepared: false,
            
        }
    }
    pub fn insert_time_range(&mut self, ts: TimeRange) {
        self.ranges.push(ts);
        self.is_prepared = false;
    }
    pub fn insert_graph_node(&mut self, node: GraphNode) {
        self.graph_nodes.insert(node.address,node);
        self.is_prepared = false;
    }

    pub fn get_graph_as_dot(&self) -> Option<String> {
        if !self.is_prepared {
            return None;
        }
        let addresses = self.address_recorder.get_all_addresses().unwrap();
        let mut buf = BufWriter::new(Vec::new());
        let it: TupleWindows<AddrIter, (usize,usize)> = addresses.tuple_windows();
    let mut f = File::create("example1.dot").unwrap();
        // render_to(&mut buf, it.collect());
        render_to(&mut f, it.collect());

        let bytes = buf.into_inner().unwrap();
        let string = String::from_utf8(bytes).unwrap();
        Some(string)
    }
    //TODO: This code is heavily flawed and was written hastily in order to get something 
    //written 
    // TODO : This code is also very fragile and /requires/ tests
    // 
    pub fn prepare(&mut self, bin_interface: &mut BinaryInterface) -> anyhow::Result<()> {
        let cont = GdbContAction {
            type_: GdbActionType::ACTION_CONTINUE,
            target: bin_interface.get_current_thread(),
            signal_to_deliver:0,
        };
        let step = GdbContAction {
            type_: GdbActionType::ACTION_STEP,
            target: bin_interface.get_current_thread(),
            signal_to_deliver:0,
        };
            
        for node in self.graph_nodes.values() {
            bin_interface.pin_mut().set_sw_breakpoint(node.address, 1);
        }
        dbg!(self.graph_nodes.len());

        let mut opened_frame_time : i64 = bin_interface.current_frame_time();
        self.address_recorder.reset_ft_for_writing(opened_frame_time as usize);

        let mut signal = 5;
        while signal == 5 {
            let rip = bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread()).to_usize();
            let current_ft = bin_interface.current_frame_time();
            if current_ft != opened_frame_time {
                self.address_recorder.finished_writing_ft();
                opened_frame_time = current_ft;
                self.address_recorder.reset_ft_for_writing(opened_frame_time as usize);
            }
            // serious problems about efficiently telling if this is an address of a node 
            // or of a timestamp
            if self.graph_nodes.contains_key(&rip) {
                self.address_recorder.insert_address(rip);

                // meaning there is a breakpoint at rip
                // so we have to step over it when there is no 
                // breakpoint
                bin_interface.pin_mut().remove_sw_breakpoint(rip,1);
                signal = bin_interface.pin_mut().continue_forward(step).unwrap();
                if signal !=5 {
                    break;
                }
                bin_interface.pin_mut().set_sw_breakpoint(rip,1);
            }
            
            signal = bin_interface.pin_mut().continue_forward(cont).unwrap();
        }
        self.address_recorder.finished_writing_ft();
        self.is_prepared = true;
        Ok(())

    }
        

}
type Nd = usize;
type Ed = (usize,usize);
struct Edges(Vec<Ed>);

pub fn render_to<W: Write>(output: &mut W, e_vec: Vec<Ed>) {
    let edges = Edges(e_vec);
    dot::render(&edges, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Edges {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example1").unwrap() }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", *n)).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Edges {
    fn nodes(&self) -> dot::Nodes<'a,Nd> {
        // (assumes that |N| \approxeq |E|)
        let &Edges(ref v) = self;
        let mut nodes = Vec::with_capacity(v.len());
        for &(s,t) in v {
            nodes.push(s); nodes.push(t);
        }
        nodes.sort();
        nodes.dedup();
        Cow::Owned(nodes)
    }

    fn edges(&'a self) -> dot::Edges<'a,Ed> {
        let &Edges(ref edges) = self;
        Cow::Borrowed(&edges[..])
    }

    fn source(&self, e: &Ed) -> Nd { e.0 }

    fn target(&self, e: &Ed) -> Nd { e.1 }
}

