use petgraph::graph::{Graph as PetGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Serialize, Deserialize)]
pub(crate) struct NextStates<S: Eq + Hash> {
    /// Mapping from state `S` to a list of next states.
    next_states: HashMap<S, Vec<S>>,
}

impl<S> NextStates<S>
where
    S: Eq + Hash + Clone + Debug,
{
    /// Creates a new graph.
    pub(crate) fn new() -> Self {
        Self {
            next_states: HashMap::new(),
        }
    }

    /// Adds a new `next_state` to `state`.
    pub(crate) fn add_next_state(&mut self, state: S, next_state: S) {
        let next_states = self.next_states.entry(state).or_default();
        debug_assert!(
            next_states.iter().all(|s| s != &next_state),
            "[modelator] unexpected repeated next state"
        );
        next_states.push(next_state);
    }

    /// Retrieves the set of next states of `state`.
    pub(crate) fn get_next_states(&mut self, state: &S) -> Option<&Vec<S>> {
        self.next_states.get(state)
    }

    /// Dot representation of self when viewed as a state graph.
    pub(crate) fn dot(&self) -> String {
        let mut state_graph = StateGraph::new();
        for (state, next_states) in self.next_states.iter() {
            for next_state in next_states {
                let state = format!("{:?}", state);
                let next_state = format!("{:?}", next_state);
                state_graph.add_neighbor(state, next_state);
            }
        }
        state_graph.dot()
    }
}

// we don't care about weights, so we use unit for them
type WeightType = ();
const WEIGHT: WeightType = ();

pub(crate) struct StateGraph<S> {
    /// Mapping from the node state `S` to its graph index.
    nodes: HashMap<S, NodeIndex>,
    /// Graph data. We use the node state `S` as the node weight, which allows
    /// us to have the inverse of the mapping above using `self.graph.node_weight`.
    graph: PetGraph<WeightType, WeightType>,
}

impl<S> StateGraph<S>
where
    S: Eq + Hash + Debug,
{
    /// Creates a new state graph.
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            graph: PetGraph::new(),
        }
    }

    /// Adds a `next_state` to `state`.
    fn add_neighbor(&mut self, state: S, next_state: S) {
        let node_index = self.add_node(state);
        let neighbor_index = self.add_node(next_state);
        self.add_edge(node_index, neighbor_index);
    }

    /// Dot representation of this state graph.
    fn dot(self) -> String {
        // reverse `self.nodes` mapping
        let nodes_len = self.nodes.len();
        let mut index_to_node: HashMap<_, _> =
            self.nodes.into_iter().map(|(k, v)| (v, k)).collect();
        assert_eq!(
            nodes_len,
            index_to_node.len(),
            "[modelator] reversed index should have the same length as values are unique"
        );

        // create a new graph, setting for each node their TLA+ state.
        let graph = self.graph.map(
            |node_index, _| {
                index_to_node
                    .remove(&node_index)
                    .expect("[modelator] graph node must be indexed")
            },
            |_, e| e,
        );

        // show no label for edges
        let config = &[petgraph::dot::Config::EdgeNoLabel];
        let dot = petgraph::dot::Dot::with_config(&graph, config);
        format!("{:?}", dot)
    }

    /// Adds a new edge (in case it hasn't been added) to the graph.
    fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        // by using `update_edge`, we ensure that the edge is never duplicated
        self.graph.update_edge(from, to, WEIGHT);
    }

    /// Adds a new node (in case it hasn't been added) to the graph and
    /// retrieves its index.
    fn add_node(&mut self, node: S) -> NodeIndex {
        if let Some(index) = self.nodes.get(&node) {
            // retrieve index if the node already exists
            *index
        } else {
            // otherwise, insert the node and return the newly assigned index
            let index = self.graph.add_node(WEIGHT);
            self.nodes.insert(node, index);
            index
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbors_test() {
        let mut next_states = NextStates::new();

        assert_eq!(next_states.get_next_states(&1), None);
        assert_eq!(next_states.get_next_states(&2), None);
        assert_eq!(next_states.get_next_states(&3), None);
        assert_eq!(next_states.get_next_states(&4), None);

        next_states.add_next_state(1, 2);
        assert_eq!(next_states.get_next_states(&1), Some(&vec![2]));
        assert_eq!(next_states.get_next_states(&2), None);
        assert_eq!(next_states.get_next_states(&3), None);
        assert_eq!(next_states.get_next_states(&4), None);

        next_states.add_next_state(1, 3);
        assert_eq!(next_states.get_next_states(&1), Some(&vec![2, 3]));
        assert_eq!(next_states.get_next_states(&2), None);
        assert_eq!(next_states.get_next_states(&3), None);
        assert_eq!(next_states.get_next_states(&4), None);

        next_states.add_next_state(2, 4);
        assert_eq!(next_states.get_next_states(&1), Some(&vec![2, 3]));
        assert_eq!(next_states.get_next_states(&2), Some(&vec![4]));
        assert_eq!(next_states.get_next_states(&3), None);
        assert_eq!(next_states.get_next_states(&4), None);

        next_states.add_next_state(3, 4);
        assert_eq!(next_states.get_next_states(&1), Some(&vec![2, 3]));
        assert_eq!(next_states.get_next_states(&2), Some(&vec![4]));
        assert_eq!(next_states.get_next_states(&3), Some(&vec![4]));
        assert_eq!(next_states.get_next_states(&4), None);

        next_states.add_next_state(4, 1);
        assert_eq!(next_states.get_next_states(&1), Some(&vec![2, 3]));
        assert_eq!(next_states.get_next_states(&2), Some(&vec![4]));
        assert_eq!(next_states.get_next_states(&3), Some(&vec![4]));
        assert_eq!(next_states.get_next_states(&4), Some(&vec![1]));
    }
}
