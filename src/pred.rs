use crate::cost::CostMetric;
use crate::instance::Instance;
use crate::pred_generator::PredictionConfig;
use crate::schedule::Schedule;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use rand::Rng;
use std::error::Error;
use std::fmt;
use std::iter::FromIterator;

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

#[derive(Clone, Debug)]
pub struct Prediction(Vec<usize>);

impl From<Vec<usize>> for Prediction {
    fn from(servers: Vec<usize>) -> Prediction {
        Prediction(servers)
    }
}

impl FromIterator<usize> for Prediction {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        Prediction(iter.into_iter().collect())
    }
}

impl IntoIterator for Prediction {
    type Item = usize;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Prediction {
    fn to_schedule(&self, instance: &Instance) -> Schedule {
        let mut schedule = Schedule::from(instance.initial_positions().clone());

        for (idx, req) in instance.requests().iter().enumerate() {
            schedule.append_move(self.0[idx], req.t);
        }
        schedule.normalize();
        schedule
    }

    pub fn get_eta(&self, solution: &Schedule, instance: &Instance) -> u32 {
        let pred_schedule = self.to_schedule(instance);
        return solution.diff(&pred_schedule);
    }

    pub fn get_predicted_server(&self, request_index: usize) -> usize {
        return self.0[request_index];
    }
}

// TODO
pub fn to_prediction(schedule: &Schedule, instance: &Instance) -> Prediction {
    schedule
        .into_iter()
        .skip(1)
        .enumerate()
        .map(|(idx, config)| {
            config
                .into_iter()
                .enumerate()
                .find(|(_, &server)| server == instance.requests()[idx].t)
                .map(|(i, _)| i)
                .unwrap_or_else(|| panic!("Cannot find predicted server. Please investigate!\nSolution={:?} Instance={}", schedule, instance))
        })
        .collect::<Prediction>()
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
    let mut step_to_predictions: Vec<Vec<Prediction>> =
        vec![vec![]; config.number_of_predictions as usize];

    let perfect_prediction = to_prediction(solution, instance);
    let ref_perfect_prediction = &to_prediction(solution, instance);
    step_to_predictions[0].push(perfect_prediction);

    let mut rng = rand::thread_rng();
    let dist = Uniform::from(0..instance.length());

    for number_of_wrong_servers in 1..instance.length() {
        for _ in 1..config.number_of_samples_per_round {
            let mut correct_preds = vec![true; instance.length()];
            (1..number_of_wrong_servers).for_each(|_| correct_preds[dist.sample(&mut rng)] = false);

            let mut pred_vec = vec![];
            for (i, &server) in ref_perfect_prediction.0.iter().enumerate() {
                let k = instance.k();
                if correct_preds[i] {
                    pred_vec.push(server);
                } else {
                    if rand::random::<bool>() {
                        // left
                        if server > 0 {
                            //left
                            pred_vec.push(predict(0, server - 1));
                        } else {
                            //right
                            pred_vec.push(predict(server + 1, k - 1));
                        }
                    } else {
                        // right
                        if server < k - 1 {
                            // right
                            pred_vec.push(predict(server + 1, k - 1));
                        } else {
                            // left
                            pred_vec.push(predict(0, server - 1));
                        }
                    }
                }
            }

            let pred = Prediction::from(pred_vec);
            let pred_schedule = pred.to_schedule(instance);
            let eta = solution.diff(&pred_schedule);
            let ratio = eta as f32 / opt_cost as f32;
            let bin_index: usize = (ratio / config.step_size).ceil() as usize;

            if bin_index < config.number_of_predictions
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
            .map(|preds| preds.choose(&mut rng).unwrap().clone())
            .collect())
    }
}
