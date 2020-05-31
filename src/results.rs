use crate::instance::Instance;

#[derive(Clone, Debug)]
pub struct SimResult {
    pub instance: Instance,
    pub opt_cost: u32,
    pub eta: u32,
    pub dc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32,
}

impl SimResult {
    pub fn is_invalid(&self) -> bool {
        if self.instance.is_taxi_instance() {
            false
        } else {
            false
        }
    }
}
