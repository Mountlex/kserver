use crate::algorithm::{double_coverage, lambda_dc};
use crate::instance::Instance;
use crate::sample::Sample;
use crate::seq::CostMetric;
use crate::seq::Sequence;
use console::style;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use std::error::Error;
use std::fmt;
use structopt::StructOpt;

use itertools_num::linspace;

#[derive(StructOpt, Debug)]
pub struct SimConfig {
    #[structopt(long = "num_lambdas", default_value = "5")]
    pub number_of_lambdas: usize,
}

#[derive(Debug)]
pub struct SimulatorError;

impl fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid sample")
    }
}

impl Error for SimulatorError {
    fn description(&self) -> &str {
        "Invalid sample!"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

pub struct SimResult {
    pub instance: Instance,
    pub solution: Sequence,
    pub eta: u32,
    pub dc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32,
}

pub fn run(samples: Vec<Sample>, config: &SimConfig) -> Result<Vec<SimResult>, Box<dyn Error>> {
    println!("{}", style("Start simulating...").bold().cyan());
    let results = simulate_samples(
        samples,
        linspace::<f32>(0., 1., config.number_of_lambdas)
            .into_iter()
            .collect::<Vec<f32>>(),
    )?;
    println!("{}", style("Simulation finished!").bold().green());

    Ok(results)
}

fn simulate_sample(sample: Sample, lambdas: &Vec<f32>) -> Result<Vec<SimResult>, SimulatorError> {
    let results = lambdas
        .iter()
        .flat_map(|lambda| {
            let dc_cost = double_coverage(&sample.instance).costs();
            sample
                .predictions
                .iter()
                .map(|pred| {
                    let alg = lambda_dc(&sample.instance, pred, *lambda);
                    let alg_cost = alg.costs();
                    let eta = pred.diff(&sample.solution);
                    SimResult {
                        instance: sample.instance.clone(),
                        solution: sample.solution.to_vec(),
                        eta,
                        dc_cost: dc_cost,
                        alg_cost: alg_cost,
                        lambda: *lambda,
                    }
                })
                .collect::<Vec<SimResult>>()
        })
        .collect::<Vec<SimResult>>();

    if results
        .iter()
        .any(|res| res.lambda == 0.0 && res.eta == 0 && res.alg_cost != res.solution.costs())
    {
        return Err(SimulatorError);
    }

    return Ok(results);
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
        .map(|sample| simulate_sample(sample, &lambdas))
        .filter_map(Result::ok)
        .collect::<Vec<Vec<SimResult>>>();

    let res = results.into_iter().flatten().collect::<Vec<SimResult>>();
    Ok(res)
}
