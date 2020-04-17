use crate::algorithm::*;
use crate::results::KServerResult;
use crate::results::KTaxiResult;
use crate::results::SimResult;
use crate::sample::KServerSample;
use crate::sample::KTaxiSample;
use crate::sample::Sample;
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

impl From<KServerSimArgs> for SimArgs {
    fn from(args: KServerSimArgs) -> SimArgs {
        SimArgs::KServer(args)
    }
}

impl From<KTaxiSimArgs> for SimArgs {
    fn from(args: KTaxiSimArgs) -> SimArgs {
        SimArgs::KTaxi(args)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KServerSimArgs {
    lambda: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct KTaxiSimArgs {
    lambda: f32,
}

pub enum SimArgs {
    KServer(KServerSimArgs),
    KTaxi(KTaxiSimArgs),
}

impl KServerSample {
    fn simulate(&self, args: KServerSimArgs) -> Result<Vec<SimResult>, SimulatorError> {
        let dc_cost = double_coverage(&self.instance).1;
        let results = self
            .predictions
            .iter()
            .map(|pred| {
                let alg_cost = lambda_dc(&self.instance, pred, args.lambda).1;
                let eta = pred.get_eta(&self.solution, &self.instance);
                let res = KServerResult {
                    instance: self.instance.clone(),
                    opt_cost: self.opt_cost,
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
        let bdc = biased_dc(&self.instance);
        let bdc_cost = bdc.1;
        if bdc_cost as f32 / self.opt_cost as f32 > 9.0 {
            println!(
                "competitive ratio of biasedDC > 9: {}, BiasedDC={:?}, Opt={:?}",
                self.instance, bdc.0, self.solution
            );
        }
        let results = self
            .predictions
            .iter()
            .map(|pred| {
                let alg_cost = lambda_biased_dc(&self.instance, pred, args.lambda).1;
                let eta = pred.get_eta(&self.solution, &self.instance);
                let res = KTaxiResult {
                    instance: self.instance.clone(),
                    opt_cost: self.opt_cost,
                    eta,
                    bdc_cost: bdc_cost,
                    alg_cost: alg_cost,
                    lambda: args.lambda,
                };
                res.into()
            })
            .collect::<Vec<SimResult>>();
        Ok(results)
    }
}

impl Sample {
    fn simulate(self: &Sample, sim_args: SimArgs) -> Result<Vec<SimResult>, SimulatorError> {
        match (self, sim_args) {
            (Sample::KServer(sample), SimArgs::KServer(args)) => sample.simulate(args),
            (Sample::KTaxi(sample), SimArgs::KTaxi(args)) => sample.simulate(args),
            _ => Err(SimulatorError::new(
                "Invalid combination of sample and arguments!".to_string(),
            )),
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
    println!(
        "{} {}",
        style("Number of results: ").bold().green(),
        style(results.len()).bold().red()
    );
    Ok(results)
}

fn simulate_sample(sample: Sample, lambdas: &Vec<f32>) -> Result<Vec<SimResult>, SimulatorError> {
    let results = lambdas
        .iter()
        .map(|lambda| {
            let args = match sample {
                Sample::KServer(_) => KServerSimArgs { lambda: *lambda }.into(),
                Sample::KTaxi(_) => KTaxiSimArgs { lambda: *lambda }.into(),
            };
            sample.simulate(args)
        })
        .filter_map(Result::ok)
        .flatten()
        .collect::<Vec<SimResult>>();

    if results.iter().any(|res| res.is_invalid()) {
        return Err(SimulatorError::new("Invalid result".to_string()));
    }
    //return Ok(results
    //    .into_iter()
    //    .filter(|res| !res.invalid_result())
    //    .collect());
    return Ok(results);
}

fn simulate_samples(
    samples: Vec<Sample>,
    lambdas: Vec<f32>,
) -> Result<Vec<SimResult>, Box<dyn Error>> {
    let number_of_samples = samples.len();
    let pb = ProgressBar::new(number_of_samples as u64);

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

    let failed_simulations = number_of_samples * lambdas.len() - results.len();

    println!(
        "{} {}",
        style("Samples with invalid simulation: ").bold().green(),
        style(failed_simulations).bold().red()
    );

    let res = results.into_iter().flatten().collect::<Vec<SimResult>>();
    Ok(res)
}
