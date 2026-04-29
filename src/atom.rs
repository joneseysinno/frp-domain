use plexus_base::{AtomId, LayerTag, Value};
use serde::{Deserialize, Serialize};

use crate::port::Port;

// ---------------------------------------------------------------------------
// AtomKind
// ---------------------------------------------------------------------------

/// The functional role of an [`Atom`] within a [`Block`](crate::block::Block).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AtomKind {
    /// Produces data from an external source (e.g. sensor, API, DB read).
    Source,
    /// Consumes data and writes it to an external sink (e.g. DB write, UI).
    Sink,
    /// Transforms data flowing through it (pure or stateful computation).
    Transform,
    /// Holds mutable state that persists across ticks.
    State,
    /// Fires events to activate downstream edges.
    Trigger,
}

impl std::fmt::Display for AtomKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AtomKind::Source => "Source",
            AtomKind::Sink => "Sink",
            AtomKind::Transform => "Transform",
            AtomKind::State => "State",
            AtomKind::Trigger => "Trigger",
        };
        write!(f, "{}", s)
    }
}

// ---------------------------------------------------------------------------
// AtomMeta
// ---------------------------------------------------------------------------

/// Descriptive metadata attached to an [`Atom`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtomMeta {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub layer: LayerTag,
}

impl AtomMeta {
    pub fn new(name: impl Into<String>, layer: LayerTag) -> Self {
        Self { name: name.into(), description: None, tags: Vec::new(), layer }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Atom
// ---------------------------------------------------------------------------

/// The fundamental computational unit of a Plexus graph.
///
/// An `Atom` has a kind (its role), metadata, a current [`Value`], and a set
/// of typed directional [`Port`]s through which data flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    pub id: AtomId,
    pub kind: AtomKind,
    pub meta: AtomMeta,
    pub value: Value,
    pub ports: Vec<Port>,
}

impl Atom {
    /// Create a new `Atom` with `Value::Null` and no ports.
    pub fn new(id: AtomId, kind: AtomKind, meta: AtomMeta) -> Self {
        Self { id, kind, meta, value: Value::Null, ports: Vec::new() }
    }

    /// Append a port to this atom.
    pub fn add_port(&mut self, port: Port) {
        self.ports.push(port);
    }

    /// Find a port by name (case-sensitive, first match).
    pub fn port_by_name(&self, name: &str) -> Option<&Port> {
        self.ports.iter().find(|p| p.name == name)
    }

    /// All input ports on this atom.
    pub fn inputs(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter().filter(|p| p.is_input())
    }

    /// All output ports on this atom.
    pub fn outputs(&self) -> impl Iterator<Item = &Port> {
        self.ports.iter().filter(|p| p.is_output())
    }
}

// ---------------------------------------------------------------------------
// loom-base integration: HasAtomId
// ---------------------------------------------------------------------------

impl loom_base::memory::HasAtomId for Atom {
    fn atom_id(&self) -> AtomId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use plexus_base::{IdGen, TypeSig};

    use super::*;
    use crate::port::Port;

    fn make_atom() -> Atom {
        let ids = IdGen::new();
        let meta = AtomMeta::new("test", LayerTag::Core);
        Atom::new(ids.next_atom_id(), AtomKind::Transform, meta)
    }

    #[test]
    fn new_atom_has_null_value_and_no_ports() {
        let a = make_atom();
        assert!(a.value.is_null());
        assert!(a.ports.is_empty());
    }

    #[test]
    fn add_and_find_port() {
        let ids = IdGen::new();
        let mut a = make_atom();
        let p = Port::new_input(ids.next_port_id(), "signal", TypeSig::Int);
        a.add_port(p);
        assert!(a.port_by_name("signal").is_some());
        assert!(a.port_by_name("missing").is_none());
    }

    #[test]
    fn inputs_and_outputs_filtered() {
        let ids = IdGen::new();
        let mut a = make_atom();
        a.add_port(Port::new_input(ids.next_port_id(), "in", TypeSig::Int));
        a.add_port(Port::new_output(ids.next_port_id(), "out", TypeSig::Int));
        assert_eq!(a.inputs().count(), 1);
        assert_eq!(a.outputs().count(), 1);
    }
}
