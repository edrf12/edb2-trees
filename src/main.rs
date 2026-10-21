use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use edb2_avl_rbtree::tree::{
    avl::AvlTree,
    ordered::OrderedTree,
    rbtree::RBTree,
    trace::{Trace, TreeSnapshot},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Clone, Copy)]
enum TreeKind {
    Avl,
    RedBlack,
}
#[derive(Clone, Copy)]
enum ModalKind {
    Insert,
    Delete,
    Search,
}
enum SelectedTree {
    Avl(AvlTree<i32, i32>),
    RedBlack(RBTree<i32, i32>),
}
struct Frame {
    label: String,
    snapshot: TreeSnapshot<i32, i32>,
}
struct Modal {
    kind: ModalKind,
    input: String,
}

struct App {
    tree_kind: TreeKind,
    tree: SelectedTree,
    modal: Option<Modal>,
    status: String,
    frames: Vec<Frame>,
    frame: usize,
    autoplay: bool,
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
        let tree = SelectedTree::Avl(AvlTree::new());
        let snapshot = match &tree {
            SelectedTree::Avl(tree) => tree.snapshot(),
            _ => unreachable!(),
        };
        Self {
            tree_kind: TreeKind::Avl,
            tree,
            modal: None,
            status: "Press i, d, or s to open an operation dialog".into(),
            frames: vec![Frame {
                label: "Ready".into(),
                snapshot,
            }],
            frame: 0,
            autoplay: false,
            last_tick: Instant::now(),
        }
    }

    fn open_modal(&mut self, kind: ModalKind) {
        self.modal = Some(Modal {
            kind,
            input: String::new(),
        });
    }

    fn submit_modal(&mut self) {
        let Some(modal) = self.modal.take() else {
            return;
        };
        let values: Vec<_> = modal
            .input
            .split_whitespace()
            .filter_map(|value| value.parse::<i32>().ok())
            .collect();
        match modal.kind {
            ModalKind::Insert if values.len() == 1 => self.record_insert(values[0]),
            ModalKind::Delete if values.len() == 1 => self.record_remove(values[0]),
            ModalKind::Search if values.len() == 1 => self.record_search(values[0]),
            ModalKind::Insert => self.status = "Insert needs one integer key".into(),
            _ => self.status = "Enter one integer key".into(),
        }
    }

    fn record_insert(&mut self, key: i32) {
        let trace = match &mut self.tree {
            SelectedTree::Avl(tree) => tree.insert_with_trace(key, 0),
            SelectedTree::RedBlack(tree) => tree.insert_with_trace(key, 0),
        };
        self.frames = frames_from_trace(trace);
        self.frame = 0;
        self.status = format!("Inserted key {key}");
    }

    fn record_remove(&mut self, key: i32) {
        let (value, trace) = match &mut self.tree {
            SelectedTree::Avl(tree) => tree.remove_with_trace(&key),
            SelectedTree::RedBlack(tree) => tree.remove_with_trace(&key),
        };
        self.frames = frames_from_trace(trace);
        self.frame = 0;
        self.status = if value.is_some() {
            format!("Removed key {key}")
        } else {
            format!("Key {key} was not found")
        };
    }

    fn record_search(&mut self, key: i32) {
        let snapshot = self.snapshot();
        let found = match &self.tree {
            SelectedTree::Avl(tree) => tree.search(&key).is_some(),
            SelectedTree::RedBlack(tree) => tree.search(&key).is_some(),
        };
        self.frames = vec![
            Frame {
                label: "Search started".into(),
                snapshot: snapshot.clone(),
            },
            Frame {
                label: format!("Search visited key {key}"),
                snapshot: snapshot.clone(),
            },
            Frame {
                label: if found {
                    "Search found key".into()
                } else {
                    "Search did not find key".into()
                },
                snapshot,
            },
        ];
        self.frame = 0;
        self.status = if found {
            format!("Found key {key}")
        } else {
            format!("Key {key} was not found")
        };
    }

    fn switch_tree(&mut self, kind: TreeKind) {
        self.tree_kind = kind;
        self.tree = match kind {
            TreeKind::Avl => SelectedTree::Avl(AvlTree::new()),
            TreeKind::RedBlack => SelectedTree::RedBlack(RBTree::new()),
        };
        self.set_single_frame("New empty tree");
        self.status = "Tree switched and reset".into();
    }

    fn reset(&mut self) {
        self.switch_tree(self.tree_kind);
        self.status = "Tree reset".into();
    }
    fn snapshot(&self) -> TreeSnapshot<i32, i32> {
        match &self.tree {
            SelectedTree::Avl(tree) => tree.snapshot(),
            SelectedTree::RedBlack(tree) => tree.snapshot(),
        }
    }
    fn set_single_frame(&mut self, label: &str) {
        self.frames = vec![Frame {
            label: label.into(),
            snapshot: self.snapshot(),
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

fn frames_from_trace(trace: Trace<i32, i32>) -> Vec<Frame> {
    let mut frames = vec![Frame {
        label: "Before operation".into(),
        snapshot: trace.initial,
    }];
    frames.extend(trace.steps.into_iter().map(|step| Frame {
        label: step.description,
        snapshot: step.snapshot,
    }));
    frames.push(Frame {
        label: "After operation".into(),
        snapshot: trace.final_state,
    });
    frames
}

fn draw<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &App) -> io::Result<()> {
    terminal.draw(|frame| {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(frame.area());
        let tree_name = match app.tree_kind {
            TreeKind::Avl => "AVL",
            TreeKind::RedBlack => "Red-black",
        };
        let current = &app.frames[app.frame];
        let tree = Paragraph::new(current.snapshot.to_string())
            .block(
                Block::default()
                    .title(format!(" {tree_name} tree | {} ", current.label))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(tree, areas[0]);
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                " i ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("insert  "),
            Span::styled(
                "d ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("delete  "),
            Span::styled(
                "s ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("search  "),
            Span::styled(
                "a/r ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("tree  "),
            Span::styled(
                "←/→ ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("step  Tab autoplay  x reset  q quit  "),
            Span::raw(format!(
                "| step {}/{} {}",
                app.frame + 1,
                app.frames.len(),
                app.status
            )),
        ]))
        .block(Block::default().borders(Borders::TOP))
        .wrap(Wrap { trim: true });
        frame.render_widget(footer, areas[1]);
        if let Some(modal) = &app.modal {
            draw_modal(frame, modal);
        }
    })?;
    Ok(())
}

fn draw_modal(frame: &mut ratatui::Frame, modal: &Modal) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);
    let (title, hint) = match modal.kind {
        ModalKind::Insert => ("Insert key", "integer key"),
        ModalKind::Delete => ("Delete key", "integer key"),
        ModalKind::Search => ("Search key", "integer key"),
    };
    let dialog = Paragraph::new(format!(
        "{}\n\n> {}\n\nEnter confirm  Esc cancel",
        hint, modal.input
    ))
    .block(
        Block::default()
            .title(format!(" {title} "))
            .borders(Borders::ALL),
    )
    .style(Style::default().bg(Color::Black))
    .wrap(Wrap { trim: false });
    frame.render_widget(dialog, area);
}

fn centered_rect(width: u16, height: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    ratatui::layout::Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
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
            if app.modal.is_some() {
                match code {
                    KeyCode::Esc => app.modal = None,
                    KeyCode::Enter => app.submit_modal(),
                    KeyCode::Backspace => {
                        if let Some(modal) = &mut app.modal {
                            modal.input.pop();
                        }
                    }
                    KeyCode::Char(ch) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(modal) = &mut app.modal {
                            modal.input.push(ch);
                        }
                    }
                    _ => {}
                }
            } else {
                match (code, modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
                    (KeyCode::Char('i'), _) => app.open_modal(ModalKind::Insert),
                    (KeyCode::Char('d'), _) => app.open_modal(ModalKind::Delete),
                    (KeyCode::Char('s'), _) => app.open_modal(ModalKind::Search),
                    (KeyCode::Char('a'), _) => app.switch_tree(TreeKind::Avl),
                    (KeyCode::Char('r'), _) => app.switch_tree(TreeKind::RedBlack),
                    (KeyCode::Char('x'), _) => app.reset(),
                    (KeyCode::Tab, _) => app.autoplay = !app.autoplay,
                    (KeyCode::Left, _) => app.previous(),
                    (KeyCode::Right, _) => app.next(),
                    _ => {}
                }
            }
        }
        if app.autoplay && app.last_tick.elapsed() >= Duration::from_millis(700) {
            app.next();
            app.last_tick = Instant::now();
            if app.frame + 1 == app.frames.len() {
                app.autoplay = false;
            }
        }
    }
    Ok(())
}
