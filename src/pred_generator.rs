use serverlib::prelude::*;

use samplelib::*;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::SliceRandom;
use rand::Rng;
use rayon::prelude::*;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct PredictionConfig {
    #[structopt(short = "p", long = "preds", default_value = "12")]
    pub number_of_predictions: usize,

    #[structopt(short = "b", long = "preds_bin_size", default_value = "0.25")]
    pub step_size: f32,

    #[structopt(short = "s", long = "preds_samples_per_round", default_value = "200")]
    pub number_of_samples_per_round: usize,

    #[structopt(short = "m", long = "preds_per_bin", default_value = "5")]
    pub preds_per_bin: usize,
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
    rng.gen_range(lower..upper + 1)
}

pub fn generate_predictions(
    instance: &Instance,
    solution: &Schedule,
    opt_cost: u32,
    config: &PredictionConfig,
) -> Result<Vec<Prediction>, PredictionError> {
    let mut step_to_predictions: Vec<Vec<(Prediction, f32)>> =
        vec![vec![]; config.number_of_predictions as usize];

    let perfect_prediction = solution.to_prediction(instance);
    let ref_perfect_prediction = &solution.to_prediction(instance);
    step_to_predictions[0].push((perfect_prediction, 0.0));

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

            let pred = Prediction::from(pred_vec);
            let pred_schedule = pred.to_schedule(instance);
            // println!(
            //     "Instance length {}, Solution length {}, Pred length {}",
            //     instance.length(),
            //     solution.size(),
            //     pred.0.len()
            // );
            let eta = solution.diff(&pred_schedule);
            let ratio = eta as f32 / opt_cost as f32;
            let bin_index: usize = (ratio / config.step_size).ceil() as usize;

            if bin_index < config.number_of_predictions {
                step_to_predictions[bin_index].push((pred, ratio as f32));
            }

            if !step_to_predictions
                .iter()
                .any(|preds| preds.len() < config.preds_per_bin)
            {
                break;
            }
        }

        if !step_to_predictions
            .iter()
            .any(|preds| preds.len() < config.preds_per_bin)
        {
            break;
        }
    }

    let missing_bins: Vec<usize> = step_to_predictions
        .iter()
        .enumerate()
        .filter(|(_, preds)| preds.is_empty())
        .map(|(i, _)| i)
        .collect();
    if !missing_bins.is_empty() {
        let msgs: Vec<String> = missing_bins
            .into_iter()
            .map(|i| {
                format!(
                    "{} - {}",
                    i as f32 * config.step_size,
                    (i + 1) as f32 * config.step_size
                )
            })
            .collect();

        return Err(PredictionError::new(format!("{} missing!", msgs.join(","))));
    } else {
        Ok(step_to_predictions
            .into_iter()
            .map(|mut preds| {
                preds.sort_by(|(_, err1), (_, err2)| err1.partial_cmp(err2).unwrap());
                let largest = preds.last().unwrap().0.clone();
                preds.swap_remove(preds.len() - 1);
                let mut others = preds
                    .choose_multiple(&mut rng, config.preds_per_bin - 1)
                    .map(|(pred, _)| pred.clone())
                    .collect::<Vec<Prediction>>();
                others.push(largest);
                others
            })
            .flatten()
            .collect())
    }
}
