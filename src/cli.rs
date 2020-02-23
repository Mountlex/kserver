use crate::sample;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "k-server-simulation")]
struct KServer {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Sample instances
    #[structopt(name = "sample")]
    Sample(sample::SampleCmd),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = KServer::from_args();

    println!("{:?}", cli);

    match cli.cmd {
        Command::Sample(config) => sample::run(&config),
    }
}
