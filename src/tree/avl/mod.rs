use crate::tree::ordered::OrderedTree;
use crate::tree::trace::{NodeMetadata, NodeSnapshot, Trace, TreeSnapshot};
use std::cmp::Ordering;

pub struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    height: i32,
}

type Link<K, V> = Option<Box<Node<K, V>>>;

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            height: 1,
            left: None,
            right: None,
        }
    }

    fn child_height(node: &Link<K, V>) -> i32 {
        node.as_ref().map_or(0, |node| node.height)
    }

    fn update_height(&mut self) {
        let left_height = Self::child_height(&self.left);
        let right_height = Self::child_height(&self.right);

        self.height = 1 + left_height.max(right_height);
    }

    fn get_balance(&self) -> i32 {
        Self::child_height(&self.left) - Self::child_height(&self.right)
    }
}

pub struct AvlTree<K, V> {
    root: Link<K, V>,
}

impl<K: Ord, V> OrderedTree<K, V> for AvlTree<K, V> {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, key: K, value: V) {
        self.root = Self::insert_inner(self.root.take(), key, value);
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let (root, value) = Self::remove_inner(self.root.take(), key);

        self.root = root;
        value
    }

    fn search(&self, key: &K) -> Option<&V> {
        let mut current = self.root.as_deref();

        while let Some(node) = current {
            match key.cmp(&node.key) {
                Ordering::Equal => return Some(&node.value),
                Ordering::Less => current = node.left.as_deref(),
                Ordering::Greater => current = node.right.as_deref(),
            }
        }

        None
    }
}

impl<K: Ord, V> AvlTree<K, V> {
    pub fn snapshot(&self) -> TreeSnapshot<K, V>
    where
        K: Clone,
        V: Clone,
    {
        TreeSnapshot {
            root: Self::snapshot_node(self.root.as_deref()),
        }
    }

    fn snapshot_node(node: Option<&Node<K, V>>) -> Option<Box<NodeSnapshot<K, V>>>
    where
        K: Clone,
        V: Clone,
    {
        node.map(|node| {
            Box::new(NodeSnapshot {
                key: node.key.clone(),
                value: node.value.clone(),
                metadata: NodeMetadata::Avl {
                    height: node.height,
                    balance: node.get_balance(),
                },
                left: Self::snapshot_node(node.left.as_deref()),
                right: Self::snapshot_node(node.right.as_deref()),
            })
        })
    }

