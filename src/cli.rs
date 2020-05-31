use crate::export;
use crate::instance_generator;
use crate::sample_generator;
use crate::sim;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "server-simulation")]
struct Cli {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(flatten)]
    instance_config: instance_generator::InstanceConfig,

    #[structopt(flatten)]
    sample_config: sample_generator::SampleConfig,

    #[structopt(flatten)]
    export_config: export::ExportConfig,

    #[structopt(subcommand)]
    generator: Generators,
}

#[derive(StructOpt, Debug)]
pub enum Generators {
    /// Sample kserver instances
    #[structopt(name = "sample")]
    Sample {
        #[structopt(flatten)]
        config: instance_generator::InstanceSampleConfig,

        #[structopt(subcommand)]
        simulator: sim::Simulators,
    },

    #[structopt(name = "load_instances")]
    LoadInstances {
        #[structopt(flatten)]
        config: instance_generator::InstanceLoadConfig,

        #[structopt(subcommand)]
        simulator: sim::Simulators,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::from_args();

    println!("{:?}", cli);
    let mut simu: sim::Simulators;
    let instances = match cli.generator {
        Generators::Sample { config, simulator } => {
            simu = simulator;
            instance_generator::generate_instances(&config, &cli.instance_config)?
        }
        Generators::LoadInstances { config, simulator } => {
            simu = simulator;
            instance_generator::load_instances(&config, &cli.instance_config)?
        }
    };

    let samples = sample_generator::run(instances, &cli.sample_config)?;

    let results = sim::run(samples, simu)?;
    export::run(results, &cli.export_config)?;

    Ok(())
}
