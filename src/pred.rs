use crate::instance::Instance;
use crate::seq::{CostMetric, Sequence, SequenceCreation};
use crate::server_config::ServerConfig;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use std::cmp::{max, min};
use std::error::Error;
use std::fmt;

pub struct PredictionConfig {
    pub number_of_preds: usize,
    pub step_size: f32,
    pub number_of_samples_per_round: usize,
    pub max_preds_per_round: usize,
}

#[derive(Debug, Clone)]
pub struct PredictionError {
    msg: String,
}

impl fmt::Display for PredictionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Not all required predictions have been found: {}!",
            self.msg
        )
    }
}

impl Error for PredictionError {
    fn description(&self) -> &str {
        "Not all required predictions have been found!"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl PredictionError {
    fn new(msg: String) -> PredictionError {
        PredictionError { msg: msg }
    }
}

pub fn generate_predictions(
    instance: &Instance,
    solution: &Sequence,
    config: &PredictionConfig,
) -> Result<Vec<Sequence>, PredictionError> {
    let mut step_to_predictions: Vec<Vec<Sequence>> = vec![vec![]; config.number_of_preds as usize];

    step_to_predictions[0].push(solution.to_vec());

    let mut rng = rand::thread_rng();
    let dist = Uniform::from(0..instance.length());

    for number_of_wrong_servers in 1..instance.length() {
        for _ in 1..config.number_of_samples_per_round {
            let mut correct_preds = vec![true; instance.length()];
            (1..number_of_wrong_servers).for_each(|_| correct_preds[dist.sample(&mut rng)] = false);

            let mut pred = Sequence::new_seq(instance.initial_positions().to_vec());
            for (i, (last, config)) in solution.iter().zip(solution.iter().skip(1)).enumerate() {
                if correct_preds[i] {
                    pred.append_config(config.to_vec());
                } else {
                    match last.moved_server(config) {
                        Some(server) => {
                            if rand::random() {
                                if server > 0 {
                                    pred.append_move(server - 1, instance.requests()[i]);
                                } else {
                                    pred.append_move(server + 1, instance.requests()[i]);
                                }
                            } else {
                                if server < instance.k() - 1 {
                                    pred.append_move(server + 1, instance.requests()[i]);
                                } else {
                                    pred.append_move(server - 1, instance.requests()[i]);
                                }
                            }
                        }
                        None => pred.append_config(config.to_vec()),
                    }
                }
            }

            let eta = solution.diff(&pred);
            let ratio = eta as f32 / solution.costs() as f32;
            let bin_index: usize = (ratio / config.step_size).ceil() as usize;

            if bin_index < config.number_of_preds
                && step_to_predictions[bin_index].len() < config.max_preds_per_round
            {
                step_to_predictions[bin_index].push(pred);
            }

            if !step_to_predictions.iter().any(|preds| preds.is_empty()) {
                break;
            }
        }

        if !step_to_predictions.iter().any(|preds| preds.is_empty()) {
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
            .map(|preds| preds.choose(&mut rng).unwrap().to_vec())
            .collect())
    }
}
