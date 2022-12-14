#![allow(unused_imports)]
#![allow(unused)]
#![allow(non_snake_case)]

extern crate actix_files;
extern crate actix_web;

use clap::{Parser, Subcommand};
use librr_rs::*;
use shared_structs::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Arc;

use crate::simulation::Simulation;

use actix_cors::Cors;
use actix_web::{
     middleware, web, App, HttpResponse, HttpServer,
};

mod address_recorder;
mod block;
mod file_parsing;
mod graph_builder;
mod erebor;
mod gdb_instance_manager;
mod recorder;
mod shared_structs;
mod simulation;
mod trampoline;

#[derive(Parser)]
#[command(author,version,about,long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Record the execution of a program for replay later
    Record {
        /// Path to the executable
        #[arg(short, long, value_name = "FILE")]
        exe: PathBuf,

        /// Directory to save the trace
        #[arg(short, long, value_name = "FOLDER")]
        save_dir: PathBuf,

        /// Should record the screen with ffmpeg and store screenshots
        #[arg(
            short,
            long,
            default_value = "false",
            value_name = "SHOULD RECORD WITH FFMPEG"
        )]
        record_screen: bool,
    },
    /// Examine a recorded trace
    Explore {
        /// Path to the save-dir of the last progrm
        trace: PathBuf,
        /// By default, this uses the procmap to fix the address offsets.
        /// Enabiling this option disables that. If you are writing your own asm
        /// or compiling glibc, you will want to enable this.
        #[arg(
            long,
            default_value = "false",
            value_name = "USE PROCMAP TO FIX ADDR OFFSETS"
        )]
        no_glibc_offsets: bool,
    },
}

