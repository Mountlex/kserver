use crate::instance::Instance;
use crate::pred::Prediction;
use crate::request::*;
use crate::schedule::{Schedule, ScheduleCreation};
use crate::server_config::*;
use std::cmp::min;

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

pub fn double_coverage(instance: &Instance) -> (Schedule, u32) {
    let dc = DoubleCoverage;
    return dc.run(instance);
}

pub fn lambda_dc(instance: &Instance, prediction: &Prediction, lambda: f32) -> (Schedule, u32) {
    let alg = LambdaDC::new(prediction.clone(), lambda);
    return alg.run(instance);
}

pub fn biased_dc(instance: &Instance) -> (Schedule, u32) {
    let bdc = BiasedDC;
    return bdc.run(instance);
}

pub fn lambda_biased_dc(
    instance: &Instance,
    prediction: &Prediction,
    lambda: f32,
) -> (Schedule, u32) {
    let lbdc = LambdaBiasedDC::new(prediction.clone(), lambda);
    return lbdc.run(instance);
}

trait Algorithm {
    fn run(&self, instance: &Instance) -> (Schedule, u32) {
        let mut schedule = Schedule::new_schedule(instance.initial_positions().to_vec());
        let mut costs = 0;

        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = schedule.last().unwrap();
            let (mut next, cost) = self.next_move(current, req, idx);
            costs += cost;
            next.sort();
            schedule.append_config(next);
        }

        (schedule, costs)
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        next_request: Request,
        request_index: usize,
    ) -> (ServerConfiguration, u32);
}

trait KTaxiAlgorithm {
    fn run(&self, instance: &Instance) -> (Schedule, u32) {
        let mut schedule = Schedule::new_schedule(instance.initial_positions().to_vec());
        let mut costs: u32 = 0;

        let mut active = 0;
        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = schedule.last().unwrap();
            let (new_active, mut next, cost) = self.next_move(current, active, req, idx);
            //println!("{}", cost);
            costs += cost;
            active = new_active;
            if next[0] > next[1] {
                active = 1 - active;
                next.sort();
            }
            schedule.append_config(next);
        }

        if costs > 200000 {
            println!("Schedule: {:?}", schedule);
        }
        (schedule, costs)
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        next_request: Request,
        req_idx: usize,
    ) -> (usize, ServerConfiguration, u32);
}

struct DoubleCoverage;

