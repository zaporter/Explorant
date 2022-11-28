#![allow(unused_imports)]
#![allow(unused)]
#![allow(non_snake_case)]

use clap::{Parser, Subcommand};
use druid_graphviz_layout::adt::dag::NodeHandle;
use erebor::Erebor;
use graph_builder::GraphBuilder;
use iced_x86::{
    Code, ConditionCode, Decoder, DecoderOptions, FlowControl, Instruction, InstructionInfoFactory,
    OpKind, Register, RflagsBits,
};
use itertools::{Itertools, Zip};
use librr_rs::*;
use procmaps::{Map, Mappings};
use rust_lapper::{Interval, Lapper};
use shared_structs::*;
use similar::{capture_diff_slices, Algorithm};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::SystemTime;
use std::{error::Error, sync::Arc};

use crate::block::{Block, BlockEvaluation, CodeFlow};
use crate::simulation::Simulation;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::{
    get, middleware, web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

mod block;
mod graph_builder;
mod address_recorder;
mod file_parsing;
// mod code_flow_graph;
// mod graph_layout;
// mod gui;
mod lcs;
mod query;
mod recorder;
mod shared_structs;
mod simulation;
mod trampoline;
mod mvp;
mod erebor;
use crate::lcs::*;
use crate::query::*;
use crate::trampoline::*;

#[derive(Parser)]
#[command(author,version,about,long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Record {
        #[arg(short, long, value_name = "FILE")]
        exe: PathBuf,
        #[arg(short, long, value_name = "FOLDER")]
        save_dir: PathBuf,
    },
    Serve {
        //#[arg(short, long, value_name = "FOLDER1 FOLDER2 ...")]
        traces: Vec<PathBuf>,
    },
    Mvp {
        #[arg(short, long, value_name = "FOLDER")]
        save_dir: PathBuf,
    },
}

