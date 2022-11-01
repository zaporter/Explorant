use std::path::PathBuf;

use crate::simulation::Simulation;

pub fn run(save_dir : &PathBuf){
    println!("Running MVP on {:?}",save_dir);
    let simulation: Simulation = Simulation::new(save_dir.clone()).unwrap();
}
