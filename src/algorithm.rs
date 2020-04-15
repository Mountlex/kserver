use crate::instance::Instance;
use crate::request::*;
use crate::schedule::{Prediction, Schedule, ScheduleCreation};
use crate::server_config::*;
use std::cmp::min;

pub fn double_coverage(instance: &Instance) -> Schedule {
    let dc = DoubleCoverage;
    return dc.run(instance);
}

pub fn lambda_dc(instance: &Instance, prediction: &Schedule, lambda: f32) -> Schedule {
    let alg = LambdaDC::new(prediction.to_vec(), lambda);
    return alg.run(instance);
}

trait Algorithm {
    fn run(&self, instance: &Instance) -> Schedule {
        let mut schedule = Schedule::new_schedule(instance.initial_positions().to_vec());

        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = schedule.last().unwrap();
            let mut next = self.next_move(current, req, idx);
            next.sort();
            schedule.append_config(next);
        }

        schedule
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        next_request: Request,
        request_index: usize,
    ) -> ServerConfiguration;
}

trait KTaxiAlgorithm {
    fn run(&self, instance: &Instance) -> Schedule {
        let mut schedule = Schedule::new_schedule(instance.initial_positions().to_vec());

        let mut active = 0;
        for &req in instance.requests().into_iter() {
            let current = schedule.last().unwrap();
            let (new_active, mut next) = self.next_move(current, active, req);
            active = new_active;
            next.sort();
            schedule.append_config(next);
        }

        schedule
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        next_request: Request,
    ) -> (usize, ServerConfiguration);
}

struct DoubleCoverage;

impl Algorithm for DoubleCoverage {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
        _req_idx: usize,
    ) -> ServerConfiguration {
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        let pos = req.get_request_pos();
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
        return res;
    }
}

struct BiasedDC;

impl KTaxiAlgorithm for BiasedDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        req: Request,
    ) -> (usize, ServerConfiguration) {
        let passive = 1 - active; // other server
        let mut res = current.to_vec();
        let pos = req.get_request_pos();

        let dp = min(2 * (pos - current[active]).abs(), (pos-current[passive]).abs()) as f32;
        let da = dp / 2.0 as f32;

        res[active] += (da * ((pos - current[active]) / (pos - current[active]).abs()) as f32) as i32;
        res[passive] += (dp * ((pos - current[passive]) / (pos - current[passive]).abs()) as f32) as i32;

        let new_active: usize = res.iter().enumerate().filter(|(_, &p)| p == pos).map(|(i, _)| i).last().unwrap();

        if let Request::Relocation(relocation) = req {
            res[new_active] = relocation.t;
        }

        return (new_active, res);
    }
}

struct LambdaDC {
    prediction: Schedule,
    lambda: f32,
}

impl LambdaDC {
    fn new(prediction: Schedule, lambda: f32) -> LambdaDC {
        LambdaDC { prediction, lambda }
    }

    fn get_distances(&self, pos_pred: i32, pos_other: i32, req: i32) -> (f32, f32) {
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
    ) -> ServerConfiguration {
        let pos = req.get_request_pos();
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        match (left, right) {
            (Some(i), Some(j)) => {
                if i == j {
                    res[i] = pos;
                } else {
                    let predicted = self.prediction.predicted_server(req_idx, req);
                    if self.lambda == 0.0 {
                        res[predicted] = pos;
                    } else {
                        let other: usize = *[i, j].iter().find(|&x| *x != predicted).unwrap();
                        let distances = self.get_distances(current[predicted], current[other], pos);
                        if i == predicted {
                            // left server
                            res[i] += distances.0.floor() as i32;
                            res[j] -= distances.1.floor() as i32;
                        } else {
                            res[i] += distances.1.floor() as i32;
                            res[j] -= distances.0.floor() as i32;
                        }
                    }
                }
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = pos;
            }
            _ => panic!("Should not happend!"),
        }
        return res;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            dc.run(&instance)
        )
    }

    #[test]
    fn test_lambda_dc_coverage() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = vec![
            vec![50, 50],
            vec![20, 50],
            vec![20, 80],
            vec![40, 80],
            vec![40, 64],
        ];
        let alg = LambdaDC::new(pred, 0.5);
        assert_eq!(
            vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 70],
                vec![43, 64],
            ],
            alg.run(&instance)
        )
    }

    #[test]
    fn test_lambda_dc_coverage_lambda_zero() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = vec![
            vec![50, 50],
            vec![20, 50],
            vec![20, 80],
            vec![40, 80],
            vec![40, 64],
        ];
        let alg = LambdaDC::new(pred, 0.0);
        assert_eq!(
            vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 80],
                vec![40, 64],
            ],
            alg.run(&instance)
        )
    }
}
