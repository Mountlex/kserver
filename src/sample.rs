use crate::instance::Instance;
use crate::instance_sample::generate_sample;
use crate::pred::{generate_predictions, PredictionConfig};
use crate::seq::{normalize_sequence, Sequence};
use crate::solver::solve;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct SampleCmd {
    pub number_of_samples: usize,

    #[structopt(short = "k", long = "servers", default_value = "2")]
    pub number_of_servers: usize,

    #[structopt(short = "l", long = "length", default_value = "50")]
    pub number_of_requests: usize,

    #[structopt(long = "min", default_value = "0")]
    pub min_value: i32,
    #[structopt(long = "max", default_value = "200")]
    pub max_value: i32,
    #[structopt(flatten)]
    pub pred_config: PredictionConfig,
}

pub type Config = SampleCmd;

#[derive(Clone)]
pub struct Sample {
    pub instance: Instance,
    pub solution: Sequence,
    pub predictions: Vec<Sequence>,
}

impl Sample {
    fn new(instance: Instance, solution: Sequence) -> Sample {
        Sample {
            instance,
            solution,
            predictions: vec![],
        }
    }
}

pub fn run(config: &Config) -> Result<Vec<Sample>, Box<dyn Error>> {
    println!("{}", style("Start sampling...").bold().cyan());
    println!("{} Generating instances...", style("[1/4]").bold().dim());
    let instances = generate_instances(config)?;
    println!("{} Solving instances...", style("[2/4]").bold().dim());
    let raw_samples = solve_instances(instances)?;
    println!("{} Normalizing solutions...", style("[3/4]").bold().dim());
    let samples = normalize_instances(raw_samples)?;
    let number_of_samples = samples.len();
    println!("{} Generating predictions...", style("[4/4]").bold().dim());
    let samples_with_preds = generate_all_predictions(samples, config)?;
    let number_of_rejected_samples = number_of_samples - samples_with_preds.len();
    if number_of_rejected_samples > 0 {
        println!(
            "{} samples have been rejected because no valid predictions have been found!",
            style(number_of_rejected_samples).bold().red()
        );
    }
    println!("{}", style("Sampling finished!").bold().green());

    Ok(samples_with_preds)
}

fn generate_all_predictions(
    samples: Vec<Sample>,
    config: &Config,
) -> Result<Vec<Sample>, Box<dyn Error>> {
    let pb = ProgressBar::new(samples.len() as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );

    let samples_with_preds = samples
        .into_par_iter()
        .progress_with(pb)
        .map(|sample| {
            match generate_predictions(&sample.instance, &sample.solution, &config.pred_config) {
                Ok(preds) => Ok(Sample {
                    predictions: preds,
                    ..sample
                }),
                Err(e) => Err(e),
            }
        })
        .filter_map(Result::ok)
        .collect();

    Ok(samples_with_preds)
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
        .into_par_iter()
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
