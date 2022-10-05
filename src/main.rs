#![allow(unused_imports)]
use druid_graphviz_layout::adt::dag::NodeHandle;
use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, Register, RflagsBits,
};
use itertools::{Itertools, Zip};
use librr_rs::*;
use procmaps::{Map, Mappings};
use rust_lapper::{Interval, Lapper};
use similar::{capture_diff_slices, Algorithm};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::time::SystemTime;
use std::{error::Error, sync::Arc};

use crate::block::{Block, BlockEvaluation, CodeFlow};

mod block;
// mod code_flow_graph;
// mod graph_layout;
mod gui;
mod recorder;
mod lcs;
mod trampoline;
mod query;
use crate::query::*;
use crate::lcs::*;
use crate::trampoline::*;

// fn get_symbols<'a>(
//     file: &'a object::File,
// ) -> Result<Vec<(String, object::Symbol<'a, 'a>)>, Box<dyn Error>> {
//     let mut to_ret = Vec::new();
//     for symbol in file.symbol_table().ok_or("No symboltable found")?.symbols() {
//         let name: String = Name::from(symbol.name().unwrap())
//             .try_demangle(DemangleOptions::name_only())
//             .to_string();
//         to_ret.push((name, symbol));
//     }
//     Ok(to_ret)
// }
fn main() {

    librr_rs::raise_resource_limits();
    gui::start_query_editor();
    // let output_directory = "/home/zack/dbfss";
    // let exe_path = "/home/zack/war_simulator";
    // recorder::record(exe_path,output_directory);
    // let addrs_no_div= get_addrs(PathBuf::from_str("/home/zack/.local/share/rr/war_simulator-3").unwrap());
    // let time = SystemTime::now();
    // let mut tree = BlockVocabulary::default();
    // tree.add_experience_to_vocabulary(&addrs_no_div);
    // dbg!(tree.num_words);

    // // tree.add_experience_to_vocabulary(&addrs_div);
    // dbg!(tree.num_words);
    // let no_div_encoded = tree.addrs_to_block_vocabulary(&addrs_no_div);
    // // let div_encoded=tree.addrs_to_block_vocabulary(&addrs_div);
    // dbg!(no_div_encoded.len());
    // // dbg!(div_encoded.len());
    // dbg!(time.elapsed().unwrap());

}
fn get_addrs(sample_dateviewer_dir: PathBuf) -> Vec<usize> {
    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir.clone());

    let cthread = bin_interface.get_current_thread();
    bin_interface.pin_mut().set_query_thread(cthread);
    bin_interface.set_pass_signals(vec![
        0,0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
    ]);
    let rip = bin_interface
        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
        .to_usize();
    dbg!(rip);

    let mut stack_info = TrampolineStackInfo {
        base_addr: 0x71000000,
        // Ive had success with 65KiB 
        // but I made it 256 MiB just in case. 
        // This shouldn't overflow
        //
        //NOTE: 
        //  This is consistently faster on my machine (~0.05 sec) if 
        //  it is given 1GiB instead of 256MiB. 
        //  I have no idea why. 
        size: 0x10000000,
        reserved_space: 0x40,
    };
    stack_info.allocate_map(&mut bin_interface);
    stack_info.setup_stack_ptr(&mut bin_interface).unwrap();
    // dbg!(bin_interface.get_proc_map());
    let mut proc_map: Lapper<usize, Map> = Lapper::new(vec![]);
    for map in bin_interface.get_proc_map().unwrap().iter() {
        proc_map.insert(Interval {
            start: map.base,
            stop: map.ceiling,
            val: map.clone(),
        });
    }

    let mut tr = TrampolineManager::new(&mut bin_interface, stack_info, &proc_map);
    tr.create_trampolines(&mut bin_interface).unwrap();
    let step = GdbContAction {
        type_: GdbActionType::ACTION_CONTINUE,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };
    let start_continue = SystemTime::now();
    let mut signal = 5;
    while signal != 9 {
        signal = bin_interface.pin_mut().continue_forward_jog_undefined(step).unwrap();
        tr.clear_address_stack(&mut bin_interface).unwrap();
    }
    dbg!(signal);
    dbg!(bin_interface.current_frame_time());
    dbg!(start_continue.elapsed().unwrap());
    let entries = tr.recorded_addresses();
    dbg!(entries.len());
    entries.clone()
}