impl Algorithm for DoubleCoverage {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
        _req_idx: usize,
    ) -> (ServerConfiguration, u32) {
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        let pos = req.s;
        match (left, right) {
            (Some(i), Some(j)) => {
                let d = min(current[j] - pos, pos - current[i]);
                res[i] += d;
                res[j] -= d;
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = pos;
            }
            _ => panic!("Should not happened!"),
        }
        let costs = config_diff(current, &res);
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
    ) -> (usize, ServerConfiguration, u32) {
        let passive = 1 - active; // other server
        let mut res = current.to_vec();
        let pos = req.s;

        let mut cost: u32 = 0;
        if res[active] != pos && res[passive] != pos {
            let dp = min!(
                2.0 * (pos - current[active]).abs() as f32,
                (pos - current[passive]).abs() as f32
            );
            let da = dp / 2.0 as f32;

            res[active] +=
                (da * ((pos - current[active]) / (pos - current[active]).abs()) as f32) as i32;
            res[passive] +=
                (dp * ((pos - current[passive]) / (pos - current[passive]).abs()) as f32) as i32;
            cost = (da + dp) as u32;
            //println!("biasedDC: da={} dp={}", da, dp);
        }
        let new_active;
        if res[active] == pos {
            new_active = active;
        } else {
            new_active = passive;
        }

        res[new_active] = req.t;
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

    fn get_distances(&self, pos_pred: i32, pos_other: i32, req: i32) -> (f32, f32) {
        //
        let d1 = (pos_pred - req).abs() as f32;
        let d2 = (pos_other - req).abs() as f32;
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
    ) -> (ServerConfiguration, u32) {
        let pos = req.s;
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        match (left, right) {
            (Some(i), Some(j)) => {
                if i == j {
                    res[i] = pos;
                } else if i == j - 1 {
                    let predicted = self.prediction.get_predicted_server(req_idx);
                    if self.lambda == 0.0 {
                        res[predicted] = pos;
                    } else if self.lambda == 1.0 {
                        let d = min(current[j] - pos, pos - current[i]);
                        res[i] += d;
                        res[j] -= d;
                    } else {
                        let other: usize = *[i, j].iter().find(|&x| *x != predicted).unwrap();
                        let distances = self.get_distances(current[predicted], current[other], pos);
                        if i == predicted {
                            // left server
                            res[i] += distances.0.floor() as i32;
                            res[j] -= distances.1.floor() as i32;
                            // Fix rounding errors
                            if res[i] > res[j] {
                                res[i] = pos;
                                res[j] = pos;
                            }
                        } else {
                            res[i] += distances.1.floor() as i32;
                            res[j] -= distances.0.floor() as i32;
                            // Fix rounding errors
                            if res[i] > res[j] {
                                res[i] = pos;
                                res[j] = pos;
                            }
                        }
                    }
                } else {
                    panic!("No adjacent servers are given!")
                }
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = pos;
            }
            _ => panic!("Should not happen!"),
        }
        let costs = config_diff(current, &res);
        return (res, costs);
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
    ) -> (usize, ServerConfiguration, u32) {
        let passive = 1 - active; // other server
        let mut res = current.to_vec();
        let pos = req.s;

        let mut cost = 0;
        if res[active] != pos && res[passive] != pos {
            let predicted = self.prediction.get_predicted_server(req_idx);

            let dp: f32;
            let da: f32;

            //println!("active={}, predicted={}", active, predicted);

            if active == predicted {
                dp = min!(
                    (1.0 + self.lambda) * ((pos - current[active]).abs() as f32),
                    (pos - current[passive]).abs() as f32
                );
                da = dp / (1.0 + self.lambda) as f32;
            } else {
                // passive == predicted
                if self.lambda == 0.0 {
                    dp = (pos - current[passive]).abs() as f32;
                    da = 0.0;
                } else {
                    dp = min!(
                        (1.0 + (1.0 / self.lambda)) * ((pos - current[active]).abs() as f32),
                        (pos - current[passive]).abs() as f32
                    );
                    da = dp / (1.0 + (1.0 / self.lambda)) as f32;
                }
            }

            res[active] +=
                (da * ((pos - current[active]) / (pos - current[active]).abs()) as f32) as i32;
            res[passive] +=
                (dp * ((pos - current[passive]) / (pos - current[passive]).abs()) as f32) as i32;
            //println!("lambdaBiasedDC: da={} dp={}", da, dp);
            cost = (da + dp) as u32;
        } else {
            //println!("Taxi already on request");
        }
        let new_active;
        if res[active] == pos {
            new_active = active;
        } else {
            new_active = passive;
        }

        res[new_active] = req.t;

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
            vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![30, 70],
                vec![30, 70],
                vec![40, 60],
                vec![50, 50]
            ],
            dc.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_dc_coverage() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = Prediction::from(vec![0, 1, 0, 1]);
        let alg = LambdaDC::new(pred, 0.5);
        assert_eq!(
            vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 70],
                vec![43, 64],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_dc_coverage_lambda_zero() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = Prediction::from(vec![0, 1, 0, 1]);
        let alg = LambdaDC::new(pred, 0.0);
        assert_eq!(
            vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 80],
                vec![40, 64],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_biased_dc() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let alg = BiasedDC;
        assert_eq!(
            vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 10],
                vec![10, 30],
                vec![0, 25],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_1() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(pred, 1.0);
        assert_eq!(
            vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 10],
                vec![10, 30],
                vec![0, 25],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_2() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(pred, 0.0);
        assert_eq!(
            vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 20],
                vec![0, 30],
                vec![0, 30],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_3() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(pred, 0.5);
        assert_eq!(
            vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 15],
                vec![5, 30],
                vec![0, 29],
            ],
            alg.run(&instance).0
        )
    }

    #[test]
    fn test_biased_dc_2() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((
            vec![(38, 38), (101, 101), (136, 50), (51, 33)],
            vec![10, 10],
        ));
        let alg = BiasedDC;
        let alg_sol = alg.run(&instance);
        let (_opt, opt_cost) = instance.solve()?;
        println!("cost alg = {}", alg_sol.1);
        println!("cost opt = {}", opt_cost);
        assert_eq!(
            vec![
                vec![10, 10],
                vec![24, 38],
                vec![76, 101],
                vec![50, 131],
                vec![33, 129]
            ],
            alg_sol.0
        );

        Ok(())
    }
}
