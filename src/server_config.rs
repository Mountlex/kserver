#[derive(Clone)]
pub struct ServerConfiguration {
    positions: Vec<i32>,
}

pub fn config_diff(config1: &ServerConfiguration, config2: &ServerConfiguration) -> u32 {
    if config1.positions.len() != config2.positions.len() {
        panic!("Server configurations must have same size!")
    }
    return config1
        .positions
        .iter()
        .zip(config2.positions.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<i32>() as u32;
}

impl ServerConfiguration {
    pub fn new(positions: Vec<i32>) -> ServerConfiguration {
        return ServerConfiguration {
            positions: positions,
        };
    }
    pub fn from_move(&self, id: u32, pos: i32) -> ServerConfiguration {
        let mut new_pos = self.positions.clone();
        new_pos[id as usize] = pos;
        return ServerConfiguration::new(new_pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_diff_works() {
        let config1 = ServerConfiguration::new(vec![10, 15, 25]);
        let config2 = ServerConfiguration::new(vec![8, 17, 25]);
        assert_eq!(4, config_diff(&config1, &config2))
    }
    #[test]
    #[should_panic]
    fn server_config_diff_panics() {
        let config1 = ServerConfiguration::new(vec![10, 15, 25]);
        let config2 = ServerConfiguration::new(vec![8, 17, 25, 3]);
        config_diff(&config1, &config2);
    }
    #[test]
    fn server_config_from_move_works() {
        let config1 = ServerConfiguration::new(vec![10, 15, 25]);
        let new_conf = config1.from_move(2, 30);
        assert_eq!(vec![10, 15, 30], new_conf.positions);
    }
}
