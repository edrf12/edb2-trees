use serde::Serialize;
use std::fmt::{self, Display};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeMetadata {
    Avl { height: i32, balance: i32 },
    RedBlack { color: &'static str },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NodeSnapshot<K, V> {
    pub key: K,
    pub value: V,
    pub metadata: NodeMetadata,
    pub left: Option<Box<NodeSnapshot<K, V>>>,
    pub right: Option<Box<NodeSnapshot<K, V>>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TreeSnapshot<K, V> {
    pub root: Option<Box<NodeSnapshot<K, V>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraceStep<K, V> {
    pub description: String,
    pub snapshot: TreeSnapshot<K, V>,
}

#[derive(Clone, Debug, Serialize)]
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

    pub fn record(&mut self, description: impl Into<String>, snapshot: TreeSnapshot<K, V>)
    where
        K: PartialEq,
        V: PartialEq,
    {
        let previous = self
            .steps
            .last()
            .map_or(&self.initial, |step| &step.snapshot);
        if previous == &snapshot {
            return;
        }
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
            let node_width = levels
                .iter()
                .flatten()
                .filter_map(|label| {
                    label
                        .as_ref()
                        .map(|label| label.split('\0').next().unwrap_or(label).len())
                })
                .max()
                .unwrap_or(1)
                .max(1);
            let metadata_width = levels
                .iter()
                .flatten()
                .filter_map(|label| {
                    label
                        .as_ref()?
                        .split('\0')
                        .nth(1)
                        .map(metadata_lines)
                        .map(|lines| lines.iter().map(String::len).max().unwrap_or(0))
                })
                .max()
                .unwrap_or(0);
            let metadata_padding = 3;
            let slot_width = node_width + metadata_padding + metadata_width + 8;
            let mut lines = Vec::new();
            for depth in 0..levels.len() {
                let gap = slot_width;
                let row_width = ((1usize << depth) * gap).max(slot_width);
                let mut row_top = vec![' '; row_width];
                let mut row_middle = vec![' '; row_width];
                let mut row_bottom = vec![' '; row_width];
                for (slot, label) in levels[depth].iter().enumerate() {
                    let Some(label) = label else { continue };
                    let cell_start = slot * gap;
                    let node_offset = (gap - node_width) / 2;
                    let key = label.split('\0').next().unwrap_or(label);
                    let metadata = metadata_lines(label.split('\0').nth(1).unwrap_or(""));
                    let rows = [
                        format!(
                            "{}{key:^node_width$}{}{}",
                            " ".repeat(node_offset),
                            " ".repeat(metadata_padding),
                            metadata[0]
                        ),
                        format!(
                            "{}{}{}",
                            " ".repeat(node_offset + node_width),
                            " ".repeat(metadata_padding),
                            metadata[1]
                        ),
                        String::new(),
                    ];
                    for (row, text) in [&mut row_top, &mut row_middle, &mut row_bottom]
                        .into_iter()
                        .zip(rows)
                    {
                        if row.len() < cell_start + text.len() {
                            row.resize(cell_start + text.len(), ' ');
                        }
                        for (offset, character) in text.chars().enumerate() {
                            row[cell_start + offset] = character;
                        }
                    }
                }
                lines.extend(
                    [row_top, row_middle, row_bottom]
                        .into_iter()
                        .map(|row| row.into_iter().collect::<String>().trim_end().to_string()),
                );
                if depth + 1 < levels.len() {
                    let child_count = 1usize << (depth + 1);
                    let mut connector_rows = vec![vec![' '; child_count * gap]; 3];
                    for (slot, parent) in levels[depth].iter().enumerate() {
                        if parent.is_none() {
                            continue;
                        }
                        let parent_center = slot * gap + gap / 2;
                        for child_slot in [slot * 2, slot * 2 + 1] {
                            if levels[depth + 1]
                                .get(child_slot)
                                .is_none_or(Option::is_none)
                            {
                                continue;
                            }
                            let child_center = child_slot * gap + gap / 2;
                            let direction = if child_center < parent_center {
                                '/'
                            } else {
                                '\\'
                            };
                            for (row_index, row) in connector_rows.iter_mut().enumerate() {
                                let numerator = row_index + 1;
                                let delta = parent_center.abs_diff(child_center);
                                let offset = delta * numerator / 4;
                                let column = if child_center < parent_center {
                                    parent_center - offset
                                } else {
                                    parent_center + offset
                                };
                                if column < row.len() {
                                    row[column] = direction;
                                }
                            }
                        }
                    }
                    lines.extend(
                        connector_rows
                            .into_iter()
                            .map(|row| row.into_iter().collect::<String>().trim_end().to_string()),
                    );
                }
            }
            lines.join("\n")
        }
    }
}

fn metadata_lines(metadata: &str) -> [String; 2] {
    if let Some((height, balance)) = metadata.split_once(", ") {
        [height.to_string(), balance.to_string()]
    } else if let Some((label, value)) = metadata.split_once('=') {
        [label.to_string(), value.to_string()]
    } else {
        [metadata.to_string(), String::new()]
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
    levels[depth][slot] = Some(format!("{}\0{metadata}", node.key));
    collect_levels(&node.left, depth + 1, slot * 2, levels);
    collect_levels(&node.right, depth + 1, slot * 2 + 1, levels);
}

impl<K: Display, V: Display> Display for TreeSnapshot<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}
