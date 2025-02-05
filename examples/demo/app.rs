use std::{error::Error, io, sync::Arc};

use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_p2p::message::IdGossipMessageKind;
use libp2p::gossipsub::IdentTopic;
use tokio::io::AsyncBufReadExt;

use crate::{
    network::{self, IdNetworkCommand, IdRequestKind},
    store::InMemoryKvStore,
};

#[derive(Debug)]
pub(crate) enum IdAppEvent {
    Resolved {
        id: String,
        peer: String,
        name: String,
    },
    GotMessage(String),
    Other(String),
}

pub(crate) async fn run(
    store: Arc<InMemoryKvStore>,
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
    event_receiver: mpsc::Receiver<IdAppEvent>,
) -> anyhow::Result<()> {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let mut network_cmd_sender = network_cmd_sender.clone();
    let mut event_receiver = event_receiver;
    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                let mut split = line.split_whitespace();
                match split.next().unwrap() {
                    "connect" => {
                        let current_user = store.get_current_user().await.unwrap();
                        for peer in current_user.peers.iter() {
                            if !peer.1 {
                                network_cmd_sender.send(IdNetworkCommand::SendRequest {
                                    peer: peer.0.to_owned(),
                                    req: IdRequestKind::Meet,
                                }).await.unwrap();
                            }
                        }
                    },
                    "resolve" => {
                        let current_user = store.get_current_user().await.unwrap();
                        let username = split.next().unwrap();
                        if let Some(user) = current_user.others.iter().find(|x| x.name == username) {
                            //store.get(&user.id.clone().unwrap()).await.unwrap();
                            network_cmd_sender
                                .send(IdNetworkCommand::Publish {
                                    topic: IdentTopic::new(user.id.clone().unwrap()),
                                    payload: IdGossipMessageKind::Resolve,
                                })
                                .await
                                .unwrap();
                        }
                    }
                    "info" => {
                        let current_user = store.get_current_user().await.unwrap();
                        let current_user_str = serde_json::to_string_pretty(&current_user).unwrap();
                        println!("{}", current_user_str);
                    }
                    "exit" => return Ok(()),
                    _ => println!("Unknown command")
                }
            }
            Some(event) = event_receiver.next() => match event {
                IdAppEvent::Other(msg) => println!("{}", msg),
                _ => todo!()
            }
        }
    }
}

