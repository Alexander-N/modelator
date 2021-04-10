use super::TlaState;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// `modelator`'s artifact containing a set of TLA+ states encoded both as TLA+ and and as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlaNextStates {
    tla_next_states: Vec<TlaAndJsonState>,
}

impl TlaNextStates {
    pub(crate) fn new() -> Self {
        Self {
            tla_next_states: Vec::new(),
        }
    }

    pub(crate) fn add(&mut self, next_state: TlaAndJsonState) {
        self.tla_next_states.push(next_state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TlaAndJsonState {
    tla_state: TlaState,
    json_state: JsonValue,
}

impl TlaAndJsonState {
    pub(crate) fn new(tla_state: TlaState, json_state: JsonValue) -> Self {
        Self {
            tla_state,
            json_state,
        }
    }
}
