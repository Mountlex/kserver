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
        (self.lambda < 0.01 && self.eta == 0 && self.alg_cost != self.opt_cost)
            || (self.lambda == 1.0 && self.dc_cost != self.alg_cost)
    }
}
