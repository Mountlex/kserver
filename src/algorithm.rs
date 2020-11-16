use crate::cost::CostMetric;
use crate::instance::Instance;
use crate::pred::Prediction;
use crate::request::*;
use crate::schedule::Schedule;
use crate::server_config::*;

macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

pub fn double_coverage(instance: &Instance) -> (Schedule, f64) {
    let dc = DoubleCoverage;
    return dc.run(instance);
}

pub fn lambda_dc(instance: &Instance, prediction: &Prediction, lambda: f32) -> (Schedule, f64) {
    let alg = LambdaDC::new(prediction.clone(), lambda);
    return alg.run(instance);
}

pub fn combine_det(instance: &Instance, prediction: &Prediction, gamma: f64) -> (Schedule, f64) {
    let alg = CombineDet::new(prediction.clone(), gamma, false);
    return alg.run(instance);
}

pub fn biased_dc(instance: &Instance) -> (Schedule, f64) {
    let bdc = BiasedDC;
    return bdc.run(instance);
}

pub fn lambda_biased_dc(
    instance: &Instance,
    prediction: &Prediction,
    lambda: f32,
) -> (Schedule, f64) {
    let lbdc = LambdaBiasedDC::new(prediction.clone(), lambda);
    return lbdc.run(instance);
}

trait Algorithm {
    fn run(&self, instance: &Instance) -> (Schedule, f64) {
        let mut schedule = Schedule::from(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = schedule.last().unwrap();
            let (mut next, cost) = self.next_move(current, req, idx);
            costs += cost;
            next.normalize();
            schedule.append_config(next);
        }

        //println!("{:?}", schedule);
        (schedule, costs)
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        next_request: Request,
        request_index: usize,
    ) -> (ServerConfiguration, f64);
}

trait KTaxiAlgorithm {
    fn run(&self, instance: &Instance) -> (Schedule, f64) {
        let mut schedule = Schedule::from(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        let mut active = 0;
        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = schedule.last().unwrap();
            let (new_active, mut next, cost) = self.next_move(current, active, req, idx);
            //println!("{}", cost);
            costs += cost;
            active = new_active;
            if next[0] > next[1] {
                active = 1 - active;
                next.normalize();
            }
            schedule.append_config(next);
        }

        (schedule, costs)
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        next_request: Request,
        req_idx: usize,
    ) -> (usize, ServerConfiguration, f64);
}

struct DoubleCoverage;

impl Algorithm for DoubleCoverage {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
        _req_idx: usize,
    ) -> (ServerConfiguration, f64) {
        let (left, right) = current.adjacent_servers(&req);
        let mut res = ServerConfiguration::from(current.0.to_vec());
        let pos = req.distance_from(&0.0);
        match (left, right) {
            (Some(i), Some(j)) => {
                let d = min!(
                    req.distance_from(&current[j]),
                    req.distance_from(&current[i])
                );
                res[i] += d;
                res[j] -= d;
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = pos;
            }
            _ => panic!("Should not happened!"),
        }
        let costs = current.diff(&res);
        return (res, costs);
    }
}

struct BiasedDC;

impl KTaxiAlgorithm for BiasedDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        req: Request,
        _req_idx: usize,
    ) -> (usize, ServerConfiguration, f64) {
        let passive = 1 - active; // other server
        let mut res = current.clone();
        let pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };

        let mut cost: f64 = 0.0;
        if res[active] != pos && res[passive] != pos {
            let dp = min!(
                2.0 * (pos - current[active]).abs() as f32,
                (pos - current[passive]).abs() as f32
            );
            let da = dp / 2.0 as f32;

            res[active] += da * ((pos - current[active]) / (pos - current[active]).abs());
            res[passive] += dp * ((pos - current[passive]) / (pos - current[passive]).abs());
            cost = (da + dp) as f64;
            //println!("biasedDC: da={} dp={}", da, dp);
        }
        let new_active;
        if res[active] == pos {
            new_active = active;
        } else {
            new_active = passive;
        }

        res[new_active] = match req {
            Request::Simple(x) => x,
            Request::Relocation(_, x) => x,
        };
        return (new_active, res, cost);
    }
}

