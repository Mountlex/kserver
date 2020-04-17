use crate::server_config::config_diff;
use crate::server_config::is_normalized;
use crate::server_config::ServerConfig;
use crate::server_config::ServerConfiguration;
use std::error::Error;

pub type Schedule = Vec<ServerConfiguration>;

/*
pub trait Prediction {
    fn predicted_server(&self, idx: usize, req: Request) -> usize;
}

impl Prediction for Schedule {
    fn predicted_server(&self, idx: usize, req: Request) -> usize {
        self[idx].moved_server(&self[idx + 1]).unwrap_or_else(|| {
            self[idx + 1]
                .iter()
                .enumerate()
                .find(|(_, &server)| match req {
                    Request::Simple(r) => server == r.pos,
                    Request::Relocation(r) => server == r.t,
                })
                .map(|(i, _)| i)
                .unwrap_or_else(|| panic!("Cannot find predicted server. Please investigate!"))
        })
    }
}*/

pub trait CostMetric {
    fn diff(&self, other: &Self) -> u32;
}

impl CostMetric for Schedule {
    fn diff(&self, other: &Self) -> u32 {
        if self.len() != other.len() {
            panic!("Schedules must have same size!")
        }
        return self
            .iter()
            .zip(other.iter())
            .map(|(c1, c2)| config_diff(c1, c2))
            .sum();
    }
}

pub trait ScheduleCreation {
    fn new_schedule(initial_configuration: ServerConfiguration) -> Self;

    fn append_config(&mut self, config: ServerConfiguration);

    fn append_move(&mut self, id: usize, position: i32);
}

impl ScheduleCreation for Schedule {
    fn new_schedule(initial_configuration: ServerConfiguration) -> Schedule {
        vec![initial_configuration]
    }

    fn append_config(&mut self, config: ServerConfiguration) {
        self.push(config);
    }

    fn append_move(&mut self, id: usize, position: i32) {
        match self.last() {
            None => println!("Cannot append move as there is no initial configuration!"),
            Some(config) => {
                let next_conf = config.from_move(id, position);
                self.push(next_conf);
            }
        }
    }
}

pub fn normalize_schedule(schedule: Schedule) -> Result<Schedule, Box<dyn Error>> {
    let mut updated = schedule;
    loop {
        match normalize_schedule_helper(&updated) {
            Some(s) => updated = s,
            None => return Ok(updated),
        }
    }
}

fn normalize_schedule_helper(schedule: &Schedule) -> Option<Schedule> {
    let first_config: ServerConfiguration = match schedule.first() {
        Some(c) => c.to_vec(),
        None => return None,
    };

    let mut fixing = false;
    let mut server_mapping: Vec<usize> = (0..first_config.len()).collect();

    let mut fixed = Schedule::new_schedule(first_config);
    for (last, config) in schedule.iter().zip(schedule.iter().skip(1)) {
        let moved_server: usize = match last.moved_server(config) {
            Some(s) => s,
            None => return None,
        };

        if !is_normalized(config) {
            if !fixing {
                if config[moved_server] < last[moved_server] {
                    server_mapping[moved_server] = moved_server - 1;
                    server_mapping[moved_server - 1] = moved_server;
                } else {
                    server_mapping[moved_server] = moved_server + 1;
                    server_mapping[moved_server + 1] = moved_server;
                }
                fixing = true;
            }
            if fixing {
                fixed.append_move(server_mapping[moved_server], config[moved_server]);
            }
        } else {
            fixed.append_config(config.to_vec());
        }
    }

    if fixing {
        Some(fixed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduleuence_diff_works() {
        let conf11 = vec![10, 15, 25];
        let conf12 = vec![8, 17, 20];
        let conf21 = vec![10, 15, 25];
        let conf22 = vec![12, 17, 30];
        let mut schedule1 = Schedule::new_schedule(conf11);
        schedule1.append_config(conf12);
        let mut schedule2 = Schedule::new_schedule(conf21);
        schedule2.append_config(conf22);
        assert_eq!(14, schedule1.diff(&schedule2));
    }
    #[test]
    #[should_panic]
    fn scheduleuence_diff_panics() {
        let mut schedule1 = Schedule::new_schedule(vec![10]);
        schedule1.append_config(vec![10]);
        let schedule2 = Schedule::new_schedule(vec![10]);
        schedule1.diff(&schedule2);
    }
    #[test]
    fn append_move_works() {
        let mut schedule = Schedule::new_schedule(vec![10, 20]);
        schedule.append_move(1, 30);
        assert_eq!(0, config_diff(&schedule.last().unwrap(), &vec![10, 30]));
    }

    #[test]
    fn normalization_small_works() -> Result<(), Box<dyn Error>> {
        let schedule: Schedule = vec![vec![50, 50], vec![30, 50], vec![30, 20]];
        assert_eq!(
            vec![vec![50, 50], vec![30, 50], vec![20, 50]],
            normalize_schedule(schedule)?
        );

        Ok(())
    }
}
