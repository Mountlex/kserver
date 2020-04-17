use crate::instance::Instance;

#[derive(Clone, Debug)]
pub enum SimResult {
    KServer(KServerResult),
    KTaxi(KTaxiResult),
}

#[derive(Clone, Debug)]
pub struct KServerResult {
    pub instance: Instance,
    pub opt_cost: u32,
    pub eta: u32,
    pub dc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32,
}

#[derive(Clone, Debug)]
pub struct KTaxiResult {
    pub instance: Instance,
    pub opt_cost: u32,
    pub eta: u32,
    pub bdc_cost: u32,
    pub alg_cost: u32,
    pub lambda: f32,
}

impl SimResult {
    pub fn is_invalid(&self) -> bool {
        match self {
            SimResult::KServer(res) => {
                res.lambda == 0.0 && res.eta == 0 && res.alg_cost != res.opt_cost
            }
            SimResult::KTaxi(res) => {
                res.lambda == 0.0 && res.eta == 0 && res.alg_cost != res.opt_cost
            }
        }
    }
}

impl From<KServerResult> for SimResult {
    fn from(result: KServerResult) -> SimResult {
        SimResult::KServer(result)
    }
}

impl From<KTaxiResult> for SimResult {
    fn from(result: KTaxiResult) -> SimResult {
        SimResult::KTaxi(result)
    }
}
