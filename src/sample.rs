use crate::instance::Instance;
use crate::pred::Prediction;
use crate::schedule::normalize_schedule;
use crate::schedule::Schedule;
use std::error::Error;

#[derive(Clone, Debug)]
pub enum Sample {
    KServer(KServerSample),
    KTaxi(KTaxiSample),
}

#[derive(Clone, Debug)]
pub struct KServerSample {
    pub instance: Instance,
    pub solution: Schedule,
    pub opt_cost: u32,
    pub predictions: Vec<Prediction>,
}

#[derive(Clone, Debug)]
pub struct KTaxiSample {
    pub instance: Instance,
    pub solution: Schedule,
    pub opt_cost: u32,
    pub predictions: Vec<Prediction>,
}

impl KTaxiSample {
    pub fn new(instance: Instance, solution: Schedule, opt_cost: u32) -> KTaxiSample {
        KTaxiSample {
            instance,
            solution,
            opt_cost,
            predictions: vec![],
        }
    }
}

impl KServerSample {
    pub fn new(instance: Instance, solution: Schedule, opt_cost: u32) -> KServerSample {
        KServerSample {
            instance,
            solution,
            opt_cost,
            predictions: vec![],
        }
    }
}

impl From<KServerSample> for Sample {
    fn from(sample: KServerSample) -> Sample {
        Sample::KServer(sample)
    }
}

impl From<KTaxiSample> for Sample {
    fn from(sample: KTaxiSample) -> Sample {
        Sample::KTaxi(sample)
    }
}

impl Sample {
    pub fn normalize(self) -> Result<Sample, Box<dyn Error>> {
        match self {
            Sample::KServer(sample) => match normalize_schedule(sample.solution) {
                Ok(s) => Ok(KServerSample::new(sample.instance, s, sample.opt_cost).into()),
                Err(e) => Err(e),
            },
            Sample::KTaxi(sample) => Ok(sample.into()),
        }
    }
}
