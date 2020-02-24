use crate::sim::SimResult;
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    numberOfServers: u64,
    numberOfRequests: u64,
    lmbda: f32,
    eta: u64,
    algCost: u64,
    dcCost: u64,
}

impl Record {
    fn from_result(res: SimResult, config: &ExportConfig) -> Record {
        Record {
            numberOfServers: res.instance.k() as u64,
            numberOfRequests: res.instance.length() as u64,
            lmbda: res.lambda,
            eta: res.eta as u64,
            algCost: res.algCost as u64,
            dcCost: res.dcCost as u64,
        }
    }
}

pub fn run(results: Vec<SimResult>, config: &ExportConfig) -> Result<(), Box<dyn Error>> {
    println!("Start exporting to {} ...", config.output_file);
    let mut wtr = Writer::from_path(config.output_file.clone())?;
    let pb = ProgressBar::new(results.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    for res in results.into_iter().progress_with(pb) {
        let record = Record::from_result(res, &config);
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    println!("Exporting finished!");

    Ok(())
}
