use std::{thread, time, env};

fn main() {
    println!("Started");
    let args: Vec<String> = env::args().collect();
    print_time("StartTime");
    if args.len() >1 {
        let delay_time : u64 = args[1].parse().expect("Failed to parse time");
        let delay = time::Duration::from_millis(delay_time);
        thread::sleep(delay);
        print_time("EndTime");
    }
    println!("Finished");
}
fn print_time(prefix: &str) {
    let local = chrono::offset::Local::now();
    println!("{}: {:?}", prefix,local);

}
