use crate::request::Request;

pub type ServerConfiguration = Vec<i32>;

pub trait ServerConfig {
    fn from_move(&self, id: usize, pos: i32) -> Self;

    fn moved_server(&self, other: &Self) -> Option<usize>;

    fn left_server(&self, req: Request) -> Option<usize>;

    fn right_server(&self, req: Request) -> Option<usize>;

    fn adjacent_servers(&self, req: Request) -> (Option<usize>, Option<usize>) {
        (self.left_server(req), self.right_server(req))
    }
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

    fn right_server(&self, req: Request) -> Option<usize> {
        self.into_iter()
            .enumerate()
            .find(|(_, &r)| r >= req.get_request_pos())
            .map(|(i, _)| i)
    }
    fn left_server(&self, req: Request) -> Option<usize> {
        self.into_iter()
            .enumerate()
            .rev()
            .find(|(_, &r)| r <= req.get_request_pos())
            .map(|(i, _)| i)
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

    #[test]
    fn server_config_find_right_server_works() {
        let config = vec![10, 15, 25, 50];
        let req1: Request = 20.into();
        let req2: Request = 25.into();
        let req3: Request = 75.into();
        assert_eq!(Some(2), config.right_server(req1));
        assert_eq!(Some(2), config.right_server(req2));
        assert_eq!(None, config.right_server(req3));
    }
    #[test]
    fn server_config_find_left_server_works() {
        let config = vec![10, 15, 25, 50];
        let req1: Request = 20.into();
        let req2: Request = 15.into();
        let req3: Request = 5.into();
        assert_eq!(Some(1), config.left_server(req1));
        assert_eq!(Some(1), config.left_server(req2));
        assert_eq!(None, config.left_server(req3));
    }
}
