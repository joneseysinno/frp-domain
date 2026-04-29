use thiserror::Error;

/// Errors produced by domain model validation logic.
#[derive(Debug, Error)]
pub enum DomainError {
    /// A `BlockSchema` or `Atom` port definition was structurally invalid.
    #[error("invalid schema: {0}")]
    InvalidSchema(String),

    /// A required port was not found.
    #[error("missing port: {0}")]
    MissingPort(String),

    /// Two or more ports share the same name within the same direction.
    #[error("duplicate port name: {0}")]
    DuplicatePort(String),

    /// A required field was not provided to a builder.
    #[error("missing required field: {0}")]
    MissingField(String),
}
