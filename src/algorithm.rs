use crate::instance::KServerInstance;
use crate::request::*;
use crate::seq::{Prediction, Sequence, SequenceCreation};
use crate::server_config::*;
use std::cmp::min;

pub fn double_coverage(instance: &KServerInstance) -> Sequence {
    let dc = DoubleCoverage;
    return dc.run(instance);
}

pub fn lambda_dc(instance: &KServerInstance, prediction: &Sequence, lambda: f32) -> Sequence {
    let alg = LambdaDC::new(prediction.to_vec(), lambda);
    return alg.run(instance);
}

trait Algorithm {
    fn run(&self, instance: &KServerInstance) -> Sequence {
        let mut seq = Sequence::new_seq(instance.initial_positions().to_vec());

        for (idx, &req) in instance.requests().into_iter().enumerate() {
            let current = seq.last().unwrap();
            let mut next = self.next_move(current, req, idx);
            next.sort();
            seq.append_config(next);
        }

        seq
    }

    fn next_move(
        &self,
        current: &ServerConfiguration,
        next_request: SimpleRequest,
        request_index: usize,
    ) -> ServerConfiguration;
}

struct DoubleCoverage;

impl Algorithm for DoubleCoverage {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: SimpleRequest,
        _req_idx: usize,
    ) -> ServerConfiguration {
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        let reqPos = req.pos;
        match (left, right) {
            (Some(i), Some(j)) => {
                let d = min(current[j] - reqPos, reqPos - current[i]);
                res[i] += d;
                res[j] -= d;
            }
            (Some(i), None) | (None, Some(i)) => {
                res[i] = reqPos;
            }
            _ => panic!("Should not happened!"),
        }
        return res;
    }
}

struct LambdaDC {
    prediction: Sequence,
    lambda: f32,
}

impl LambdaDC {
    fn new(prediction: Sequence, lambda: f32) -> LambdaDC {
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
        req: SimpleRequest,
        req_idx: usize,
    ) -> ServerConfiguration {
        let reqPos = req.pos;
        let (left, right) = current.adjacent_servers(req);
        let mut res = current.to_vec();
        match (left, right) {
            (Some(i), Some(j)) => {
                if i == j {
                    res[i] = reqPos;
                } else {
                    let predicted = self.prediction.predicted_server(req_idx, req);
                    if self.lambda == 0.0 {
                        res[predicted] = reqPos;
                    } else {
                        let other: usize = *[i, j].iter().find(|&x| *x != predicted).unwrap();
                        let distances = self.get_distances(current[predicted], current[other], reqPos);
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
                res[i] = reqPos;
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
        let instance = KServerInstance::new(vec![20, 80, 30, 70, 60, 50].into(), vec![50, 50]);
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
        let instance = KServerInstance::new(vec![20, 80, 40, 64].into(), vec![50, 50]);
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
        let instance = KServerInstance::new(vec![20, 80, 40, 64].into(), vec![50, 50]);
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
