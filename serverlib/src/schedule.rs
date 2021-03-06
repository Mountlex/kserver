use crate::cost::CostMetric;
use crate::instance::Instance;
use crate::pred::Prediction;
use crate::server_config::ServerConfiguration;

#[derive(Debug, Clone, PartialEq)]
pub struct Schedule(Vec<ServerConfiguration>);

impl Schedule {
    pub fn empty() -> Schedule {
        Schedule(Vec::new())
    }

    pub fn with_initial_config(initial_config: ServerConfiguration) -> Self {
        Schedule(vec![initial_config])
    }

    pub fn append_config(&mut self, config: ServerConfiguration) {
        self.0.push(config);
    }

    pub fn append_move(&mut self, id: usize, position: f32) {
        match self.0.last() {
            None => println!("Cannot append move as there is no initial configuration!"),
            Some(config) => {
                let next_conf = config.from_move(id, position);
                self.0.push(next_conf);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn normalize(&mut self) {
        for config in self {
            config.normalize();
        }
    }

    pub fn last(&self) -> Option<&ServerConfiguration> {
        self.0.last()
    }

    pub fn cost(&self) -> f64 {
        if self.len() <= 1 {
            0.0
        } else {
            let mut cost = 0.0;
            for (from, to) in self
                .0
                .iter()
                .take(self.0.len() - 1)
                .zip(self.0.iter().skip(1))
            {
                cost += from.diff(to);
            }
            cost
        }
    }

    pub fn to_prediction(&self, instance: &Instance) -> Prediction {
        self
        .into_iter()
        .skip(1)
        .enumerate()
        .map(|(idx, config)| {
            config
            .into_iter()
            .enumerate()
            .find(|(_, server)| instance[idx].distance_to(server) == 0.0)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Cannot find predicted server. Please investigate!\nSolution={:?} Instance={}", self, instance))
        })
        .collect::<Prediction>()
    }

    pub fn to_lazy(mut self, instance: &Instance) -> Schedule {
        let mut lazy_schedule = Schedule::empty();
        let initial = self.0.remove(0);
        lazy_schedule.append_config(initial);

        for (idx, config) in self.into_iter().enumerate() {
            let moved_server = config
                .into_iter()
                .enumerate()
                .find(|(_, server)| instance[idx].distance_to(server) == 0.0)
                .map(|(i, _)| i)
                .unwrap();
            lazy_schedule.append_move(moved_server, *instance[idx].pos())
        }

        lazy_schedule
    }
}

impl std::iter::IntoIterator for Schedule {
    type Item = ServerConfiguration;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a Schedule {
    type Item = &'a ServerConfiguration;
    type IntoIter = std::slice::Iter<'a, ServerConfiguration>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> std::iter::IntoIterator for &'a mut Schedule {
    type Item = &'a mut ServerConfiguration;
    type IntoIter = std::slice::IterMut<'a, ServerConfiguration>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl std::iter::FromIterator<ServerConfiguration> for Schedule {
    fn from_iter<I: IntoIterator<Item = ServerConfiguration>>(iter: I) -> Self {
        let mut c = Schedule::empty();
        for i in iter {
            c.append_config(i);
        }
        c
    }
}

impl From<Vec<ServerConfiguration>> for Schedule {
    fn from(config_list: Vec<ServerConfiguration>) -> Self {
        config_list.into_iter().collect()
    }
}

impl From<Vec<Vec<i32>>> for Schedule {
    fn from(config_list: Vec<Vec<i32>>) -> Self {
        config_list
            .into_iter()
            .map(|vec| ServerConfiguration::from(vec))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_diff_works() {
        let conf11 = ServerConfiguration::from(vec![10, 15, 25]);
        let conf12 = ServerConfiguration::from(vec![8, 17, 20]);
        let conf21 = ServerConfiguration::from(vec![10, 15, 25]);
        let conf22 = ServerConfiguration::from(vec![12, 17, 30]);
        let mut schedule1 = Schedule::from(vec![conf11]);
        schedule1.append_config(conf12);
        let mut schedule2 = Schedule::from(vec![conf21]);
        schedule2.append_config(conf22);
        assert_eq!(14.0, schedule1.diff(&schedule2));
    }
    #[test]
    #[should_panic]
    fn schedule_diff_panics() {
        let mut schedule1: Schedule = Schedule::from(vec![ServerConfiguration::from(vec![10])]);
        schedule1.append_config(vec![10].into());
        let schedule2 = Schedule::from(vec![ServerConfiguration::from(vec![10])]);
        schedule1.diff(&schedule2);
    }
    #[test]
    fn append_move_works() {
        let mut schedule = Schedule::from(vec![ServerConfiguration::from(vec![10, 20])]);
        schedule.append_move(1, 30.0);
        assert_eq!(
            schedule.last().unwrap(),
            &ServerConfiguration::from(vec![10, 30])
        );
    }

    #[test]
    fn test_lazy() {
        let instance = Instance::from((vec![1, 3, 6, 9], vec![0, 10]));
        let mut schedule = Schedule::empty();
        schedule.append_config(instance.initial_positions().clone());
        schedule.append_config(vec![1, 9].into());
        schedule.append_config(vec![3, 7].into());
        schedule.append_config(vec![4, 6].into());
        schedule.append_config(vec![4, 9].into());

        let lazy = schedule.to_lazy(&instance);
        assert_eq!(lazy.0[0], vec![0, 10].into());
        assert_eq!(lazy.0[1], vec![1, 10].into());
        assert_eq!(lazy.0[2], vec![3, 10].into());
        assert_eq!(lazy.0[3], vec![3, 6].into());
        assert_eq!(lazy.0[4], vec![3, 9].into());
    }
}
