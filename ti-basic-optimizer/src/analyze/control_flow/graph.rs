//! # Graph Implementation
//! Existing graph libraries won't do for the sorts of invasive manipulations we need to be able to
//! do efficiently.

use std::cell::{Ref, RefCell, RefMut};

/// Opaque, immutable "name" for a node in a [`Digraph`]. Can be used to obtain a mutable or immutable
/// reference to the associated data with [`Digraph::node`] and [`Digraph::node_mut`].
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct NodeIndex(usize);

/// Implements a directed cyclic graph.
pub struct Digraph<T> {
    nodes: Vec<RefCell<T>>,
    out_arcs: Vec<Vec<usize>>,
    in_arcs: Vec<Vec<usize>>,
}

impl<T> Digraph<T> {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            out_arcs: vec![],
            in_arcs: vec![],
        }
    }

    /// Returns a Vec with the direct successors of the provided block.
    pub fn out_arcs(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.out_arcs[idx.0].iter().map(|&x| NodeIndex(x)).collect()
    }

    /// Returns a Vec with the direct predecessors of the provided block.
    pub fn in_arcs(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        self.in_arcs[idx.0].iter().map(|&x| NodeIndex(x)).collect()
    }

    pub fn out_degree(&self, idx: NodeIndex) -> usize {
        self.out_arcs[idx.0].len()
    }

    pub fn in_degree(&self, idx: NodeIndex) -> usize {
        self.in_arcs[idx.0].len()
    }

    /// Access and mutate the data associated with the provided [`NodeIndex`].
    pub fn node_mut(&self, idx: NodeIndex) -> RefMut<T> {
        self.nodes[idx.0].borrow_mut()
    }

    /// Access the data associated with the provided [`NodeIndex`].
    pub fn node(&self, idx: NodeIndex) -> Ref<T> {
        self.nodes[idx.0].borrow()
    }

    /// Insert a node into the digraph. No edges are created.
    ///
    /// Returns a [`NodeIndex`] for the node which was just inserted.
    pub fn insert_node(&mut self, node: T) -> NodeIndex {
        self.nodes.push(RefCell::new(node));
        self.out_arcs.push(vec![]);
        self.in_arcs.push(vec![]);

        NodeIndex(self.nodes.len() - 1)
    }

    pub fn insert_arc(&mut self, from: NodeIndex, to: NodeIndex) {
        self.out_arcs[from.0].push(to.0);
        self.in_arcs[to.0].push(from.0);
    }
}

#[cfg(test)]
mod tests {
    use crate::analyze::control_flow::graph::Digraph;

    #[test]
    #[should_panic]
    fn cant_have_two_refmut_to_same_block() {
        struct Node(u8);

        let mut g = Digraph::new();

        let a = g.insert_node(Node(1));
        let b = g.insert_node(Node(2));
        g.insert_arc(a, b);

        let a_mut = g.node_mut(a);
        let a_mut_2 = g.node_mut(a);
    }

    #[test]
    fn interior_mutability() {
        struct Node(u8);

        let mut g = Digraph::new();

        let a = g.insert_node(Node(1));
        let b = g.insert_node(Node(2));
        g.insert_arc(a, b);

        {
            let mut a_mut = g.node_mut(a);
            let mut b_mut = g.node_mut(b);

            a_mut.0 += 1;
            b_mut.0 += 1;
        }

        assert_eq!(g.node(a).0, 2);
        assert_eq!(g.node(b).0, 3);
    }
}
