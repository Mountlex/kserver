use crate::request::Request;

/// Represents a state of the servers on the line.algorithm
///
/// A configuration is a snapshot of the server's positions at a fixed point in time.
/// For the k-server problem on the line, we assume that the configuration of the servers is ordered.
///
/// ## Examples
///
/// ### Instantiation
/// A server configuration implements the `From`-trait for a vector of integers:
/// ```
/// # use serversim::server_config::ServerConfiguration;
/// let config: ServerConfiguration = vec![1,4,7].into();
/// assert_eq!(3, config.size());
/// ```
/// It can also directly be build using `new`:
/// ```
/// # use serversim::server_config::ServerConfiguration;
/// let config = ServerConfiguration::from(vec![1.0,4.0,7.0]);
/// assert_eq!(3, config.size());
/// ```
/// Given a configuration, one can derive another configuration based on moving a server. These operations can also be chained.
/// ```
/// # use serversim::server_config::ServerConfiguration;
/// let config = ServerConfiguration::from(vec![1,4,7]);
/// let next = config.from_move(1, 6.0).from_move(2, 8.0);
/// assert_eq!(ServerConfiguration::from(vec![1,6,8]), next);
/// ```
///
/// ### Normalization
/// A server configuration can be sorted using the `normalize` method.
/// ```
/// # use serversim::server_config::ServerConfiguration;
/// let mut config: ServerConfiguration = vec![3,5,1].into();
/// config.normalize();
/// assert_eq!(ServerConfiguration::from(vec![1,3,5]), config);
/// ```
///
/// ### Inspection
/// Given a request, we can search the configuration for adjacent servers using the `adjacent_servers` method.
/// ```
/// # use serversim::server_config::ServerConfiguration;
/// # use serversim::request::Request;
/// let config: ServerConfiguration = vec![1,5,10].into();
/// assert_eq!((Some(0), Some(1)), config.adjacent_servers(&Request::from(3)));
/// assert_eq!((None, Some(0)), config.adjacent_servers(&Request::from(-1)));
/// assert_eq!((Some(1), Some(1)), config.adjacent_servers(&Request::from(5)));
/// assert_eq!((Some(2), Some(2)), config.adjacent_servers(&Request::from(10)));
/// assert_eq!((Some(2), None), config.adjacent_servers(&Request::from(12)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ServerConfiguration(pub Vec<f32>);

impl ServerConfiguration {
    pub fn new(mut positions: Vec<f32>) -> ServerConfiguration {
        positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ServerConfiguration(positions)
    }

    pub fn from_move(&self, id: usize, pos: f32) -> ServerConfiguration {
        let mut new_pos = ServerConfiguration(self.0.to_vec());
        new_pos.0[id] = pos;
        return new_pos;
    }

    pub fn normalize(&mut self) {
        self.0.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn adjacent_servers(&self, req: &Request) -> (Option<usize>, Option<usize>) {
        let mut right_index: Option<usize> = None;
        let &pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };
        for (idx, &server) in self.into_iter().enumerate() {
            if server >= pos {
                right_index = Some(idx);
                break;
            }
        }
        match right_index {
            Some(0) => (None, right_index),
            Some(right) => {
                if self[right] == pos {
                    (right_index, right_index)
                } else {
                    assert!(self[right] >= pos);
                    assert!(self[right - 1] <= pos);
                    (Some(right - 1), Some(right))
                }
            }
            None => (Some(self.size() - 1), None),
        }
    }
}



impl From<Vec<f32>> for ServerConfiguration {
    fn from(vec: Vec<f32>) -> ServerConfiguration {
        ServerConfiguration::new(vec)
    }
}

impl From<Vec<i32>> for ServerConfiguration {
    fn from(vec: Vec<i32>) -> ServerConfiguration {
        ServerConfiguration::new(vec.into_iter().map(|e| e as f32).collect())
    }
}

impl<'a> IntoIterator for &'a ServerConfiguration {
    type Item = &'a f32;
    type IntoIter = std::slice::Iter<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ServerConfiguration {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::ops::Index<usize> for ServerConfiguration {
    type Output = f32;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl std::ops::IndexMut<usize> for ServerConfiguration {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_config_diff_works() {
        let config1: ServerConfiguration = vec![10, 15, 25].into();
        let config2: ServerConfiguration = vec![8, 17, 25].into();
        assert_eq!(4.0, config1.diff(&config2))
    }

    #[test]
    fn server_config_from_move_works() {
        let config1: ServerConfiguration = vec![10, 15, 25].into();
        let new_conf = config1.from_move(2, 30.0);
        assert_eq!(ServerConfiguration::from(vec![10, 15, 30]), new_conf);
    }
}
