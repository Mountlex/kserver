use crate::server_config::config_diff;
use crate::server_config::ServerConfig;
use crate::server_config::ServerConfiguration;

pub type Schedule = Vec<ServerConfiguration>;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_diff_works() {
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
    fn schedule_diff_panics() {
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
}