// ASSUMPTIONS
// All code is run from the same binary
// with ASLR turned off.
// Anything else is undefined behavior and
// will fail silenty.
struct SimulationStorage {
    traces: Vec<Simulation>,
    settings: Mutex<Settings>,
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
        response.frames.insert(frame_name, Vec::new());
    }
    HttpResponse::Ok().json(response)
}
async fn get_general_info(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<EmptyRequest>,
) -> HttpResponse {
    let mut traces: Vec<TraceGeneralInfo> = Vec::new();
    let mut binary_name: Option<String> = None;
    let mut recording_dir: Option<PathBuf> = None;
    for (id, simulation) in data.as_ref().traces.iter().enumerate() {
        let mut binary_interface = match simulation.bin_interface.lock() {
            Ok(k) => k,
            Err(k) => return HttpResponse::InternalServerError().body(k.to_string()),
        };
        recording_dir = Some(simulation.save_directory.clone());
        if binary_name.is_none() {
            binary_name = Some(binary_interface.get_exec_file().into());
        } // TODO ensure binary name is the same across all traces
        let mut frame_time_map = match simulation.frame_time_map.lock() {
            Ok(k) => k,
            Err(k) => return HttpResponse::InternalServerError().body(k.to_string()),
        };
        traces.push(TraceGeneralInfo {
            id,
            frame_time_map: frame_time_map.clone(),
            proc_maps: binary_interface.get_proc_map().unwrap().to_vec(),
        });
    }
    let data = GeneralInfoResponse {
        binary_name: binary_name.unwrap(),
        recording_dir: recording_dir.unwrap(),
        traces,
    };
    HttpResponse::Ok().json(data)
}
async fn get_current_graph(
    data: web::Data<Arc<SimulationStorage>>,
    packet_version: web::Data<Arc<Mutex<usize>>>,
    _req: web::Json<CurrentGraphRequest>,
) -> HttpResponse {
    // let mut packet_version = packet_version.get_ref().lock().unwrap();
    // *packet_version+=1;

    let dwarf_data = data.get_ref().traces[0].dwarf_data.lock().unwrap();
    let mut graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let settings = data.get_ref().settings.lock().unwrap();
    let dot_data = graph_builder.get_graph_as_dot(&dwarf_data,&settings).unwrap();
    // println!("{}",&dot_data.clone().unwrap());
    // dbg!(&data.get_ref().traces.len());
    let response: CurrentGraphResponse = CurrentGraphResponse {
        version: 0,
        dot: dot_data.unwrap(),
    };
    HttpResponse::Ok().json(response)
}
async fn create_gdb_server(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<CreateGdbServerRequest>,
) -> HttpResponse {
    let req = req.0;
    let simulation = &data.get_ref().traces[0];
    let mut gdb_instance_manager = data.get_ref().traces[0].gdb_instance_mgr.lock().unwrap();
    let to_ret = gdb_instance_manager.create_instance(&req.start_time, simulation);
    // TODO
    let response = CreateGdbServerResponse {
        value: to_ret.unwrap_or_else(|m| format!("ERROR CREATING SERVER. {m}")),
    };
    HttpResponse::Ok().json(response)
}
async fn get_addr_occurrences(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<AddrOccurrencesRequest>,
) -> HttpResponse {
    let req = req.0;
    let graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let occurs = graph_builder.get_addr_occurrences(req.synoptic_node_id);
    //TODO
    let response = AddrOccurrenceResponse { val: occurs };
    HttpResponse::Ok().json(response)
}
async fn get_node_data(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<NodeDataRequest>,
) -> HttpResponse {
    let graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let resp = NodeDataResponse {
        modules: graph_builder.modules.clone(),
        nodes: graph_builder.synoptic_nodes.clone(),
    };
    HttpResponse::Ok().json(resp)
}
async fn get_raw_nodes_and_modules(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<GetRawNodesAndModulesRequest>,
) -> HttpResponse {
    let graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    let resp = GetRawNodesAndModulesResponse {
        modules: graph_builder.modules.clone(),
        nodes: graph_builder.nodes.clone(),
    };
    HttpResponse::Ok().json(resp)
}
async fn update_raw_nodes_and_modules(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<UpdateRawNodesAndModulesRequest>,
) -> HttpResponse {
    let req = req.0;

    let mut bin_interface =
        BinaryInterface::new_at_target_event(0, data.get_ref().traces[0].save_directory.clone());
    let cthread = bin_interface.get_current_thread();
    bin_interface.pin_mut().set_query_thread(cthread);
    bin_interface.set_pass_signals(vec![
        0, 0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
    ]);
    let settings: &mut Settings = &mut data.get_ref().settings.lock().unwrap();
    let erebor = data.get_ref().traces[0].dwarf_data.lock().unwrap();
    let mut graph_builder = data.get_ref().traces[0].graph_builder.lock().unwrap();
    settings.selected_node_id = None;
    graph_builder.update_raw_modules(req.modules).unwrap();
    graph_builder.update_raw_nodes(req.nodes, &erebor).unwrap();
    graph_builder.prepare(&mut bin_interface, req.rerun_level);

    let resp = UpdateRawNodesAndModulesResponse {};
    HttpResponse::Ok().json(resp)
}
async fn get_all_source_files(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<NodeDataRequest>,
) -> HttpResponse {
    let erebor = data.get_ref().traces[0].dwarf_data.lock().unwrap();
    let out = erebor.files.keys().map(|k| (*k).clone()).collect();

    let resp = AllSourceFilesResponse { files: out };
    HttpResponse::Ok().json(resp)
}
async fn get_settings(
    data: web::Data<Arc<SimulationStorage>>,
    _req: web::Json<GetSettingsRequest>,
) -> HttpResponse {
    let settings: Settings = data.get_ref().settings.lock().unwrap().clone();
    HttpResponse::Ok().json(settings)
}
async fn set_settings(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<SetSettingsRequest>,
) -> HttpResponse {
    let req = req.0;
    let settings: &mut Settings = &mut data.get_ref().settings.lock().unwrap();
    *settings = req.settings;
    settings.version += 1;
    HttpResponse::Ok().json(settings.clone())
}
async fn get_source_file(
    data: web::Data<Arc<SimulationStorage>>,
    req: web::Json<SourceFileRequest>,
) -> HttpResponse {
    let req = req.0;
    // TODO Checks here
    let contents = std::fs::read_to_string(req.file_name);
    let resp = SourceFileResponse {
        data: contents.unwrap_or("[empty]".into()),
    };
    HttpResponse::Ok().json(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    procspawn::init();

    librr_rs::raise_resource_limits();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Record {
            exe,
            save_dir,
            record_screen,
        } => {
            recorder::record(exe, save_dir, *record_screen, None);
            Ok(())
        }
        Commands::Explore {
            trace,
            no_glibc_offsets,
        } => {
            return run_server(vec![trace.clone()], !*no_glibc_offsets).await;
        }
    }
}
fn react_frontend_app() -> actix_web::Result<actix_files::NamedFile> {
    let path: PathBuf = PathBuf::from("./frontend/build/index.html");
    Ok(actix_files::NamedFile::open(path)?)
}
async fn run_server(traces: Vec<PathBuf>, offset_addrs_with_map: bool) -> std::io::Result<()> {
    if traces.len() == 0 {
        log::error!("You must pass at least one trace");
        // TODO: Anyhow this with proper msg
        return Ok(());
    }
    let traces = traces
        .iter()
        .map(|t| Simulation::new(t.clone(), offset_addrs_with_map).unwrap())
        .collect();
    let simulation: Arc<SimulationStorage> = Arc::new(SimulationStorage {
        traces,
        settings: Mutex::new(Settings::default()),
    });
    let packet_version: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let port = 12000;
    let ip = "0.0.0.0";
    log::info!("Starting HTTP server at {}:{}", &ip, port);
    std::thread::spawn(move || {
        while port_scanner::local_port_available(12000) {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        std::process::Command::new("/usr/bin/xdg-open")
            .arg(format!("http://localhost:{}", port))
            .output()
            .expect("failed to start web browser at port");
    });
    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(simulation.clone()))
            .app_data(web::Data::new(packet_version.clone()))
            .app_data(web::JsonConfig::default().limit(1073741824))
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
            .service(web::resource("/set_settings").route(web::post().to(set_settings)))
            .service(web::resource("/get_settings").route(web::post().to(get_settings)))
            .service(web::resource("/create_gdb_server").route(web::post().to(create_gdb_server)))
            .service(web::resource("/addr_occurrences").route(web::post().to(get_addr_occurrences)))
            .service(web::resource("/source_files").route(web::post().to(get_all_source_files)))
            .service(
                web::resource("/get_raw_nodes_and_modules")
                    .route(web::post().to(get_raw_nodes_and_modules)),
            )
            .service(
                web::resource("/update_raw_nodes_and_modules")
                    .route(web::post().to(update_raw_nodes_and_modules)),
            )
            // .route("/", web::get().to(react_frontend_app))
            .service(actix_files::Files::new("/", "./frontend/build").index_file("index.html"))
    })
    .workers(8)
    .bind((ip, port))?
    .run()
    .await
}
//fn get_addrs(sample_dateviewer_dir: PathBuf) -> Vec<usize> {
//    let mut bin_interface = BinaryInterface::new_at_target_event(0, sample_dateviewer_dir.clone());

