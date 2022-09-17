#![allow(unused_imports)]
use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, RflagsBits, Register,
};
use itertools::Itertools;
use librr_rs::*;
use rust_lapper::{Interval, Lapper};
use std::path::PathBuf;
use std::str::FromStr;
use std::{error::Error, sync::Arc};
use std::time::SystemTime;

use crate::block::{Block, BlockEvaluation, CodeFlow};


mod block;
// mod code_flow_graph;
// mod graph_layout;
// mod gui;
mod trampoline;
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
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/hello_world-5").unwrap();
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/fizzbuzz-5/").unwrap();
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/date_viewer-102").unwrap();
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/cargo-1").unwrap();
    // let main_addr :usize = 0x558ce6f8b060;
    let sample_dateviewer_dir =
        PathBuf::from_str("/home/zack/.local/share/rr/war_simulator-3").unwrap();
    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir.clone());

    let rip = bin_interface
        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
        .to_usize();
    dbg!(rip);

    let start = SystemTime::now();

    let code_flow = create_code_flow(&mut bin_interface).unwrap();
    let duration = start.elapsed().unwrap();
    dbg!(duration);
    dbg!(code_flow.blocks.len());
    dbg!(code_flow.path.len());
    // let first_block = code_flow.blocks.into_iter().next().unwrap().val;
    // let instructions = first_block.instructions();
    // let instructions = &read_instructions(&bin_interface, 0x401000, 0x1000);
    // let instructions = &read_instructions(&bin_interface, 0x55555555a000, 0x3a000);
    let instructions = &read_instructions(&bin_interface, 0x55555555b000, 0x42000);
    // dbg!(&instructions);

    //let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir);
    let rip = bin_interface
        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
        .to_usize();
    dbg!(rip);
    // build_trampoline_for_instr(&mut bin_interface, &int_instr).unwrap();
    let mut tr = TrampolineManager::default();
    let possible_vuln_jumps = tr.identify_possible_vulnerable_jumps(instructions);
    dbg!(possible_vuln_jumps.len());
    tr.setup_stack_ptr(&mut bin_interface).unwrap();
    tr.create_trampolines(&mut bin_interface, instructions, possible_vuln_jumps.into_keys().collect_vec())
        .unwrap();
    // tr.patch_jumps_into_trampoline(&mut bin_interface, possible_vuln_jumps);
    dbg!(tr.unwatched_instructions.len());
    dbg!(tr.allocations.len());
    // let code_flow = create_code_flow(&mut bin_interface).unwrap();
    // let first_block = code_flow.blocks.into_iter().next().unwrap().val;
    let step = GdbContAction {
        type_: GdbActionType::ACTION_CONTINUE,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };
    let mut num_instructions: u128 = 0;
    let mut signal = 5;
    while signal == 5 {
        num_instructions+=1;
        signal = bin_interface.pin_mut().continue_forward(step);
        // let rip = bin_interface
        //     .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
        //     .to_usize();
        // dbg!(rip);
        // let current = get_current_instr(&bin_interface);
        // println!("{}",current);
    }
    dbg!(num_instructions);
    tr.clear_address_stack(&mut bin_interface).unwrap();
    let entries = tr.recorded_addresses();
    dbg!(entries.len());

    // gui::start_code_flow_examiner(code_flow);
}

/**
 * DO NOT USE FOR PROD. 
 * VERY SLOW
 */
fn get_current_instr(bin_interface: &BinaryInterface)->Instruction{
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
    let cthread = bin_interface.get_current_thread();
    bin_interface.pin_mut().set_query_thread(cthread);
    bin_interface.set_pass_signals(vec![
        0, 0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
    ]);
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
        signal = bin_interface.pin_mut().continue_forward(cont);
        if signal != 5 {
            break 'outer;
        }
        bin_interface
            .pin_mut()
            .remove_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
        signal = bin_interface.pin_mut().continue_forward(step);
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
