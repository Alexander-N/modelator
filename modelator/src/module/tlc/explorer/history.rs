use super::graph::Graph;
use std::collections::{BTreeMap, HashSet};

/// A TLA+ state is represented as a mapping from the TLA+ variable to the TLA+
/// value associated with that variable.
#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) struct TlaState {
    state: BTreeMap<String, String>,
}

/// A set of histories found by the model checker.
pub(crate) struct Histories {
    /// The initial state of the TLA+ model.
    initial_state: TlaState,
    /// A graph representing all histories found by the model checker (and
    /// possibly more).
    graph: Graph<TlaState>,
    /// The maximum length of an history found by the model checker.
    max_history_len: usize,
}

impl Histories {
    /// Creates a new set of histories.
    pub(crate) fn new(initial_state: TlaState) -> Self {
        Self {
            initial_state,
            graph: Graph::new(),
            max_history_len: 0,
        }
    }

    /// Adds a new history to the set of `TlaStates`.
    pub(crate) fn add_history(&mut self, history: Vec<TlaState>) {
        self.max_history_len = std::cmp::max(self.max_history_len, history.len());
        self.graph.add_path(history);
    }

    /// Computes the set of histories found by the model checker, potentially
    /// guessing other ones.
    pub(crate) fn all_histories(&self) -> HashSet<Vec<TlaState>> {
        // by setting `max_len` to `self.max_history_len + 1`, we allow our
        // graph to correctly guess histories that it hasn't observe yet
        let max_len = self.max_history_len + 1;
        self.graph.all_paths(&self.initial_state, max_len)
    }
}
