pub trait OrderedTree<K: Ord, V> {
    fn new() -> Self;

    /// Inserts value in the tree using the position of the key
    ///
    /// This method rebalances the tree automatically
    ///
    /// Calling this method with a key that already exists will **replace** its value
    fn insert(&mut self, key: K, value: V);

    /// Removes key from the tree and returns the value, giving ownership
    fn remove(&mut self, key: &K) -> Option<V>;

    /// Searches the tree for key ands returns a borrowed reference to its value
    fn search(&self, key: &K) -> Option<&V>;
}
