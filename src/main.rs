use kserver::cli::run;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