    pub fn insert_with_trace(&mut self, key: K, value: V) -> Trace<K, V>
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        let mut trace = Trace::new(self.snapshot());
        self.root = Self::insert_traced(self.root.take(), key, value, &mut Vec::new(), &mut trace);
        trace.final_state = self.snapshot();
        trace.record("Insertion complete", trace.final_state.clone());
        trace
    }

    pub fn remove_with_trace(&mut self, key: &K) -> (Option<V>, Trace<K, V>)
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        let mut trace = Trace::new(self.snapshot());
        let (root, value) = Self::remove_traced(self.root.take(), key, &mut Vec::new(), &mut trace);
        self.root = root;
        trace.final_state = self.snapshot();
        trace.record("Tree changed after removal", trace.final_state.clone());
        (value, trace)
    }

    // Insert Recursion
    fn insert_inner(node: Link<K, V>, key: K, value: V) -> Link<K, V> {
        match node {
            None => Some(Box::new(Node::new(key, value))),
            Some(mut node) => {
                if key < node.key {
                    node.left = Self::insert_inner(node.left.take(), key, value);
                } else if key > node.key {
                    node.right = Self::insert_inner(node.right.take(), key, value);
                } else {
                    node.value = value;
                    return Some(node);
                }

                Some(Self::rebalance(node))
            }
        }
    }

    fn insert_traced(
        node: Link<K, V>,
        key: K,
        value: V,
        path: &mut Vec<(NodeSnapshot<K, V>, bool)>,
        trace: &mut Trace<K, V>,
    ) -> Link<K, V>
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        match node {
            None => {
                let node = Box::new(Node::new(key, value));
                Self::record_traced_subtree(&node, path, "Insert leaf", trace);
                Some(node)
            }
            Some(mut node) => {
                match key.cmp(&node.key) {
                    Ordering::Less => {
                        path.push((
                            Self::snapshot_node(Some(&node)).unwrap().as_ref().clone(),
                            true,
                        ));
                        node.left = Self::insert_traced(node.left.take(), key, value, path, trace);
                        path.pop();
                    }
                    Ordering::Greater => {
                        path.push((
                            Self::snapshot_node(Some(&node)).unwrap().as_ref().clone(),
                            false,
                        ));
                        node.right =
                            Self::insert_traced(node.right.take(), key, value, path, trace);
                        path.pop();
                    }
                    Ordering::Equal => {
                        node.value = value;
                        return Some(node);
                    }
                }
                Some(Self::rebalance_traced(node, path, trace))
            }
        }
    }

    fn record_traced_subtree(
        node: &Node<K, V>,
        path: &[(NodeSnapshot<K, V>, bool)],
        label: &str,
        trace: &mut Trace<K, V>,
    ) where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        trace.record(label, Self::snapshot_with_path(node, path));
    }

    fn record_traced_rotation(
        node: &Node<K, V>,
        path: &[(NodeSnapshot<K, V>, bool)],
        label: &str,
        pivot: K,
        trace: &mut Trace<K, V>,
    ) where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        trace.record_highlighted(label, Self::snapshot_with_path(node, path), pivot);
    }

    fn snapshot_with_path(
        node: &Node<K, V>,
        path: &[(NodeSnapshot<K, V>, bool)],
    ) -> TreeSnapshot<K, V>
    where
        K: Clone,
        V: Clone,
    {
        let mut root = Self::snapshot_node(Some(node));
        for (ancestor, descended_left) in path.iter().rev() {
            let mut ancestor = ancestor.clone();
            if *descended_left {
                ancestor.left = root;
            } else {
                ancestor.right = root;
            }
            let left_height = snapshot_height(&ancestor.left);
            let right_height = snapshot_height(&ancestor.right);
            ancestor.metadata = NodeMetadata::Avl {
                height: 1 + left_height.max(right_height),
                balance: left_height - right_height,
            };
            root = Some(Box::new(ancestor));
        }
        TreeSnapshot { root }
    }

    fn rebalance_traced(
        mut node: Box<Node<K, V>>,
        path: &[(NodeSnapshot<K, V>, bool)],
        trace: &mut Trace<K, V>,
    ) -> Box<Node<K, V>>
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        node.update_height();
        match node.get_balance() {
            balance if balance > 1 => {
                let left_balance = node.left.as_ref().map_or(0, |child| child.get_balance());
                if left_balance < 0 {
                    node.left = Some(Self::rotate_left(
                        node.left.take().expect("LR rotation needs a left child"),
                    ));
                    let pivot = node.left.as_ref().unwrap().key.clone();
                    Self::record_traced_rotation(
                        &node,
                        path,
                        "LR rotation: left child",
                        pivot,
                        trace,
                    );
                }
                let case = if left_balance < 0 { "LR" } else { "LL" };
                let node = Self::rotate_right(node);
                let pivot = node.key.clone();
                Self::record_traced_rotation(
                    &node,
                    path,
                    &format!("{case} rotation: right"),
                    pivot,
                    trace,
                );
                node
            }
            balance if balance < -1 => {
                let right_balance = node.right.as_ref().map_or(0, |child| child.get_balance());
                if right_balance > 0 {
                    node.right = Some(Self::rotate_right(
                        node.right.take().expect("RL rotation needs a right child"),
                    ));
                    let pivot = node.right.as_ref().unwrap().key.clone();
                    Self::record_traced_rotation(
                        &node,
                        path,
                        "RL rotation: right child",
                        pivot,
                        trace,
                    );
                }
                let case = if right_balance > 0 { "RL" } else { "RR" };
                let node = Self::rotate_left(node);
                let pivot = node.key.clone();
                Self::record_traced_rotation(
                    &node,
                    path,
                    &format!("{case} rotation: left"),
                    pivot,
                    trace,
                );
                node
            }
            _ => node,
        }
    }

    fn remove_traced(
        node: Link<K, V>,
        key: &K,
        path: &mut Vec<(NodeSnapshot<K, V>, bool)>,
        trace: &mut Trace<K, V>,
    ) -> (Link<K, V>, Option<V>)
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        let Some(mut node) = node else {
            return (None, None);
        };
        match key.cmp(&node.key) {
            Ordering::Equal => {
                let removed = node.value.clone();
                let removed_snapshot = Self::snapshot_node(Some(&node)).unwrap().as_ref().clone();
                match (node.left, node.right) {
                    (Some(left), Some(right)) => {
                        path.push((removed_snapshot, false));
                        let (right, mut successor) = Self::remove_min_traced(right, path, trace);
                        path.pop();
                        successor.left = Some(left);
                        successor.right = right;
                        let successor = Self::rebalance_traced(successor, path, trace);
                        Self::record_traced_subtree(
                            &successor,
                            path,
                            "Replace removed node",
                            trace,
                        );
                        (Some(successor), Some(removed))
                    }
                    (Some(child), None) | (None, Some(child)) => {
                        Self::record_traced_subtree(&child, path, "Remove node", trace);
                        (Some(child), Some(removed))
                    }
                    (None, None) => (None, Some(removed)),
                }
            }
            direction => {
                let went_left = direction == Ordering::Less;
                let ancestor = Self::snapshot_node(Some(&node)).unwrap().as_ref().clone();
                let removed = if went_left {
                    path.push((ancestor, true));
                    let result = Self::remove_traced(node.left.take(), key, path, trace);
                    path.pop();
                    node.left = result.0;
                    result.1
                } else {
                    path.push((ancestor, false));
                    let result = Self::remove_traced(node.right.take(), key, path, trace);
                    path.pop();
                    node.right = result.0;
                    result.1
                };
                if removed.is_some() {
                    let node = Self::rebalance_traced(node, path, trace);
                    Self::record_traced_subtree(&node, path, "Rebalance after removal", trace);
                    (Some(node), removed)
                } else {
                    (Some(node), None)
                }
            }
        }
    }

    fn remove_min_traced(
        mut node: Box<Node<K, V>>,
        path: &mut Vec<(NodeSnapshot<K, V>, bool)>,
        trace: &mut Trace<K, V>,
    ) -> (Link<K, V>, Box<Node<K, V>>)
    where
        K: Clone + PartialEq,
        V: Clone + PartialEq,
    {
        match node.left.take() {
            None => {
                let remainder = node.right.take();
                if let Some(remainder_node) = remainder.as_deref() {
                    Self::record_traced_subtree(remainder_node, path, "Remove successor", trace);
                }
                (remainder, node)
            }
            Some(left) => {
                path.push((
                    Self::snapshot_node(Some(&node)).unwrap().as_ref().clone(),
                    true,
                ));
                let (left, min) = Self::remove_min_traced(left, path, trace);
                path.pop();
                node.left = left;
                let node = Self::rebalance_traced(node, path, trace);
                Self::record_traced_subtree(&node, path, "Rebalance successor path", trace);
                (Some(node), min)
            }
        }
    }

    // Remove Recursion
    fn remove_inner(node: Link<K, V>, key: &K) -> (Link<K, V>, Option<V>) {
        match node {
            None => (node, None),
            Some(mut node) => match key.cmp(&node.key) {
                Ordering::Equal => match (node.left, node.right) {
                    (Some(left), Some(right)) => {
                        let (right, mut successor) = Self::remove_min(right);
                        successor.left = Some(left);
                        successor.right = right;

                        (Some(Self::rebalance(successor)), Some(node.value))
                    }
                    (None, Some(right)) => (Some(right), Some(node.value)),
                    (Some(left), None) => (Some(left), Some(node.value)),
                    (None, None) => (None, Some(node.value)),
                },
                Ordering::Greater => {
                    let (child, value) = Self::remove_inner(node.right.take(), key);
                    node.right = child;

                    (Some(Self::rebalance(node)), value)
                }
                Ordering::Less => {
                    let (child, value) = Self::remove_inner(node.left.take(), key);
                    node.left = child;

                    (Some(Self::rebalance(node)), value)
                }
            },
        }
    }

    fn remove_min(mut node: Box<Node<K, V>>) -> (Link<K, V>, Box<Node<K, V>>) {
        match node.left.take() {
            None => (node.right.take(), node),
            Some(left) => {
                let (new_left, min) = Self::remove_min(left);
                node.left = new_left;

                (Some(Self::rebalance(node)), min)
            }
        }
    }

    /// Rebalances the tree in one of the four patterns (LL, RR, LR, RL)
    fn rebalance(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        node.update_height();

        match node.get_balance() {
            balance if balance > 1 => {
                let left_balance = node.left.as_ref().map_or(0, |child| child.get_balance());

                if left_balance < 0 {
                    node.left = Some(Self::rotate_left(
                        node.left.take().expect("LR rotation needs a left child"),
                    ));
                }

                Self::rotate_right(node)
            }
            balance if balance < -1 => {
                let right_balance = node.right.as_ref().map_or(0, |child| child.get_balance());

                if right_balance > 0 {
                    node.right = Some(Self::rotate_right(
                        node.right.take().expect("RL rotation needs a right child"),
                    ));
                }

                Self::rotate_left(node)
            }
            _ => node,
        }
    }

    fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut pivot = node.left.take().expect("right rotation needs a left child");

        node.left = pivot.right.take();
        node.update_height();

        pivot.right = Some(node);
        pivot.update_height();

        pivot
    }

    fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut pivot = node
            .right
            .take()
            .expect("left rotation needs a right child");

        node.right = pivot.left.take();
        node.update_height();

        pivot.left = Some(node);
        pivot.update_height();

        pivot
    }
}

