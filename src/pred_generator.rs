use serverlib::prelude::*;

use samplelib::*;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PredictionConfig {
    #[structopt(short = "p", long = "preds", default_value = "12")]
    pub number_of_predictions: usize,

    #[structopt(short = "b", long = "preds_per_bin", default_value = "2")]
    pub preds_per_bin: usize,

    #[structopt(short = "s", long = "preds_samples_per_round", default_value = "200")]
    pub number_of_samples_per_round: usize,
}

trait PredictionAdder {
    fn add_predictions(self, config: &PredictionConfig) -> Result<Sample, PredictionError>;
}

impl PredictionAdder for Sample {
    fn add_predictions(self, config: &PredictionConfig) -> Result<Sample, PredictionError> {
        match generate_predictions(&self.instance, &self.solution, self.opt_cost, config) {
            Ok(preds) => Ok(Sample {
                predictions: preds,
                ..self
            }),
            Err(e) => Err(e),
        }
    }
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
        .map(|sample| sample.add_predictions(config))
        .filter_map(Result::ok)
        .collect();

    Ok(samples_with_preds)
}

/// Lower and upper inclusive
fn predict(lower: usize, upper: usize) -> usize {
    if lower == upper {
        return lower;
    }
    let mut rng = rand::thread_rng();
    rng.gen_range(lower, upper + 1)
}

pub fn generate_predictions(
    instance: &Instance,
    solution: &Schedule,
    opt_cost: u32,
    config: &PredictionConfig,
) -> Result<Vec<Prediction>, PredictionError> {
    let mut predictions: Vec<Prediction> =
        Vec::with_capacity(instance.length() * config.number_of_samples_per_round + 1);

    let perfect_prediction = solution.to_prediction(instance);
    let ref_perfect_prediction = &solution.to_prediction(instance);
    predictions.push(perfect_prediction);

    let mut rng = rand::thread_rng();
    let dist = Uniform::from(0..instance.length());
    for number_of_wrong_servers in 1..instance.length() {
        for _ in 1..config.number_of_samples_per_round {
            let mut correct_preds = vec![true; instance.length()];
            (1..number_of_wrong_servers).for_each(|_| correct_preds[dist.sample(&mut rng)] = false);

            let mut pred_vec = vec![];
            for (i, &server) in ref_perfect_prediction.into_iter().enumerate() {
                let k = instance.k();
                if correct_preds[i] {
                    pred_vec.push(server);
                } else {
                    let p = predict(0, k - 1);
                    if p == server {
                        if p == 0 {
                            pred_vec.push(p + 1);
                        } else {
                            pred_vec.push(p - 1);
                        }
                    } else {
                        pred_vec.push(p);
                    }
                }
            }

            predictions.push(Prediction::from(pred_vec));
        }
    }

    let preds_with_error: Vec<(Prediction, usize)> = predictions
        .into_iter()
        .map(|pred| {
            let pred_schedule = pred.to_schedule(instance);
            let eta = solution.diff(&pred_schedule).floor() as usize;
            (pred, eta)
        })
        .collect();
    let max_error = preds_with_error
        .iter()
        .max_by_key(|(_, error)| error)
        .unwrap()
        .1;
    let bin_size = max_error as f64 / config.number_of_predictions as f64;
    let mut bins: HashMap<usize, Vec<Prediction>> = HashMap::new();
    for (pred, error) in preds_with_error {
        let bin = (error as f64 / bin_size).ceil() as usize;
        if bin < config.number_of_predictions {
            let preds_in_bin = bins.entry(bin).or_default();
            preds_in_bin.push(pred);
        }
    }

    for bin in bins.values_mut() {
        bin.truncate(config.number_of_predictions);
    }

    let final_predictions = bins.into_iter().map(|(_, bin)| bin).flatten().collect();

    Ok(final_predictions)
}
