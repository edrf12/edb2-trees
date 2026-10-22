use edb2_avl_rbtree::tree::{avl::AvlTree, ordered::OrderedTree, rbtree::RBTree};

fn exercise_common_operations<T: OrderedTree<i32, i32>>() {
    insertion_orders::<T>();
    search_existing_and_missing::<T>();
    remove_leaf_one_child_and_two_children::<T>();
    remove_missing::<T>();
    empty_and_singleton::<T>();
}

fn insertion_orders<T: OrderedTree<i32, i32>>() {
    let cases = [
        vec![1, 2, 3, 4, 5, 6, 7],
        vec![7, 6, 5, 4, 3, 2, 1],
        vec![1, 3, 5, 7, 2, 4, 6],
    ];

    for keys in cases {
        let mut tree = T::new();
        for &key in &keys {
            tree.insert(key, key * 10);
        }
        for key in 1..=7 {
            assert_eq!(tree.search(&key), Some(&(key * 10)));
        }
    }
}

fn search_existing_and_missing<T: OrderedTree<i32, i32>>() {
    let mut tree = T::new();
    for key in [8, 3, 12, 1, 6, 10, 14] {
        tree.insert(key, key * 10);
    }

    assert_eq!(tree.search(&6), Some(&60));
    assert_eq!(tree.search(&9), None);
}

fn remove_leaf_one_child_and_two_children<T: OrderedTree<i32, i32>>() {
    // Leaf node.
    let mut leaf_tree = T::new();
    for key in [20, 10, 30] {
        leaf_tree.insert(key, key);
    }
    assert_eq!(leaf_tree.remove(&10), Some(10));
    assert_eq!(leaf_tree.search(&10), None);

    // Node with one child.
    let mut one_child_tree = T::new();
    for key in [20, 10, 30, 5] {
        one_child_tree.insert(key, key);
    }
    assert_eq!(one_child_tree.remove(&10), Some(10));
    assert_eq!(one_child_tree.search(&5), Some(&5));

    // Node with two children.
    let mut two_children_tree = T::new();
    for key in [20, 10, 30, 5, 15, 25, 35] {
        two_children_tree.insert(key, key);
    }
    assert_eq!(two_children_tree.remove(&20), Some(20));
    assert_eq!(two_children_tree.search(&15), Some(&15));
    assert_eq!(two_children_tree.search(&25), Some(&25));
}

fn remove_missing<T: OrderedTree<i32, i32>>() {
    let mut tree = T::new();
    tree.insert(1, 10);
    assert_eq!(tree.remove(&2), None);
    assert_eq!(tree.search(&1), Some(&10));
}

fn empty_and_singleton<T: OrderedTree<i32, i32>>() {
    let mut tree = T::new();
    assert_eq!(tree.search(&1), None);
    assert_eq!(tree.remove(&1), None);

    tree.insert(1, 10);
    assert_eq!(tree.search(&1), Some(&10));
    assert_eq!(tree.remove(&1), Some(10));
    assert_eq!(tree.search(&1), None);
}

#[test]
fn avl_tree_common_operations() {
    exercise_common_operations::<AvlTree<i32, i32>>();
}

#[test]
fn rbtree_common_operations() {
    exercise_common_operations::<RBTree<i32, i32>>();
}
