use std::collections::HashSet;

use librr_rs::BinaryInterface;

use crate::{address_recorder::AddressRecorder, shared_structs::{GraphNode, TimeStamp}, query::node::TimeRange};

// TODO
// Implement a clever prepared scheme 
// that saves prior work.
//
// This implementation is comically inefficient
//
pub struct GraphBuilder {
    address_recorder : AddressRecorder,
    graph_nodes : Vec<GraphNode>,
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
            graph_nodes: Vec::new(),
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
        self.graph_nodes.push(node);
        self.is_prepared = false;
    }
    pub fn get_graph_as_dot(&self) -> Option<String> {
        if !self.is_prepared {
            return None;
        }
        None
    }
    pub fn prepare(&mut self, bin_interface: &mut BinaryInterface) -> anyhow::Result<()> {
        
        Ok(())
    }
}

