use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, File};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{thread, u128};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io::Write;

use librr_rs::RecordingInterface;

const RECORDING_TEMP_FILE_NAME: &str = "unique_temp_recording_output.mkv";
const RECORDING_TEMP_TIMES_NAME: &str = "unique_temp_recording_output_times.txt";

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct FrameTimeMap {
    frames: Vec<(i64, u128, String)>,
    times: HashMap<i64, u128>,
}

pub fn record(exe_path: &str, output_directory: &str) -> Result<(), Box<dyn Error>> {
    remove_dir_all(output_directory);

    // https://stackoverflow.com/questions/53391150/ffmpeg-obtain-the-system-time-corresponding-to-each-frame-present-in-a-video
    let child = Command::new("/usr/bin/ffmpeg")
        .arg("-f")
        .arg("x11grab")
        .arg("-framerate")
        .arg("30")
        .arg("-i")
        .arg(":0.0+0,0")
        .arg("-filter_complex")
        .arg("settb=1/1000,setpts=RTCTIME/1000-1500000000000,mpdecimate,split[out][ts];[out]setpts=N/FRAME_RATE/TB[out]")
        .arg("-map")
        .arg("[out]")
        .arg("-vcodec")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-preset")
        .arg("fast")
        .arg("-crf")
        .arg("0")
        .arg("-threads")
        .arg("0")
        .arg(RECORDING_TEMP_FILE_NAME)
        .arg("-map")
        .arg("[ts]")
        .arg("-f")
        .arg("mkvtimestamp_v2")
        .arg(RECORDING_TEMP_TIMES_NAME)
        .arg("-vsync")
        .arg("0")
        .spawn()?;

    thread::sleep(Duration::from_millis(1000));
    let mut rec_interface = RecordingInterface::new(format!("-o {output_directory} {exe_path}"));
    let mut frame_times_to_system_milis: HashMap<i64, u128> = HashMap::new();
    while rec_interface.pin_mut().continue_recording() {
        if frame_times_to_system_milis.contains_key(&rec_interface.current_frame_time()) {
            continue; // This should never happen but it is worth handling properly
        }
        frame_times_to_system_milis.insert(
            rec_interface.current_frame_time(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        );
    }
    thread::sleep(Duration::from_millis(1000));
    dbg!(child.id());
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT).unwrap();

    thread::sleep(Duration::from_millis(3000));

    create_dir_all(format!("{}/frames", output_directory))?;
    // extract all frames
    let mut child = Command::new("/usr/bin/ffmpeg")
        .arg("-i")
        .arg(RECORDING_TEMP_FILE_NAME)
        .arg(format!("{}/frames/out-%06d.jpg", output_directory))
        .spawn()?;
    child.wait()?;
    let frames_dir = read_dir(format!("{}/frames", output_directory))?;
    let frames_names = frames_dir.map(|f| f.unwrap().file_name()).sorted();
    let frame_times_str = read_to_string(RECORDING_TEMP_TIMES_NAME)?;
    let mut frametimemap = FrameTimeMap::default();
    // correlate the recorded times and frames with the recorded times and frame_times
    // insert everything into the frametimemap
    for (time_str, frame_img_entry) in frame_times_str.split("\n").skip(1).zip(frames_names) {
        let time = time_str.parse::<u128>()? + 1500000000000;
        dbg!(time);
        dbg!(&frame_img_entry);
        let mut last_real_time = 0;
        'inner: for (frame_time, real_time) in frame_times_to_system_milis.iter() {
            dbg!(real_time);
            dbg!(frame_time);
            if time > last_real_time && time <= *real_time {
                frametimemap.frames.push((
                    *frame_time,
                    *real_time,
                    format!("frames/{}", frame_img_entry.to_str().unwrap()),
                ));
                break 'inner;
            }
            last_real_time = *real_time;
        }
    }
    frametimemap.times=frame_times_to_system_milis;
    let mut frametimemapfile = File::create(format!("{}/frame_time_map.json5", output_directory))?;
    write!(frametimemapfile, "{}", json5::to_string(&frametimemap)?)?;

    remove_file(RECORDING_TEMP_TIMES_NAME)?;
    remove_file(RECORDING_TEMP_FILE_NAME)?;
    thread::sleep(Duration::from_millis(3000));

    println!("Finished Recording");
    Ok(())
}
