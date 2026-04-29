use std::sync::Arc;
use std::time::Duration;

use plexus_core::{EdgeId, PortId, Value};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// EdgeTransform
// ---------------------------------------------------------------------------

/// The computation applied to the input values of a [`HyperEdge`] to produce
/// an output value.
#[derive(Clone)]
pub enum EdgeTransform {
    /// Pass the first input through unchanged. Returns `Value::Null` if there
    /// are no inputs.
    PassThrough,
    /// Look up a named transform function in the engine's
    /// [`TransformRegistry`](crate) at evaluation time.
    Named(String),
    /// An inline closure that is invoked directly. Not serializable.
    Inline(Arc<dyn Fn(&[Value]) -> Value + Send + Sync>),
    /// A Rhai script evaluated at runtime. The script receives the input
    /// values as an `inputs` array variable and must return a single value.
    ///
    /// Unlike `Inline`, this variant serializes and deserializes correctly,
    /// allowing complete functional graphs to be persisted and transferred.
    Script(String),
}

impl std::fmt::Debug for EdgeTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeTransform::PassThrough => write!(f, "EdgeTransform::PassThrough"),
            EdgeTransform::Named(name) => write!(f, "EdgeTransform::Named({:?})", name),
            EdgeTransform::Inline(_) => write!(f, "EdgeTransform::Inline(<fn>)"),
            EdgeTransform::Script(code) => {
                write!(f, "EdgeTransform::Script(<{} bytes>)", code.len())
            }
        }
    }
}

// Manual Serialize/Deserialize: Inline closures are not serializable; they
// round-trip as `PassThrough` so that deserialized graphs are always valid
// (callers must re-attach closures after loading from persistent storage).
impl Serialize for EdgeTransform {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(tag = "type", content = "value")]
        enum Repr<'a> {
            PassThrough,
            Named(&'a str),
            Inline, // serialized as PassThrough sentinel
            Script(&'a str),
        }
        match self {
            EdgeTransform::PassThrough => Repr::PassThrough,
            EdgeTransform::Named(n) => Repr::Named(n.as_str()),
            EdgeTransform::Inline(_) => Repr::Inline,
            EdgeTransform::Script(code) => Repr::Script(code.as_str()),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EdgeTransform {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", content = "value")]
        enum Repr {
            PassThrough,
            Named(String),
            Inline,
            Script(String),
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::PassThrough | Repr::Inline => EdgeTransform::PassThrough,
            Repr::Named(n) => EdgeTransform::Named(n),
            Repr::Script(code) => EdgeTransform::Script(code),
        })
    }
}

// ---------------------------------------------------------------------------
// EdgeSchedule
// ---------------------------------------------------------------------------

/// When a [`HyperEdge`] should be evaluated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeSchedule {
    /// Re-evaluate whenever any source port value changes.
    OnChange,
    /// Re-evaluate on every scheduler tick that exceeds the given interval.
    OnTick(Duration),
    /// Re-evaluate when the named event is fired on the graph.
    OnEvent(String),
}

// ---------------------------------------------------------------------------
// HyperEdge
// ---------------------------------------------------------------------------

/// A directed, multi-input/multi-output data-flow edge in a Plexus graph.
///
/// A `HyperEdge` reads from one or more source [`Port`](crate::port::Port)s,
/// applies its [`EdgeTransform`], and writes the result to one or more target
/// ports, according to its [`EdgeSchedule`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperEdge {
    pub id: EdgeId,
    pub sources: Vec<PortId>,
    pub targets: Vec<PortId>,
    pub transform: EdgeTransform,
    pub schedule: EdgeSchedule,
    /// If `true`, this edge's output is buffered and applied to `port_values`
    /// at the start of the *next* execution cycle rather than immediately.
    /// This allows feedback loops (cycles in the graph) without violating the
    /// topological sort — delay edges are excluded from the sort entirely.
    #[serde(default)]
    pub delay: bool,
}

impl HyperEdge {
    pub fn new(
        id: EdgeId,
        sources: Vec<PortId>,
        targets: Vec<PortId>,
        transform: EdgeTransform,
        schedule: EdgeSchedule,
    ) -> Self {
        Self { id, sources, targets, transform, schedule, delay: false }
    }

    /// Mark this edge as a delay edge, enabling feedback loops.
    ///
    /// The output of a delay edge is buffered and applied at the start of the
    /// next execution cycle, giving exactly one tick of delay.
    pub fn with_delay(mut self) -> Self {
        self.delay = true;
        self
    }
}

// ---------------------------------------------------------------------------
// loom-core integration: HasEdgeId
// ---------------------------------------------------------------------------

impl loom_core::memory::HasEdgeId for HyperEdge {
    fn edge_id(&self) -> EdgeId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use plexus_core::IdGen;

    use super::*;

    #[test]
    fn passthrough_debug() {
        let t = EdgeTransform::PassThrough;
        assert!(format!("{:?}", t).contains("PassThrough"));
    }

    #[test]
    fn inline_debug_shows_fn() {
        let t = EdgeTransform::Inline(Arc::new(|_| Value::Null));
        assert!(format!("{:?}", t).contains("<fn>"));
    }

    #[test]
    fn named_debug() {
        let t = EdgeTransform::Named("double".into());
        assert!(format!("{:?}", t).contains("double"));
    }

    #[test]
    fn hyperedge_construction() {
        let ids = IdGen::new();
        let e = HyperEdge::new(
            ids.next_edge_id(),
            vec![ids.next_port_id()],
            vec![ids.next_port_id()],
            EdgeTransform::PassThrough,
            EdgeSchedule::OnChange,
        );
        assert_eq!(e.sources.len(), 1);
        assert_eq!(e.targets.len(), 1);
    }

    #[test]
    fn on_tick_schedule() {
        let s = EdgeSchedule::OnTick(Duration::from_millis(100));
        assert!(matches!(s, EdgeSchedule::OnTick(_)));
    }

    #[test]
    fn script_debug() {
        let t = EdgeTransform::Script("x + 1".to_string());
        let s = format!("{:?}", t);
        assert!(s.contains("Script"));
        assert!(s.contains("bytes"));
    }

    /// Script transforms must survive a serde JSON round-trip intact — unlike
    /// `Inline` which degrades to `PassThrough` on deserialization.
    #[test]
    fn script_serde_round_trip() {
        let original = EdgeTransform::Script("inputs[0] + 1".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let restored: EdgeTransform = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(restored, EdgeTransform::Script(ref s) if s == "inputs[0] + 1"),
            "expected Script variant, got {:?}",
            restored
        );
    }

    /// Inline closures still degrade to PassThrough (unrepresentable in JSON).
    #[test]
    fn inline_degrades_to_passthrough_on_serde() {
        let original = EdgeTransform::Inline(Arc::new(|_| Value::Null));
        let json = serde_json::to_string(&original).unwrap();
        let restored: EdgeTransform = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, EdgeTransform::PassThrough));
    }
}
