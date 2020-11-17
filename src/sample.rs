use kserver::prelude::*;
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
