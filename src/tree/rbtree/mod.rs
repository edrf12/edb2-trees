use std::cmp::Ordering;

use crate::tree::ordered::OrderedTree;
use crate::tree::trace::{NodeMetadata, NodeSnapshot, Trace, TreeSnapshot};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

type Link<K, V> = Option<Box<Node<K, V>>>;
struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    color: Color,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            color: Color::Red,
            left: None,
            right: None,
        }
    }
}

pub struct RBTree<K, V> {
    root: Link<K, V>,
}

impl<K: Ord, V> OrderedTree<K, V> for RBTree<K, V> {
    fn new() -> Self {
        Self { root: None }
    }

    fn insert(&mut self, key: K, value: V) {
        self.root = Self::insert_inner(self.root.take(), key, value);
        if let Some(root) = &mut self.root {
            root.color = Color::Black;
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.search(key)?;

        let mut root = self.root.take().expect("the key was found in the tree");
        if !Self::is_red(&root.left) && !Self::is_red(&root.right) {
            root.color = Color::Red;
        }
        let (root, value) = Self::remove_inner(root, key);
        self.root = root;
        if let Some(root) = &mut self.root {
            root.color = Color::Black;
        }
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

impl<K: Ord, V> RBTree<K, V> {
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
                metadata: NodeMetadata::RedBlack {
                    color: if node.color == Color::Red {
                        "red"
                    } else {
                        "black"
                    },
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
        trace.record(
            "Red-black rotation/recoloring complete",
            trace.final_state.clone(),
        );
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
        trace.record(
            "Red-black rotation/recoloring complete",
            trace.final_state.clone(),
        );
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

    // Top-down LLRB deletion: move a red link toward the subtree before descending.
    fn remove_inner(mut node: Box<Node<K, V>>, key: &K) -> (Link<K, V>, Option<V>) {
        let removed;

        if key < &node.key {
            if !Self::is_red(&node.left) && !Self::left_child_is_red(&node.left) {
                node = Self::move_red_left(node);
            }
            let (left, value) = Self::remove_inner(
                node.left
                    .take()
                    .expect("the searched key is in the left subtree"),
                key,
            );
            node.left = left;
            removed = value;
        } else {
            if Self::is_red(&node.left) {
                node = Self::rotate_right(node);
            }

            if key == &node.key && node.right.is_none() {
                return (None, Some(node.value));
            }

            if !Self::is_red(&node.right) && !Self::right_child_is_red(&node.right) {
                node = Self::move_red_right(node);
            }

            if key == &node.key {
                let (right, successor) = Self::delete_min(
                    node.right
                        .take()
                        .expect("a node with two children has a successor"),
                );
                node.right = right;
                node.key = successor.key;
                removed = Some(std::mem::replace(&mut node.value, successor.value));
            } else {
                let (right, value) = Self::remove_inner(
                    node.right
                        .take()
                        .expect("the searched key is in the right subtree"),
                    key,
                );
                node.right = right;
                removed = value;
            }
        }

        (Some(Self::rebalance(node)), removed)
    }

    fn delete_min(mut node: Box<Node<K, V>>) -> (Link<K, V>, Box<Node<K, V>>) {
        if node.left.is_none() {
            return (None, node);
        }

        if !Self::is_red(&node.left) && !Self::left_child_is_red(&node.left) {
            node = Self::move_red_left(node);
        }

        let (left, min) = Self::delete_min(
            node.left
                .take()
                .expect("the minimum is in the left subtree"),
        );
        node.left = left;
        (Some(Self::rebalance(node)), min)
    }

    fn left_child_is_red(node: &Link<K, V>) -> bool {
        node.as_ref().is_some_and(|node| Self::is_red(&node.left))
    }

    fn right_child_is_red(node: &Link<K, V>) -> bool {
        node.as_ref().is_some_and(|node| Self::is_red(&node.left))
    }

    fn flip_colors(node: &mut Box<Node<K, V>>) {
        node.color = Self::opposite(node.color);
        if let Some(left) = &mut node.left {
            left.color = Self::opposite(left.color);
        }
        if let Some(right) = &mut node.right {
            right.color = Self::opposite(right.color);
        }
    }

    fn opposite(color: Color) -> Color {
        match color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        }
    }

    fn move_red_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        Self::flip_colors(&mut node);
        if Self::right_child_is_red(&node.right) {
            node.right = Some(Self::rotate_right(
                node.right.take().expect("right child exists"),
            ));
            node = Self::rotate_left(node);
            Self::flip_colors(&mut node);
        }
        node
    }

    fn move_red_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        Self::flip_colors(&mut node);
        if Self::left_child_is_red(&node.left) {
            node = Self::rotate_right(node);
            Self::flip_colors(&mut node);
        }
        node
    }

    fn is_red(node: &Link<K, V>) -> bool {
        node.as_ref().is_some_and(|node| node.color == Color::Red)
    }

    /// Restores the left-leaning red-black invariants after insertion or deletion.
    fn rebalance(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        if Self::is_red(&node.right) && !Self::is_red(&node.left) {
            node = Self::rotate_left(node);
        }

        let left_has_red_left = node
            .left
            .as_ref()
            .is_some_and(|left| Self::is_red(&left.left));
        if Self::is_red(&node.left) && left_has_red_left {
            node = Self::rotate_right(node);
        }

        if Self::is_red(&node.left) && Self::is_red(&node.right) {
            node.color = Color::Red;
            node.left.as_mut().expect("red left child exists").color = Color::Black;
            node.right.as_mut().expect("red right child exists").color = Color::Black;
        }

        node
    }

    fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut pivot = node.left.take().expect("right rotation needs a left child");

        node.left = pivot.right.take();
        pivot.right = Some(node);
        pivot.color = pivot.right.as_ref().expect("rotated node exists").color;
        pivot.right.as_mut().expect("rotated node exists").color = Color::Red;
        pivot
    }

    fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
        let mut pivot = node
            .right
            .take()
            .expect("left rotation needs a right child");

        node.right = pivot.left.take();
        pivot.left = Some(node);
        pivot.color = pivot.left.as_ref().expect("rotated node exists").color;
        pivot.left.as_mut().expect("rotated node exists").color = Color::Red;
        pivot
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Link, RBTree};
    use crate::tree::ordered::OrderedTree;
    use proptest::prelude::*;

    fn assert_valid<K: Ord, V>(node: &Link<K, V>, lower: Option<&K>, upper: Option<&K>) -> usize {
        let Some(node) = node else {
            return 1;
        };

        assert!(lower.is_none_or(|lower| lower < &node.key));
        assert!(upper.is_none_or(|upper| &node.key < upper));
        assert!(
            !RBTree::<K, V>::is_red(&node.right),
            "red links must lean left"
        );
        if node.color == Color::Red {
            assert!(!RBTree::<K, V>::is_red(&node.left));
            assert!(!RBTree::<K, V>::is_red(&node.right));
        }

        let left_height = assert_valid(&node.left, lower, Some(&node.key));
        let right_height = assert_valid(&node.right, Some(&node.key), upper);
        assert_eq!(left_height, right_height, "black heights must match");
        left_height + usize::from(node.color == Color::Black)
    }

    fn assert_tree_valid<K: Ord, V>(tree: &RBTree<K, V>) {
        if let Some(root) = &tree.root {
            assert!(root.color == Color::Black);
        }
        assert_valid(&tree.root, None, None);
    }

    #[test]
    fn insert_and_remove_preserve_red_black_properties() {
        let mut tree = RBTree::new();
        let count = 257;

        for i in 0..count {
            let key = (i * 73) % count;
            tree.insert(key, key * 10);
            assert_tree_valid(&tree);
        }

        for i in 0..count {
            let key = (i * 97) % count;
            assert_eq!(tree.remove(&key), Some(key * 10));
            assert_tree_valid(&tree);
        }

        assert!(tree.remove(&0).is_none());
        assert_tree_valid(&tree);
    }

    proptest! {
        #[test]
        fn operations_preserve_red_black_invariants(operations in prop::collection::vec((any::<bool>(), -50_i32..50), 0..150)) {
            let mut tree = RBTree::new();

            for (insert, key) in operations {
                if insert {
                    tree.insert(key, key * 10);
                } else {
                    tree.remove(&key);
                }
                assert_tree_valid(&tree);
            }
        }
    }
}
