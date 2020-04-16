use crate::schedule::CostMetric;
use crate::sim::SimResult;
use console::style;
use csv::Writer;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use serde::Serialize;
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ExportConfig {
    #[structopt(short = "o", long = "output", default_value = "result.csv")]
    pub output_file: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    numberOfServers: u64,
    numberOfRequests: u64,
    lmbda: f32,
    eta: u64,
    optCost: u64,
    algCost: u64,
    dcCost: u64,
}

impl Record {
    #[allow(unused_variables)]
    fn from_result(sim_res: SimResult, config: &ExportConfig) -> Record {
        match sim_res {
            SimResult::KServer(res) => {
                let rec = Record {
                    numberOfServers: res.instance.k() as u64,
                    numberOfRequests: res.instance.length() as u64,
                    lmbda: res.lambda,
                    eta: res.eta as u64,
                    optCost: res.opt_cost as u64,
                    algCost: res.alg_cost as u64,
                    dcCost: res.dc_cost as u64,
                };
                if rec.eta == 0 && rec.lmbda == 0.0 && rec.optCost != rec.algCost {
                    panic!("This should not happen!")
                }
                return rec;
            },
            SimResult::KTaxi(res) => {
                let rec = Record {
                    numberOfServers: res.instance.k() as u64,
                    numberOfRequests: res.instance.length() as u64,
                    lmbda: res.lambda,
                    eta: res.eta as u64,
                    optCost: res.opt_cost as u64,
                    algCost: res.alg_cost as u64,
                    dcCost: res.bdc_cost as u64,
                };
                if rec.eta == 0 && rec.lmbda == 0.0 && rec.optCost != rec.algCost {
                    panic!("This should not happen!")
                }
                return rec;
            },
        }
    }
}

pub fn run(results: Vec<SimResult>, config: &ExportConfig) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        style(format!("Start exporting to {}...", config.output_file))
            .bold()
            .cyan()
    );
    let mut wtr = Writer::from_path(config.output_file.clone())?;
    let pb = ProgressBar::new(results.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({percent}%)"),
    );
    for res in results.into_iter().progress_with(pb) {
        let record = Record::from_result(res, &config);
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    println!("{}", style("Exporting finished!").bold().green());

    Ok(())
}