/**
 * DO NOT USE FOR PROD.
 * VERY SLOW
 */
fn get_current_instr(bin_interface: &BinaryInterface) -> Instruction {
    let rip = bin_interface
        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
        .to_usize();

    let bytes = bin_interface.get_mem(rip, 18);
    let mut decoder = Decoder::with_ip(64, &bytes, rip as u64, DecoderOptions::NONE);
    let mut instr = Instruction::default();
    decoder.decode_out(&mut instr);
    instr
}
fn create_code_flow(bin_interface: &mut BinaryInterface) -> Result<CodeFlow, Box<dyn Error>> {
    let step = GdbContAction {
        type_: GdbActionType::ACTION_STEP,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };

    let cont = GdbContAction {
        type_: GdbActionType::ACTION_CONTINUE,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };
    // bin_interface.pin_mut().set_sw_breakpoint(main_addr,1);
    // bin_interface.pin_mut().continue_forward(cont);
    // bin_interface.pin_mut().remove_sw_breakpoint(main_addr,1);
    let mut code_flow = CodeFlow::default();
    let mut signal = 5;

    'outer: while signal == 5 {
        let rip = bin_interface
            .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
            .to_usize();
        code_flow.path.push(rip);

        // let instructions = read_instructions_till_flow_change(&bin_interface, rip);
        let block = if let Some(block) = code_flow.blocks.find(rip, rip).next() {
            block.val.clone()
        } else {
            let instrs = read_instructions_till_flow_change(&bin_interface, rip);
            let start = rip;
            let stop = instrs.last().unwrap().ip() as usize;
            let block = Block::new(start, stop, instrs);
            code_flow.blocks.insert(Interval {
                start: start - 1,
                stop: stop + 1,
                val: Arc::new(block),
            });
            code_flow.blocks.find(rip, rip).next().unwrap().val.clone()
        };

        break 'outer;
        let instructions = block.instructions();

        bin_interface
            .pin_mut()
            .set_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
        signal = bin_interface.pin_mut().continue_forward(cont).unwrap();
        if signal != 5 {
            break 'outer;
        }
        bin_interface
            .pin_mut()
            .remove_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
        signal = bin_interface.pin_mut().continue_forward(step).unwrap();
    }

    Ok(code_flow)
}

fn read_instructions(
    bin_interface: &BinaryInterface,
    start_addr: usize,
    size: usize,
) -> Vec<Instruction> {
    let bytes = bin_interface.get_mem(start_addr, size);
    let mut instructions = Vec::new();
    let mut decoder = Decoder::with_ip(64, &bytes, start_addr as u64, DecoderOptions::NONE);
    let mut instr = Instruction::default();
    while decoder.can_decode() {
        decoder.decode_out(&mut instr);
        instructions.push(instr);
    }
    instructions
}

const READ_CHUNK_SIZE: usize = 40;
fn read_instructions_till_flow_change(
    bin_interface: &BinaryInterface,
    ip: usize,
) -> Vec<Instruction> {
    let mut base = ip;
    let mut instructions = Vec::new();
    loop {
        let bytes = bin_interface.get_mem(base, READ_CHUNK_SIZE);

        let mut decoder = Decoder::with_ip(64, &bytes, base as u64, DecoderOptions::NONE);
        let mut instr = Instruction::default();
        let mut last_successful_position = 0;
        while decoder.can_decode() {
            decoder.decode_out(&mut instr);
            if instr.code() != Code::INVALID {
                last_successful_position = decoder.position();
            } else {
                break;
            }

            let non_next_flow = instr.flow_control() != FlowControl::Next;
            instructions.push(std::mem::take(&mut instr));
            if non_next_flow {
                return instructions;
            }
        }
        base += last_successful_position;
    }
}