// ASSUMPTIONS
// All code is run from the same binary 
// with ASLR turned off.
// Anything else is undefined behavior and 
// will fail silenty. 
struct SimulationStorage {
    traces: Vec<Simulation>,
    //dwarf_data: Mutex<Erebor>,
    //graph_builder: Mutex<GraphBuilder>,

}
async fn ping(req: web::Json<PingRequest>) -> HttpResponse {
    let req = req.0;
    HttpResponse::Ok().json(PingResponse { id: req.id })
}
async fn get_instruction_pointer(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<InstructionPointerRequest>,
) -> HttpResponse {
    let ip = data.get_ref().traces[req.trace_id].last_rip.lock();
    match ip {
        Ok(instruction_pointer) => HttpResponse::Ok().json(InstructionPointerResponse {
            instruction_pointer: *instruction_pointer,
        }),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
async fn get_recorded_frames(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<RecordedFramesRequest>,
) -> HttpResponse {
    let mut frame_time_map = match data.get_ref().traces[req.trace_id].frame_time_map.lock() {
        Ok(k) => k,
        Err(k) => return HttpResponse::InternalServerError().body(k.to_string()),
    };
    let save_dir = data.as_ref().traces[req.trace_id].save_directory.clone();
    let to_load: Vec<String> = frame_time_map
        .frames
        .iter()
        .map(|(_, _, frame_name)| frame_name.clone())
        .collect();
    let mut response = RecordedFramesResponse {
        frames: HashMap::new(),
    };
    for frame_name in to_load {
        let dir = save_dir.join(frame_name.clone());
        response
            .frames
            .insert(frame_name, Vec::new());
    }
    HttpResponse::Ok().json(response)
}
async fn get_general_info(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<EmptyRequest>,
) -> HttpResponse {
    let mut traces : Vec<TraceGeneralInfo> = Vec::new();
    let mut binary_name : Option<String> = None;
    for (id,simulation) in data.as_ref().traces.iter().enumerate() {
        let mut binary_interface = match simulation.bin_interface.lock() {
            Ok(k) => k,
            Err(k) => return HttpResponse::InternalServerError().body(k.to_string()),
        };
        if binary_name.is_none() {
            binary_name = Some(binary_interface.get_exec_file().into());
        } // TODO ensure binary name is the same across all traces
        let mut frame_time_map = match simulation.frame_time_map.lock() {
            Ok(k) => k,
            Err(k) => return HttpResponse::InternalServerError().body(k.to_string()),
        };
        traces.push(TraceGeneralInfo { id, frame_time_map: frame_time_map.clone(), proc_maps: binary_interface.get_proc_map().unwrap().to_vec() });
    }
    let data = GeneralInfoResponse {
        binary_name : binary_name.unwrap(),
        traces,
    };
    HttpResponse::Ok().json(data)
}
async fn get_current_graph(
    data: web::Data<Arc<SimulationStorage>>,
    packet_version : web::Data<Arc<Mutex<usize>>>,
    _req: web::Json<CurrentGraphRequest>,
) -> HttpResponse {
    // let mut packet_version = packet_version.get_ref().lock().unwrap();
    // *packet_version+=1;

    let mut graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let dot_data = graph_builder.get_graph_as_dot().unwrap();
    dbg!(&dot_data);
    dbg!(&data.get_ref().traces.len());
    let response : CurrentGraphResponse = CurrentGraphResponse {
        version : 0,
        dot: dot_data.unwrap()
    };
    HttpResponse::Ok().json(response)
}
async fn get_node_data(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<NodeDataRequest>,
) -> HttpResponse {

    let graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let resp = NodeDataResponse{
        modules: graph_builder.modules.clone(),
        nodes: graph_builder.synoptic_nodes.clone(),
    };
    HttpResponse::Ok().json(resp)
}
async fn get_source_file(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<SourceFileRequest>,
) -> HttpResponse {
    let req = req.0;
    // TODO Checks here 
    let contents = std::fs::read_to_string(req.file_name);
    let resp = SourceFileResponse {
        data:contents.unwrap_or("[empty]".into()),
    };
    HttpResponse::Ok().json(resp)

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    librr_rs::raise_resource_limits();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Record { exe, save_dir } => {
            recorder::record(exe, save_dir, None);
            Ok(())
        },
        Commands::Serve { traces } => {
            return run_server(traces.clone()).await;
        },
        Commands::Mvp {save_dir} => {
            mvp::run(save_dir);
            Ok(())
        },
    }
}
async fn run_server(traces: Vec<PathBuf>) -> std::io::Result<()> {
    if traces.len() == 0 {
        log::error!("You must pass at least one trace");
        // TODO: Anyhow this with proper msg
        return Ok(());
    }
    let traces = traces.iter().map(|t| Simulation::new(t.clone()).unwrap()).collect();
    let simulation: Arc<SimulationStorage> = Arc::new(SimulationStorage { traces });
    let packet_version : Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let port = 8080;
    let ip = "127.0.0.1";
    log::info!("Starting HTTP server at {}:{}", &ip, port);

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(simulation.clone()))
            .app_data(web::Data::new(packet_version.clone()))
            .app_data(web::JsonConfig::default().limit(40000096)) 
            .service(web::resource("/ping").route(web::post().to(ping)))
            .service(
                web::resource("/instruction_pointer")
                    .route(web::post().to(get_instruction_pointer)),
            )
            .service(web::resource("/general_info").route(web::post().to(get_general_info)))
            .service(web::resource("/recorded_frames").route(web::post().to(get_recorded_frames)))
            .service(web::resource("/current_graph").route(web::post().to(get_current_graph)))
            .service(web::resource("/node_data").route(web::post().to(get_node_data)))
            .service(web::resource("/source_file").route(web::post().to(get_source_file)))

        // .service(web::resource("/createsheet").route(web::post().to(create_sheet)))
        // .service(web::resource("/getsheet").route(web::post().to(get_sheet)))
        // .service(web::resource("/updatesheet").route(web::post().to(update_sheet)))
        // .service(web::resource("/forksheet").route(web::post().to(fork_sheet)))
    })
    .bind((ip, port))?
    .run()
    .await
}
// fn main() {
//     librr_rs::raise_resource_limits();
//     gui::start_query_editor();
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

// }
fn get_addrs(sample_dateviewer_dir: PathBuf) -> Vec<usize> {
    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir.clone());

    let cthread = bin_interface.get_current_thread();
    bin_interface.pin_mut().set_query_thread(cthread);
    bin_interface.set_pass_signals(vec![
        0, 0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
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
        signal = bin_interface
            .pin_mut()
            .continue_forward_jog_undefined(step)
            .unwrap();
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
