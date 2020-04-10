use crate::request::*;

#[derive(Clone, Debug)]
pub struct Instance<T> {
    requests: Vec<T>,
    initial_positions: Vec<i32>,
}

impl<T> Instance<T> {
    pub fn new(requests: Vec<T>, initial_positions: Vec<i32>) -> Instance<T> {
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
    pub fn requests(&self) -> &Vec<T> {
        &self.requests
    }
    pub fn initial_positions(&self) -> &Vec<i32> {
        &self.initial_positions
    }
    pub fn req(&self, index: &usize) -> T {
        return self.requests[*index];
    }
}

pub type KServerInstance = Instance<SimpleRequest>;

impl From<Vec<i32>> for Vec<SimpleRequest> {
    fn from(requests: Vec<i32>) -> Vec<SimpleRequest> {
        requests.into_iter().map(|req| req.into()).collect()
    }
}

pub type KTaxiInstance = Instance<RelocationRequest>;

impl From<Vec<(i32,i32)>> for Vec<RelocationRequest> {
    fn from(requests: Vec<(i32, i32)>) -> Vec<RelocationRequest> {
        requests.into_iter().map(|req| req.into()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_instance() -> KServerInstance {
        return KServerInstance::new(vec![1, 3, 6, 9].into(), vec![5, 5]);
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
