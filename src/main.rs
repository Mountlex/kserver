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
enum Command {
    /// Sample instances
    #[structopt(name = "sample")]
    Sample {
        #[structopt(short = "s", long = "samples")]
        number_of_samples: u32,

        #[structopt(short = "p", long = "preds")]
        number_of_predictions: u32,

        #[structopt(short = "b", long = "preds_bin_size")]
        step_size: f32,
    },
}

fn main() {
    let cli = KServer::from_args();
}
