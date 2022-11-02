use std::{path::PathBuf, sync::Mutex, error::Error};

use librr_rs::{BinaryInterface,GdbRegister};
use procmaps::Map;
use rust_lapper::{Lapper, Interval};
use symbolic_common::{Language, Name};
use symbolic_demangle::{Demangle, DemangleOptions};
use object::{
    Object, ObjectSection, ObjectSymbol, ObjectSymbolTable, Section, SectionKind, Segment
};

use crate::{trampoline::{TrampolineManager, TrampolineStackInfo}, shared_structs::FrameTimeMap, erebor::Erebor};

pub struct Simulation{
    pub bin_interface : Mutex<BinaryInterface>,
    pub trampoline_manager : Mutex<TrampolineManager>,
    pub proc_map: Mutex<Lapper<usize,Map>>,
    pub frame_time_map : Mutex<FrameTimeMap>,
    // pub symbol_table: Mutex<Vec<(String, object::Symbol<'static,'static>)>>,
    pub last_rip: Mutex<usize>,
    pub save_directory : PathBuf,
    pub dwarf_data : Erebor,
}
// SAFETY: const *cxx:void is not send and sync
// because if a thread context switches while running
// a method in the c++ code then we have undefined behavior. 
//
// However, because bin_interface is behind a mutex, we should 
// be able to ensure that a context switch will not start running
// other c++ code. 
unsafe impl Send for Simulation {}
unsafe impl Sync for Simulation {}

fn get_symbols<'a>(file : &'a object::File) -> Result<Vec<(String,object::Symbol<'a,'a>)>, Box<dyn Error>> {
    let mut to_ret = Vec::new();
    for symbol in file.symbol_table().ok_or("No symboltable found")?.symbols() {
        let name : String = Name::from(symbol.name().unwrap()).try_demangle(DemangleOptions::name_only()).to_string();
        to_ret.push((name, symbol));
    }
    Ok(to_ret)
}

impl Simulation {
    pub fn new(directory:PathBuf) -> anyhow::Result<Self> {
        let mut bin_interface = BinaryInterface::new_at_target_event(0, directory.clone());
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
            //  This is consistently faster on my machine if 
            //  it is given 1GiB instead of 256MiB. 
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
        // Read symbol file and parse symbols 
        let symbol_file = bin_interface.get_exec_file();
        dbg!(&symbol_file);

        let symbol_str = std::fs::read(symbol_file).unwrap();
        let obj_file = object::File::parse(&*symbol_str).unwrap();
        // 
        let dwarf_data = Erebor::new(obj_file);

        let frame_time_map : FrameTimeMap= {
            let file = std::fs::File::open(directory.join("frame_time_map.json"))?;
            let reader = std::io::BufReader::new(file);

            serde_json::from_reader(reader)?
        };

        let trampoline_manager = TrampolineManager::new(&mut bin_interface, stack_info, &proc_map);

        Ok(Self{
            bin_interface: Mutex::new(bin_interface),
            trampoline_manager: Mutex::new(trampoline_manager),
            proc_map: Mutex::new(proc_map),
            frame_time_map: Mutex::new(frame_time_map),
            last_rip: Mutex::new(rip),
            save_directory: directory,
            dwarf_data,
            // symbol_table:Mutex::new(symbols),
        })

    }
}
