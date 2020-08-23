use super::compiler::{SRef, NRef};

#[derive(Debug, Clone)]
pub struct Spec {
    pub coil: NRef,
    pub a: SRef,
    pub b: SRef,
}

impl Spec {
    pub fn build(&self) -> Relay {
        Relay {
            spec: self,
            level: false,
            is_middle: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Relay<'a> {
    spec: &'a Spec,

    level: bool,
    is_middle: bool,
}

impl<'a> Relay<'a> {
    fn a(&self) -> bool {
        !self.is_middle && self.level
    }

    fn b(&self) -> bool {
        !self.is_middle && !self.level
    }

    pub fn update(&mut self, state: &mut [bool], net: &[bool]) {
        let level = self.spec.coil.get(net);
        if self.is_middle {
            self.level = level;
            self.is_middle = false;
        } else if self.level != level {
            self.is_middle = true;
        }
        self.spec.a.set(state, self.a());
        self.spec.b.set(state, self.b());
    }
}
