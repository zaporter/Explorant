use std::{ops::DerefMut, rc::Rc, sync::Arc};

use iced_x86::Instruction;
use rust_lapper::Lapper;

pub enum Fidelity {
    // Dynamic code segment instrumentation with singlesteps
    EveryInstruction,
    // Use static code segment instrumentation
    StaticHighFidelity,
    StaticLowFidelity,
}

#[derive(Debug, Clone)]
pub struct CodeFlow {
    pub blocks: Lapper<usize, Arc<Block>>,
    pub path: Vec<usize>,
}
impl Default for CodeFlow {
    fn default() -> Self {
        CodeFlow {
            blocks: Lapper::new(vec![]),
            path: Vec::new(),
        }
    }
}
// blocks are non-overlapping and thus can be placed in a tree to find address of any instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    base: usize,
    ceiling: usize,
    //
    // data flow instructions are everything but the last one
    // the last one is a data flow on RIP...?
    // Actually every single one is a data flow on RIP but I dont want to do that...
    instructions: Vec<Instruction>,
    // sorted by event_time_uid
    evaluations: Vec<Arc<BlockEvaluation>>,
    jumps_to: Vec<usize>,
}

impl Block {
    pub fn new(base: usize, ceiling: usize, instructions: Vec<Instruction>) -> Self {
        Block {
            base,
            ceiling,
            instructions,
            evaluations: Vec::new(),
            jumps_to: Vec::new(),
        }
    }
    pub fn add_evaluation(&mut self, ebi: Arc<BlockEvaluation>) {
        // ensure that we are strictly sorted by event_time_uid
        if let Some(last) = self.evaluations.last() {
            assert!(last.frame_time_uid < ebi.frame_time_uid);
        }
        self.evaluations.push(ebi);
    }
    pub fn base(&self) -> &usize {
        &self.base
    }
    pub fn ceiling(&self) -> &usize {
        &self.ceiling
    }
    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    pub fn evaluations(&self) -> &Vec<Arc<BlockEvaluation>> {
        &self.evaluations
    }
}
// This just serves as a marker to indicate that it is possible to come back here and do
// computation. Add code to come back to this INSTANCE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEvaluation {
    pub caller: Option<Arc<BlockEvaluation>>,
    pub dest: Option<Arc<BlockEvaluation>>,
    pub entry_address: usize,
    pub evaluated_block: Arc<Block>,
    pub frame_time_uid: u64,
}
