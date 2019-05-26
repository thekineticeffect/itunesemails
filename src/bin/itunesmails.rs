use std::env;
use std::process;

use itunesmails::*;

fn main() {
    let directory: Vec<String> = env::args().collect();
    if let Some(dir) = directory.get(1) {
        process_folder(dir).unwrap_or_else(|error| {
            println!("Had error: {}", error);
            process::exit(1)
        });
    } else {
        println!("Requires a folder as the first argument!");
        process::exit(1);
    }
}
