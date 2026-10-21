use std::io::{self, Write};

use edb2_avl_rbtree::tree::avl::AvlTree;
use edb2_avl_rbtree::tree::ordered::OrderedTree;
use edb2_avl_rbtree::tree::rbtree::RBTree;
use edb2_avl_rbtree::tree::trace::Trace;

enum SelectedTree {
    Avl(AvlTree<i32, i32>),
    RedBlack(RBTree<i32, i32>),
}

fn main() {
    println!("Tree visualizer (type 'help' for commands)");
    let mut selected = SelectedTree::Avl(AvlTree::new());
    let stdin = io::stdin();

    loop {
        print!("tree> ");
        io::stdout().flush().expect("stdout is available");
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let mut parts = line.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };
        match command.to_ascii_lowercase().as_str() {
            "help" => print_help(),
            "use" => match parts.next().map(str::to_ascii_lowercase).as_deref() {
                Some("avl") => {
                    selected = SelectedTree::Avl(AvlTree::new());
                    println!("Using AVL tree");
                }
                Some("rb") | Some("red-black") | Some("redblack") => {
                    selected = SelectedTree::RedBlack(RBTree::new());
                    println!("Using red-black tree");
                }
                _ => println!("Usage: use avl|rb"),
            },
            "insert" | "i" => {
                let Some((key, value)) = parse_pair(&mut parts) else {
                    continue;
                };
                match &mut selected {
                    SelectedTree::Avl(tree) => show_trace(tree.insert_with_trace(key, value)),
                    SelectedTree::RedBlack(tree) => show_trace(tree.insert_with_trace(key, value)),
                }
            }
            "remove" | "delete" | "d" => {
                let Some(key) = parse_key(&mut parts) else {
                    continue;
                };
                match &mut selected {
                    SelectedTree::Avl(tree) => {
                        let (value, trace) = tree.remove_with_trace(&key);
                        println!("Removed: {value:?}");
                        show_trace(trace);
                    }
                    SelectedTree::RedBlack(tree) => {
                        let (value, trace) = tree.remove_with_trace(&key);
                        println!("Removed: {value:?}");
                        show_trace(trace);
                    }
                }
            }
            "search" | "s" => {
                let Some(key) = parse_key(&mut parts) else {
                    continue;
                };
                let value = match &selected {
                    SelectedTree::Avl(tree) => tree.search(&key).copied(),
                    SelectedTree::RedBlack(tree) => tree.search(&key).copied(),
                };
                println!(
                    "{}",
                    value.map_or_else(|| "Not found".to_string(), |v| format!("Found value {v}"))
                );
            }
            "print" | "p" => print_tree(&selected),
            "reset" | "r" => reset(&mut selected),
            "quit" | "exit" | "q" => break,
            _ => println!("Unknown command. Type 'help' for commands."),
        }
    }
}

fn parse_key<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Option<i32> {
    parts.next().and_then(|value| {
        value
            .parse()
            .map_err(|_| println!("Key must be an integer"))
            .ok()
    })
}

fn parse_pair<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Option<(i32, i32)> {
    let key = parse_key(parts)?;
    let Some(value) = parts.next() else {
        println!("Usage: insert <key> <value>");
        return None;
    };
    Some((
        key,
        value
            .parse()
            .map_err(|_| println!("Value must be an integer"))
            .ok()?,
    ))
}

fn show_trace<K, V>(trace: Trace<K, V>)
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    println!("Initial state:\n{}", trace.initial);
    for (index, step) in trace.steps.iter().enumerate() {
        println!("\nStep {}: {step}", index + 1);
    }
    println!("\nFinal state:\n{}", trace.final_state);
}

fn print_tree(selected: &SelectedTree) {
    match selected {
        SelectedTree::Avl(tree) => println!("{}", tree.snapshot()),
        SelectedTree::RedBlack(tree) => println!("{}", tree.snapshot()),
    }
}

fn reset(selected: &mut SelectedTree) {
    *selected = match selected {
        SelectedTree::Avl(_) => SelectedTree::Avl(AvlTree::new()),
        SelectedTree::RedBlack(_) => SelectedTree::RedBlack(RBTree::new()),
    };
    println!("Tree reset");
}

fn print_help() {
    println!(
        "Commands:\n  use avl|rb\n  insert <key> <value>\n  remove <key>\n  search <key>\n  print\n  reset\n  quit"
    );
}
