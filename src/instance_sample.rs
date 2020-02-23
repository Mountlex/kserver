use crate::instance::Instance;
use crate::sample::Config;
use rand::distributions::{Distribution, Uniform};

pub fn generate_sample(config: &Config) -> Instance {
    let mut rng = rand::thread_rng();
    let dist = Uniform::from(config.min_value..config.max_value);

    let requests: Vec<i32> = dist
        .sample_iter(rng)
        .take(config.number_of_requests)
        .collect();
    let initial_pos: i32 = dist.sample(&mut rng);

    let initial_positions: Vec<i32> = vec![initial_pos; config.number_of_servers];
    Instance::new(requests, initial_positions)
}
