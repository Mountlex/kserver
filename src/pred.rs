use crate::instance::Instance;
use crate::seq::Sequence;

pub struct PredictionConfig {
    pub number_of_preds: u32,
    pub step_size: f32,
    pub number_of_samples_per_round: u32,
    pub max_preds_per_round: u32,
}

pub fn generate(
    instance: &Instance,
    solution: &Sequence,
    config: &PredictionConfig,
) -> Option<Vec<Sequence>> {
    let mut stepToPredictions: Vec<Vec<Sequence>> = vec![vec![]; config.number_of_preds as usize];

    None
}
