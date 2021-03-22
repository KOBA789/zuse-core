use crate::compiler::SRef;

#[derive(Debug, Clone, Default)]
pub struct Node {
    edges: Vec<Edge>,
}

#[derive(Debug, Clone, Default)]
pub struct Edge {
    sref: SRef,
    node_idx: usize,
}

#[derive(Debug, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    pub fn new(num_nodes: usize) -> Self {
        let mut nodes = vec![];
        nodes.resize_with(num_nodes, Default::default);
        Self { nodes }
    }

    pub fn add_edge(&mut self, n1: usize, n2: usize, sref: SRef) {
        let exists = self.nodes[n1]
            .edges
            .iter()
            .any(|edge| edge.node_idx == n2 && edge.sref == sref);
        if exists {
            return;
        }
        self.nodes[n1].edges.push(Edge { node_idx: n2, sref });
        self.nodes[n2].edges.push(Edge { node_idx: n1, sref });
    }

    pub fn update(&self, state: &[bool], net: &mut [bool], node_idx: usize) {
        if net[node_idx] {
            return;
        }
        net[node_idx] = true;
        for edge in &self.nodes[node_idx].edges {
            if state[edge.sref.idx()] {
                self.update(state, net, edge.node_idx);
            }
        }
    }
}