//    let cthread = bin_interface.get_current_thread();
//    bin_interface.pin_mut().set_query_thread(cthread);
//    bin_interface.set_pass_signals(vec![
//        0, 0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
//    ]);
//    let rip = bin_interface
//        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
//        .to_usize();
//    dbg!(rip);

//    let mut stack_info = TrampolineStackInfo {
//        base_addr: 0x71000000,
//        // Ive had success with 65KiB
//        // but I made it 256 MiB just in case.
//        // This shouldn't overflow
//        //
//        //NOTE:
//        //  This is consistently faster on my machine if
//        //  it is given 1GiB instead of 256MiB.
//        //  I have no idea why.
//        size: 0x10000000,
//        reserved_space: 0x40,
//    };
//    stack_info.allocate_map(&mut bin_interface);
//    stack_info.setup_stack_ptr(&mut bin_interface).unwrap();
//    // dbg!(bin_interface.get_proc_map());
//    let mut proc_map: Lapper<usize, Map> = Lapper::new(vec![]);
//    for map in bin_interface.get_proc_map().unwrap().iter() {
//        proc_map.insert(Interval {
//            start: map.base,
//            stop: map.ceiling,
//            val: map.clone(),
//        });
//    }

//    let mut tr = TrampolineManager::new(&mut bin_interface, stack_info, &proc_map);
//    tr.create_trampolines(&mut bin_interface).unwrap();
//    let step = GdbContAction {
//        type_: GdbActionType::ACTION_CONTINUE,
//        target: bin_interface.get_current_thread(),
//        signal_to_deliver: 0,
//    };
//    let start_continue = SystemTime::now();
//    let mut signal = 5;
//    while signal != 9 {
//        signal = bin_interface
//            .pin_mut()
//            .continue_forward_jog_undefined(step)
//            .unwrap();
//        tr.clear_address_stack(&mut bin_interface).unwrap();
//    }
//    dbg!(signal);
//    dbg!(bin_interface.current_frame_time());
//    dbg!(start_continue.elapsed().unwrap());
//    let entries = tr.recorded_addresses();
//    dbg!(entries.len());
//    entries.clone()
//}

