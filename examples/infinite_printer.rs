use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Start2");
    loop {
        println!("You should never see me");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}
