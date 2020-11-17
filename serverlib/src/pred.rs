use crate::cost::CostMetric;
use crate::instance::Instance;
use crate::schedule::Schedule;
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
    pub fn new(msg: String) -> PredictionError {
        PredictionError { msg: msg }
    }
}

#[derive(Clone, Debug, Default)]
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

impl<'a> IntoIterator for &'a Prediction {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl std::ops::Index<usize> for Prediction {
    type Output = usize;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl Prediction {
    pub fn to_schedule(&self, instance: &Instance) -> Schedule {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());

        for (idx, req) in instance.requests().iter().enumerate() {
            schedule.append_move(self[idx], req.distance_to(&0.0));
        }
        //schedule.normalize();
        schedule
    }

    pub fn eta(&self, solution: &Schedule, instance: &Instance) -> f64 {
        let pred_schedule = self.to_schedule(instance);
        return solution.diff(&pred_schedule);
    }

    pub fn predicted_server(&self, request_index: usize) -> usize {
        return self[request_index];
    }
}

