use crate::algorithm::{double_coverage, lambda_dc};
use crate::instance::Instance;
use crate::sample::Sample;
use crate::seq::CostMetric;
use crate::seq::Sequence;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use structopt::StructOpt;

use itertools_num::linspace;

#[derive(StructOpt, Debug)]
pub struct SimConfig {
    #[structopt(long = "num_lambdas", default_value = "5")]
    pub number_of_lambdas: usize,
}

pub struct SimResult {
    pub instance: Instance,
    pub solution: Sequence,
    pub eta: u32,
    pub dcCost: u32,
    pub algCost: u32,
    pub lambda: f32,
}

pub fn run(samples: Vec<Sample>, config: &SimConfig) -> Result<Vec<SimResult>, Box<dyn Error>> {
    println!("Starting simulation...");
    let results = simulate_samples(
        samples,
        linspace::<f32>(0., 1., config.number_of_lambdas)
            .into_iter()
            .collect::<Vec<f32>>(),
    )?;
    println!("Simulation finished!");

    Ok(results)
}

fn simulate_samples(
    samples: Vec<Sample>,
    lambdas: Vec<f32>,
) -> Result<Vec<SimResult>, Box<dyn Error>> {
    let pb = ProgressBar::new(samples.len() as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );

    let results: Vec<Vec<SimResult>> = samples
        .into_iter()
        .progress_with(pb)
        .map(|sample| {
            lambdas
                .iter()
                .flat_map(|lambda| {
                    let dcCost = double_coverage(&sample.instance).costs();
                    sample
                        .predictions
                        .iter()
                        .map(|pred| {
                            let algCost = lambda_dc(&sample.instance, pred, *lambda).costs();
                            SimResult {
                                instance: sample.instance.clone(),
                                solution: sample.solution.to_vec(),
                                eta: pred.diff(&sample.solution),
                                dcCost,
                                algCost,
                                lambda: *lambda,
                            }
                        })
                        .collect::<Vec<SimResult>>()
                })
                .collect::<Vec<SimResult>>()
        })
        .collect::<Vec<Vec<SimResult>>>();

    let res = results.into_iter().flatten().collect::<Vec<SimResult>>();
    Ok(res)
}