struct LambdaDC {
    prediction: Prediction,
    lambda: f32,
}

impl LambdaDC {
    fn new(prediction: Prediction, lambda: f32) -> LambdaDC {
        LambdaDC { prediction, lambda }
    }

    fn get_distances(&self, pos_pred: f32, pos_other: f32, req: f32) -> (f32, f32) {
        //
        let d1 = (pos_pred - req).abs();
        let d2 = (pos_other - req).abs();
        if d2 > self.lambda * d1 {
            (d1, self.lambda * d1)
        } else {
            (d2 / self.lambda, d2)
        }
    }
}

impl Algorithm for LambdaDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
        req_idx: usize,
    ) -> (ServerConfiguration, f64) {
        let pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };
        let (left, right) = current.adjacent_servers(&req);
        let mut res = ServerConfiguration::from(current.0.to_vec());
        match (left, right) {
            (Some(i), Some(j)) => {
                if i == j - 1 {
                    assert!(current[i] < pos);
                    assert!(current[j] > pos);
                    // neither i nor j are on the request
                    let predicted = self.prediction.get_predicted_server(req_idx);
                    let fast_server = if predicted <= i { i } else { j };
                    if self.lambda == 0.0 {
                        res[fast_server] = pos;
                    } else {
                        let other: usize = if fast_server == i { j } else { i };
                        let (fast, slow) =
                            self.get_distances(current[fast_server], current[other], pos);
                        if i == fast_server {
                            // left server
                            res[i] += fast;
                            res[j] -= slow;
                            // Fix rounding errors
                            if res[i] > res[j] {
                                res[i] = pos;
                                res[j] = pos;
                            }
                        } else {
                            // j == fast_server
                            res[i] += slow;
                            res[j] -= fast;
                            // Fix rounding errors
                            if res[i] > res[j] {
                                res[i] = pos;
                                res[j] = pos;
                            }
                        }
                        assert!(self.lambda < 1.0 || fast == slow);
                        assert!(res[i] == pos || res[j] == pos);
                    }
                } else {
                    assert!(res[i] == pos);
                }
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = pos;
            }
            _ => panic!("Should not happen!"),
        }
        let costs = current.diff(&res);
        return (res, costs);
    }
}

struct CombineDet {
    prediction: Prediction,
    gamma: f64,
    lazy: bool
}

impl CombineDet {
    fn new(prediction: Prediction, gamma: f64, lazy: bool) -> CombineDet {
        CombineDet { prediction, gamma, lazy }
    }
}

impl Algorithm for CombineDet {
    fn run(&self, instance: &Instance) -> (Schedule, f64) {
        let mut schedule = Schedule::from(instance.initial_positions().clone());
        let mut dc_schedule = Schedule::from(instance.initial_positions().clone());
        let mut ftp_schedule = Schedule::from(instance.initial_positions().clone());
        let mut dc_costs: f64 = 0.0;
        let mut ftp_costs: f64 = 0.0;
        let mut current_dc = false;
        let mut bound = 1.0;

        let dc = DoubleCoverage;

        for (idx, (&req, pred)) in instance.requests().into_iter().zip(self.prediction.clone().into_iter()).enumerate() {
            ftp_costs += req.distance_from(&ftp_schedule.last().unwrap().0[pred]) as f64;
            ftp_schedule.append_move(pred, *req.pos());
            
            let (next, cost) = dc.next_move(dc_schedule.last().unwrap(), req, idx);
            dc_costs += cost;
            dc_schedule.append_config(next);

            while (current_dc && dc_costs > bound) || (!current_dc && ftp_costs > bound) {
                current_dc = !current_dc;
                bound *= 1.0 + self.gamma;
            }

            if current_dc {
                schedule.append_config(dc_schedule.last().unwrap().clone());
            } else {
                schedule.append_config(ftp_schedule.last().unwrap().clone());
            }
        }

        let costs = schedule.cost();

        //println!("{:?}", schedule);
        (schedule, costs)
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        next_request: Request,
        request_index: usize,
    ) -> (ServerConfiguration, f64) {
        (current.clone(), 0.0)
    }
}

