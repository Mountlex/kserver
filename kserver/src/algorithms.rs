use serverlib::prelude::*;

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

pub fn learning_augmented_alg<A: PredAlgorithm>(alg: A, instance: &Instance, prediction: &Prediction) -> (Schedule, f64) {
    alg.run(instance, prediction)
}

pub fn deterministic_alg<A: DetAlgorithm>(alg: A, instance: &Instance) -> (Schedule, f64) {
    alg.run_det(instance)
}


pub trait DetAlgorithm {
    fn run_det(&self, instance: &Instance) -> (Schedule, f64) {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        for &req in instance.requests() {
            let current = schedule.last().unwrap();
            let (mut next, cost) = self.next_det_move(current, req);
            costs += cost;
            next.normalize();
            schedule.append_config(next);
        }

        //println!("{:?}", schedule);
        (schedule, costs)
    }

    fn next_det_move(
        &self,
        current: &ServerConfiguration,
        next_request: Request,
    ) -> (ServerConfiguration, f64);
}

pub trait PredAlgorithm {
    fn run(&self, instance: &Instance, pred: &Prediction) -> (Schedule, f64) {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        for (&req, &pred) in instance.requests().into_iter().zip(pred.into_iter()) {
            let current = schedule.last().unwrap();
            let (mut next, cost) = self.next_move(current, req, pred);
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
        prediction: usize
    ) -> (ServerConfiguration, f64);
}

impl <T: DetAlgorithm> PredAlgorithm for T {
    fn next_move(&self, current: &ServerConfiguration, next_request: Request, _: usize) -> (ServerConfiguration, f64) {
        self.next_det_move(current, next_request)
    }
}

pub struct DoubleCoverage;

impl DetAlgorithm for DoubleCoverage {
    fn next_det_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
    ) -> (ServerConfiguration, f64) {
        let (left, right) = current.adjacent_servers(&req);
        let mut res = current.clone();
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



pub struct LambdaDC {
    lambda: f32,
}

impl LambdaDC {
    pub fn new(lambda: f32) -> LambdaDC {
        LambdaDC { lambda }
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

impl PredAlgorithm for LambdaDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        req: Request,
        predicted: usize,
    ) -> (ServerConfiguration, f64) {
        let pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };
        let (left, right) = current.adjacent_servers(&req);
        let mut res = current.clone();
        match (left, right) {
            (Some(i), Some(j)) => {
                if i == j - 1 {
                    assert!(current[i] < pos);
                    assert!(current[j] > pos);
                    // neither i nor j are on the request
                   
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

pub struct CombineDet {
    gamma: f64,
}

impl CombineDet {
    pub fn new(gamma: f64) -> CombineDet {
        CombineDet {gamma }
    }
}

impl PredAlgorithm for CombineDet {
    fn run(&self, instance: &Instance, prediction: &Prediction) -> (Schedule, f64) {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut dc_schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut ftp_schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut dc_costs: f64 = 0.0;
        let mut ftp_costs: f64 = 0.0;
        let mut current_dc = false;
        let mut bound = 1.0;

        let dc = DoubleCoverage;
        
        for (&req, &pred) in instance.requests().into_iter().zip(prediction.into_iter()) {
            let ftp = LambdaDC::new(0.0);
            let (next, cost) = ftp.next_move(ftp_schedule.last().unwrap(), req, pred);
            ftp_costs += cost;
            ftp_schedule.append_config(next);
            
            let (next, cost) = dc.next_move(dc_schedule.last().unwrap(), req, pred);
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
        _next_request: Request,
        _request_index: usize,
    ) -> (ServerConfiguration, f64) {
        (current.clone(), 0.0)
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
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![30, 70],
                vec![30, 70],
                vec![40, 60],
                vec![50, 50]
            ]),
            dc.run_det(&instance).0
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
        let alg = LambdaDC::new( 0.5);
        assert_eq!(
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 70],
                vec![43, 64],
            ]),
            alg.run(&instance, &pred).0
        )
    }

    #[test]
    fn test_lambda_dc_coverage_lambda_zero() {
        let instance = Instance::from((vec![20, 80, 40, 64], vec![50, 50]));
        let pred = Prediction::from(vec![0, 1, 0, 1]);
        let alg = LambdaDC::new( 0.0);
        assert_eq!(
            Schedule::from(vec![
                vec![50, 50],
                vec![20, 50],
                vec![20, 80],
                vec![40, 80],
                vec![40, 64],
            ]),
            alg.run(&instance, &pred).0
        )
    }

   
}
