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

pub struct TraceStep<K, V> {
    pub description: String,
    pub snapshot: TreeSnapshot<K, V>,
}

pub struct Trace<K, V> {
    pub initial: TreeSnapshot<K, V>,
    pub steps: Vec<TraceStep<K, V>>,
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

    pub fn record(&mut self, description: impl Into<String>, snapshot: TreeSnapshot<K, V>) {
        self.steps.push(TraceStep {
            description: description.into(),
            snapshot,
        });
    }
}

impl<K: Display, V: Display> TreeSnapshot<K, V> {
    pub fn render(&self) -> String {
        let mut levels = Vec::new();
        collect_levels(&self.root, 0, 0, &mut levels);
        if levels.is_empty() {
            "(empty)".to_string()
        } else {
            let label_width = levels
                .iter()
                .flatten()
                .filter_map(|label| label.as_ref().map(String::len))
                .max()
                .unwrap_or(3)
                + 2;
            let mut lines = Vec::new();
            for depth in 0..levels.len() {
                let slots = 1usize << depth;
                let gap = label_width + 2;
                let mut line = vec![' '; (slots * gap).max(label_width)];
                for (slot, label) in levels[depth].iter().enumerate() {
                    let Some(label) = label else { continue };
                    let center = slot * gap + gap / 2;
                    let start = center.saturating_sub(label.len() / 2);
                    for (offset, character) in label.chars().enumerate() {
                        if start + offset >= line.len() {
                            line.resize(start + offset + 1, ' ');
                        }
                        line[start + offset] = character;
                    }
                }
                lines.push(line.into_iter().collect::<String>().trim_end().to_string());
                if depth + 1 < levels.len() {
                    let child_gap = gap;
                    let mut arrows =
                        vec![' '; ((1usize << (depth + 1)) * child_gap).max(label_width)];
                    for (slot, parent) in levels[depth].iter().enumerate() {
                        if parent.is_none() {
                            continue;
                        }
                        let parent_center = slot * gap + gap / 2;
                        let left = parent_center.saturating_sub(child_gap / 2);
                        let right = parent_center + child_gap / 2;
                        if slot * 2 < levels[depth + 1].len()
                            && levels[depth + 1][slot * 2].is_some()
                            && left < arrows.len()
                        {
                            arrows[left] = '/';
                        }
                        if slot * 2 + 1 < levels[depth + 1].len()
                            && levels[depth + 1][slot * 2 + 1].is_some()
                            && right < arrows.len()
                        {
                            arrows[right] = '\\';
                        }
                    }
                    lines.push(
                        arrows
                            .into_iter()
                            .collect::<String>()
                            .trim_end()
                            .to_string(),
                    );
                }
            }
            lines.join("\n")
        }
    }
}

fn collect_levels<K: Display, V: Display>(
    node: &Option<Box<NodeSnapshot<K, V>>>,
    depth: usize,
    slot: usize,
    levels: &mut Vec<Vec<Option<String>>>,
) {
    let Some(node) = node else { return };
    let metadata = match node.metadata {
        NodeMetadata::Avl { height, balance } => format!("h={height}, bf={balance}"),
        NodeMetadata::RedBlack { color } => color.to_string(),
    };
    while levels.len() <= depth {
        levels.push(Vec::new());
    }
    let required = (1usize << depth).max(slot + 1);
    levels[depth].resize(required, None);
    levels[depth][slot] = Some(format!("[{} {metadata}]", node.key));
    collect_levels(&node.left, depth + 1, slot * 2, levels);
    collect_levels(&node.right, depth + 1, slot * 2 + 1, levels);
}

impl<K: Display, V: Display> Display for TreeSnapshot<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}
