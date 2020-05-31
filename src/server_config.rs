use crate::request::Request;

pub type ServerConfiguration = Vec<i32>;

pub trait ServerConfig {
    fn from_move(&self, id: usize, pos: i32) -> Self;

    fn moved_server(&self, other: &Self) -> Option<usize>;

    fn adjacent_servers(&self, req: Request) -> (Option<usize>, Option<usize>);
}

pub fn is_normalized(config: &ServerConfiguration) -> bool {
    !config
        .iter()
        .zip(config.iter().skip(1))
        .any(|(&a, &b)| a > b)
}

pub fn config_diff(config1: &ServerConfiguration, config2: &ServerConfiguration) -> u32 {
    if config1.len() != config2.len() {
        panic!("Server configurations must have same size!")
    }
    return config1
        .iter()
        .zip(config2.iter())
        .map(|(a, b)| (a - b).abs())
        .sum::<i32>() as u32;
}

impl ServerConfig for ServerConfiguration {
    fn from_move(&self, id: usize, pos: i32) -> ServerConfiguration {
        let mut new_pos = self.clone();
        new_pos[id] = pos;
        return new_pos;
    }

    fn moved_server(&self, other: &ServerConfiguration) -> Option<usize> {
        if self.len() != other.len() {
            return None;
        }
        let moved_servers: Vec<usize> = self
            .iter()
            .zip(other.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .collect();
        if moved_servers.len() != 1 {
            return None;
        }
        let res = moved_servers.first().cloned();
        res
    }

    fn adjacent_servers(&self, req: Request) -> (Option<usize>, Option<usize>) {
        let mut right_index: Option<usize> = None;
        for (idx, &server) in self.iter().enumerate() {
            if server >= req.s {
                right_index = Some(idx);
                break;
            }
        }
        match right_index {
            Some(0) => (None, right_index),
            Some(right) => {
                if self[right] == req.s {
                    (right_index, right_index)
                } else {
                    (Some(right - 1), right_index)
                }
            }
            None => (Some(self.len() - 1), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_diff_works() {
        let config1 = vec![10, 15, 25];
        let config2 = vec![8, 17, 25];
        assert_eq!(4, config_diff(&config1, &config2))
    }
    #[test]
    #[should_panic]
    fn server_config_diff_panics() {
        let config1 = vec![10, 15, 25];
        let config2 = vec![8, 17, 25, 3];
        config_diff(&config1, &config2);
    }
    #[test]
    fn server_config_from_move_works() {
        let config1 = vec![10, 15, 25];
        let new_conf = config1.from_move(2, 30);
        assert_eq!(vec![10, 15, 30], new_conf);
    }
}
