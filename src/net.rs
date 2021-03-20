use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Netlist {
    pub relays: Vec<Relay>,
    pub switches: Vec<Switch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relay {
    pub coil: String,
    pub a: String,
    pub b: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub state: String,
    pub l: String,
    pub r: String,
}

pub const V_P: &str = "N0";
