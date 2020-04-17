use crate::instance::*;
use crate::pred_generator::{run_generate_predictions, PredictionConfig};
use crate::sample::Sample;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct SampleConfig {
    #[structopt(flatten)]
    pub pred_config: PredictionConfig,
}

pub fn run(instances: Vec<Instance>, config: &SampleConfig) -> Result<Vec<Sample>, Box<dyn Error>> {
    println!("{}", style("Start generating samples...").bold().cyan());
    println!("{} Solving instances...", style("[1/3]").bold().dim());
    let raw_samples = solve_instances(instances)?;
    println!("{} Normalizing solutions...", style("[2/3]").bold().dim());
    let samples = normalize_solutions(raw_samples)?;
    let number_of_samples = samples.len();
    println!("{} Generating predictions...", style("[3/3]").bold().dim());
    let samples_with_preds = run_generate_predictions(samples, &config.pred_config)?;
    let number_of_rejected_samples = number_of_samples - samples_with_preds.len();
    if number_of_rejected_samples > 0 {
        println!(
            "{} samples have been rejected because no valid predictions have been found!",
            style(number_of_rejected_samples).bold().red()
        );
    }
    println!("{}", style("Finished!").bold().green());

    Ok(samples_with_preds)
}

fn normalize_solutions(samples: Vec<Sample>) -> Result<Vec<Sample>, Box<dyn Error>> {
    let pb = ProgressBar::new(samples.len() as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    let solutions = samples
        .into_iter()
        .progress_with(pb)
        .map(|sample| sample.normalize())
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
        .map(|instance| instance.build_sample())
        .filter_map(Result::ok)
        .collect();

    Ok(solutions)
}
