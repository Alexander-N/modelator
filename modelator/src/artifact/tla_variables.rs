use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// `modelator`'s artifact representing a set of TLA+ variables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlaVariables {
    vars: HashSet<String>,
}

impl TlaVariables {
    pub(crate) fn new(vars: HashSet<String>) -> Self {
        Self { vars }
    }

    /// Returns the set of TLA+ variables.
    pub fn vars(&self) -> &HashSet<String> {
        &self.vars
    }
}

impl std::fmt::Display for TlaVariables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.vars)
    }
}
