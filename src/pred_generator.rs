use crate::pred::generate_predictions;
use crate::sample_generator::Sample;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PredictionConfig {
    #[structopt(short = "p", long = "preds", default_value = "12")]
    pub number_of_predictions: usize,

    #[structopt(short = "b", long = "preds_bin_size", default_value = "0.25")]
    pub step_size: f32,

    #[structopt(long = "preds_samples_per_round", default_value = "100")]
    pub number_of_samples_per_round: usize,

    #[structopt(long = "max_preds_per_bin", default_value = "5")]
    pub max_preds_per_round: usize,
}

pub fn run_generate_predictions(
    samples: Vec<Sample>,
    config: &PredictionConfig,
) -> Result<Vec<Sample>, Box<dyn Error>> {
    let pb = ProgressBar::new(samples.len() as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );

    let samples_with_preds = samples
        .into_par_iter()
        .progress_with(pb)
        .map(
            |sample| match generate_predictions(&sample.instance, &sample.solution, &config) {
                Ok(preds) => Ok(Sample {
                    predictions: preds,
                    ..sample
                }),
                Err(e) => Err(e),
            },
        )
        .filter_map(Result::ok)
        .collect();

    Ok(samples_with_preds)
}