/*use crate::network::{IdNetworkCommand, IdRequestKind};
use crate::store::InMemoryKvStore;
use crate::user::UserState;
use futures::channel::mpsc;
use futures::SinkExt;
use idp2p_p2p::message::{IdGossipMessageKind, IdMessageDirection};
use layout::Flex;
use libp2p::gossipsub::IdentTopic;
use ratatui::crossterm::event::KeyEvent;
use ratatui::prelude::*;
use ratatui::widgets::Clear;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::sync::Arc;
use std::{error::Error, io};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug)]
pub(crate) enum IdAppEvent {
    Resolved {
        id: String,
        peer: String,
        name: String,
    },
    GotMessage(String),
    Other(String),
}

#[derive(Debug, PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Store
    store: Arc<InMemoryKvStore>,
    /// Current user
    current_user: UserState,
    /// Current value of the input box
    input: Input,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    // Show help popup
    show_help_popup: bool,
    // Event sender
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
    // Event receiver
    event_receiver: mpsc::Receiver<IdAppEvent>,
}

impl App {
    async fn new(
        store: Arc<InMemoryKvStore>,
        network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
        event_receiver: mpsc::Receiver<IdAppEvent>,
    ) -> Self {
        let current_user = store.get_current_user().await.unwrap();
        Self {
            store: store,
            current_user: current_user,
            input: Input::default(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            show_help_popup: false,
            network_cmd_sender: network_cmd_sender,
            event_receiver: event_receiver,
        }
    }

    async fn handle_event(&mut self, event: IdAppEvent) {
        println!("Got event: {:?}", event);
        let current_user = self.store.get_current_user().await.unwrap();

        match event {
            IdAppEvent::Other(msg) => {
                self.messages.push(msg);
            }
            IdAppEvent::Resolved { id, peer, name } => {
                let msg = format!("Resolved {} as {}", id, name);
                self.messages.push(msg);
                self.current_user = current_user;
            }
            IdAppEvent::GotMessage(msg) => {
                let msg = format!("Got message: {}", msg);
                self.messages.push(msg);
            }
        }
    }
}

pub(crate) async fn run(
    store: Arc<InMemoryKvStore>,
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
    event_receiver: mpsc::Receiver<IdAppEvent>,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // let current_user = store.get_current_user().await.unwrap();
    // create app and run it
    let app = App::new(store, network_cmd_sender, event_receiver).await;

    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let current_user = app.store.get_current_user().await.unwrap();
        while let Ok(Some(event)) = app.event_receiver.try_next() {
            app.handle_event(event).await;
        }
        if let Event::Key(key) = event::read()? {
            if app.input_mode == InputMode::Normal && key.code == KeyCode::Char('q') {
                return Ok(());
            }
            key_event(key, &mut app, &current_user).await;
        }
    }
}

async fn resolve(username: &str, app: &mut App, current_user: &UserState) {
    if let Some(user) = current_user.others.iter().find(|x| x.name == username) {
        app.network_cmd_sender
            .send(IdNetworkCommand::Publish {
                topic: IdentTopic::new(user.id.clone().unwrap()),
                payload: IdGossipMessageKind::Resolve,
            })
            .await
            .unwrap();
    }
}

async fn key_event(key: KeyEvent, app: &mut App, current_user: &UserState) {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('c') => {
                println!("Typed c, current user: {:#?}", current_user);
                //println!("Current peers: {:#?}", current_user.peers);
                for peer in current_user.peers.iter() {
                    if !peer.1 {
                        app.network_cmd_sender.send(IdNetworkCommand::SendRequest {
                            peer: peer.0.to_owned(),
                            req: IdRequestKind::Meet,
                        }).await.unwrap();
                    }
                }
            }
            KeyCode::Char('e') => {
                if current_user.username.as_str() != "alice" {
                    resolve("alice", app, current_user).await;
                }
                if current_user.username.as_str() != "bob" {
                    resolve("bob", app, current_user).await;
                }
                if current_user.username.as_str() != "dog" {
                    resolve("dog", app, current_user).await;
                }

                app.input_mode = InputMode::Editing;
                app.show_help_popup = false;
            }
            _ => {}
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                if !app.input.value().is_empty() {
                    /*let topic = match app.current_user.username.as_str() {
                        "alice" => bob.id.clone().unwrap(),
                        "bob" => alice.id.clone().unwrap(),
                        _ => panic!(""),
                    };
                    let topic = IdentTopic::new(topic);
                    app.network_cmd_sender
                        .send(IdNetworkCommand::Publish {
                            topic,
                            payload: IdGossipMessageKind::NotifyMessage {
                                direction: IdMessageDirection::To,
                                id: alice.id.unwrap(),
                                providers: vec![app.current_peer.to_string()],
                            },
                        })
                        .await
                        .unwrap();*/
                }
                app.input.reset();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {
                app.input.handle_event(&Event::Key(key));
            }
        },
    };
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    if app.show_help_popup {
        let area = popup_area(area, 60, 20);

        let popup = Paragraph::new("Unsupported key pressed. Press any key to continue.")
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(Clear, area); // Clear background beneath the popup
        f.render_widget(popup, area);
        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(2), // Space for header
                Constraint::Length(3), // Space for help message
                Constraint::Length(3), // Space for input
                Constraint::Min(1),    // Remaining space for messages
            ]
            .as_ref(),
        )
        .split(f.area());

    let title = format!(
        "{} - {}",
        app.current_user.username.to_uppercase(),
        app.current_user.id
    );
    // Render header
    let header = Paragraph::new(Span::styled(
        &title,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ));
    f.render_widget(header, chunks[0]);

    let text = "[c] to connect, [r] to resolve, [e] to edit, [q] to quit".to_string();

    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[1]);

    // Render input box
    let width = chunks[1].width.max(3) - 3; // Keep 2 for borders and 1 for cursor
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    // Set cursor position when editing
    if let InputMode::Editing = app.input_mode {
        f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        ));
    }

    // Render messages
    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(_, m)| {
            let content = vec![Line::from(Span::raw(format!("{}", m)))];
            ListItem::new(content)
        })
        .collect();
    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[3]);
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}*/
