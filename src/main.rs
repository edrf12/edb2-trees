use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use edb2_avl_rbtree::tree::{avl::AvlTree, ordered::OrderedTree, rbtree::RBTree, trace::Trace};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

#[derive(Clone, Copy)]
enum TreeKind {
    Avl,
    RedBlack,
}

enum SelectedTree {
    Avl(AvlTree<i32, i32>),
    RedBlack(RBTree<i32, i32>),
}

struct Frame {
    label: String,
    tree: String,
}

struct App {
    tree_kind: TreeKind,
    tree: SelectedTree,
    input: String,
    status: String,
    frames: Vec<Frame>,
    frame: usize,
    autoplay: bool,
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            tree_kind: TreeKind::Avl,
            tree: SelectedTree::Avl(AvlTree::new()),
            input: String::new(),
            status: "Type 'h' for help. Try: i 30 300".into(),
            frames: vec![Frame {
                label: "Ready".into(),
                tree: "(empty)".into(),
            }],
            frame: 0,
            autoplay: false,
            last_tick: Instant::now(),
        }
    }

    fn execute(&mut self) {
        let command = std::mem::take(&mut self.input);
        let mut parts = command.split_whitespace();
        let Some(op) = parts.next() else { return };
        match op.to_ascii_lowercase().as_str() {
            "i" | "insert" => {
                let (Some(key), Some(value)) = (parse_i32(parts.next()), parse_i32(parts.next()))
                else {
                    self.status = "Usage: i <key> <value>".into();
                    return;
                };
                self.record_insert(key, value);
            }
            "d" | "delete" | "remove" => {
                let Some(key) = parse_i32(parts.next()) else {
                    self.status = "Usage: d <key>".into();
                    return;
                };
                self.record_remove(key);
            }
            "s" | "search" => {
                let Some(key) = parse_i32(parts.next()) else {
                    self.status = "Usage: s <key>".into();
                    return;
                };
                let value = match &self.tree {
                    SelectedTree::Avl(tree) => tree.search(&key).copied(),
                    SelectedTree::RedBlack(tree) => tree.search(&key).copied(),
                };
                self.status = value.map_or_else(
                    || format!("{key} was not found"),
                    |v| format!("{key} = {v}"),
                );
            }
            "a" | "avl" => self.switch_tree(TreeKind::Avl),
            "r" | "rb" | "redblack" => self.switch_tree(TreeKind::RedBlack),
            "reset" => self.reset(),
            "h" | "help" => {
                self.status =
                    "i/d/s <args>  a=AVL  r=red-black  ←/→ step  Tab autoplay  x reset  q quit"
                        .into()
            }
            _ => self.status = "Unknown command; press h for help".into(),
        }
    }

    fn record_insert(&mut self, key: i32, value: i32) {
        let trace = match &mut self.tree {
            SelectedTree::Avl(tree) => tree.insert_with_trace(key, value),
            SelectedTree::RedBlack(tree) => tree.insert_with_trace(key, value),
        };
        self.frames = frames_from_trace(trace);
        self.frame = 0;
        self.status = format!("Inserted {key}; use ←/→ to inspect the transition");
    }

    fn record_remove(&mut self, key: i32) {
        let (value, trace) = match &mut self.tree {
            SelectedTree::Avl(tree) => tree.remove_with_trace(&key),
            SelectedTree::RedBlack(tree) => tree.remove_with_trace(&key),
        };
        self.frames = frames_from_trace(trace);
        self.frame = 0;
        self.status = format!("Removed {value:?}; use ←/→ to inspect the transition");
    }

    fn switch_tree(&mut self, kind: TreeKind) {
        self.tree_kind = kind;
        self.tree = match self.tree_kind {
            TreeKind::Avl => SelectedTree::Avl(AvlTree::new()),
            TreeKind::RedBlack => SelectedTree::RedBlack(RBTree::new()),
        };
        self.set_current_frame("New empty tree");
        self.status = "Tree switched and reset".into();
    }

    fn reset(&mut self) {
        self.switch_tree(self.tree_kind);
        self.status = "Tree reset".into();
    }

    fn set_current_frame(&mut self, label: &str) {
        let tree = match &self.tree {
            SelectedTree::Avl(tree) => tree.snapshot().to_string(),
            SelectedTree::RedBlack(tree) => tree.snapshot().to_string(),
        };
        self.frames = vec![Frame {
            label: label.into(),
            tree,
        }];
        self.frame = 0;
    }

    fn next(&mut self) {
        self.frame = (self.frame + 1).min(self.frames.len() - 1);
    }
    fn previous(&mut self) {
        self.frame = self.frame.saturating_sub(1);
    }
}

fn frames_from_trace<K, V>(trace: Trace<K, V>) -> Vec<Frame>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    let mut frames = vec![Frame {
        label: "Before operation".into(),
        tree: trace.initial.to_string(),
    }];
    frames.extend(trace.steps.into_iter().map(|step| Frame {
        label: step.description,
        tree: step.snapshot.to_string(),
    }));
    frames.push(Frame {
        label: "After operation".into(),
        tree: trace.final_state.to_string(),
    });
    frames
}

fn parse_i32(value: Option<&str>) -> Option<i32> {
    value?.parse().ok()
}

fn draw<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &App) -> io::Result<()> {
    terminal.draw(|frame| {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .split(frame.area());
        let title = match app.tree_kind {
            TreeKind::Avl => "AVL tree",
            TreeKind::RedBlack => "Red-black tree",
        };
        let current = &app.frames[app.frame];
        let tree = Paragraph::new(current.tree.as_str())
            .block(
                Block::default()
                    .title(format!(" {title} | {} ", current.label))
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(tree, areas[0]);
        let input = Paragraph::new(Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.input),
        ]))
        .block(Block::default().title(" Command ").borders(Borders::ALL));
        frame.render_widget(input, areas[1]);
        let footer = Paragraph::new(Line::from(vec![
            Span::raw(format!("Step {}/{}  ", app.frame + 1, app.frames.len())),
            Span::styled(
                if app.autoplay { "[AUTO] " } else { "" },
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.status),
        ]));
        frame.render_widget(footer, areas[2]);
    })?;
    Ok(())
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run(&mut terminal);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();
    loop {
        draw(terminal, &app)?;
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
        {
            match (code, modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => break,
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                (KeyCode::Enter, _) => app.execute(),
                (KeyCode::Tab, _) => app.autoplay = !app.autoplay,
                (KeyCode::Left, _) => app.previous(),
                (KeyCode::Right, _) => app.next(),
                (KeyCode::Backspace, _) => {
                    app.input.pop();
                }
                (KeyCode::Char('x'), _) => app.reset(),
                (KeyCode::Char(' '), _) => app.input.push(' '),
                (KeyCode::Char(ch), _) => app.input.push(ch),
                _ => {}
            }
        }
        if app.autoplay && app.last_tick.elapsed() >= Duration::from_millis(700) {
            app.next();
            if app.frame + 1 == app.frames.len() {
                app.autoplay = false;
            }
            app.last_tick = Instant::now();
        }
    }
    Ok(())
}
