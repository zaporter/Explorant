
use object::{
    Object, ObjectSection, ObjectSymbol, ObjectSymbolTable, Section, SectionKind, Segment,
};
use std::str::FromStr;
use std::error::Error;
use std::path::PathBuf;
use std::pin::Pin;
use symbolic_common::{Language, Name};
use symbolic_demangle::{Demangle, DemangleOptions};
use librr_rs::*;


fn get_symbols<'a>(file : &'a object::File) -> Result<Vec<(String,object::Symbol<'a,'a>)>, Box<dyn Error>> {
    let mut to_ret = Vec::new();
    for symbol in file.symbol_table().ok_or("No symboltable found")?.symbols() {
        let name : String = Name::from(symbol.name().unwrap()).try_demangle(DemangleOptions::name_only()).to_string();
        to_ret.push((name, symbol));
    }
    Ok(to_ret)
}
enum Direction{
    Forward,
    Backward,
}
fn goto(bin_interface : &mut BinaryInterface, addr: usize, max_iter : usize, direction : Direction)->Result<(), Box<dyn Error>> {
    let cont = GdbContAction {
        type_: GdbActionType::ACTION_CONTINUE,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };
    let mut iter_count = 0;
    while {
        assert!(bin_interface.pin_mut().set_sw_breakpoint(addr, 1));
        match direction {
            Direction::Forward  => bin_interface.pin_mut().continue_forward(cont),
            Direction::Backward => bin_interface.pin_mut().continue_backward(cont),
        };
        assert!(bin_interface.pin_mut().remove_sw_breakpoint(addr,1));
        let rip = bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread());
        rip.get_value_u128() != addr as u128 
    } {
        iter_count += 1;
        if iter_count > max_iter {
            Err(format!("Exceeded maximum iterations going to addr {}", addr))?;
        }
    }
    Ok(())

}
fn main() {
    let sample_dateviewer_dir =
        PathBuf::from_str("/home/zack/.local/share/rr/date_viewer-94").unwrap();
    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir);

    let mut found_mapping = false;
    let current_thread = bin_interface.get_current_thread();
    let mappings = bin_interface.get_proc_map().unwrap();
    let rip = bin_interface
        .get_register(GdbRegister::DREG_RIP, current_thread.clone())
        .get_value_u128() as usize;
    for mapping in mappings.iter() {
        if rip < mapping.ceiling && rip > mapping.base {
            assert!(mapping.perms.readable);
            assert!(!mapping.perms.writable);
            assert!(mapping.perms.executable);
            // should be ld-linux-x86-64.so.2
            dbg!(rip);
            dbg!(mapping);
            found_mapping = true;
        }
    }
    assert!(found_mapping);
    // let thread_locals_map = mappings.iter().find(|k| 
    //     match &k.pathname {
    //         procmaps::Path::MappedFile(path) => path.contains("thread_locals"),
    //         _ => false
    //     }).unwrap();
    // dbg!(thread_locals_map);
    // let thread_locals_string = if let procmaps::Path::MappedFile(k) = &thread_locals_map.pathname {k} else {panic!("thread locals is not a mapped file!")};
    // {
    //     let symbol_str = std::fs::read(thread_locals_string).unwrap();
    //     let obj_file = object::File::parse(&*symbol_str).unwrap();
    // }

    // Identify the proc map entry for the binary
    let base_exe = mappings.iter().find(|k|
        match &k.pathname {
            procmaps::Path::MappedFile(path) => path.contains("date"),
            _ => false
        }).unwrap().base;
    
    // Read symbol file and parse symbols 
    let symbol_file = bin_interface.get_exec_file();
    dbg!(&symbol_file);

    let symbol_str = std::fs::read(symbol_file).unwrap();
    let obj_file = object::File::parse(&*symbol_str).unwrap();
    let symbols = get_symbols(&obj_file).unwrap();
    // identify address of date_viewer::main
    let main_addr = symbols.into_iter().find(|k| k.0 == "date_viewer::main").unwrap().1.address() as usize;
    let main_addr = main_addr + base_exe;
    dbg!(main_addr);

    // Set the query and continue threads
    let cthread = bin_interface.get_current_thread();
    dbg!(bin_interface.pin_mut().set_continue_thread(cthread));
    let cthread = bin_interface.get_current_thread();
    dbg!(bin_interface.pin_mut().set_query_thread(cthread));


    bin_interface.set_pass_signals(vec![
        0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
    ]);

    dbg!(bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread()));
    
    // _dl_debug_state and main
    // let valid_breaks : Vec<u128>= vec![0x55b8b3fed060, 0x7fe727f1c090];

    // goto main
    goto(&mut bin_interface, main_addr, 10000, Direction::Forward).unwrap();
    println!("At date_viewer::main");
    dbg!(bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread()));

    // step till the end of the program
    let step = GdbContAction {
        type_: GdbActionType::ACTION_STEP,
        target: bin_interface.get_current_thread(),
        signal_to_deliver: 0,
    };
    println!("Stepping 30000 times");
    for _ in 0..30000 {
        bin_interface.pin_mut().continue_forward(step);
    }
    dbg!(bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread()));
    println!("Going back to the start of main");
    goto(&mut bin_interface, main_addr, 10000, Direction::Backward).unwrap();
    dbg!(bin_interface.get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread()));
    println!("Stepping 10000 times");
    for _ in 0..10000 {
        bin_interface.pin_mut().continue_forward(step);
    }

}