fn snapshot_height<K, V>(node: &Option<Box<NodeSnapshot<K, V>>>) -> i32 {
    node.as_ref().map_or(0, |node| match node.metadata {
        NodeMetadata::Avl { height, .. } => height,
        NodeMetadata::RedBlack { .. } => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{AvlTree, Link};
    use crate::tree::ordered::OrderedTree;
    use proptest::prelude::*;

    fn assert_valid<K: Ord, V>(node: &Link<K, V>, lower: Option<&K>, upper: Option<&K>) -> i32 {
        let Some(node) = node else {
            return 0;
        };

        assert!(lower.is_none_or(|lower| lower < &node.key));
        assert!(upper.is_none_or(|upper| &node.key < upper));

        let left_height = assert_valid(&node.left, lower, Some(&node.key));
        let right_height = assert_valid(&node.right, Some(&node.key), upper);
        assert_eq!(node.height, 1 + left_height.max(right_height));
        assert!(
            left_height.abs_diff(right_height) <= 1,
            "AVL balance violated"
        );
        node.height
    }

    proptest! {
        #[test]
        fn operations_preserve_avl_invariants(operations in prop::collection::vec((any::<bool>(), -50_i32..50), 0..150)) {
            let mut tree = AvlTree::new();

            for (insert, key) in operations {
                if insert {
                    tree.insert(key, key * 10);
                } else {
                    tree.remove(&key);
                }
                assert_valid(&tree.root, None, None);
            }
        }
    }
}
