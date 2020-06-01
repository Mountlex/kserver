use crate::algorithm::*;
use crate::results::SimResult;
use crate::sample::Sample;
use console::style;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use itertools_num::linspace;
use rayon::prelude::*;
use std::error::Error;
use std::fmt;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Copy, Clone)]
pub struct SimConfig {
    #[structopt(long = "lambdas", default_value = "5")]
    pub number_of_lambdas: usize,
}

#[derive(StructOpt, Debug, Copy, Clone)]
pub enum Simulators {
    #[structopt(name = "kserver")]
    KServer(SimConfig),
    #[structopt(name = "ktaxi")]
    KTaxi(SimConfig),
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

fn simulate_kserver(sample: &Sample, lambda: f32) -> Result<Vec<SimResult>, SimulatorError> {
    let dc_cost = double_coverage(&sample.instance).1;
    let results = sample
        .predictions
        .iter()
        .map(|pred| {
            let (_, alg_cost) = lambda_dc(&sample.instance, pred, lambda);
            let eta = pred.get_eta(&sample.solution, &sample.instance);
            let k = sample.instance.k() as f64;
            if alg_cost as f64 > (1.0 + (k - 1.0) * lambda as f64) * (sample.opt_cost as f64 + 2.0 * eta as f64) {
                println!("LambdaDC does not achieve the theoretical competitive ratio: {} > (1+{})({} + 2{})",
                    alg_cost, lambda, sample.opt_cost, eta);
            }            
            let res = SimResult {
                instance: sample.instance.clone(),
                opt_cost: sample.opt_cost,
                eta,
                dc_cost: dc_cost,
                alg_cost: alg_cost,
                lambda: lambda,
            };
            res.into()
        })
        .collect::<Vec<SimResult>>();
    Ok(results)
}

fn simulate_ktaxi(sample: &Sample, lambda: f32) -> Result<Vec<SimResult>, SimulatorError> {
    let bdc = biased_dc(&sample.instance);
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
            let (_, alg_cost) = lambda_biased_dc(&sample.instance, pred, lambda);
            let eta = pred.get_eta(&sample.solution, &sample.instance);
            let res = SimResult {
                instance: sample.instance.clone(),
                opt_cost: sample.opt_cost,
                eta,
                dc_cost: bdc_cost,
                alg_cost: alg_cost,
                lambda: lambda,
            };
            res.into()
        })
        .collect::<Vec<SimResult>>();
    Ok(results)
}

impl Sample {
    fn simulate(
        self: &Sample,
        simulator: Simulators,
        lambda: f32,
    ) -> Result<Vec<SimResult>, SimulatorError> {
        match simulator {
            Simulators::KServer(_) => simulate_kserver(self, lambda),
            Simulators::KTaxi(_) => simulate_ktaxi(self, lambda),
        }
    }
}

pub fn run(samples: Vec<Sample>, simulator: Simulators) -> Result<Vec<SimResult>, Box<dyn Error>> {
    println!("{}", style("Start simulating...").bold().cyan());
    let number_of_lambdas = match simulator {
        Simulators::KTaxi(config) => config.number_of_lambdas,
        Simulators::KServer(config) => config.number_of_lambdas,
    };
    let lambdas = linspace::<f32>(0., 1., number_of_lambdas)
        .into_iter()
        .collect::<Vec<f32>>();
    let results = simulate_samples(samples, lambdas, simulator)?;
    println!("{}", style("Simulation finished!").bold().green());
    println!(
        "{} {}",
        style("Number of results: ").bold().green(),
        style(results.len()).bold().red()
    );
    Ok(results)
}

fn simulate_sample(
    sample: Sample,
    lambdas: &Vec<f32>,
    simulator: Simulators,
) -> Result<Vec<SimResult>, SimulatorError> {
    let results = lambdas
        .iter()
        .map(|lambda| sample.simulate(simulator, *lambda))
        .filter_map(Result::ok)
        .flatten()
        .collect::<Vec<SimResult>>();

    if results.iter().any(|res| res.is_invalid()) {
        return Err(SimulatorError::new("Invalid result".to_string()));
    }
    return Ok(results);
}

fn simulate_samples(
    samples: Vec<Sample>,
    lambdas: Vec<f32>,
    simulator: Simulators,
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
        .map(|sample| simulate_sample(sample, &lambdas, simulator))
        .filter_map(Result::ok)
        .collect::<Vec<Vec<SimResult>>>();

    let failed_simulations = number_of_samples - results.len();
    if failed_simulations > 0 {
        println!(
            "{} {}",
            style("Samples with invalid simulation: ").bold().green(),
            style(failed_simulations).bold().red()
        );
    }
    let res = results.into_iter().flatten().collect::<Vec<SimResult>>();
    Ok(res)
}
