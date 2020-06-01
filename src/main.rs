use serversim::cli::run;
use std::process;

extern crate itertools_num;

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
