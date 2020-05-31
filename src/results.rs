use crate::instance::Instance;
use crate::schedule::Schedule;

#[derive(Clone, Debug)]
pub struct SimResult {
    pub instance: Instance,
    pub schedule: Schedule,
    pub opt_cost: u32,
    pub eta: u32,
    pub dc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32,
}

impl SimResult {
    pub fn is_invalid(&self) -> bool {
        if (self.lambda < 0.01 && self.eta == 0 && self.alg_cost != self.opt_cost)
            || (self.lambda == 1.0 && self.dc_cost != self.alg_cost)
        {
            println!("ALG: {:?}", self.schedule);
            println!("Lambda: {}", self.lambda);
            println!("alg_cost: {}", self.alg_cost);
            println!("dc_cost: {}", self.dc_cost);
            println!("eta: {}", self.eta);
            println!("opt_cost: {}", self.opt_cost);
            true
        } else {
            false
        }
    }
}
