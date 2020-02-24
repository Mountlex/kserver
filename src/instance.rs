#[derive(Clone, Debug)]
pub struct Instance {
    requests: Vec<i32>,
    initial_positions: Vec<i32>,
}

impl Instance {
    pub fn new(requests: Vec<i32>, initial_positions: Vec<i32>) -> Instance {
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
    pub fn requests(&self) -> &Vec<i32> {
        &self.requests
    }
    pub fn initial_positions(&self) -> &Vec<i32> {
        &self.initial_positions
    }
    pub fn req(&self, index: &usize) -> i32 {
        return self.requests[*index];
    }
}

#[cfg(test)]
mod tests {
    use super::Instance;

    fn get_instance() -> Instance {
        return Instance::new(vec![1, 3, 6, 9], vec![5, 5]);
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
