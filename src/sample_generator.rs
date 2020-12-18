use serverlib::prelude::Instance;

use crate::pred_generator::{run_generate_predictions, PredictionConfig};
use samplelib::*;

use crate::solver::SampleBuilder;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
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
    println!("{} Solving instances...", style("[1/2]").bold().dim());
    let samples = solve_instances(instances)?;
    let number_of_samples = samples.len();
    println!("{} Generating predictions...", style("[2/2]").bold().dim());
    let samples_with_preds = run_generate_predictions(samples, &config.pred_config)?;
    let number_of_rejected_samples = number_of_samples - samples_with_preds.len();
    if number_of_rejected_samples > 0 {
        println!(
            "{} samples have been rejected because no valid predictions have been found!",
            style(number_of_rejected_samples).bold().red()
        );
        println!(
            "{} instances remain!",
            style(samples_with_preds.len()).bold().green()
        );
    }
    println!("{}", style("Finished!").bold().green());

    Ok(samples_with_preds)
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
