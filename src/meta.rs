use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Shared metadata attached to domain entities (blocks, atoms, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    /// Unix timestamp (milliseconds) when this entity was created.
    pub created_at: u64,
    /// Unix timestamp (milliseconds) of the last update.
    pub updated_at: u64,
    /// Arbitrary key-value labels (sorted for deterministic serialization).
    pub labels: BTreeMap<String, String>,
}

impl Meta {
    /// Create new `Meta` with `created_at` and `updated_at` set to now.
    pub fn new() -> Self {
        let now = now_millis();
        Self { created_at: now, updated_at: now, labels: BTreeMap::new() }
    }

    /// Add or replace a label, consuming and returning `self` for chaining.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Update `updated_at` to the current time.
    pub fn touch(&mut self) {
        self.updated_at = now_millis();
    }
}

impl Default for Meta {
    fn default() -> Self {
        Self::new()
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_meta_has_timestamps() {
        let m = Meta::new();
        assert!(m.created_at > 0);
        assert_eq!(m.created_at, m.updated_at);
    }

    #[test]
    fn with_label_chains() {
        let m = Meta::new().with_label("env", "prod").with_label("tier", "1");
        assert_eq!(m.labels["env"], "prod");
        assert_eq!(m.labels["tier"], "1");
    }
}
