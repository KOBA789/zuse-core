use std::collections::HashMap;

use crate::graph;
use crate::net;
use crate::relay::{self, Relay};

#[derive(Debug, Clone, Copy, Default)]
pub struct SRef(usize);
impl SRef {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn idx(&self) -> usize {
        self.0
    }

    pub fn set(&self, state: &mut [bool], value: bool) {
        state[self.0] = value;
    }

    pub fn get(&self, state: &[bool]) -> bool {
        state[self.0]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NRef(usize);
impl NRef {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn idx(&self) -> usize {
        self.0
    }

    pub fn get(&self, net: &[bool]) -> bool {
        net[self.0]
    }
}


#[derive(Debug, Clone)]
pub struct CircuitSpec {
    smap: StateMap,
    nmap: NetMap,
    graph: graph::Graph,
    relays: Vec<relay::Spec>,
}

impl CircuitSpec {
    pub fn build(&self) -> Circuit {
        let state = vec![false; self.smap.map.len()];
        let net = vec![false; self.nmap.map.len()];
        let relays = self.relays.iter().map(relay::Spec::build).collect();
        let mut circuit = Circuit {
            spec: self,
            state,
            net,
            relays,
        };
        // initialize
        circuit.run_models();
        circuit
    }
}

pub struct Circuit<'a> {
    spec: &'a CircuitSpec,
    relays: Vec<Relay<'a>>,
    state: Vec<bool>,
    net: Vec<bool>,
}

impl<'a> Circuit<'a> {
    pub fn get_state(&self, name: &str) -> Option<bool> {
        self.spec
            .smap
            .map
            .get(name)
            .map(|sref| sref.get(&self.state))
    }

    pub fn set_state(&mut self, name: &str, value: bool) {
        if let Some(sref) = self.spec.smap.map.get(name) {
            sref.set(&mut self.state, value)
        }
    }

    pub fn get_net(&self, name: &str) -> Option<bool> {
        self.spec.nmap.map.get(name).map(|nref| nref.get(&self.net))
    }

    fn clear_net(&mut self) {
        let len = self.net.len();
        self.net.clear();
        self.net.resize(len, false);
    }

    fn run_models(&mut self) {
        for relay in self.relays.iter_mut() {
            relay.update(&mut self.state, &self.net);
        }
    }

    pub fn simulate(&mut self) {
        self.clear_net();
        self.spec.graph.update(&self.state, &mut self.net, 0);
        self.run_models();
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateMap {
    map: HashMap<String, SRef>,
}

impl StateMap {
    fn by_name(&mut self, name: &str) -> SRef {
        match self.map.get(name) {
            Some(sref) => *sref,
            None => {
                let idx = self.map.len();
                let sref = SRef::new(idx);
                self.map.insert(name.to_string(), sref);
                sref
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetMap {
    map: HashMap<String, NRef>,
}

impl Default for NetMap {
    fn default() -> Self {
        let mut nmap = Self {
            map: HashMap::new(),
        };
        nmap.by_name(net::V_P);
        nmap
    }
}

impl NetMap {
    fn by_name(&mut self, name: &str) -> NRef {
        match self.map.get(name) {
            Some(sref) => *sref,
            None => {
                let idx = self.map.len();
                let nref = NRef::new(idx);
                self.map.insert(name.to_string(), nref);
                nref
            }
        }
    }
}

pub fn compile(netlist: &net::Netlist) -> CircuitSpec {
    let mut smap = StateMap::default();
    let mut nmap = NetMap::default();
    for sw in &netlist.switches {
        nmap.by_name(&sw.l);
        nmap.by_name(&sw.r);
    }
    let relays = netlist
        .relays
        .iter()
        .map(|relay| {
            let a = smap.by_name(&relay.a);
            let b = smap.by_name(&relay.b);
            let coil = nmap.by_name(&relay.coil);
            relay::Spec { a, b, coil }
        })
        .collect();
    let mut graph = graph::Graph::new(nmap.map.len());
    for sw in &netlist.switches {
        let sref = smap.by_name(&sw.state);
        let l_nref = nmap.by_name(&sw.l);
        let r_nref = nmap.by_name(&sw.r);
        graph.add_edge(l_nref.idx(), r_nref.idx(), sref);
    }
    CircuitSpec {
        smap,
        nmap,
        graph,
        relays,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multivibrator() {
        let netlist = net::Netlist {
            relays: vec![
                net::Relay {
                    coil: "N1".to_string(),
                    a: "R1.A".to_string(),
                    b: "R1.B".to_string(),
                },
            ],
            switches: vec![
                net::Switch {
                    state: "R1.B".to_string(),
                    l: "V+".to_string(),
                    r: "N1".to_string(),
                },
            ],
        };
        let circuit_spec = compile(&netlist);
        let mut circuit = circuit_spec.build();
        circuit.simulate();
        assert_eq!(Some(true), circuit.get_net("N1"));
        circuit.simulate();
        assert_eq!(Some(false), circuit.get_net("N1"));
    }


    #[test]
    fn test_cyclic() {
        let netlist = net::Netlist {
            relays: vec![],
            switches: vec![
                net::Switch {
                    state: "SW1.A".to_string(),
                    l: "V+".to_string(),
                    r: "N2".to_string(),
                },
                net::Switch {
                    state: "SW2.A".to_string(),
                    l: "V+".to_string(),
                    r: "N5".to_string(),
                },
                net::Switch {
                    state: "SW3.A".to_string(),
                    l: "N2".to_string(),
                    r: "N5".to_string(),
                },
            ],
        };
        let circuit_spec = compile(&netlist);
        let mut circuit = circuit_spec.build();
        circuit.simulate();
        assert_eq!(Some(false), circuit.get_net("N2"));
        assert_eq!(Some(false), circuit.get_net("N5"));
        
        circuit.set_state("SW1.A", true);
        circuit.simulate();
        assert_eq!(Some(true), circuit.get_net("N2"));
        assert_eq!(Some(false), circuit.get_net("N5"));

        circuit.set_state("SW3.A", true);
        circuit.simulate();
        assert_eq!(Some(true), circuit.get_net("N2"));
        assert_eq!(Some(true), circuit.get_net("N5"));

        circuit.set_state("SW2.A", true);
        circuit.simulate();
        assert_eq!(Some(true), circuit.get_net("N2"));
        assert_eq!(Some(true), circuit.get_net("N5"));

        circuit.set_state("SW1.A", false);
        circuit.simulate();
        assert_eq!(Some(true), circuit.get_net("N2"));
        assert_eq!(Some(true), circuit.get_net("N5"));

        circuit.set_state("SW3.A", false);
        circuit.simulate();
        assert_eq!(Some(false), circuit.get_net("N2"));
        assert_eq!(Some(true), circuit.get_net("N5"));

        circuit.set_state("SW2.A", false);
        circuit.simulate();
        assert_eq!(Some(false), circuit.get_net("N2"));
        assert_eq!(Some(false), circuit.get_net("N5"));
    }
}
