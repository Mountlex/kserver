use crate::server_config::config_diff;
use crate::server_config::is_normalized;
use crate::server_config::ServerConfig;
use crate::server_config::ServerConfiguration;
use crate::request::OnTheLine;
use std::error::Error;

pub type Sequence = Vec<ServerConfiguration>;

pub trait Prediction {
    fn predicted_server(&self, idx: usize, req: impl OnTheLine) -> usize;
}

impl Prediction for Sequence {
    fn predicted_server(&self, idx: usize, req: impl OnTheLine) -> usize {
        self[idx].moved_server(&self[idx + 1]).unwrap_or_else(|| {
            self[idx + 1]
                .iter()
                .enumerate()
                .find(|(_, &server)| server == req.start_pos())
                .map(|(i, _)| i)
                .unwrap_or_else(|| panic!("Cannot find predicted server. Please investigate!"))
        })
    }
}

pub trait CostMetric {
    fn costs(&self) -> u32;
    fn diff(&self, other: &Self) -> u32;
}

impl CostMetric for Sequence {
    fn costs(&self) -> u32 {
        return self
            .iter()
            .zip(self.iter().skip(1))
            .map(|(c1, c2)| config_diff(c1, c2))
            .sum();
    }

    fn diff(&self, other: &Self) -> u32 {
        if self.len() != other.len() {
            panic!("Sequences must have same size!")
        }
        return self
            .iter()
            .zip(other.iter())
            .map(|(c1, c2)| config_diff(c1, c2))
            .sum();
    }
}

pub trait SequenceCreation {
    fn new_seq(initial_configuration: ServerConfiguration) -> Self;

    fn append_config(&mut self, config: ServerConfiguration);

    fn append_move(&mut self, id: usize, position: i32);
}

impl SequenceCreation for Sequence {
    fn new_seq(initial_configuration: ServerConfiguration) -> Sequence {
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

pub fn normalize_sequence(seq: Sequence) -> Result<Sequence, Box<dyn Error>> {
    let mut updated = seq;
    loop {
        match normalize_sequence_helper(&updated) {
            Some(s) => updated = s,
            None => return Ok(updated),
        }
    }
}

fn normalize_sequence_helper(seq: &Sequence) -> Option<Sequence> {
    let first_config: ServerConfiguration = match seq.first() {
        Some(c) => c.to_vec(),
        None => return None,
    };

    let mut fixing = false;
    let mut server_mapping: Vec<usize> = (0..first_config.len()).collect();

    let mut fixed = Sequence::new_seq(first_config);
    for (last, config) in seq.iter().zip(seq.iter().skip(1)) {
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
    fn sequence_diff_works() {
        let conf11 = vec![10, 15, 25];
        let conf12 = vec![8, 17, 20];
        let conf21 = vec![10, 15, 25];
        let conf22 = vec![12, 17, 30];
        let mut seq1 = Sequence::new_seq(conf11);
        seq1.append_config(conf12);
        let mut seq2 = Sequence::new_seq(conf21);
        seq2.append_config(conf22);
        assert_eq!(14, seq1.diff(&seq2));
    }
    #[test]
    #[should_panic]
    fn sequence_diff_panics() {
        let mut seq1 = Sequence::new_seq(vec![10]);
        seq1.append_config(vec![10]);
        let seq2 = Sequence::new_seq(vec![10]);
        seq1.diff(&seq2);
    }
    #[test]
    fn append_move_works() {
        let mut seq = Sequence::new_seq(vec![10, 20]);
        seq.append_move(1, 30);
        assert_eq!(0, config_diff(&seq.last().unwrap(), &vec![10, 30]));
    }
    #[test]
    fn costs_works() {
        let mut seq = Sequence::new_seq(vec![10, 20]);
        seq.append_move(1, 30);
        seq.append_move(0, 15);
        seq.append_move(1, 20);
        assert_eq!(25, seq.costs());
    }

    #[test]
    fn normalization_small_works() -> Result<(), Box<dyn Error>> {
        let seq: Sequence = vec![vec![50, 50], vec![30, 50], vec![30, 20]];
        assert_eq!(
            vec![vec![50, 50], vec![30, 50], vec![20, 50]],
            normalize_sequence(seq)?
        );

        Ok(())
    }
}
