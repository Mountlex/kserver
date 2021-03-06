pub mod algorithms;

use samplelib::*;
use crate::algorithms::*;


pub fn simulate_kserver(sample: &Sample, gamma: f64, lambda: f32, lazy: bool) -> Vec<SimResult> {


    let (dc_schedule, mut dc_cost) =  deterministic_alg(DoubleCoverage, &sample.instance);
    if lazy {
        dc_cost = dc_schedule.to_lazy(&sample.instance).cost();
    }
    let results = sample
        .predictions
        .iter()
        .map(|pred| {
            let (alg_schedule, mut alg_cost) = learning_augmented_alg(LambdaDC::new(lambda), &sample.instance, pred);
            if lazy {
                alg_cost = alg_schedule.to_lazy(&sample.instance).cost();
            }
            let (combine_schedule, mut combine_cost) = learning_augmented_alg(CombineDet::new(gamma), &sample.instance, pred);
            if lazy {
                combine_cost = combine_schedule.to_lazy(&sample.instance).cost();

            }

            let eta = pred.eta(&sample.solution, &sample.instance);
            let k = sample.instance.k() as f64;
            if alg_cost as f64 > (1.0 + (k - 1.0) * lambda as f64) * (sample.opt_cost as f64 + 2.0 * eta as f64) {
                println!("LambdaDC does not achieve the theoretical competitive ratio: {} > (1+{})({} + 2{})",
                    alg_cost, lambda, sample.opt_cost, eta);
            }    
            if lambda == 0.0 && eta == 0.0 && alg_cost as f64 != sample.opt_cost as f64 {
                println!("LambdaDC with lambda = eta = 0, but ALG = {} != {} = OPT", alg_cost, sample.opt_cost);
            }  
            if (alg_cost as f64) < sample.opt_cost as f64 {
                println!("LambdaDC ALG = {} < {} = OPT", alg_cost, sample.opt_cost);
            }  
            let cost_list: Vec<(String, f64)> = vec![("DC".into(), dc_cost), ("LDC".into(), alg_cost), ("RobustFtp".into(), combine_cost)];
            
            let res = SimResult {
                instance: sample.instance.clone(),
                opt_cost: sample.opt_cost,
                eta,
                alg_costs: cost_list,
                lambda,
            };
            res.into()
        })
        .collect::<Vec<SimResult>>();
    results
}