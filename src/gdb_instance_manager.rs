use std::{thread::JoinHandle, collections::HashMap};

use anyhow::bail;
use librr_rs::*;

use crate::{shared_structs::TimeStamp, simulation::Simulation};

const SHOULD_WAIT_TILL_RUNNING : bool = false;

#[derive(Default)]
pub struct GdbInstanceManager {
    instances: HashMap<TimeStamp, RunningGdbInstance>,
}
impl GdbInstanceManager {
    pub fn is_running(&self, ts: &TimeStamp) -> bool {
        let Some(inst) = self.instances.get(ts) else {
            return false;
        };
        inst.is_alive()
    }
    // pub fn get_instance(&self, ts:&TimeStamp) -> &
    pub fn create_instance(&mut self, ts: &TimeStamp, simulation: &Simulation)->anyhow::Result<String>{
        let inst = RunningGdbInstance::spawn(ts, simulation)?;
        let to_ret = inst.2.clone();
        if SHOULD_WAIT_TILL_RUNNING && inst.wait_till_running().is_err(){
            return bail!("Err: {:?}",inst.join());

        }else {
            self.instances.insert(ts.clone(), inst);
            Ok(to_ret)
        }
    }
    pub fn kill(&self, ts: &TimeStamp) {
        todo!()
    }
}

struct RunningGdbInstance(TimeStamp, JoinHandle<anyhow::Result<()>>, String, u16);
impl RunningGdbInstance {
    pub fn spawn(start_loc: &TimeStamp, simulation: &Simulation) -> anyhow::Result<Self> {
        dbg!(&start_loc);
        let port = port_scanner::request_open_port().ok_or(anyhow::Error::msg("No open ports!"))?;
        let exe= simulation.bin_interface.lock().unwrap().get_exec_file();
        
        let conn_str = format!("gdb '-l' '10000' '-ex' 'set sysroot /' '-ex' 'target extended-remote 127.0.0.1:{port}' {exe}");
        let save_directory = simulation.save_directory.clone();
        let start_loc = start_loc.clone();
        let handler = std::thread::spawn(move || {
            let mut bin_interface = BinaryInterface::new_at_target_event(
                start_loc.frame_time as i64 - 1,
                save_directory,
            );

            let cthread = bin_interface.get_current_thread();
            bin_interface.pin_mut().set_query_thread(cthread);
            bin_interface.set_pass_signals(vec![
                0, 0xe, 0x14, 0x17, 0x1a, 0x1b, 0x1c, 0x21, 0x24, 0x25, 0x2c, 0x4c, 0x97,
            ]);
            let cont = GdbContAction {
                type_: GdbActionType::ACTION_CONTINUE,
                target: bin_interface.get_current_thread(),
                signal_to_deliver: 0,
            };
            let step = GdbContAction {
                type_: GdbActionType::ACTION_STEP,
                target: bin_interface.get_current_thread(),
                signal_to_deliver: 0,
            };
            if let Some(addr) = start_loc.addr {
                if let Some(desired_times) = start_loc.instance_of_addr {
                    let mut times_reached = 0;
                    bin_interface.pin_mut().set_sw_breakpoint(addr, 1);
                    loop {
                        let rip = bin_interface
                            .get_register(GdbRegister::DREG_RIP, bin_interface.get_current_thread())
                            .to_usize();
                        dbg!(&rip);
                        let current_ft = bin_interface.current_frame_time() as usize;
                        if current_ft != start_loc.frame_time {
                            bail!("Failed to start gdb instance. At the wrong frame at: {}, expected: {}!", current_ft, start_loc.frame_time);
                        }
                        if rip == addr {
                            times_reached += 1;
                            // HAPPY PATH
                            if times_reached == desired_times {
                                bin_interface.pin_mut().remove_sw_breakpoint(rip, 1);
                                break;
                            }
                            bin_interface.pin_mut().remove_sw_breakpoint(rip, 1);
                            let signal = bin_interface.pin_mut().continue_forward(step).unwrap();
                            if signal != 5 {
                                bail!("Failed to start gdb instance. Step signal!");
                            }
                            bin_interface.pin_mut().set_sw_breakpoint(rip, 1);
                        }

                        let signal = bin_interface
                            .pin_mut()
                            .continue_forward_jog_undefined(cont)
                            .unwrap();

                        if signal != 5 {
                            bail!("Failed to start gdb instance. Continue signal");
                        }
                    }
                }
            }
            bin_interface.pin_mut().serve_current_state_as_gdbserver(port);
            Ok(())
        });
        Ok(Self(start_loc.clone(), handler, conn_str,port))
    }
    pub fn join(self) -> Result<anyhow::Result<()>,std::boxed::Box<(dyn std::any::Any + Send + 'static)>> {
        self.1.join()
    }
    pub fn wait_till_running(&self) -> anyhow::Result<()>{
        // While it is available, we haven't finished
        while !port_scanner::scan_port(self.3) {
            if !self.is_alive() {
                bail!("Failed to launch before port opened!");
            }else {
                // still alive
                std::thread::sleep_ms(50);
            }
        }
        // Not available, must have been grabbed by our program
        Ok(())
    }
    pub fn is_alive(&self) -> bool {
        self.1.is_finished()
    }
}
