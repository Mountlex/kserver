use crate::instance::Instance;
use crate::pred::Prediction;
use crate::schedule::normalize_schedule;
use crate::schedule::Schedule;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct Sample {
    pub instance: Instance,
    pub solution: Schedule,
    pub opt_cost: u32,
    pub predictions: Vec<Prediction>,
}

impl Sample {
    pub fn new(instance: Instance, solution: Schedule, opt_cost: u32) -> Sample {
        Sample {
            instance,
            solution,
            opt_cost,
            predictions: vec![],
        }
    }
}

impl Sample {
    pub fn normalize(self) -> Result<Sample, Box<dyn Error>> {
        if !self.instance.is_taxi_instance() {
            match normalize_schedule(self.solution) {
                Ok(s) => Ok(Sample {
                    solution: s,
                    ..self
                }),
                Err(e) => Err(e),
            }
        } else {
            Ok(self)
        }
    }
}
