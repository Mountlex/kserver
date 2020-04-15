use crate::algorithm::{double_coverage, lambda_dc};
use crate::instance::*;
use crate::sample_generator::*;
use crate::schedule::CostMetric;
use crate::schedule::Schedule;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::error::Error;
use std::fmt;
use structopt::StructOpt;

use itertools_num::linspace;

#[derive(StructOpt, Debug)]
pub struct SimConfig {
    #[structopt(long = "lambdas", default_value = "5")]
    pub number_of_lambdas: usize,
}

#[derive(Debug)]
pub struct SimulatorError {
    msg: String,
}

#[allow(dead_code)]
impl SimulatorError {
    fn new(msg: String) -> SimulatorError {
        SimulatorError { msg: msg }
    }
}

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

#[derive(Clone, Debug)]
pub struct KServerResult {
    pub instance: Instance,
    pub solution: Schedule,
    pub eta: u32,
    pub dc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32
}

impl From<KServerResult> for SimResult {
    fn from(result: KServerResult) -> SimResult {
        SimResult::KServer(result)
    }
}

impl From<KServerSimArgs> for SimArgs {
    fn from(args: KServerSimArgs) -> SimArgs {
        SimArgs::KServer(args)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KServerSimArgs {
    lambda: f32
}

#[derive(Copy, Clone, Debug)]
pub struct KTaxiSimArgs {
    lambda: f32
}

pub enum SimArgs {
    KServer(KServerSimArgs),
    KTaxi(KTaxiSimArgs)
}

pub enum SimResult {
    KServer(KServerResult),
    KTaxi(KServerResult)
}

impl SimResult {
    fn invalid_result(&self) -> bool {
        match self {
            SimResult::KServer(res) => 
                res.lambda == 0.0 && res.eta == 0 && res.alg_cost != res.solution.costs(),
            SimResult::KTaxi(res) => 
                res.lambda == 0.0 && res.eta == 0 && res.alg_cost != res.solution.costs() 
        }
    }
}

impl KServerSample {
    fn simulate(&self, args: KServerSimArgs) -> Result<Vec<SimResult>, SimulatorError> {
            let dc_cost = double_coverage(&self.instance).costs();
            let results = self.predictions
                .iter()
                .map(|pred| {
                        let alg = lambda_dc(&self.instance, pred, args.lambda);
                        let alg_cost = alg.costs();
                        let eta = pred.diff(&self.solution);
                        let res = KServerResult {
                            instance: self.instance.clone(),
                            solution: self.solution.to_vec(),
                            eta,
                            dc_cost: dc_cost,
                            alg_cost: alg_cost,
                            lambda: args.lambda,
                        };
                        res.into()
                    })
                    .collect::<Vec<SimResult>>();
            Ok(results)
    }
}

impl KTaxiSample {
    fn simulate(&self, args: KTaxiSimArgs) -> Result<Vec<SimResult>, SimulatorError> {
       Err(SimulatorError::new("Not implemented".to_string()))
    }
}

impl Sample {
    fn simulate(self: &Sample, sim_args: SimArgs) -> Result<Vec<SimResult>, SimulatorError> {
        match (self, sim_args) {
            (Sample::KServer(sample), SimArgs::KServer(args)) => sample.simulate(args),
            (Sample::KTaxi(sample), SimArgs::KTaxi(args)) => sample.simulate(args),
            _ => Err(SimulatorError::new("Invalid combination of sample and arguments!".to_string()))
        }
    }
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
        .map(|lambda| {
            sample.simulate(KServerSimArgs {lambda: *lambda}.into())
        })
        .filter_map(Result::ok)
        .flatten()
        .collect::<Vec<SimResult>>();

    if results.iter().any(|res| res.invalid_result()) {
        return Err(SimulatorError::new("Invalid result".to_string()));
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
        .into_par_iter()
        .progress_with(pb)
        .map(|sample| simulate_sample(sample, &lambdas))
        .filter_map(Result::ok)
        .collect::<Vec<Vec<SimResult>>>();

    let res = results.into_iter().flatten().collect::<Vec<SimResult>>();
    Ok(res)
}
