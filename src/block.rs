use std::collections::HashSet;

use plexus_base::{AtomId, BlockId};
use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::meta::Meta;
use crate::port::{Port, PortDirection};

// ---------------------------------------------------------------------------
// BlockSchema
// ---------------------------------------------------------------------------

/// Declares the typed port interface of a [`Block`]: which inputs it accepts
/// and which outputs it produces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockSchema {
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

impl BlockSchema {
    /// Create a new schema from explicit port lists.
    pub fn new(inputs: Vec<Port>, outputs: Vec<Port>) -> Self {
        Self { inputs, outputs }
    }

    /// Validate that:
    /// - All input ports have `direction == Input`
    /// - All output ports have `direction == Output`
    /// - No two inputs share a name
    /// - No two outputs share a name
    pub fn validate(&self) -> Result<(), DomainError> {
        let mut seen_inputs = HashSet::new();
        for p in &self.inputs {
            if p.direction != PortDirection::Input {
                return Err(DomainError::InvalidSchema(format!(
                    "port '{}' listed as input but has direction {}",
                    p.name, p.direction
                )));
            }
            if !seen_inputs.insert(&p.name) {
                return Err(DomainError::DuplicatePort(p.name.clone()));
            }
        }

        let mut seen_outputs = HashSet::new();
        for p in &self.outputs {
            if p.direction != PortDirection::Output {
                return Err(DomainError::InvalidSchema(format!(
                    "port '{}' listed as output but has direction {}",
                    p.name, p.direction
                )));
            }
            if !seen_outputs.insert(&p.name) {
                return Err(DomainError::DuplicatePort(p.name.clone()));
            }
        }

        Ok(())
    }

    /// Find an input port by name.
    pub fn find_input(&self, name: &str) -> Option<&Port> {
        self.inputs.iter().find(|p| p.name == name)
    }

    /// Find an output port by name.
    pub fn find_output(&self, name: &str) -> Option<&Port> {
        self.outputs.iter().find(|p| p.name == name)
    }

    /// Find any port (input or output) by name.
    pub fn find_port(&self, name: &str) -> Option<&Port> {
        self.find_input(name).or_else(|| self.find_output(name))
    }
}

// ---------------------------------------------------------------------------
// Block
// ---------------------------------------------------------------------------

/// A composable unit in a Plexus graph: a group of [`Atom`](crate::atom::Atom)s
/// with a shared typed port interface ([`BlockSchema`]) and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub schema: BlockSchema,
    /// IDs of the atoms that make up this block.
    pub atoms: Vec<AtomId>,
    pub meta: Meta,
}

impl Block {
    /// Create a block directly (prefer [`BlockBuilder`] for ergonomics).
    pub fn new(id: BlockId, schema: BlockSchema, atoms: Vec<AtomId>, meta: Meta) -> Self {
        Self { id, schema, atoms, meta }
    }
}

// ---------------------------------------------------------------------------
// loom-base integration: HasBlockId
// ---------------------------------------------------------------------------

impl loom_base::memory::HasBlockId for Block {
    fn block_id(&self) -> BlockId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// BlockBuilder
// ---------------------------------------------------------------------------

/// Fluent builder for [`Block`].
#[derive(Default)]
pub struct BlockBuilder {
    id: Option<BlockId>,
    schema: Option<BlockSchema>,
    atoms: Vec<AtomId>,
    meta: Meta,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: BlockId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn schema(mut self, schema: BlockSchema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn atom(mut self, id: AtomId) -> Self {
        self.atoms.push(id);
        self
    }

    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta = self.meta.with_label(key, value);
        self
    }

    /// Build the [`Block`], validating the schema before returning.
    pub fn build(self) -> Result<Block, DomainError> {
        let id = self.id.ok_or_else(|| DomainError::MissingField("id".into()))?;
        let schema = self.schema.ok_or_else(|| DomainError::MissingField("schema".into()))?;
        schema.validate()?;
        Ok(Block::new(id, schema, self.atoms, self.meta))
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

    fn make_schema(ids: &IdGen) -> BlockSchema {
        BlockSchema::new(
            vec![Port::new_input(ids.next_port_id(), "x", TypeSig::Int)],
            vec![Port::new_output(ids.next_port_id(), "y", TypeSig::Int)],
        )
    }

    #[test]
    fn schema_validate_passes() {
        let ids = IdGen::new();
        assert!(make_schema(&ids).validate().is_ok());
    }

    #[test]
    fn schema_duplicate_input_name_fails() {
        let ids = IdGen::new();
        let schema = BlockSchema::new(
            vec![
                Port::new_input(ids.next_port_id(), "x", TypeSig::Int),
                Port::new_input(ids.next_port_id(), "x", TypeSig::Float),
            ],
            vec![],
        );
        assert!(matches!(schema.validate(), Err(DomainError::DuplicatePort(_))));
    }

    #[test]
    fn block_builder_builds_successfully() {
        let ids = IdGen::new();
        let block = BlockBuilder::new()
            .id(ids.next_block_id())
            .schema(make_schema(&ids))
            .label("env", "test")
            .build()
            .unwrap();
        assert_eq!(block.meta.labels["env"], "test");
    }

    #[test]
    fn block_builder_missing_id_fails() {
        let ids = IdGen::new();
        let err = BlockBuilder::new().schema(make_schema(&ids)).build().unwrap_err();
        assert!(matches!(err, DomainError::MissingField(_)));
    }

    #[test]
    fn schema_find_port() {
        let ids = IdGen::new();
        let schema = make_schema(&ids);
        assert!(schema.find_input("x").is_some());
        assert!(schema.find_output("y").is_some());
        assert!(schema.find_port("x").is_some());
        assert!(schema.find_port("z").is_none());
    }
}
