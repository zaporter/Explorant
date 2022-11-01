use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, File};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{thread, u128};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io::Write;

use librr_rs::RecordingInterface;

use crate::shared_structs::FrameTimeMap;

const RECORDING_TEMP_FILE_NAME: &str = "unique_temp_recording_output.mkv";
const RECORDING_TEMP_TIMES_NAME: &str = "unique_temp_recording_output_times.txt";

pub fn record(
    exe_path: &PathBuf,
    output_directory: &PathBuf,
    exe_args: Option<&str>,
) -> anyhow::Result<()> {
    remove_dir_all(output_directory);
    let output_directory_str = output_directory
        .to_str()
        .ok_or_else(|| anyhow::Error::msg("Output directory cannot be turned into a str"))?;
    let exe_path_str = exe_path
        .to_str()
        .ok_or_else(|| anyhow::Error::msg("Exe path cannot be turned into a str"))?;

    // https://stackoverflow.com/questions/53391150/ffmpeg-obtain-the-system-time-corresponding-to-each-frame-present-in-a-video
    // TODO: The waiting in this is terrrible and
    // brittle. Refactor to use pipes from the child
    // to read stdout and stderr to decide when
    // ffmpeg is ready to start recording frames
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_millis(5000));
    let mut rec_interface = RecordingInterface::new(format!(
        "--output-trace-dir {} {} {}",
        output_directory_str,
        exe_path_str,
        exe_args.unwrap_or("")
    ));
    let mut frame_times_to_system_milis: HashMap<i64, u128> = HashMap::new();

    while rec_interface.pin_mut().continue_recording() {
        if frame_times_to_system_milis.contains_key(&rec_interface.current_frame_time()) {
            continue;
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
    // dbg!(child.id());
    signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)?;

    thread::sleep(Duration::from_millis(2000));

    create_dir_all(format!("{}/frames", output_directory_str))?;
    // extract all frames
    let mut child = Command::new("/usr/bin/ffmpeg")
        .arg("-i")
        .arg(RECORDING_TEMP_FILE_NAME)
        .arg(format!("{}/frames/out-%06d.jpg", output_directory_str))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    child.wait()?;
    let frames_dir = read_dir(format!("{}/frames", output_directory_str))?;
    let frames_names = frames_dir.map(|f| f.unwrap().file_name()).sorted();
    let frame_times_str = read_to_string(RECORDING_TEMP_TIMES_NAME)?;
    let mut frametimemap = FrameTimeMap {
        frames: Vec::new(),
        times: HashMap::new(),
    };
    // correlate the recorded times and frames with the recorded times and frame_times
    // insert everything into the frametimemap
    // let mut last_real_time = 0;
    // Add first frame before starting
    frametimemap.frames.push((
        1,
        *frame_times_to_system_milis.get(&1).ok_or_else(|| {
            anyhow::Error::msg("Didn't have an entry for the first frame time. Please report this.")
        })?,
        "frames/out-000001.jpg".into(),
    ));
    // Add all frames during execution
    
    for (time_str, frame_img_entry) in frame_times_str.split("\n").skip(1).zip(frames_names.clone()) {
        let time = time_str.parse::<u128>()? + 1500000000000;
        for (current_entry, next_entry) in frame_times_to_system_milis.iter().sorted_by_key(|a| a.0).tuple_windows() {
            if current_entry.1 < &time && next_entry.1>= &time {
                frametimemap.frames.push((
                    *current_entry.0,
                    time,
                    format!("frames/{}", frame_img_entry.to_str().unwrap()),
                ));
            }
        }
    }
    // Add first frame after execution
    {
        let (last_ft, last_ft_time) = frame_times_to_system_milis.iter().sorted_by_key(|a| a.0).last().ok_or_else(|| anyhow::Error::msg("No last frame to save"))?;
        let last_frame_name = frames_names.last().ok_or_else(|| anyhow::Error::msg("No last frame. Did ffmpeg start?"))?;
        frametimemap.frames.push((
            *last_ft,
            *last_ft_time,
            format!("frames/{}", last_frame_name.to_str().unwrap()),
        ));

        
    }

    frametimemap.times = frame_times_to_system_milis;
    let mut frametimemapfile =
        File::create(format!("{}/frame_time_map.json", output_directory_str))?;
    write!(
        frametimemapfile,
        "{}",
        serde_json::to_string(&frametimemap)?
    )?;

    remove_file(RECORDING_TEMP_TIMES_NAME)?;
    remove_file(RECORDING_TEMP_FILE_NAME)?;
    thread::sleep(Duration::from_millis(3000));

    println!("Finished Recording");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{io::Read, sync::Once};

    use super::*;
    use gag::BufferRedirect;
    use rand::prelude::*;

    static INIT: Once = Once::new();
    fn initialize() {
        INIT.call_once(|| {
            std::env::set_var("RUST_LOG", "debug");
            std::env::set_var("RUST_BACKTRACE", "1");
            env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
            librr_rs::raise_resource_limits();
        });
    }

    #[test]
    #[serial_test::serial]
    fn date_viewer_no_args() -> anyhow::Result<()> {
        initialize();
        let exe_dir = std::env::current_dir()
            .unwrap()
            .join("test-executables/build")
            .join("date_viewer");
        let random_number: u64 = rand::thread_rng().gen();
        let save_dir = std::env::temp_dir().join(format!("mqp_temp_{}", random_number.to_string()));
        let mut output = String::new();
        let mut stdout_buf = BufferRedirect::stdout().unwrap();
        super::record(&exe_dir, &save_dir, None)?;
        stdout_buf.read_to_string(&mut output).unwrap();
        drop(stdout_buf);
        assert!(output.contains("Started"));
        assert!(output.contains("StartTime"));
        assert!(!output.contains("EndTime"));
        assert!(output.contains("Finished"));
        Ok(())
    }
    #[test]
    #[serial_test::serial]
    fn date_viewer_args() -> anyhow::Result<()> {
        initialize();
        let exe_dir = std::env::current_dir()
            .unwrap()
            .join("test-executables/build")
            .join("date_viewer");
        let random_number: u64 = rand::thread_rng().gen();
        let save_dir = std::env::temp_dir().join(format!("mqp_temp_{}", random_number.to_string()));
        let mut output = String::new();
        let mut stdout_buf = BufferRedirect::stdout().unwrap();
        super::record(&exe_dir, &save_dir, Some("100"))?;
        stdout_buf.read_to_string(&mut output).unwrap();
        drop(stdout_buf);
        assert!(output.contains("Started"));
        assert!(output.contains("StartTime"));
        assert!(output.contains("EndTime"));
        assert!(output.contains("Finished"));
        let file = File::open(save_dir.join("frame_time_map.json"))?;
        let reader = std::io::BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let map: FrameTimeMap = serde_json::from_reader(reader)?;
        // this happens when there is not enough delay after
        // starting the recording. It makes the first avaiable frame equal to the last.
        //assert!(map.frames.last().unwrap().2 != "frames/out-000002.jpg");
        Ok(())
    }
    #[test]
    #[serial_test::serial]
    fn date_viewer_frame_time_map() -> anyhow::Result<()> {
        initialize();
        let exe_dir = std::env::current_dir()
            .unwrap()
            .join("test-executables/build")
            .join("date_viewer");
        let random_number: u64 = rand::thread_rng().gen();
        let save_dir = std::env::temp_dir().join(format!("mqp_temp_{}", random_number.to_string()));
        // log::error!("{:?}",&save_dir);
        let mut output = String::new();
        let mut stdout_buf = BufferRedirect::stdout().unwrap();
        super::record(&exe_dir, &save_dir, Some("10000"))?;
        stdout_buf.read_to_string(&mut output).unwrap();
        drop(stdout_buf);
        assert!(output.contains("Started"));
        assert!(output.contains("StartTime"));
        assert!(output.contains("EndTime"));
        assert!(output.contains("Finished"));

        let file = File::open(save_dir.join("frame_time_map.json"))?;
        let reader = std::io::BufReader::new(file);

        // Read the JSON contents of the file as an instance of `User`.
        let map: FrameTimeMap = serde_json::from_reader(reader)?;
        assert!(map.frames.len() > 1);
        assert!(map.frames.len() < 20);
        // this happens when there is not enough delay after
        // starting the recording. It makes the first avaiable frame equal to the last.
        assert!(map.frames.last().unwrap().2 != "frames/out-000002.jpg");
        //assert_eq!(map.times.len(),*map.times.keys().max().unwrap() as usize);
        Ok(())
    }
}
