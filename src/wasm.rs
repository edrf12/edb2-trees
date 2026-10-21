use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::tree::{
    avl::AvlTree,
    ordered::OrderedTree,
    rbtree::RBTree,
    trace::{NodeSnapshot, TraceStep, TreeSnapshot},
};

enum Tree {
    Avl(AvlTree<i32, ()>),
    RedBlack(RBTree<i32, ()>),
}

#[derive(Serialize)]
struct Frame {
    label: String,
    snapshot: TreeSnapshot<i32, ()>,
    affected_key: Option<i32>,
}

#[derive(Serialize)]
struct OperationResult {
    result: String,
    frames: Vec<Frame>,
}

#[wasm_bindgen]
pub struct BrowserTree {
    tree: Tree,
}

#[wasm_bindgen]
impl BrowserTree {
    #[wasm_bindgen(constructor)]
    pub fn new(kind: &str) -> Result<BrowserTree, JsValue> {
        Ok(Self {
            tree: make_tree(kind)?,
        })
    }

    pub fn switch_tree(&mut self, kind: &str) -> Result<String, JsValue> {
        self.tree = make_tree(kind)?;
        self.snapshot_json()
    }

    pub fn reset(&mut self) -> Result<String, JsValue> {
        self.tree = match self.tree {
            Tree::Avl(_) => Tree::Avl(AvlTree::new()),
            Tree::RedBlack(_) => Tree::RedBlack(RBTree::new()),
        };
        self.snapshot_json()
    }

    pub fn snapshot_json(&self) -> Result<String, JsValue> {
        match &self.tree {
            Tree::Avl(tree) => encode(&tree.snapshot()),
            Tree::RedBlack(tree) => encode(&tree.snapshot()),
        }
    }

    pub fn insert(&mut self, key: i32) -> Result<String, JsValue> {
        let frames = match &mut self.tree {
            Tree::Avl(tree) => {
                let trace = tree.insert_with_trace(key, ());
                trace_frames(trace.initial, trace.steps, trace.final_state)
            }
            Tree::RedBlack(tree) => {
                let trace = tree.insert_with_trace(key, ());
                trace_frames(trace.initial, trace.steps, trace.final_state)
            }
        };
        encode(&OperationResult {
            result: format!("Inserted {key}"),
            frames,
        })
    }

    pub fn delete(&mut self, key: i32) -> Result<String, JsValue> {
        let (found, frames) = match &mut self.tree {
            Tree::Avl(tree) => {
                let (value, trace) = tree.remove_with_trace(&key);
                (
                    value.is_some(),
                    trace_frames(trace.initial, trace.steps, trace.final_state),
                )
            }
            Tree::RedBlack(tree) => {
                let (value, trace) = tree.remove_with_trace(&key);
                (
                    value.is_some(),
                    trace_frames(trace.initial, trace.steps, trace.final_state),
                )
            }
        };
        let result = if found {
            format!("Deleted {key}")
        } else {
            format!("{key} was not found")
        };
        encode(&OperationResult { result, frames })
    }

    pub fn search(&self, key: i32) -> Result<String, JsValue> {
        let snapshot = match &self.tree {
            Tree::Avl(tree) => tree.snapshot(),
            Tree::RedBlack(tree) => tree.snapshot(),
        };
        let mut frames = vec![Frame {
            label: "Search started".into(),
            snapshot: snapshot.clone(),
            affected_key: None,
        }];
        let found = search_path(snapshot.root.as_deref(), key, &snapshot, &mut frames);
        let result = if found {
            format!("Found {key}")
        } else {
            format!("{key} was not found")
        };
        frames.push(Frame {
            label: result.clone(),
            snapshot,
            affected_key: Some(key),
        });
        encode(&OperationResult { result, frames })
    }
}

fn make_tree(kind: &str) -> Result<Tree, JsValue> {
    match kind {
        "avl" => Ok(Tree::Avl(AvlTree::new())),
        "rb" | "red_black" => Ok(Tree::RedBlack(RBTree::new())),
        _ => Err(JsValue::from_str("Tree type must be 'avl' or 'rb'")),
    }
}

fn encode(value: &impl Serialize) -> Result<String, JsValue> {
    serde_json::to_string(value).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn trace_frames(
    initial: TreeSnapshot<i32, ()>,
    steps: Vec<TraceStep<i32, ()>>,
    final_state: TreeSnapshot<i32, ()>,
) -> Vec<Frame> {
    let mut frames = vec![Frame {
        label: "Before operation".into(),
        snapshot: initial,
        affected_key: None,
    }];
    frames.extend(steps.into_iter().map(|step| Frame {
        label: step.description,
        snapshot: step.snapshot,
        affected_key: step.highlighted_key,
    }));
    if frames
        .last()
        .is_none_or(|frame| frame.snapshot != final_state)
    {
        frames.push(Frame {
            label: "After operation".into(),
            snapshot: final_state,
            affected_key: None,
        });
    }
    frames
}

fn search_path(
    node: Option<&NodeSnapshot<i32, ()>>,
    key: i32,
    snapshot: &TreeSnapshot<i32, ()>,
    frames: &mut Vec<Frame>,
) -> bool {
    let Some(node) = node else { return false };
    frames.push(Frame {
        label: format!("Visit {}", node.key),
        snapshot: snapshot.clone(),
        affected_key: Some(node.key),
    });
    match key.cmp(&node.key) {
        std::cmp::Ordering::Equal => true,
        std::cmp::Ordering::Less => search_path(node.left.as_deref(), key, snapshot, frames),
        std::cmp::Ordering::Greater => search_path(node.right.as_deref(), key, snapshot, frames),
    }
}
