use crate::cost::CostMetric;
use crate::request::Request;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ServerConfiguration(Vec<i32>);

impl ServerConfiguration {
    pub fn from_move(&self, id: usize, pos: i32) -> ServerConfiguration {
        let mut new_pos = ServerConfiguration(self.0.to_vec());
        new_pos.0[id] = pos;
        return new_pos;
    }

    pub fn normalize(&mut self) {
        self.0.sort();
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn adjacent_servers(&self, req: Request) -> (Option<usize>, Option<usize>) {
        let mut right_index: Option<usize> = None;
        for (idx, &server) in self.into_iter().enumerate() {
            if server >= req.s {
                right_index = Some(idx);
                break;
            }
        }
        match right_index {
            Some(0) => (None, right_index),
            Some(right) => {
                if self[right] == req.s {
                    (right_index, right_index)
                } else {
                    (Some(right - 1), right_index)
                }
            }
            None => (Some(self.size() - 1), None),
        }
    }
}

impl CostMetric<u32> for ServerConfiguration {
    fn diff(&self, other: &ServerConfiguration) -> u32 {
        return self
            .into_iter()
            .zip(other.into_iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<i32>() as u32;
    }
}

impl From<Vec<i32>> for ServerConfiguration {
    fn from(vec: Vec<i32>) -> ServerConfiguration {
        ServerConfiguration(vec)
    }
}

impl<'a> IntoIterator for &'a ServerConfiguration {
    type Item = &'a i32;
    type IntoIter = std::slice::Iter<'a, i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for ServerConfiguration {
    type Item = i32;
    type IntoIter = std::vec::IntoIter<i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::ops::Index<usize> for ServerConfiguration {
    type Output = i32;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl std::ops::IndexMut<usize> for ServerConfiguration {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_config_diff_works() {
        let config1: ServerConfiguration = vec![10, 15, 25].into();
        let config2: ServerConfiguration = vec![8, 17, 25].into();
        assert_eq!(4, config1.diff(&config2))
    }

    #[test]
    fn server_config_from_move_works() {
        let config1: ServerConfiguration = vec![10, 15, 25].into();
        let new_conf = config1.from_move(2, 30);
        assert_eq!(ServerConfiguration::from(vec![10, 15, 30]), new_conf);
    }
}