// * DO NOT USE FOR PROD.
// * VERY SLOW
// */
//fn get_current_instr(bin_interface: &BinaryInterface) -> Instruction {
//    let rip = bin_interface
//        .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
//        .to_usize();

//    let bytes = bin_interface.get_mem(rip, 18);
//    let mut decoder = Decoder::with_ip(64, &bytes, rip as u64, DecoderOptions::NONE);
//    let mut instr = Instruction::default();
//    decoder.decode_out(&mut instr);
//    instr
//}
//fn create_code_flow(bin_interface: &mut BinaryInterface) -> Result<CodeFlow, Box<dyn Error>> {
//    let step = GdbContAction {
//        type_: GdbActionType::ACTION_STEP,
//        target: bin_interface.get_current_thread(),
//        signal_to_deliver: 0,
//    };

//    let cont = GdbContAction {
//        type_: GdbActionType::ACTION_CONTINUE,
//        target: bin_interface.get_current_thread(),
//        signal_to_deliver: 0,
//    };
//    // bin_interface.pin_mut().set_sw_breakpoint(main_addr,1);
//    // bin_interface.pin_mut().continue_forward(cont);
//    // bin_interface.pin_mut().remove_sw_breakpoint(main_addr,1);
//    let mut code_flow = CodeFlow::default();
//    let mut signal = 5;

//    'outer: while signal == 5 {
//        let rip = bin_interface
//            .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
//            .to_usize();
//        code_flow.path.push(rip);

//        // let instructions = read_instructions_till_flow_change(&bin_interface, rip);
//        let block = if let Some(block) = code_flow.blocks.find(rip, rip).next() {
//            block.val.clone()
//        } else {
//            let instrs = read_instructions_till_flow_change(&bin_interface, rip);
//            let start = rip;
//            let stop = instrs.last().unwrap().ip() as usize;
//            let block = Block::new(start, stop, instrs);
//            code_flow.blocks.insert(Interval {
//                start: start - 1,
//                stop: stop + 1,
//                val: Arc::new(block),
//            });
//            code_flow.blocks.find(rip, rip).next().unwrap().val.clone()
//        };

//        break 'outer;
//        let instructions = block.instructions();

//        bin_interface
//            .pin_mut()
//            .set_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
//        signal = bin_interface.pin_mut().continue_forward(cont).unwrap();
//        if signal != 5 {
//            break 'outer;
//        }
//        bin_interface
//            .pin_mut()
//            .remove_sw_breakpoint(instructions.last().unwrap().ip() as usize, 1);
//        signal = bin_interface.pin_mut().continue_forward(step).unwrap();
//    }

//    Ok(code_flow)
//}

//fn read_instructions(
//    bin_interface: &BinaryInterface,
//    start_addr: usize,
//    size: usize,
//) -> Vec<Instruction> {
//    let bytes = bin_interface.get_mem(start_addr, size);
//    let mut instructions = Vec::new();
//    let mut decoder = Decoder::with_ip(64, &bytes, start_addr as u64, DecoderOptions::NONE);
//    let mut instr = Instruction::default();
//    while decoder.can_decode() {
//        decoder.decode_out(&mut instr);
//        instructions.push(instr);
//    }
//    instructions
//}

//const READ_CHUNK_SIZE: usize = 40;
//fn read_instructions_till_flow_change(
//    bin_interface: &BinaryInterface,
//    ip: usize,
//) -> Vec<Instruction> {
//    let mut base = ip;
//    let mut instructions = Vec::new();
//    loop {
//        let bytes = bin_interface.get_mem(base, READ_CHUNK_SIZE);

//        let mut decoder = Decoder::with_ip(64, &bytes, base as u64, DecoderOptions::NONE);
//        let mut instr = Instruction::default();
//        let mut last_successful_position = 0;
//        while decoder.can_decode() {
//            decoder.decode_out(&mut instr);
//            if instr.code() != Code::INVALID {
//                last_successful_position = decoder.position();
//            } else {
//                break;
//            }

//            let non_next_flow = instr.flow_control() != FlowControl::Next;
//            instructions.push(std::mem::take(&mut instr));
//            if non_next_flow {
//                return instructions;
//            }
//        }
//        base += last_successful_position;
//    }
//}
