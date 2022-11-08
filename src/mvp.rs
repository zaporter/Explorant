use std::path::PathBuf;

use std::io::prelude::*;
use crate::simulation::Simulation;
use librr_rs::*;

pub fn run(save_dir: &PathBuf) {
    println!("Running MVP on {:?}", save_dir);
    let mut simulation: Simulation = Simulation::new(save_dir.clone()).unwrap();
    let mut addresses = Vec::new();
    let bin_interface = simulation.bin_interface.get_mut().unwrap();
    let erebor = simulation.dwarf_data.lock().unwrap();

    let step = GdbContAction {
        type_: GdbActionType::ACTION_STEP,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };

    // bin_interface.pin_mut().set_sw_breakpoint(main_addr,1);
    // bin_interface.pin_mut().continue_forward(cont);
    // bin_interface.pin_mut().remove_sw_breakpoint(main_addr,1);
    let mut signal = 5;

    'outer: while signal == 5 {
        let rip = bin_interface
            .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
            .to_usize();
        addresses.push(rip);
        signal = bin_interface.pin_mut().continue_forward(step).unwrap();
    }
    println!("Erebor lines size: {}", erebor.lines.len());
    println!("Done stepping num_addrs: {}", addresses.len());
    dbg!(simulation.proc_map);
    let mut locations = Vec::new();
    let mut addr_locs = Vec::new();
    let mut uniq_files = Vec::new();
    let mut past_main = false;
    for address in addresses {
        // TODO THIS NEEDS TO BE ASLR CORRECTED 
        // WITH THE PROC MAP SOMEHOW
        let loc  = erebor.lines.get(&address);
        if let Some(loc) = loc {
            if !past_main {
                let file_info = erebor.files.get(&loc.file);
                if let Some(file_info) = file_info {
                    for func in &file_info.functions {
                        // TODO : add checks here to ensure addr is in range of func
                        if func.demangled_name == "main" {
                            past_main= true;
                        }
                    }
                    if !past_main {
                        continue;
                    }
                }
            }
            if !uniq_files.contains(&loc.file){
                uniq_files.push(loc.file.clone());
            }
            locations.push(loc);
            addr_locs.push(address);
        }
    }
    dbg!(locations.len());
    dbg!(uniq_files);
    let malloc_path = PathBuf::from("/home/zack/Tools/MQP/glibc2/malloc/malloc.c");
    // let malloc_src = std::fs::read_to_string(malloc_path).expect("unable to read malloc.c");
    let file = std::fs::File::open(malloc_path.clone()).unwrap();
    let reader = std::io::BufReader::new(file);

    let mut malloc_lines = Vec::new();
    for line in reader.lines() {
        malloc_lines.push(line.unwrap());
    }
    dbg!(malloc_lines.len());
    let mut finished_malloc_lines = Vec::new();
    let malloc_file_info = erebor.files.get(&malloc_path).unwrap();
    'outer: for line_num in 0..malloc_lines.len()-1 {
        if malloc_file_info.lines.contains_key(&(line_num as u32 +1)){
            //
            let possible_addrs = malloc_file_info.lines.get(&(line_num as u32 +1)).unwrap();
            'inner: for p_addr in possible_addrs {
                if addr_locs.contains(p_addr) {
                    dbg!(p_addr);
                    finished_malloc_lines.push(format!("* {}",&malloc_lines[line_num].chars().skip(1).collect::<String>()));
                    continue 'outer;
                }
            }
            finished_malloc_lines.push(malloc_lines[line_num].clone());
        } else {
            finished_malloc_lines.push(malloc_lines[line_num].clone());
        }
    }
    dbg!(finished_malloc_lines.len());
    // dbg!(finished_malloc_lines);
    std::fs::write("malloc.c", finished_malloc_lines.join("\n")).expect("");
}
