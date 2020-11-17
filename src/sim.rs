use kserver::simulate_kserver;
use ktaxi::simulate_ktaxi;
use samplelib::*;
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



trait Simulate {
    fn simulate(
        &self,
        simulator: Simulators,
        lambda: f32,
    ) -> Vec<SimResult>;
}

impl Simulate for Sample {
    fn simulate(
        &self,
        simulator: Simulators,
        lambda: f32,
    ) -> Vec<SimResult> {
        match simulator {
            Simulators::KServer(_) => simulate_kserver(self, lambda),
            Simulators::KTaxi(_) => simulate_ktaxi(self, lambda),
        }
    }
}

pub fn run(samples: Vec<Sample>, simulator: Simulators) -> Vec<SimResult> {
    println!("{}", style("Start simulating...").bold().cyan());
    let number_of_lambdas = match simulator {
        Simulators::KTaxi(config) => config.number_of_lambdas,
        Simulators::KServer(config) => config.number_of_lambdas,
    };
    let lambdas = linspace::<f32>(0., 1., number_of_lambdas)
        .into_iter()
        .collect::<Vec<f32>>();
    let results = simulate_samples(samples, lambdas, simulator);
    println!("{}", style("Simulation finished!").bold().green());
    println!(
        "{} {}",
        style("Number of results: ").bold().green(),
        style(results.len()).bold().red()
    );
    results
}

fn simulate_sample(
    sample: Sample,
    lambdas: &Vec<f32>,
    simulator: Simulators,
) -> Vec<SimResult> {
    let results = lambdas
        .iter()
        .map(|lambda| sample.simulate(simulator, *lambda))
        .flatten()
        .collect::<Vec<SimResult>>();

    
    return results;
}

fn simulate_samples(
    samples: Vec<Sample>,
    lambdas: Vec<f32>,
    simulator: Simulators,
) -> Vec<SimResult> {
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
    res
}
