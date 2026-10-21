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
        K: Clone,
        V: Clone,
    {
        let mut trace = Trace::new(self.snapshot());
        trace.record("Insert key", self.snapshot());
        self.insert(key, value);
        trace.final_state = self.snapshot();
        trace.record("AVL rotation/rebalance complete", trace.final_state.clone());
        trace
    }

    pub fn remove_with_trace(&mut self, key: &K) -> (Option<V>, Trace<K, V>)
    where
        K: Clone,
        V: Clone,
    {
        let mut trace = Trace::new(self.snapshot());
        trace.record("Remove key", self.snapshot());
        let value = self.remove(key);
        trace.final_state = self.snapshot();
        trace.record("AVL rotation/rebalance complete", trace.final_state.clone());
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
