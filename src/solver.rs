use crate::instance::*;
use crate::sample_generator::*;
use crate::schedule::{Schedule, ScheduleCreation};
use crate::request::*;
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

        let relocation_costs: i32 = self.requests().iter().filter_map(|req| match *req {
            Request::Relocation(r) => Some(r),
            _ => None,
        }).map(|req| (req.s - req.t).abs()).sum();

        let fixed_costs = costs as i32 + (-COST_CONST * self.length() as i32) - relocation_costs;
    
        return Ok((schedule, fixed_costs as u32));
    }

    pub fn build_sample(self: Instance) -> Result<Sample, SolverError> {
        let (solution, costs) = self.solve()?;
        match self.instance_type {
            InstanceType::KServer => Ok(KServerSample::new(self, solution, costs).into()),
            InstanceType::KTaxi => Ok(KTaxiSample::new(self, solution, costs).into()),
        }
        
    }
}


fn add_source_and_init_vertices(graph: &mut GraphBuilder<VertexType>, instance: &Instance) {
    for (i, _) in instance.initial_positions().iter().enumerate() {
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
        for (j, y) in instance.initial_positions().iter().enumerate() {
            graph.add_edge(
                VertexType::InitVertex(j),
                VertexType::FromVertex(i),
                Capacity(1),
                Cost((x.get_request_pos() - y).abs()),
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
                let relocated_pos = match x {
                    Request::Simple(req) => req.pos,
                    Request::Relocation(req) => req.t
                };
                graph.add_edge(
                    VertexType::ToVertex(i),
                    VertexType::FromVertex(j),
                    Capacity(1),
                    Cost((relocated_pos - y.get_request_pos()).abs()),
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
    let mut schedule = Schedule::new_schedule(instance.initial_positions().to_vec());
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
    let mut fixed_tuples = order_servers_correctly(tuples, instance);
    fixed_tuples.sort_by_key(|&(_, r)| r);
    for (s, r) in fixed_tuples.iter() {
        schedule.append_move(*s, instance.req(r).get_request_pos());
    }
    schedule
}

fn order_servers_correctly(
    tuples: Vec<(usize, usize)>,
    instance: &Instance,
) -> Vec<(usize, usize)> {
    // (server_id, req_idx, req)
    let mut first_requests: Vec<(usize, usize, i32)> = instance
        .initial_positions()
        .iter()
        .enumerate()
        .flat_map(|(i, _)| tuples.iter().find(|(s, _)| i == *s))
        .map(|s| *s)
        .map(|(s, r)| (s, r, instance.req(&r).get_request_pos()))
        .collect();
    first_requests.sort_by(|a, b| a.2.cmp(&b.2));
    let mut server_mapping: HashMap<_, _> = (0..instance.k()).enumerate().collect();
    first_requests
        .into_iter()
        .enumerate()
        .for_each(|(i, (s, _, _))| {
            server_mapping.insert(i, s);
        });

    return tuples
        .iter()
        .map(|(s, r)| (server_mapping[s], *r))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::CostMetric;

    #[test]
    fn solver_costs_1_works() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((vec![78, 77, 30, 8, 15, 58, 37, 19, 11, 7], vec![91, 91]));
        let solution = instance.solve();
        let (schedule, costs) = solution?;
        assert_eq!(160, costs);
        assert_eq!(160, schedule.costs());
        Ok(())
    }

    #[test]
    fn solver_order_works() {
        let instance = Instance::from((vec![40, 60, 50], vec![50, 50]));
        let tuples = vec![(1, 0), (0, 1), (1, 2)];
        assert_eq!(
            vec![(0, 0), (1, 1), (0, 2)],
            order_servers_correctly(tuples, &instance)
        );
    }

    #[test]
    fn solver_works() -> Result<(), Box<dyn Error>> {
        let instance = Instance::from((vec![38, 72, 183, 149, 135, 104], vec![32, 32]));
        let solution = vec![
            vec![32, 32],
            vec![32, 38],
            vec![32, 72],
            vec![32, 183],
            vec![32, 149],
            vec![32, 135],
            vec![32, 104],
        ];
        assert_eq!(solution, instance.solve()?.0);
        Ok(())
    }
}
