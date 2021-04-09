use petgraph::graph::{Graph as PetGraph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// we don't care about weights, so we use unit for them
type WeightType = ();
const WEIGHT: WeightType = ();

pub(crate) struct Graph<N> {
    /// Mapping from the node data `N` to its index
    nodes: HashMap<N, NodeIndex>,
    /// Graph data. We use the node data `N` as the node weight, which allows us
    /// to have the inverse of the mapping above using `self.graph.node_weight`.
    graph: PetGraph<N, WeightType>,
}

impl<N> Graph<N>
where
    N: Eq + Hash + Clone,
{
    /// Creates a new graph.
    pub(crate) fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            graph: PetGraph::new(),
        }
    }

    /// Adds a new path (in case it hasn't been added) to the graph.
    pub(crate) fn add_path(&mut self, path: Vec<N>) {
        let mut path_iter = path.into_iter();
        if let Some(mut previous_node) = path_iter.next() {
            // if the path is non-empty, then iterate the remaining nodes in the
            // path and add an edge between consecutive nodes
            for node in path_iter {
                self.add_edge(&previous_node, &node);
                previous_node = node;
            }
        }
    }

    /// Computes all paths starting from node `from` with a length up to `max_len`.
    pub(crate) fn all_paths(&self, from: &N, max_len: usize) -> HashSet<Vec<N>> {
        if max_len == 0 {
            return HashSet::new();
        }

        let mut result = HashSet::new();
        if let Some(from_index) = self.nodes.get(from) {
            // if the node `from` exists, compute paths starting with it
            self.do_all_paths(*from_index, max_len, Vec::new(), &mut result);
        }
        result
    }

    /// Perform a depth-first-search until the path traversed reaches the
    /// supplied max length.
    fn do_all_paths(
        &self,
        node_index: NodeIndex,
        max_len: usize,
        mut path: Vec<N>,
        result: &mut HashSet<Vec<N>>,
    ) {
        // retrieve node data
        let node = self
            .graph
            .node_weight(node_index)
            .expect("[modelator] indexed node should exist in the graph")
            .clone();
        // add `node` to the path
        path.push(node);

        // add new path to results
        assert!(
            result.insert(path.clone()),
            "[modelator] paths cannot be duplicated"
        );

        // if we have reached the max path length, give up
        if path.len() == max_len {
            return;
        }

        for neighbor_index in self.graph.neighbors(node_index) {
            self.do_all_paths(neighbor_index, max_len, path.clone(), result);
        }
    }

    /// Adds a new edge (in case it hasn't been added) to the graph.
    fn add_edge(&mut self, from: &N, to: &N) {
        let from = self.node_index(from);
        let to = self.node_index(to);
        // by using `update_edge`, we ensure that the edge is never duplicated
        self.graph.update_edge(from, to, WEIGHT);
    }

    /// Adds a new node (in case it hasn't been added) to the graph and
    /// retrieves its index.
    fn node_index(&mut self, node: &N) -> NodeIndex {
        if let Some(index) = self.nodes.get(&node) {
            // retrieve index if the node already exists
            *index
        } else {
            // otherwise, insert the node and return the newly assigned index
            let index = self.graph.add_node(node.clone());
            self.nodes.insert(node.clone(), index);
            index
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    fn entry(
        a: impl Into<Option<usize>>,
        b: impl Into<Option<usize>>,
    ) -> (Option<usize>, Option<usize>) {
        (a.into(), b.into())
    }

    #[test]
    fn all_paths_test() {
        let mut graph = Graph::new();
        let start_node = entry(None, None);
        let history0 = vec![start_node];
        let history1 = vec![start_node, entry(0, 0)];
        let history2 = vec![start_node, entry(0, 0), entry(1, 0)];
        let history3 = vec![start_node, entry(0, 0), entry(0, 2)];
        let history4 = vec![start_node, entry(0, 0), entry(0, 0)];
        let history5 = vec![start_node, entry(0, 0), entry(0, 0), entry(1, 0)];
        let history6 = vec![start_node, entry(0, 0), entry(0, 0), entry(0, 2)];
        let history7 = vec![start_node, entry(0, 0), entry(0, 0), entry(0, 0)];

        graph.add_path(history0.clone());
        assert_eq!(
            graph.all_paths(&start_node, 1),
            HashSet::from_iter(vec![history0.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 2),
            HashSet::from_iter(vec![history0.clone()])
        );

        graph.add_path(history1.clone());
        assert_eq!(
            graph.all_paths(&start_node, 1),
            HashSet::from_iter(vec![history0.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 2),
            HashSet::from_iter(vec![history0.clone(), history1.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 3),
            HashSet::from_iter(vec![history0.clone(), history1.clone()])
        );

        graph.add_path(history2.clone());
        assert_eq!(
            graph.all_paths(&start_node, 1),
            HashSet::from_iter(vec![history0.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 2),
            HashSet::from_iter(vec![history0.clone(), history1.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 3),
            HashSet::from_iter(vec![history0.clone(), history1.clone(), history2.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 4),
            HashSet::from_iter(vec![history0.clone(), history1.clone(), history2.clone()])
        );

        graph.add_path(history3.clone());
        assert_eq!(
            graph.all_paths(&start_node, 1),
            HashSet::from_iter(vec![history0.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 2),
            HashSet::from_iter(vec![history0.clone(), history1.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 3),
            HashSet::from_iter(vec![
                history0.clone(),
                history1.clone(),
                history2.clone(),
                history3.clone()
            ])
        );
        assert_eq!(
            graph.all_paths(&start_node, 4),
            HashSet::from_iter(vec![
                history0.clone(),
                history1.clone(),
                history2.clone(),
                history3.clone()
            ])
        );

        // here we add a cycle, so we already have infinite paths.
        // this is nice because it means we can avoid some queries to the model
        // checker (e.g., after seeing history4, given the previous histories,
        // we can guess history5, history6 and history7)
        graph.add_path(history4.clone());
        assert_eq!(
            graph.all_paths(&start_node, 1),
            HashSet::from_iter(vec![history0.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 2),
            HashSet::from_iter(vec![history0.clone(), history1.clone()])
        );
        assert_eq!(
            graph.all_paths(&start_node, 3),
            HashSet::from_iter(vec![
                history0.clone(),
                history1.clone(),
                history2.clone(),
                history3.clone(),
                history4.clone()
            ])
        );
        assert_eq!(
            graph.all_paths(&start_node, 4),
            HashSet::from_iter(vec![
                history0.clone(),
                history1.clone(),
                history2.clone(),
                history3.clone(),
                history4.clone(),
                history5.clone(),
                history6.clone(),
                history7.clone(),
            ])
        );
    }
}
