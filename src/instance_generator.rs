use kserver::prelude::*;

use console::style;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use std::error::Error;
use std::io::{self, BufRead};
use std::{fmt, fs, path};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct InstanceConfig {
    #[structopt(short = "k", long = "servers", default_value = "2")]
    pub number_of_servers: usize,

    #[structopt(short = "l", long = "length", default_value = "50")]
    pub number_of_requests: usize,

    #[structopt(long = "min", default_value = "0")]
    pub min_value: i32,
    #[structopt(long = "max", default_value = "4000")]
    pub max_value: i32,
}

#[derive(StructOpt, Debug)]
pub struct InstanceSampleConfig {
    pub number_of_instances: usize,

    #[structopt(long = "relocations", short = "r", default_value = "0.0")]
    pub percentage_of_relocations: f32,
}

#[derive(StructOpt, Debug)]
pub struct InstanceLoadConfig {
    #[structopt(short = "n", long, default_value = "-1")]
    pub number_of_instances: i64,

    #[structopt(short, long, default_value = ".")]
    pub directory: String,

    pub file_prefix: String,
}

#[derive(Debug, Clone)]
pub struct InstanceError {
    msg: String,
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for InstanceError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[allow(dead_code)]
impl InstanceError {
    fn new(msg: String) -> InstanceError {
        InstanceError { msg: msg }
    }
}

pub fn generate_instances(
    sample_config: &InstanceSampleConfig,
    config: &InstanceConfig,
) -> Result<Vec<Instance>, Box<dyn Error>> {
    println!("{}", style("Start generating instances...").bold().cyan());
    let number_of_instances = sample_config.number_of_instances;
    let pb = ProgressBar::new(number_of_instances as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );
    let mut instances = Vec::with_capacity(number_of_instances);

    for _ in (0..number_of_instances).progress_with(pb) {
        instances.push(generate_instance(config, sample_config));
    }
    println!("{}", style("Finished generation!").bold().green());
    Ok(instances)
}

pub fn load_instances(
    load_config: &InstanceLoadConfig,
    config: &InstanceConfig,
) -> Result<Vec<Instance>, Box<dyn Error>> {
    println!("{}", style("Start loading instances...").bold().cyan());

    let entries: Vec<path::PathBuf> = fs::read_dir(load_config.directory.clone())?
        .into_iter()
        .filter_map(Result::ok)
        .map(|r| r.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            p.file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(&load_config.file_prefix)
        })
        .collect();

    println!("Found {} files.", entries.len());

    let pb = ProgressBar::new(entries.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len})"),
    );

    let instances: Vec<Instance> = entries
        .into_iter()
        .progress_with(pb)
        .map(|path| load_instance(&path.as_path(), &config))
        .filter_map(Result::ok)
        .collect::<Vec<Instance>>();

    println!("{}", style("Finished loading!").bold().green());
    if load_config.number_of_instances < 0 {
        return Ok(instances);
    }

    let selected_instances: Vec<Instance> = instances
        .into_iter()
        .take(load_config.number_of_instances as usize)
        .collect();

    Ok(selected_instances)
}

fn generate_instance(config: &InstanceConfig, sample_config: &InstanceSampleConfig) -> Instance {
    let mut rng = rand::thread_rng();
    let dist = Uniform::from(config.min_value..config.max_value);

    let mut requests: Vec<(i32, i32)> = vec![];
    let mut count = 0;
    while count < config.number_of_requests {
        let s = dist.sample(&mut rng);
        if sample_config.percentage_of_relocations > 0.0
            && rng.gen::<f32>() < sample_config.percentage_of_relocations
        {
            let t = dist.sample(&mut rng);
            requests.push((s, s));
            requests.push((s, t));
            count += 2;
        } else {
            requests.push((s, s));
            count += 1;
        }
    }
    let initial_pos: i32 = dist.sample(&mut rng);

    let initial_positions: Vec<i32> = vec![initial_pos; config.number_of_servers];
    Instance::from((requests, initial_positions))
}

fn load_instance(path: &path::Path, config: &InstanceConfig) -> Result<Instance, Box<dyn Error>> {
    let file = fs::File::open(path)?;

    let mut raw_requests: Vec<f64> = io::BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| line.parse::<f64>().ok())
        //.filter(|req| req > &0.0)
        .collect();

    raw_requests.dedup();
    if raw_requests.len() < config.number_of_requests {
        return Err(InstanceError::new("Filtered instance is too short!".to_string()).into());
    }
    raw_requests.resize_with(config.number_of_requests, || 1.0);

    let raw_int_requests = raw_requests
        .into_iter()
        .map(|req| (req * 10000.0) as i32)
        .collect::<Vec<i32>>();

    let min_val = raw_int_requests.iter().min().unwrap();
    let max_val = raw_int_requests.iter().max().unwrap();

    let requests = raw_int_requests
        .iter()
        .map(|&req| interpolate(req, *min_val, *max_val, config.min_value, config.max_value).into())
        .collect::<Vec<Request>>();

    let initial_pos = (config.max_value - config.min_value) / 2;

    Ok(Instance::new(
        requests,
        ServerConfiguration::from(vec![initial_pos; config.number_of_servers]),
    ))
}

fn interpolate(req: i32, in_min: i32, in_max: i32, out_min: i32, out_max: i32) -> i32 {
    (((req - in_min) as f64) / ((in_max - in_min) as f64) * (out_max - out_min) as f64) as i32
        + out_min
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_working() {
        assert_eq!(10, interpolate(20, 0, 100, 0, 50));
        assert_eq!(20, interpolate(120, 100, 200, 10, 60));
    }
}
