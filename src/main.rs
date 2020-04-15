use std::process;
use kserver::cli::run;

extern crate itertools_num;

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
