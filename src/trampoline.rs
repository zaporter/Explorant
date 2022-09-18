use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, Register, RflagsBits,
};
use librr_rs::*;
use procmaps::Map;
use rust_lapper::{Interval, Lapper};
use std::{collections::HashMap, error::Error};

use iced_x86::code_asm;

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
pub struct TrampolineStackInfo {
    pub base_addr: usize,
    pub size: usize,
    pub reserved_space: usize,
}
impl TrampolineStackInfo {
    pub fn allocate_map(&self, binary_interface:&mut BinaryInterface){
        binary_interface.pin_mut().mmap_stack(self.base_addr, self.size);
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
pub struct TrampolineHeapInfo {
    pub base_addr: usize,
    pub size: usize,
}
impl TrampolineHeapInfo {
    pub fn allocate_map(&self, binary_interface:&mut BinaryInterface){
        binary_interface.pin_mut().mmap_heap(self.base_addr, self.size);
    }
}
#[derive(Clone, Eq, PartialEq)]
pub struct TrampolineInfo {
    pub replaced_instructions: Vec<Instruction>,
    pub trampoline_instructions: Vec<Instruction>,
}
pub struct TrampolineManager {
    pub stack_info: TrampolineStackInfo,
    pub heap_info: TrampolineHeapInfo,
    // [] space inside my heap
    pub allocations: Lapper<usize, TrampolineInfo>,
    // trampoline -> usize inside of allocation
    pub trampolines: Lapper<usize, usize>,
    pub unwatched_instructions: Vec<Instruction>,
    pub recorded_addresses: Vec<usize>,
    pub space_allocated: usize,
}
impl Default for TrampolineManager {
    fn default() -> Self {
        TrampolineManager::new(
            TrampolineStackInfo {
                base_addr: 0x71000000,
                size: 0x100000,
                reserved_space: 0x40,
            },
            TrampolineHeapInfo {
                base_addr:0x73000000,//0x555554553000,// 0x73000000,
                // base_addr: 0x555554553000, // 0x73000000,
                size: 0x100000,
            },
        )
    }
}
impl TrampolineManager {
    pub fn new(stack_info: TrampolineStackInfo, heap_info: TrampolineHeapInfo) -> Self {
        TrampolineManager {
            stack_info,
            heap_info,
            allocations: Lapper::new(vec![]),
            trampolines: Lapper::new(vec![]),
            unwatched_instructions: Vec::new(),
            recorded_addresses: Vec::new(),
            space_allocated: 0,
        }
    }
    pub fn new_for(bin_interface: &mut BinaryInterface, stack_info: TrampolineStackInfo, target: &Map, maps: &Lapper<usize,Map>) -> Self{
        let heap_possible_bottom = target.ceiling - 2_usize.pow(31);
        let heap_possible_top = target.base + 2_usize.pow(31);
        let heap_size = 0x100000;
        let heap_base = maps.find_free_interval(heap_possible_bottom, heap_possible_top, heap_size).unwrap();
        let heap_info = TrampolineHeapInfo { base_addr: heap_base, size: heap_size };
        heap_info.allocate_map(bin_interface);
        TrampolineManager {
            stack_info,
            heap_info,
            allocations: Lapper::new(vec![]),
            trampolines: Lapper::new(vec![]),
            unwatched_instructions: Vec::new(),
            recorded_addresses: Vec::new(),
            space_allocated: 0,
        }

    }
    /**
     * Identify all of the jumps where we know the
     * address we are pointing to. Then when generating
     * the trampolines, we can update these if we need
     * to
     */
    pub fn identify_possible_vulnerable_jumps(
        &self,
        instructions: &Vec<Instruction>,
    ) -> HashMap<usize, Instruction> {
        let mut map = HashMap::new();
        for instr in instructions {
            match instr.flow_control() {
                // Ignore nexts as we always replace the instructions
                // at a previous good location
                FlowControl::Next => {}
                //
                FlowControl::UnconditionalBranch => {
                    if instr.is_jmp_short_or_near() {
                        map.insert(instr.near_branch_target() as usize, instr.clone());
                    } else {
                        dbg!(instr);
                        todo!();
                    }
                }
                FlowControl::IndirectBranch => {
                    // TODO
                }
                FlowControl::ConditionalBranch => {
                    if instr.is_jcc_short_or_near() {
                        map.insert(instr.near_branch_target() as usize, instr.clone());
                    } else {
                        dbg!(instr);
                        todo!();
                    }
                }
                // returns are fine because we wont overwrite the calls
                // they are related to
                FlowControl::Return => {}
                // This probably isnt too much of an issue
                // because the start of a call almost
                // certainly wont be overwritten
                FlowControl::Call => {
                    if instr.is_call_near() {
                        //TODO
                        //map.insert(instr.next_ip() as usize, instr.clone());
                    } else {
                        dbg!(instr);
                        //todo!();
                    }
                }
                FlowControl::IndirectCall => {
                    //TODO
                }
                // Justification todo
                FlowControl::Interrupt => {}
                FlowControl::XbeginXabortXend => {}
                FlowControl::Exception => {}
            }
        }

        map
    }
    fn find_next_free_slot(&self, size: usize) -> Result<usize, Box<dyn Error>> {
        if self.space_allocated + size > self.heap_info.size {
            return Err("Ran out of heap space for trampolines")?;
        }
        Ok(self.space_allocated)
    }
    pub fn setup_stack_ptr(
        &mut self,
        bin_interface: &mut BinaryInterface,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = (self.stack_info.base_addr + self.stack_info.size)
            .to_le_bytes()
            .to_vec();
        bin_interface.set_bytes(self.stack_info.base_addr, bytes)?;
        Ok(())
    }
    pub fn recorded_addresses(&self) -> &Vec<usize> {
        &self.recorded_addresses
    }
    pub fn clear_address_stack(
        &mut self,
        bin_interface: &mut BinaryInterface,
    ) -> Result<(), Box<dyn Error>> {
        let stack_end_addr = self.stack_info.base_addr + self.stack_info.size;
        let rsp_bytes = bin_interface.get_mem(self.stack_info.base_addr, 8);
        let rsp = usize::from_le_bytes(rsp_bytes.try_into().unwrap());
        let num_entries: usize = (stack_end_addr - rsp) / 8;
        for entry in 1..=num_entries {
            let entry_bytes = bin_interface.get_mem(stack_end_addr - entry * 8, 8);
            self.recorded_addresses
                .push(usize::from_le_bytes(entry_bytes.try_into().unwrap()));
        }
        self.setup_stack_ptr(bin_interface)?;
        Ok(())
    }
    pub fn patch_jumps_into_trampoline(
        &mut self,
        bin_interface: &mut BinaryInterface,
        possible_vuln_jumps: HashMap<usize, Instruction>,
    ) {
        for (jump_addr, instr) in possible_vuln_jumps {
            // each jump had better only point into at most
            // one trampoline
            let mut trampoline_iter = self.trampolines.find(jump_addr, jump_addr);
            if let Some(trampoline) = trampoline_iter.next() {
                match instr.flow_control() {
                    // Ignore nexts as we always replace the instructions
                    // at a previous good location
                    FlowControl::Next => {}
                    //
                    FlowControl::UnconditionalBranch => {
                        if instr.is_jmp_short_or_near() {
                            let mut new_instr = instr.clone();
                            match instr.op0_kind() {
                                OpKind::NearBranch64 => {
                                    new_instr.set_near_branch64(trampoline.start as u64);
                                }
                                _ => {
                                    todo!()
                                }
                            }
                            self.overwrite_single_instr(bin_interface, &instr, &new_instr)
                                .unwrap();
                        } else {
                            dbg!(instr);
                            todo!();
                        }
                    }
                    FlowControl::IndirectBranch => {
                        // TODO
                    }
                    FlowControl::ConditionalBranch => {
                        if instr.is_jcc_short_or_near() {
                            let mut new_instr = instr.clone();
                            match instr.op0_kind() {
                                OpKind::NearBranch64 => {
                                    new_instr.set_near_branch64(trampoline.start as u64);
                                }
                                _ => {
                                    todo!()
                                }
                            }
                            self.overwrite_single_instr(bin_interface, &instr, &new_instr)
                                .unwrap();
                        } else {
                            dbg!(instr);
                            todo!();
                        }
                    }
                    // returns are fine because we wont overwrite the calls
                    // they are related to
                    FlowControl::Return => {}
                    // This probably isnt too much of an issue
                    // because the start of a call almost
                    // certainly wont be overwritten
                    FlowControl::Call => {
                        todo!()
                    }
                    FlowControl::IndirectCall => {
                        todo!()
                    }
                    // Justification todo
                    FlowControl::Interrupt => {}
                    FlowControl::XbeginXabortXend => {}
                    FlowControl::Exception => {}
                }
            }
        }
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
    pub fn create_trampolines(
        &mut self,
        bin_interface: &mut BinaryInterface,
        instructions: &Vec<Instruction>,
        dangerous_addresses: Vec<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let mut instruction_stack = Vec::new();
        let mut instruction_stack_code_size = 0;
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
                // We need to create a trampoline

                // We need at least 9 bytes to do the jmp
                // 1 for the jmp, 8 for the address
                // if instruction_stack_code_size >= 5 {
                if instruction_stack.len()==0 {
                        self.unwatched_instructions.push(*instr);
                        instruction_stack_code_size = 0;
                        instruction_stack.clear();
                    continue;
                }
                if instruction_stack.last().unwrap().is_ip_rel_memory_operand(){
                        self.unwatched_instructions.push(*instr);
                        instruction_stack_code_size = 0;
                        instruction_stack.clear();
                    continue;
                }
                if instruction_stack.last().unwrap().len() >= 5 {
                    let mut instrs_to_replace = Vec::new();
                    let mut grabbed_size = 0;
                    while grabbed_size < 5 {
                        let instr_to_replace = instruction_stack.pop().unwrap();
                        grabbed_size += instr_to_replace.len();
                        instrs_to_replace.push(instr_to_replace.clone());
                    }
                    // reverse it to get back to the normal memory ordering
                    instrs_to_replace.reverse();
                    if Self::range_contains_marked_addr(
                        instrs_to_replace.first().unwrap().ip() as usize,
                        instrs_to_replace.last().unwrap().ip() as usize,
                        &dangerous_addresses,
                    ) {
                        self.unwatched_instructions.push(*instr);
                        instruction_stack_code_size = 0;
                        instruction_stack.clear();
                        continue;
                    }
                    self.create_flow_reading_trampoline(bin_interface, instrs_to_replace, instr)?;
                } else {
                    self.unwatched_instructions.push(*instr);
                }
                // don't use instructions from previous jumps.
                // This is a possible future optimization for
                // jumps that ignore RIP but it is dangerous
                instruction_stack_code_size = 0;
                instruction_stack.clear();
            }
        }
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

    pub fn create_flow_reading_trampoline(
        &mut self,
        bin_interface: &mut BinaryInterface,
        replaced_instructions: Vec<Instruction>,
        flow_instruction: &Instruction,
    ) -> Result<(), Box<dyn Error>> {
        let heap_code_base_addr = self.find_next_free_slot(40)? + self.heap_info.base_addr;
        let first_replaced_instruction_ip = replaced_instructions
            .get(0)
            .ok_or("Must pass Instructions to create_flow_reading_trampoline")?
            .ip();
        let mut ca = code_asm::CodeAssembler::new(64)?;
        ca.jmp(heap_code_base_addr as u64)?;
        let jump_bytes = ca.assemble(first_replaced_instruction_ip)?;
        let jump_bytes_len = jump_bytes.len();
        // record it in the trampolines
        self.trampolines.insert(Interval {
            start: first_replaced_instruction_ip as usize,
            stop: first_replaced_instruction_ip as usize + jump_bytes_len,
            val: heap_code_base_addr,
        });
        // dbg!(&jump_bytes);
        bin_interface.set_bytes(first_replaced_instruction_ip as usize, jump_bytes)?;

        let mut ca = code_asm::CodeAssembler::new(64)?;

        let mut space_for_replaced_instructions = 0;
        for instr in &replaced_instructions {
            space_for_replaced_instructions += instr.len();
            // if instr.memory_base() == Register::RIP {
            //     if instr.code() == Code::Mov_r64_rm64 {
            //         continue;
            //     }
            //     if instr.code() == Code::Lea_r64_m{
            //         continue;
            //     }
            //     if instr.code() == Code::Cmp_rm8_imm8{
            //         continue;
            //     }
            //     if instr.code() == Code::Cmp_rm64_imm8{
            //         continue;
            //     }
            //     if instr.code() == Code::Movaps_xmm_xmmm128{
            //         continue;
            //     }
            //     return Err(format!("RIP relative memory base {} {:?}",instr,instr))?;
            // };
            for i in 0..5 {
                if instr.op_register(i) == Register::RIP {
                    return Err(format!("RIP operand  {}", instr))?;
                }
            }
            ca.add_instruction(*instr)?;
        }
        if space_for_replaced_instructions < jump_bytes_len {
            return Err(format!("Only given {space_for_replaced_instructions} but needs {jump_bytes_len} bytes for the jump instruction from main"))?;
        }
        for k in jump_bytes_len..space_for_replaced_instructions {
            bin_interface
                .pin_mut()
                .set_byte(first_replaced_instruction_ip as usize + k, 0xcc);
        }

        ca.xchg(code_asm::ptr(self.stack_info.base_addr), code_asm::rsp)?;
        ca.xchg(code_asm::ptr(self.stack_info.base_addr+8), code_asm::rax)?;
        ca.mov(code_asm::rax, flow_instruction.ip())?;
        ca.push(code_asm::rax)?;
        ca.xchg(code_asm::ptr(self.stack_info.base_addr), code_asm::rsp)?;
        ca.xchg(code_asm::ptr(self.stack_info.base_addr+8), code_asm::rax)?;
        ca.jmp(replaced_instructions.last().unwrap().next_ip())?;

        let heap_trampoline_bytes = ca.assemble(heap_code_base_addr as u64)?;
        self.allocations.insert(Interval {
            start: heap_code_base_addr,
            stop: heap_code_base_addr + heap_trampoline_bytes.len(),
            val: TrampolineInfo {
                replaced_instructions,
                trampoline_instructions: Vec::new(),
            },
        });
        self.space_allocated += heap_trampoline_bytes.len();
        bin_interface.set_bytes(heap_code_base_addr, heap_trampoline_bytes)?;

        Ok(())
    }
}
