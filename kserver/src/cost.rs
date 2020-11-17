use crate::server_config::ServerConfiguration;
use crate::schedule::Schedule;

pub trait CostMetric<T> {
    fn diff(&self, other: &Self) -> T;
}

impl CostMetric<f64> for ServerConfiguration {
    fn diff(&self, other: &ServerConfiguration) -> f64 {
        return self
            .into_iter()
            .zip(other.into_iter())
            .map(|(a, b)| (a - b).abs() as f64)
            .sum::<f64>();
    }
}

impl CostMetric<f64> for Schedule {
    fn diff(&self, other: &Self) -> f64 {
        if self.len() != other.len() {
            panic!("Schedules must have same size!")
        }
        return self
            .into_iter()
            .zip(other.into_iter())
            .map(|(c1, c2)| c1.diff(c2) as f64)
            .sum();
    }
}