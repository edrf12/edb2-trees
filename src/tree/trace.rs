use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub enum NodeMetadata {
    Avl { height: i32, balance: i32 },
    RedBlack { color: &'static str },
}

#[derive(Clone, Debug)]
pub struct NodeSnapshot<K, V> {
    pub key: K,
    pub value: V,
    pub metadata: NodeMetadata,
    pub left: Option<Box<NodeSnapshot<K, V>>>,
    pub right: Option<Box<NodeSnapshot<K, V>>>,
}

#[derive(Clone, Debug)]
pub struct TreeSnapshot<K, V> {
    pub root: Option<Box<NodeSnapshot<K, V>>>,
}

pub struct Trace<K, V> {
    pub initial: TreeSnapshot<K, V>,
    pub steps: Vec<String>,
    pub final_state: TreeSnapshot<K, V>,
}

impl<K, V> Trace<K, V> {
    pub fn new(initial: TreeSnapshot<K, V>) -> Self
    where
        K: Clone,
        V: Clone,
    {
        Self {
            final_state: initial.clone(),
            initial,
            steps: Vec::new(),
        }
    }

    pub fn record(&mut self, description: impl Into<String>) {
        self.steps.push(description.into());
    }
}

impl<K: Display, V: Display> TreeSnapshot<K, V> {
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        render_node(&self.root, "", true, &mut lines);
        if lines.is_empty() {
            "(empty)".to_string()
        } else {
            lines.join("\n")
        }
    }
}

fn render_node<K: Display, V: Display>(
    node: &Option<Box<NodeSnapshot<K, V>>>,
    prefix: &str,
    is_tail: bool,
    lines: &mut Vec<String>,
) {
    let Some(node) = node else { return };
    render_node(&node.right, &format!("{prefix}    "), false, lines);
    let connector = if prefix.is_empty() {
        ""
    } else if is_tail {
        "`-- "
    } else {
        "+-- "
    };
    let metadata = match node.metadata {
        NodeMetadata::Avl { height, balance } => format!("h={height}, bf={balance}"),
        NodeMetadata::RedBlack { color } => color.to_string(),
    };
    lines.push(format!(
        "{prefix}{connector}{}:{} [{metadata}]",
        node.key, node.value
    ));
    render_node(&node.left, &format!("{prefix}    "), true, lines);
}

impl<K: Display, V: Display> Display for TreeSnapshot<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}
