use plexus_core::{PortId, TypeSig};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// PortDirection
// ---------------------------------------------------------------------------

/// Whether a port receives data (Input) or emits data (Output).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

impl std::fmt::Display for PortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortDirection::Input => write!(f, "Input"),
            PortDirection::Output => write!(f, "Output"),
        }
    }
}

// ---------------------------------------------------------------------------
// Port
// ---------------------------------------------------------------------------

/// A typed, directional connection point on an [`Atom`](crate::atom::Atom)
/// or [`BlockSchema`](crate::block::BlockSchema).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub direction: PortDirection,
    pub type_sig: TypeSig,
}

impl Port {
    /// Create a new input port.
    pub fn new_input(id: PortId, name: impl Into<String>, type_sig: TypeSig) -> Self {
        Self { id, name: name.into(), direction: PortDirection::Input, type_sig }
    }

    /// Create a new output port.
    pub fn new_output(id: PortId, name: impl Into<String>, type_sig: TypeSig) -> Self {
        Self { id, name: name.into(), direction: PortDirection::Output, type_sig }
    }

    /// Returns `true` if this port accepts incoming data.
    pub fn is_input(&self) -> bool {
        self.direction == PortDirection::Input
    }

    /// Returns `true` if this port emits outgoing data.
    pub fn is_output(&self) -> bool {
        self.direction == PortDirection::Output
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}({})", self.direction, self.name, self.type_sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_input_port() {
        let p = Port::new_input(PortId::new(1), "x", TypeSig::Int);
        assert!(p.is_input());
        assert!(!p.is_output());
        assert_eq!(p.name, "x");
    }

    #[test]
    fn new_output_port() {
        let p = Port::new_output(PortId::new(2), "y", TypeSig::Float);
        assert!(p.is_output());
        assert!(!p.is_input());
    }
}
