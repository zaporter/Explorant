use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, RflagsBits,
};
use librr_rs::*;
use object::{
    Object, ObjectSection, ObjectSymbol, ObjectSymbolTable, Section, SectionKind, Segment,
};
use rust_lapper::{Interval, Lapper};
use std::path::PathBuf;
use std::pin::Pin;
use std::str::FromStr;
use std::{error::Error, sync::Arc};
use symbolic_common::{Language, Name};
use symbolic_demangle::{Demangle, DemangleOptions};

use std::time::SystemTime;

use crate::block::{Block, BlockEvaluation, CodeFlow};

mod block;
mod gui;
mod code_flow_graph;
mod graph_layout;

fn get_symbols<'a>(
    file: &'a object::File,
) -> Result<Vec<(String, object::Symbol<'a, 'a>)>, Box<dyn Error>> {
    let mut to_ret = Vec::new();
    for symbol in file.symbol_table().ok_or("No symboltable found")?.symbols() {
        let name: String = Name::from(symbol.name().unwrap())
            .try_demangle(DemangleOptions::name_only())
            .to_string();
        to_ret.push((name, symbol));
    }
    Ok(to_ret)
}
fn main() {
    // let sample_dateviewer_dir = PathBuf::from_str("/home/zack/.local/share/rr/binary-0").unwrap();
    let sample_dateviewer_dir = PathBuf::from_str("/home/zack/.local/share/rr/fizzbuzz-0/").unwrap();
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/date_viewer-95").unwrap();
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/cargo-1").unwrap();
    // let main_addr :usize = 0x558ce6f8b060;
    // let sample_dateviewer_dir =
    //     PathBuf::from_str("/home/zack/.local/share/rr/war_simulator-2").unwrap();
    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir);

    let start = SystemTime::now();

    let code_flow = create_code_flow(&mut bin_interface).unwrap();
    let duration = start.elapsed().unwrap();
    dbg!(duration);
    dbg!(code_flow.blocks.len());
    dbg!(code_flow.path.len());
    gui::start_code_flow_examiner(code_flow);
}
// fn fill_in_jumps(code_flow: &mut CodeFlow){


// }

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

        let instructions = block.instructions();

        bin_interface
            .pin_mut()
            .set_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
        signal = bin_interface.pin_mut().continue_forward(cont);
        if signal != 5 {
            dbg!("SIGNAL 9");
            break 'outer;
        }
        bin_interface
            .pin_mut()
            .remove_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
        signal = bin_interface.pin_mut().continue_forward(step);
    }

    Ok(code_flow)
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
