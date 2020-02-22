use std::collections::HashMap;

use crate::instance::Instance;
use crate::seq::Sequence;
use crate::server_config::ServerConfiguration;
use mcmf::*;

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
enum VertexType {
    InitVertex(usize),
    FromVertex(usize),
    ToVertex(usize),
}

const COST_CONST: i32 = -100000;

pub fn solve(instance: &Instance) -> Result<(Sequence, u32), String> {
    let mut graph = GraphBuilder::new();
    add_source_and_init_vertices(&mut graph, instance);
    add_request_verticies(&mut graph, instance);
    add_request_edges(&mut graph, instance);

    let (costs, paths) = graph.mcmf();
    let seq = create_sequence(paths, instance);
    let fixed_costs = costs as i32 + (-COST_CONST * instance.length() as i32);

    return Ok((seq, fixed_costs as u32));
}

fn add_source_and_init_vertices(graph: &mut GraphBuilder<VertexType>, instance: &Instance) {
    for (i, _) in instance.initial_positions().iter().enumerate() {
        graph.add_edge(
            Vertex::Source,
            VertexType::InitVertex(i),
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
                Cost((x - y).abs()),
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
                    Cost((x - y).abs()),
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

fn create_sequence(paths: Vec<mcmf::Path<VertexType>>, instance: &Instance) -> Sequence {
    let mut seq = Sequence::new(ServerConfiguration::new(
        instance.initial_positions().to_vec(),
    ));
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
        seq.append_move(*s as u32, instance.req(r));
    }
    seq
}

fn order_servers_correctly(
    tuples: Vec<(usize, usize)>,
    instance: &Instance,
) -> Vec<(usize, usize)> {
    let mut first_requests: Vec<(usize, usize, i32)> = instance
        .initial_positions()
        .iter()
        .enumerate()
        .flat_map(|(i, _)| tuples.iter().find(|(s, _)| i == *s))
        .map(|s| *s)
        .map(|(s, r)| (s, r, instance.req(&r)))
        .collect();
    first_requests.sort_by(|a, b| a.2.cmp(&b.2));
    let server_mapping: HashMap<_, _> = first_requests
        .iter()
        .enumerate()
        .map(|(i, (s, _, _))| (i, s))
        .collect();

    return tuples
        .iter()
        .map(|(s, r)| (*server_mapping[s], *r))
        .collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solver_costs_1_works() -> Result<(), String> {
        let instance = Instance::new(vec![78, 77, 30, 8, 15, 58, 37, 19, 11, 7], vec![91, 91]);
        let solution = solve(&instance);
        let (seq, costs) = solution?;
        assert_eq!(160, costs);
        assert_eq!(160, seq.costs());
        Ok(())
    }
}
