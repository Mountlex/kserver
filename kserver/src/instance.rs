use crate::request::*;
use crate::server_config::ServerConfiguration;

/// Represents an instance for an online server problem on the line.
///
/// An instance is composed of an `initial_configuration` and a sequence of `requests`.
///
/// ## Examples
///
/// An instance with initial configuration (3,6) and requests 2,8,5 can be directly build from vectors using the `From`-trait:
/// ```
/// # use serversim::instance::Instance;
/// let instance = Instance::from((vec![2,8,5], vec![3,6]));
/// assert_eq!(2, instance.k());
/// assert_eq!(3, instance.length());
/// ```
#[derive(Clone, Debug)]
pub struct Instance {
    requests: Vec<Request>,
    initial_positions: ServerConfiguration,
}

impl Instance {
    pub fn new(requests: Vec<Request>, initial_positions: ServerConfiguration) -> Instance {
        Instance {
            requests: requests,
            initial_positions: initial_positions,
        }
    }
    pub fn length(&self) -> usize {
        self.requests.len()
    }
    pub fn k(&self) -> usize {
        self.initial_positions.size()
    }
    pub fn requests(&self) -> &Vec<Request> {
        &self.requests
    }
    pub fn initial_positions(&self) -> &ServerConfiguration {
        &self.initial_positions
    }
    pub fn req(&self, index: &usize) -> Request {
        return self.requests[*index];
    }

    pub fn is_taxi_instance(&self) -> bool {
        self.requests().iter().any(|&req| !req.is_simple())
    }
}

impl std::iter::IntoIterator for Instance {
    type Item = Request;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.requests.into_iter()
    }
}

impl std::ops::Index<usize> for Instance {
    type Output = Request;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.requests[idx]
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Instance: ({:?}) [{}]",
            self.initial_positions,
            self.requests
                .iter()
                .fold(String::new(), |acc, &num| acc + &num.to_string() + ", ")
        )
    }
}

impl From<(Vec<i32>, Vec<i32>)> for Instance {
    fn from(instance: (Vec<i32>, Vec<i32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, ServerConfiguration::from(instance.1))
    }
}

impl From<(Vec<(i32, i32)>, Vec<i32>)> for Instance {
    fn from(instance: (Vec<(i32, i32)>, Vec<i32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, ServerConfiguration::from(instance.1))
    }
}

impl From<(Vec<f32>, Vec<f32>)> for Instance {
    fn from(instance: (Vec<f32>, Vec<f32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, ServerConfiguration::from(instance.1))
    }
}

impl From<(Vec<(f32, f32)>, Vec<f32>)> for Instance {
    fn from(instance: (Vec<(f32, f32)>, Vec<f32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, ServerConfiguration::from(instance.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_instance() -> Instance {
        return Instance::from((vec![1, 3, 6, 9], vec![5, 5]));
    }

    #[test]
    fn instance_k_works() {
        let instance = get_instance();
        assert_eq!(2, instance.k());
    }

    #[test]
    fn instance_length_works() {
        let instance = get_instance();
        assert_eq!(2, instance.k());
    }
}
