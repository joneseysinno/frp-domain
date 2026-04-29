//! Domain model for the frp graph layer.
//!
//! Defines the core domain types: [`Atom`], [`Block`], [`Port`], [`HyperEdge`],
//! and shared [`Meta`] — plus the [`DomainError`] type for validation failures.

pub mod atom;
pub mod block;
pub mod edge;
pub mod error;
pub mod meta;
pub mod port;

pub use atom::{Atom, AtomKind, AtomMeta};
pub use block::{Block, BlockBuilder, BlockSchema};
pub use edge::{EdgeSchedule, EdgeTransform, HyperEdge};
pub use error::DomainError;
pub use meta::Meta;
pub use port::{Port, PortDirection};
