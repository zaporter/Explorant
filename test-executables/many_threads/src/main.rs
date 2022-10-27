use std::thread;
use std::time::Duration;

/*
 * This program spawns 10 threads, each of which say that they've started, wait 10 ms, and then say
 * they're finished. 
 *
 * This is used to test the get_thread_list() functionality of the library
 *
 * I could have used sync constants to limit the number of sleeps I need, but I don't want to add
 * extra code to the final binary.
 *
 */
fn main() {
    println!("Started");
    let mut handles = Vec::new();
    for thread_num in 0..10 {
        handles.push(thread::spawn(move || {
            println!("Thread {thread_num} has started");
            // ensure we dont finish before we can count the threads
            thread::sleep(Duration::from_millis(10));
            println!("Thread {thread_num} is done");
        }));
    };
    // give the threads time to start
    thread::sleep(Duration::from_millis(2));
    println!("Done Spawning");
    // Give me time to count the threads
    thread::sleep(Duration::from_millis(1));
    for h in handles {
        h.join().unwrap();
    }
    println!("Finished");
}
