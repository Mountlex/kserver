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

pub fn learning_augmented_alg<A: KTaxiPredAlgorithm>(alg: A, instance: &Instance, prediction: &Prediction) -> (Schedule, f64) {
    alg.run(instance, prediction)
}

pub fn deterministic_alg<A: KTaxiDetAlgorithm>(alg: A, instance: &Instance) -> (Schedule, f64) {
    alg.run_det(instance)
}

pub trait KTaxiDetAlgorithm {
    fn run_det(&self, instance: &Instance) -> (Schedule, f64) {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        let mut active = 0;
        for &req in instance.requests() {
            let current = schedule.last().unwrap();
            let (new_active, mut next, cost) = self.next_det_move(current, active, req);
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

    fn next_det_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        next_request: Request,
    ) -> (usize, ServerConfiguration, f64);
}



pub trait KTaxiPredAlgorithm {
    fn run(&self, instance: &Instance, prediction: &Prediction) -> (Schedule, f64) {
        let mut schedule = Schedule::with_initial_config(instance.initial_positions().clone());
        let mut costs: f64 = 0.0;

        let mut active = 0;
        for (&req, &pred) in instance.requests().into_iter().zip(prediction.into_iter()) {
            let current = schedule.last().unwrap();
            let (new_active, mut next, cost) = self.next_move(current, active, req, pred);
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
        prediction: usize,
    ) -> (usize, ServerConfiguration, f64);
}



pub struct BiasedDC;

impl KTaxiDetAlgorithm for BiasedDC {
    fn next_det_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        req: Request,
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



pub struct LambdaBiasedDC {
    lambda: f32,
}

impl LambdaBiasedDC {
    pub fn new(lambda: f32) -> LambdaBiasedDC {
        LambdaBiasedDC { lambda }
    }
}

impl KTaxiPredAlgorithm for LambdaBiasedDC {
    fn next_move(
        &self,
        current: &ServerConfiguration,
        active: usize,
        req: Request,
        predicted: usize,
    ) -> (usize, ServerConfiguration, f64) {
        let passive = 1 - active; // other server
        let mut res = current.clone();
        let pos = match req {
            Request::Simple(x) => x,
            Request::Relocation(x, _) => x,
        };

        let mut cost: f64 = 0.0;
        if res[active] != pos && res[passive] != pos {

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
            alg.run_det(&instance).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_1() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(1.0);
        assert_eq!(
            Schedule::from(vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 10],
                vec![10, 30],
                vec![0, 25],
            ]),
            alg.run(&instance, &pred).0
        )
    }

    #[test]
    fn test_lambda_biased_dc_2() {
        let instance = Instance::from((vec![(0, 0), (10, 0), (30, 30), (0, 0)], vec![0, 30]));
        let pred = Prediction::from(vec![0, 0, 1, 0]);
        let alg = LambdaBiasedDC::new(0.0);
        assert_eq!(
            Schedule::from(vec![
                vec![0, 30],
                vec![0, 30],
                vec![0, 20],
                vec![0, 30],
                vec![0, 30],
            ]),
            alg.run(&instance, &pred).0
        )
    }
}
