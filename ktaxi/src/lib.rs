pub mod algorithms;

use samplelib::*;
use crate::algorithms::*;

pub fn simulate_ktaxi(sample: &Sample, lambda: f32) -> Vec<SimResult> {
    let bdc = deterministic_alg(BiasedDC, &sample.instance);
    let bdc_cost = bdc.1;
    if bdc_cost as f32 / sample.opt_cost as f32 > 9.0 {
        println!(
            "competitive ratio of biasedDC > 9: {}, BiasedDC={:?}, Opt={:?}",
            sample.instance, bdc.0, sample.solution
        );
    }
    let results = sample
        .predictions
        .iter()
        .map(|pred| {
            let (_, alg_cost) = learning_augmented_alg(LambdaBiasedDC::new(lambda), &sample.instance, pred);
            let eta = pred.eta(&sample.solution, &sample.instance);

            let cost_list: Vec<(String, f64)> = vec![("BDC".into(), bdc_cost), ("LBDC".into(), alg_cost)];

            let res = SimResult {
                instance: sample.instance.clone(),
                opt_cost: sample.opt_cost,
                eta,
                alg_costs: cost_list,
                lambda: lambda,
            };
            res.into()
        })
        .collect::<Vec<SimResult>>();
    results
}