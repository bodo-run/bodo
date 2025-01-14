use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Start2");
    loop {
        print!("You should never see me\n");
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}
