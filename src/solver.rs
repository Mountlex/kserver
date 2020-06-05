use crate::instance::*;
use crate::request::*;
use crate::sample::*;
use crate::schedule::Schedule;
use mcmf::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
enum VertexType {
    InitVertex(usize),
    FromVertex(usize),
    ToVertex(usize),
}

#[derive(Debug, Clone)]
pub struct SolverError {
    msg: String,
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Not all required predictions have been found: {}!",
            self.msg
        )
    }
}

impl Error for SolverError {
    fn description(&self) -> &str {
        "Not all required predictions have been found!"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

#[allow(dead_code)]
impl SolverError {
    fn new(msg: String) -> SolverError {
        SolverError { msg: msg }
    }
}

const COST_CONST: i32 = -100000;

impl Instance {
    pub fn solve(self: &Instance) -> Result<(Schedule, u32), SolverError> {
        let mut graph = GraphBuilder::new();
        add_source_and_init_vertices(&mut graph, self);
        add_request_verticies(&mut graph, self);
        add_request_edges(&mut graph, self);
        let (costs, paths) = graph.mcmf();
        let schedule = create_schedule(paths, self);

        let fixed_costs = costs as i32 + (-COST_CONST * self.length() as i32);
        return Ok((schedule, fixed_costs as u32));
    }

    pub fn build_sample(self: Instance) -> Result<Sample, SolverError> {
        let (solution, costs) = self.solve()?;
        Ok(Sample::new(self, solution, costs))
    }
}

fn add_source_and_init_vertices(graph: &mut GraphBuilder<VertexType>, instance: &Instance) {
    for (i, _) in instance.initial_positions().into_iter().enumerate() {
        graph.add_edge(
            Vertex::Source,
            VertexType::InitVertex(i),
            Capacity(1),
            Cost(0),
        );
        graph.add_edge(
            VertexType::InitVertex(i),
            Vertex::Sink,
            Capacity(1),
            Cost(0),
        );
    }
}

fn add_request_verticies(graph: &mut GraphBuilder<VertexType>, instance: &Instance) {
    for (i, x) in instance.requests().iter().enumerate() {
        for (j, y) in instance.initial_positions().into_iter().enumerate() {
            graph.add_edge(
                VertexType::InitVertex(j),
                VertexType::FromVertex(i),
                Capacity(1),
                Cost(x.distance_from(y) as i32),
            );
        }
        graph.add_edge(
            VertexType::FromVertex(i),
            VertexType::ToVertex(i),
            Capacity(1),
            Cost(COST_CONST),
        );
        graph.add_edge(VertexType::ToVertex(i), Vertex::Sink, Capacity(1), Cost(0));
    }
}

fn add_request_edges(graph: &mut GraphBuilder<VertexType>, instance: &Instance) {
    for (i, x) in instance.requests().iter().enumerate() {
        for (j, y) in instance.requests().iter().enumerate() {
            if i < j {
                graph.add_edge(
                    VertexType::ToVertex(i),
                    VertexType::FromVertex(j),
                    Capacity(1),
                    Cost(x.distance_to_req(y) as i32),
                );
            }
        }
    }
}

fn get_type_of_vertex(v: &Vertex<VertexType>) -> Option<VertexType> {
    match v {
        Vertex::Node(x) => Some::<VertexType>(*x),
        _ => None,
    }
}

fn is_move_edge(v1: &Vertex<VertexType>, v2: &Vertex<VertexType>) -> Option<usize> {
    let (type1, type2) = (get_type_of_vertex(v1), get_type_of_vertex(v2));
    match (type1, type2) {
        (Some(VertexType::FromVertex(i)), Some(VertexType::ToVertex(_))) => Some(i),
        _ => None,
    }
}

fn create_schedule(paths: Vec<mcmf::Path<VertexType>>, instance: &Instance) -> Schedule {
    let mut schedule = Schedule::from(instance.initial_positions().clone());
    // server index to request index
    let tuples: Vec<(usize, usize)> = paths
        .iter()
        .enumerate()
        .flat_map(|(index, path)| {
            path.flows
                .iter()
                .filter_map(|flow| is_move_edge(&flow.a, &flow.b))
                .map(move |request| (index, request))
        })
        .collect();

    //println!("{:?}", tuples);
    let number_of_requests = instance.length();
    let mut req_to_server: HashMap<usize, usize> = tuples
        .into_iter()
        .map(|(server, request)| (request, server))
        .collect();

    //println!("Initial: {:?}", req_to_server);
    for (req_index, &req) in instance.requests().iter().enumerate() {
        let last_config = schedule.last().unwrap();
        match last_config.adjacent_servers(&req) {
            (Some(i), Some(j)) => {
                if req_to_server[&req_index] < i {
                    //println!("< i: ({}, {})", i, j);
                    let other = req_to_server[&req_index];
                    switch_server_indices(&mut req_to_server, number_of_requests, other, i);
                } else if req_to_server[&req_index] > j {
                    //println!("> j: ({}, {})", i, j);
                    let other = req_to_server[&req_index];
                    switch_server_indices(&mut req_to_server, number_of_requests, other, j);
                }
            }
            (None, Some(i)) => {
                if i != req_to_server[&req_index] {
                    //println!("(None, {})", i);
                    let other = req_to_server[&req_index];
                    switch_server_indices(&mut req_to_server, number_of_requests, other, i);
                }
            }
            (Some(i), None) => {
                if i != req_to_server[&req_index] {
                    //println!("({}, None)", i);
                    let other = req_to_server[&req_index];
                    switch_server_indices(&mut req_to_server, number_of_requests, other, i);
                }
            }
            (None, None) => panic!("Something went wrong!"),
        }
        let next_conf = last_config.from_move(
            req_to_server[&req_index],
            match req {
                Request::Simple(x) => x,
                Request::Relocation(_, x) => x,
            },
        );
        //next_conf.sort();
        schedule.append_config(next_conf);
    }
    schedule
}

fn switch_server_indices(mapping: &mut HashMap<usize, usize>, length: usize, i: usize, j: usize) {
    for req_index in 0..length {
        if mapping.get(&req_index) == Some(&i) {
            mapping.insert(req_index, j);
        } else if mapping.get(&req_index) == Some(&j) {
            mapping.insert(req_index, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_costs_1_works() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((vec![78, 77, 30, 8, 15, 58, 37, 19, 11, 7], vec![91, 91]));
        let solution = instance.solve();
        let (_, costs) = solution?;
        assert_eq!(160, costs);
        Ok(())
    }

    #[test]
    fn solver_works() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((vec![38, 72, 183, 149, 135, 104], vec![32, 32]));
        let solution = Schedule::from(vec![
            vec![32, 32],
            vec![32, 38],
            vec![32, 72],
            vec![32, 183],
            vec![32, 149],
            vec![32, 135],
            vec![32, 104],
        ]);
        assert_eq!(solution, instance.solve()?.0);
        Ok(())
    }

    #[test]
    fn solver_works2() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((vec![17, 17, 5, 14, 16, 17], vec![14, 14]));
        let solution = Schedule::from(vec![
            vec![14, 14],
            vec![14, 17],
            vec![14, 17],
            vec![5, 17],
            vec![5, 14],
            vec![5, 16],
            vec![5, 17],
        ]);
        assert_eq!(solution, instance.solve()?.0);
        Ok(())
    }
}
