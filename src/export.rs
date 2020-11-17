use samplelib::*;

use console::style;
use csv::WriterBuilder;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use std::error::Error;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ExportConfig {
    #[structopt(short = "o", long = "output", default_value = "result.csv")]
    pub output_file: String,
}



pub fn run(results: Vec<SimResult>, config: &ExportConfig) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        style(format!("Start exporting to {}...", config.output_file))
            .bold()
            .cyan()
    );
    let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .from_path(config.output_file.clone())?;
    let pb = ProgressBar::new(results.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({percent}%)"),
    );

    let mut headers = vec!["NumberOfServers", "NumberOfRequests", "Lmbda", "Eta", "OptCost"];
    if let Some(first) = results.first() {
        let mut cost_names = first.alg_costs.iter().map(|(name, _)| name.as_ref()).collect();
        headers.append(&mut cost_names);
        wtr.write_record(headers)?;

        for res in results.into_iter().progress_with(pb) {
            let mut record = vec![
                res.instance.k().to_string(), 
                res.instance.length().to_string(),
                res.lambda.to_string(),
                res.eta.to_string(),
                res.opt_cost.to_string()
                ];
                let mut cost_values = res.alg_costs.iter().map(|(_, value)| value.to_string()).collect();
                record.append(&mut cost_values);

            wtr.write_record(record)?;
        }
    }

    
    wtr.flush()?;
    println!("{}", style("Exporting finished!").bold().green());

    Ok(())
}