struct LambdaBiasedDC {
    prediction: Prediction,
    lambda: f32,
}

impl LambdaBiasedDC {
    fn new(prediction: Prediction, lambda: f32) -> LambdaBiasedDC {
        LambdaBiasedDC { prediction, lambda }
    }
}

impl KTaxiAlgorithm for LambdaBiasedDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        req: Request,
        req_idx: usize,
    ) -> (usize, ServerConfiguration, f64) {
        let passive = 1 - active; // other server
        let mut res = current.clone();
        let pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };

        let mut cost: f64 = 0.0;
        if res[active] != pos && res[passive] != pos {
            let predicted = self.prediction.get_predicted_server(req_idx);

            let dp: f32;
            let da: f32;

            //println!("active={}, predicted={}", active, predicted);

            if active == predicted {
                dp = min!(
                    (1.0 + self.lambda) * (pos - current[active]).abs(),
                    (pos - current[passive]).abs()
                );
                da = dp / (1.0 + self.lambda);
            } else {
                // passive == predicted
                if self.lambda == 0.0 {
                    dp = (pos - current[passive]).abs();
                    da = 0.0;
                } else {
                    dp = min!(
                        (1.0 + (1.0 / self.lambda)) * ((pos - current[active]).abs()),
                        (pos - current[passive]).abs()
                    );
                    da = dp / (1.0 + (1.0 / self.lambda));
                }
            }

            res[active] += da * ((pos - current[active]) / (pos - current[active]).abs());
            res[passive] += dp * ((pos - current[passive]) / (pos - current[passive]).abs());
            //println!("lambdaBiasedDC: da={} dp={}", da, dp);
            cost = (da + dp) as f64;
        } else {
            //println!("Taxi already on request");
        }
        let new_active;
        if res[active] == pos {
            new_active = active;
        } else {
            new_active = passive;
        }

        res[new_active] = match req {
            Request::Simple(x) => x,
            Request::Relocation(_, x) => x,
        };

        return (new_active, res, cost);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_double_coverage() {
        let instance = Instance::from((vec![20, 80, 30, 70, 60, 50], vec![50, 50]));
        let dc = DoubleCoverage;
        assert_eq!(
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![30, 70],
                vec![30, 70],
                vec![40, 60],
                vec![50, 50]
            ]),
            dc.run(&instance).0
        )
    }
    #[test]
    fn test_min() {
        assert_eq!(4, min!(4, 6));
        assert_eq!(4.0, min!(4.0, 6.0));
    }

    #[test]
    fn test_lambda_dc_coverage() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = Prediction::from(vec![0, 1, 0, 1]);
        let alg = LambdaDC::new(pred, 0.5);
        assert_eq!(
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 70],
                vec![43, 64],
            ]),
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_dc_coverage_lambda_zero() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = Prediction::from(vec![0, 1, 0, 1]);
        let alg = LambdaDC::new(pred, 0.0);
        assert_eq!(
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 80],
                vec![40, 64],
            ]),
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_biased_dc() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let alg = BiasedDC;
        assert_eq!(
            Schedule::from(vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 10],
                vec![10, 30],
                vec![0, 25],
            ]),
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_1() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(pred, 1.0);
        assert_eq!(
            Schedule::from(vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 10],
                vec![10, 30],
                vec![0, 25],
            ]),
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_2() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(pred, 0.0);
        assert_eq!(
            Schedule::from(vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 20],
                vec![0, 30],
                vec![0, 30],
            ]),
            alg.run(&instance).0
        )
    }
}
