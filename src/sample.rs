use crate::instance::Instance;
use crate::instance_sample::generate_sample;
use crate::seq::{normalize_sequence, Sequence};
use crate::solver::solve;
use console::style;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct SampleCmd {
    #[structopt(short = "s", long = "samples")]
    pub number_of_samples: usize,

    #[structopt(short = "k", long = "servers", default_value = "2")]
    pub number_of_servers: usize,

    #[structopt(short = "l", long = "length", default_value = "20")]
    pub number_of_requests: usize,

    #[structopt(short = "p", long = "preds", default_value = "20")]
    pub number_of_predictions: usize,

    #[structopt(long = "min", default_value = "0")]
    pub min_value: i32,

    #[structopt(long = "max", default_value = "100")]
    pub max_value: i32,

    #[structopt(short = "b", long = "preds_bin_size", default_value = "0.25")]
    pub step_size: f32,
}

pub type Config = SampleCmd;

struct Sample {
    pub instance: Instance,
    pub solution: Sequence,
}

impl Sample {
    fn new(instance: Instance, solution: Sequence) -> Sample {
        Sample { instance, solution }
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("Starting sampling...");
    println!("{} Generating instances...", style("[1/4]").bold().dim());
    let instances = generate_instances(config)?;
    println!("{} Solving instances...", style("[2/4]").bold().dim());
    let raw_samples = solve_instances(instances)?;
    println!("{} Normalizing solutions...", style("[3/4]").bold().dim());
    let samples = normalize_instances(raw_samples)?;

    Ok(())
}

fn normalize_instances(samples: Vec<Sample>) -> Result<Vec<Sample>, Box<dyn Error>> {
    let pb = ProgressBar::new(samples.len() as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    let solutions = samples
        .into_iter()
        .progress_with(pb)
        .map(|sample| match normalize_sequence(sample.solution) {
            Ok(s) => Ok(Sample::new(sample.instance, s)),
            Err(e) => Err(e),
        })
        .filter_map(Result::ok)
        .collect();

    Ok(solutions)
}

fn solve_instances(instances: Vec<Instance>) -> Result<Vec<Sample>, Box<dyn Error>> {
    let pb = ProgressBar::new(instances.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    let solutions = instances
        .into_iter()
        .progress_with(pb)
        .map(|instance| match solve(&instance) {
            Ok(s) => Ok(Sample::new(instance, s.0)),
            Err(e) => Err(e),
        })
        .filter_map(Result::ok)
        .collect();

    Ok(solutions)
}

fn generate_instances(config: &Config) -> Result<Vec<Instance>, Box<dyn Error>> {
    let pb = ProgressBar::new(config.number_of_samples as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    let mut samples = Vec::with_capacity(config.number_of_samples);

    for _ in (0..config.number_of_samples).progress_with(pb) {
        samples.push(generate_sample(config));
    }

    Ok(samples)
}
