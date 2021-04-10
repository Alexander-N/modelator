use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// `modelator`'s artifact representing a set of TLA+ variables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlaVariables {
    tla_variables: HashSet<String>,
}

impl TlaVariables {
    pub(crate) fn new(tla_variables: HashSet<String>) -> Self {
        Self { tla_variables }
    }

    /// Returns the set of TLA+ variables.
    pub fn vars(&self) -> &HashSet<String> {
        &self.tla_variables
    }
}
