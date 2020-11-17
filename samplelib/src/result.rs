use serverlib::prelude::Instance;


#[derive(Clone, Debug)]
pub struct SimResult {
    pub instance: Instance,
    pub opt_cost: u32,
    pub eta: f64,
    pub alg_costs: Vec<(String, f64)>,
    pub lambda: f32,
}
