use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// `modelator`'s artifact representing a set of TLA+ variables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlaVariables {
    tla_variables: BTreeSet<String>,
}

impl TlaVariables {
    pub(crate) fn new(tla_variables: BTreeSet<String>) -> Self {
        Self { tla_variables }
    }

    /// Returns the set of TLA+ variables.
    pub fn vars(&self) -> &BTreeSet<String> {
        &self.tla_variables
    }
}
