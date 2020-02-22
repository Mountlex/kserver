use crate::server_config::config_diff;
use crate::server_config::ServerConfiguration;

#[derive(Clone)]
pub struct Sequence {
    configurations: Vec<ServerConfiguration>,
}


pub fn sequence_diff(seq1: &Sequence, seq2: &Sequence) -> u32 {
    if seq1.configurations.len() != seq2.configurations.len() {
        panic!("Sequences must have same size!")
    }
    return seq1
        .configurations
        .iter()
        .zip(seq2.configurations.iter())
        .map(|(c1, c2)| config_diff(c1, c2))
        .sum();
}

pub fn normalize_sequence(seq: Sequence, instance &Instance) -> Sequence {

    return ()
}

impl Sequence {
    pub fn new(initial_configuration: ServerConfiguration) -> Sequence {
        return Sequence {
            configurations: vec![initial_configuration],
        };
    }
    pub fn empty() -> Sequence {
        return Sequence {
            configurations: vec![],
        };
    }
    pub fn costs(&self) -> u32 {
        return self
            .configurations
            .iter()
            .zip(self.configurations.iter().skip(1))
            .map(|(c1, c2)| config_diff(c1, c2))
            .sum();
    }
    pub fn append_configuration(&mut self, configuration: ServerConfiguration) {
        self.configurations.push(configuration);
    }

    pub fn append_move(&mut self, id: u32, position: i32) {
        match self.configurations.last() {
            None => println!("Cannot append move as there is no initial configuration!"),
            Some(config) => {
                let next_conf = config.from_move(id, position);
                self.configurations.push(next_conf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_diff_works() {
        let conf11 = ServerConfiguration::new(vec![10, 15, 25]);
        let conf12 = ServerConfiguration::new(vec![8, 17, 20]);
        let conf21 = ServerConfiguration::new(vec![10, 15, 25]);
        let conf22 = ServerConfiguration::new(vec![12, 17, 30]);
        let mut seq1 = Sequence::new(conf11);
        seq1.append_configuration(conf12);
        let mut seq2 = Sequence::new(conf21);
        seq2.append_configuration(conf22);
        assert_eq!(14, sequence_diff(&seq1, &seq2));
    }
    #[test]
    #[should_panic]
    fn sequence_diff_panics() {
        let mut seq1 = Sequence::new(ServerConfiguration::new(vec![10]));
        seq1.append_configuration(ServerConfiguration::new(vec![10]));
        let seq2 = Sequence::new(ServerConfiguration::new(vec![10]));
        sequence_diff(&seq1, &seq2);
    }
    #[test]
    fn append_move_works() {
        let mut seq = Sequence::new(ServerConfiguration::new(vec![10, 20]));
        seq.append_move(1, 30);
        assert_eq!(
            0,
            config_diff(
                &seq.configurations.last().unwrap(),
                &ServerConfiguration::new(vec![10, 30])
            )
        );
    }
    #[test]
    fn costs_works() {
        let mut seq = Sequence::new(ServerConfiguration::new(vec![10, 20]));
        seq.append_move(1, 30);
        seq.append_move(0, 15);
        seq.append_move(1, 20);
        assert_eq!(25, seq.costs());
    }
}
