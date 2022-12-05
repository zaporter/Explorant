use druid::piet::util::first_strong_rtl;
use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, Register, RflagsBits,
};
use librr_rs::*;
use procmaps::Map;
use rust_lapper::{Interval, Lapper};
use std::{collections::HashMap, error::Error, fmt, time::SystemTime};

use iced_x86::code_asm;
use procmaps::Path::MappedFile;

/**
 * The trampoline stack is meant to store
 * information about
 *
 * [*base_addr
 *  rsp = base_addr+size
 *  free
 *  free
 *  ..
 *  *base_addr+reserved_space
 *  ..
 *  ..
 *  ..
 * *base_addr+size]
 */
#[derive(Debug, Clone)]
pub struct TrampolineStackInfo {
    pub base_addr: usize,
    pub size: usize,
    pub reserved_space: usize,
}
impl TrampolineStackInfo {
    pub fn allocate_map(&self, binary_interface: &mut BinaryInterface) {
        binary_interface
            .pin_mut()
            .mmap_stack(self.base_addr, self.size);
    }

    pub fn setup_stack_ptr(
        &mut self,
        bin_interface: &mut BinaryInterface,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = (self.base_addr + self.size).to_le_bytes().to_vec();
        bin_interface.set_bytes(self.base_addr, bytes)?;

        bin_interface
            .pin_mut()
            .set_byte(self.base_addr + self.reserved_space, 0x90);
        Ok(())
    }
}
/**
 * repeated sections of:
 * xchg TrampolineStackInfo.base_addr rsp
 * instruction replaced
 * push (**)
 * xchg TrampolineStackInfo.base_addr rsp
 * jmp k
 */
#[derive(Clone)]
pub struct TrampolineHeapInfo {
    pub base_addr: usize,
    pub size: usize,
    pub bytes_used: usize,
    // Map of heap_addr -> trampoline
    pub allocations: Lapper<usize, TrampolineInfo>,
    // Instructions not being trampolined
    pub unwatched_instructions: Vec<Instruction>,
}
impl fmt::Debug for TrampolineHeapInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrampolineHeapInfo")
            .field("base_addr", &self.base_addr)
            .field("size", &self.size)
            .field("bytes_used", &self.bytes_used)
            .field("num_allocations", &self.allocations.len())
            .finish()
    }
}
impl TrampolineHeapInfo {
    pub fn allocate_map(&self, binary_interface: &mut BinaryInterface) {
        binary_interface
            .pin_mut()
            .mmap_heap(self.base_addr, self.size);
    }

    fn find_next_free_slot(&self, size: usize) -> Result<usize, Box<dyn Error>> {
        if self.bytes_used + size > self.size {
            return Err("Ran out of heap space for trampolines")?;
        }
        Ok(self.bytes_used)
    }
}
#[derive(Clone, Eq, PartialEq)]
pub struct TrampolineInfo {
    pub replaced_instructions: Vec<Instruction>,
    pub trampoline_instructions: Vec<Instruction>,
}
pub struct TrampolineManager {
    pub stack_info: TrampolineStackInfo,
    // Map of Map->heap_base_addr
    pub trampoline_maps: HashMap<Map, TrampolineHeapInfo>,
    // I dont know
    pub recorded_addresses: Vec<usize>,
}
impl fmt::Debug for TrampolineManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrampolineManager")
            .field("stack_info:", &self.stack_info)
            .field("trampoline_maps:", &self.trampoline_maps)
            .finish()
    }
}
impl TrampolineManager {
    pub fn new(
        bin_interface: &mut BinaryInterface,
        stack_info: TrampolineStackInfo,
        maps: &Lapper<usize, Map>,
    ) -> Self {
        let mut trampoline_maps = HashMap::new();
        for map in maps.iter() {
            if map.val.perms.readable && map.val.perms.executable && !map.val.perms.writable {
                if let MappedFile(path) = &map.val.pathname {
                    if path.contains("librrpage.so") {
                        continue;
                    }
                }
                let heap_info =
                    TrampolineManager::create_heap_for_map(bin_interface, &map.val, maps);
                trampoline_maps.insert(map.val.clone(), heap_info);
            }
        }
        TrampolineManager {
            stack_info,
            trampoline_maps,
            recorded_addresses: Vec::new(),
        }
    }
    //
    // TODO:
    // 2^31 = +/- 2GB. I am using 2^30 because 31 wasnt working
    // If I use 2^31, make sure to offset base by 4096 bytes
    // otherwise it will be too close
    //
    fn create_heap_for_map(
        bin_interface: &mut BinaryInterface,
        target: &Map,
        maps: &Lapper<usize, Map>,
    ) -> TrampolineHeapInfo {
        let heap_possible_bottom = target.ceiling.checked_sub(2_usize.pow(30)).unwrap_or(0);
        let heap_possible_top = target.base + 2_usize.pow(30);
        let heap_size = 0x10000000;
        let heap_base = maps
            .find_free_interval(heap_possible_bottom, heap_possible_top, heap_size)
            .unwrap();
        let heap_info = TrampolineHeapInfo {
            base_addr: heap_base,
            size: heap_size,
            bytes_used: 0,
            unwatched_instructions: Vec::new(),
            allocations: Lapper::new(vec![]),
        };
        heap_info.allocate_map(bin_interface);
        heap_info
    }

