use crate::export;
use crate::instance_generator;
use crate::sample_generator;
use crate::sim;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "k-server-simulation")]
struct KServer {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(flatten)]
    sim_config: sim::SimConfig,

    #[structopt(flatten)]
    instance_config: instance_generator::InstanceConfig,

    #[structopt(flatten)]
    sample_config: sample_generator::SampleConfig,

    #[structopt(flatten)]
    export_config: export::ExportConfig,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Sample kserver instances
    #[structopt(name = "kserver_sample")]
    SampleKServer(instance_generator::KServerInstanceSampleConfig),

    /// Sample ktaxi instances
    #[structopt(name = "ktaxi_sample")]
    SampleKTaxi(instance_generator::KTaxiInstanceSampleConfig),

    #[structopt(name = "load_instances")]
    LoadInstances(instance_generator::InstanceLoadConfig),
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = KServer::from_args();

    println!("{:?}", cli);

    let instances = match cli.cmd {
        Command::SampleKServer(config) => {
            instance_generator::generate_kserver_instances(&config, &cli.instance_config)?
        }        
        Command::SampleKTaxi(config) => {
            instance_generator::generate_ktaxi_instances(&config, &cli.instance_config)?
        }
        Command::LoadInstances(config) => {
            instance_generator::load_instances(&config, &cli.instance_config)?
        }
    };

    let samples = sample_generator::run(instances, &cli.sample_config)?;

    let results = sim::run(samples, &cli.sim_config)?;
    export::run(results, &cli.export_config)?;

    Ok(())
}
