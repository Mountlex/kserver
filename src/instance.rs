use crate::request::*;

#[derive(Clone, Debug)]
pub struct Instance {
    requests: Vec<Request>,
    initial_positions: Vec<i32>,
}

impl Instance {
    pub fn new(requests: Vec<Request>, initial_positions: Vec<i32>) -> Instance {
        Instance {
            requests: requests,
            initial_positions: initial_positions,
        }
    }    
    pub fn length(&self) -> usize {
        self.requests.len()
    }
    pub fn k(&self) -> usize {
        self.initial_positions.len()
    }
    pub fn requests(&self) -> &Vec<Request> {
        &self.requests
    }
    pub fn initial_positions(&self) -> &Vec<i32> {
        &self.initial_positions
    }
    pub fn req(&self, index: &usize) -> Request {
        return self.requests[*index];
    }
}


impl From<(Vec<i32>, Vec<i32>)> for Instance {
    fn from(instance: (Vec<i32>, Vec<i32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, instance.1)
    }
}

impl From<(Vec<(i32, i32)>, Vec<i32>)> for Instance {
    fn from(instance: (Vec<(i32, i32)>, Vec<i32>)) -> Instance {
        let requests = instance.0.into_iter().map(|req| req.into()).collect();
        Instance::new(requests, instance.1)
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