    pub fn recorded_addresses(&self) -> &Vec<usize> {
        &self.recorded_addresses
    }
    pub fn clear_address_stack(
        &mut self,
        bin_interface: &mut BinaryInterface,
    ) -> Result<(), Box<dyn Error>> {
        // TODO:
        // increase the vec capacity before this program runs
        // to ensure we don't have to reallocate halfway through
        // each insert
        let stack_end_addr = self.stack_info.base_addr + self.stack_info.size;
        let rsp_bytes = bin_interface.get_mem(self.stack_info.base_addr, 8);
        let rsp = usize::from_le_bytes(rsp_bytes.try_into().unwrap());
        let mut bytes = bin_interface.get_mem(rsp, stack_end_addr - rsp);
        bytes.reverse();
        // we reverse it so that when reading, we read the last thing
        // added first. That also means that we have to read as be
        // bytes
        for be_num_bytes in bytes.chunks(8) {
            self.recorded_addresses
                .push(usize::from_be_bytes(be_num_bytes.try_into().unwrap()));
        }
        self.stack_info.setup_stack_ptr(bin_interface)?;
        Ok(())
    }

    pub fn overwrite_single_instr(
        &self,
        bin_interface: &mut BinaryInterface,
        original: &Instruction,
        new: &Instruction,
    ) -> Result<(), Box<dyn Error>> {
        let mut ca = code_asm::CodeAssembler::new(64)?;
        ca.add_instruction(*new)?;
        let new_bytes = ca.assemble(original.ip())?;
        if new_bytes.len() != original.len() {
            return Err(format!(
                "Unable to overwrite instruction {} with {} as lengths differ! ({} vs {})",
                original,
                new,
                original.len(),
                new_bytes.len()
            ))?;
        }
        bin_interface.set_bytes(original.ip() as usize, new_bytes)?;

        Ok(())
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
    pub fn create_trampolines(
        &mut self,
        bin_interface: &mut BinaryInterface,
    ) -> Result<(), Box<dyn Error>> {
        self.trampoline_maps
            .iter_mut()
            .map(|(map, heap)| {
                Self::create_trampolines_for_map(&self.stack_info, bin_interface, map, heap)
                    .unwrap();
                (map, heap)
            })
            .count();
        Ok(())
    }
    fn create_trampolines_for_map(
        stack_info: &TrampolineStackInfo,
        bin_interface: &mut BinaryInterface,
        map: &Map,
        heap: &mut TrampolineHeapInfo,
    ) -> Result<(), Box<dyn Error>> {
        let instructions =
            Self::read_instructions(bin_interface, map.base + 32, map.ceiling - (map.base + 32));
        let mut instruction_stack = Vec::new();
        let mut instruction_stack_code_size = 0;
        let mut added = 0;
        let mut not_added = 0;
        for instr in instructions {
            if instr.flow_control() == FlowControl::Next {
                if instr.code() == Code::Endbr64 {
                    // TODO We could still overwrite this
                    // if we are not careful
                    continue;
                }
                instruction_stack_code_size += instr.len();
                instruction_stack.push(instr);
            } else {
                if instr.is_invalid() {
                    continue;
                }
                if instr.len() >5 /* && !instr.is_ip_rel_memory_operand()*/ && !instr.is_invalid() {
                    Self::insert_trampoline(stack_info, bin_interface, instr, None, heap)?;
                    added += 1;
                } else {
                    not_added += 1;
                }
            }
        }
        dbg!(added);
        dbg!(not_added);
        Ok(())
    }
    /**
     * X \in (a,b]
     */
    fn range_contains_marked_addr(start: usize, end: usize, marked: &Vec<usize>) -> bool {
        for m in marked {
            if *m > start && *m <= end {
                return true;
            }
        }
        false
    }

    pub fn insert_trampoline(
        stack_info: &TrampolineStackInfo,
        bin_interface: &mut BinaryInterface,
        replaced_instruction: Instruction,
        flow_instruction: Option<&Instruction>,
        heap: &mut TrampolineHeapInfo,
    ) -> Result<(), Box<dyn Error>> {
        let heap_code_base_addr = heap.find_next_free_slot(40)? + heap.base_addr;
        let first_replaced_instruction_ip = replaced_instruction.ip();
        let mut ca = code_asm::CodeAssembler::new(64)?;
        ca.jmp(heap_code_base_addr as u64)?;
        let jump_bytes = ca.assemble(first_replaced_instruction_ip)?;
        let jump_bytes_len = jump_bytes.len();
        // record it in the trampolines
        // self.trampolines.insert(Interval {
        //     start: first_replaced_instruction_ip as usize,
        //     stop: first_replaced_instruction_ip as usize + jump_bytes_len,
        //     val: heap_code_base_addr,
        // });
        // dbg!(&jump_bytes);
        bin_interface.set_bytes(first_replaced_instruction_ip as usize, jump_bytes)?;

        let mut ca = code_asm::CodeAssembler::new(64)?;

        for i in 0..5 {
            if replaced_instruction.op_register(i) == Register::RIP {
                return Err(format!("RIP operand  {}", replaced_instruction))?;
            }
        }
        if replaced_instruction.len() < jump_bytes_len {
            return Err(format!(
                "Only given {} but needs {jump_bytes_len} bytes for the jump instruction from main",
                replaced_instruction.len()
            ))?;
        }
        for k in jump_bytes_len..replaced_instruction.len() {
            bin_interface
                .pin_mut()
                .set_byte(first_replaced_instruction_ip as usize + k, 0x00);
        }
        ca.add_instruction(replaced_instruction)?;
        let mut noop_to_replace = ca.create_label();
        let overflow_protection = false;
        if !overflow_protection {
            ca.xchg(code_asm::ptr(stack_info.base_addr), code_asm::rsp)?;
            ca.xchg(code_asm::ptr(stack_info.base_addr + 8), code_asm::rax)?;
            // RECORD DATA
            ca.mov(code_asm::rax, replaced_instruction.ip())?;
            ca.push(code_asm::rax)?;
            // CLEANUP
            ca.xchg(code_asm::ptr(stack_info.base_addr), code_asm::rsp)?;
            ca.xchg(code_asm::ptr(stack_info.base_addr + 8), code_asm::rax)?;
            ca.jmp(replaced_instruction.next_ip())?;
        } else {
            // SETUP
            ca.xchg(code_asm::ptr(stack_info.base_addr), code_asm::rsp)?;
            ca.xchg(code_asm::ptr(stack_info.base_addr + 8), code_asm::rax)?;
            // RECORD DATA
            ca.mov(code_asm::rax, replaced_instruction.ip())?;
            ca.push(code_asm::rax)?;
            // FLOW PROT
            ca.mov(code_asm::rax, 0xcccccccc_u64)?;
            ca.push(code_asm::rax)?;
            ca.pop(code_asm::rax)?;

            ca.mov(
                code_asm::al,
                code_asm::byte_ptr(stack_info.base_addr + stack_info.reserved_space),
            )?;
            ca.mov(code_asm::byte_ptr(noop_to_replace), code_asm::al)?;
            ca.set_label(&mut noop_to_replace)?;
            ca.nop()?;
            // CLEANUP
            ca.xchg(code_asm::ptr(stack_info.base_addr), code_asm::rsp)?;
            ca.xchg(code_asm::ptr(stack_info.base_addr + 8), code_asm::rax)?;
            ca.jmp(replaced_instruction.next_ip())?;
        }
        // ca.jmp(flow_instruction.ip())?;

        let heap_trampoline_bytes = ca.assemble(heap_code_base_addr as u64)?;
        heap.allocations.insert(Interval {
            start: heap_code_base_addr,
            stop: heap_code_base_addr + heap_trampoline_bytes.len(),
            val: TrampolineInfo {
                replaced_instructions: vec![replaced_instruction],
                trampoline_instructions: Vec::new(),
            },
        });
        heap.bytes_used += heap_trampoline_bytes.len();
        bin_interface.set_bytes(heap_code_base_addr, heap_trampoline_bytes)?;

        Ok(())
    }
}
//impl TrampolineMapCreator {
//    /**
//     * Identify all of the jumps where we know the
//     * address we are pointing to. Then when generating
//     * the trampolines, we can update these if we need
//     * to
//     */
//    pub fn identify_possible_vulnerable_jumps(
//        &self,
//        instructions: &Vec<Instruction>,
//    ) -> HashMap<usize, Instruction> {
//        let mut map = HashMap::new();
//        for instr in instructions {
//            match instr.flow_control() {
//                // Ignore nexts as we always replace the instructions
//                // at a previous good location
//                FlowControl::Next => {}
//                //
//                FlowControl::UnconditionalBranch => {
//                    if instr.is_jmp_short_or_near() {
//                        map.insert(instr.near_branch_target() as usize, instr.clone());
//                    } else {
//                        dbg!(instr);
//                        todo!();
//                    }
//                }
//                FlowControl::IndirectBranch => {
//                    // TODO
//                }
//                FlowControl::ConditionalBranch => {
//                    if instr.is_jcc_short_or_near() {
//                        map.insert(instr.near_branch_target() as usize, instr.clone());
//                    } else {
//                        dbg!(instr);
//                        todo!();
//                    }
//                }
//                // returns are fine because we wont overwrite the calls
//                // they are related to
//                FlowControl::Return => {}
//                // This probably isnt too much of an issue
//                // because the start of a call almost
//                // certainly wont be overwritten
//                FlowControl::Call => {
//                    if instr.is_call_near() {
//                        //TODO
//                        //map.insert(instr.next_ip() as usize, instr.clone());
//                    } else {
//                        dbg!(instr);
//                        //todo!();
//                    }
//                }
//                FlowControl::IndirectCall => {
//                    //TODO
//                }
//                // Justification todo
//                FlowControl::Interrupt => {}
//                FlowControl::XbeginXabortXend => {}
//                FlowControl::Exception => {}
//            }
//        }

//        map
//    }
//pub fn patch_jumps_into_trampoline(
//    &mut self,
//    bin_interface: &mut BinaryInterface,
//    possible_vuln_jumps: HashMap<usize, Instruction>,
//) {
//    for (jump_addr, instr) in possible_vuln_jumps {
//        // each jump had better only point into at most
//        // one trampoline
//        let mut trampoline_iter = self.trampolines.find(jump_addr, jump_addr);
//        if let Some(trampoline) = trampoline_iter.next() {
//            match instr.flow_control() {
//                // Ignore nexts as we always replace the instructions
//                // at a previous good location
//                FlowControl::Next => {}
//                //
//                FlowControl::UnconditionalBranch => {
//                    if instr.is_jmp_short_or_near() {
//                        let mut new_instr = instr.clone();
//                        match instr.op0_kind() {
//                            OpKind::NearBranch64 => {
//                                new_instr.set_near_branch64(trampoline.start as u64);
//                            }
//                            _ => {
//                                todo!()
//                            }
//                        }
//                        self.overwrite_single_instr(bin_interface, &instr, &new_instr)
//                            .unwrap();
//                    } else {
//                        dbg!(instr);
//                        todo!();
//                    }
//                }
//                FlowControl::IndirectBranch => {
//                    // TODO
//                }
//                FlowControl::ConditionalBranch => {
//                    if instr.is_jcc_short_or_near() {
//                        let mut new_instr = instr.clone();
//                        match instr.op0_kind() {
//                            OpKind::NearBranch64 => {
//                                new_instr.set_near_branch64(trampoline.start as u64);
//                            }
//                            _ => {
//                                todo!()
//                            }
//                        }
//                        self.overwrite_single_instr(bin_interface, &instr, &new_instr)
//                            .unwrap();
//                    } else {
//                        dbg!(instr);
//                        todo!();
//                    }
//                }
//                // returns are fine because we wont overwrite the calls
//                // they are related to
//                FlowControl::Return => {}
//                // This probably isnt too much of an issue
//                // because the start of a call almost
//                // certainly wont be overwritten
//                FlowControl::Call => {
//                    todo!()
//                }
//                FlowControl::IndirectCall => {
//                    todo!()
//                }
//                // Justification todo
//                FlowControl::Interrupt => {}
//                FlowControl::XbeginXabortXend => {}
//                FlowControl::Exception => {}
//            }
//        }
//    }
//}
