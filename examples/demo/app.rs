use crate::network::IdNetworkCommand;
use crate::store::InMemoryKvStore;
use futures::channel::mpsc;
use futures::SinkExt;
use idp2p_p2p::message::IdGossipMessageKind;
use layout::Flex;
use libp2p::gossipsub::IdentTopic;
use libp2p::PeerId;
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

pub(crate) enum IdAppEvent {
    ListenOn(String),
    Resolved {
        id: String,
        peer: String,
        name: String,
    },
    GotMessage(String),
}

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current user
    current_user: String,
    /// Current peer
    current_peer: PeerId,
    /// List of users
    users: Vec<String>,
    /// Store
    store: Arc<InMemoryKvStore>,
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
    fn new(
        current_user: String,
        current_peer: PeerId,
        store: Arc<InMemoryKvStore>,
        network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
        event_receiver: mpsc::Receiver<IdAppEvent>,
    ) -> Self {
        Self {
            current_user,
            current_peer,
            users: vec![],
            store: store,
            input: Input::default(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            show_help_popup: false,
            network_cmd_sender: network_cmd_sender,
            event_receiver: event_receiver,
        }
    }

    fn handle_event(&mut self, event: IdAppEvent) {
        match event {
            IdAppEvent::ListenOn(addr) => {
                //println!("Listening on {} as {}", addr, self.current_user);
            }
            IdAppEvent::Resolved { id, peer, name } => todo!(),
            IdAppEvent::GotMessage(_) => todo!(),
        }
    }
}
pub(crate) async fn run(
    current_user: String,
    current_peer: PeerId,
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
  
    // create app and run it
    let app = App::new(current_user, current_peer, store, network_cmd_sender, event_receiver);

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
        let alice = app.store.get_user("alice").await.unwrap().unwrap();
        let bob = app.store.get_user("bob").await.unwrap().unwrap();
        let dog = app.store.get_user("dog").await.unwrap().unwrap();
        let mut users = vec![];
        if let Some(id) = alice.id.clone() {
            users.push(format!("Alice - {}", id));
        }
        if let Some(id) = bob.id.clone() {
            users.push(format!("Bob - {}", id));
        }
        if let Some(id) = dog.id.clone() {
            users.push(format!("Dog - {}", id));
        }
        app.users = users;

        while let Ok(Some(event)) = app.event_receiver.try_next() {
            app.handle_event(event);
        }
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        let alice = app.store.get_user("alice").await.unwrap().unwrap();
                        let bob = app.store.get_user("bob").await.unwrap().unwrap();
                        let dog = app.store.get_user("dog").await.unwrap().unwrap();

                        if app.current_user != "alice" {
                            if let Some(id) = alice.id.clone()  {
                                let topic = IdentTopic::new(id.to_string());
                                app.network_cmd_sender
                                .send(IdNetworkCommand::Publish { topic, payload: IdGossipMessageKind::Resolve })
                                .await
                                .unwrap();
                            }
                        
                        }
                        if app.current_user != "bob" {
                            if let Some(id) = bob.id.clone()  {
                                let topic = IdentTopic::new(id.to_string());
                                app.network_cmd_sender
                                .send(IdNetworkCommand::Publish { topic, payload: IdGossipMessageKind::Resolve })
                                .await
                                .unwrap();
                            }
                        }
                        if app.current_user != "dog"  {
                            if let Some(id) = dog.id  {
                                let topic = IdentTopic::new(id.to_string());
                                app.network_cmd_sender
                                .send(IdNetworkCommand::Publish { topic, payload: IdGossipMessageKind::Resolve })
                                .await
                                .unwrap();
                            }
                        }

                        app.input_mode = InputMode::Editing;
                        app.show_help_popup = false;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        if !app.input.value().is_empty() {
                            let topic =match app.current_user.as_str() {
                               "alice" => bob.id.clone().unwrap(),
                               "bob" => alice.id.clone().unwrap(),
                               _ => panic!("")
                            };
                            let topic = IdentTopic::new(topic);
                            app.network_cmd_sender
                                .send(IdNetworkCommand::Publish{
                                    topic,
                                    payload: IdGossipMessageKind::NotifyMessage { id: alice.id.unwrap(), providers: vec![app.current_peer.to_string()] }
                                })
                                .await
                                .unwrap();
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
            }
        }
    }
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

    let title = app.current_user.to_uppercase();
    // Render header
    let header = Paragraph::new(Span::styled(
        &title,
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(header, chunks[0]);

    // Render help message
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = "".to_owned();
    for user in app.users.iter() {
        text.push_str(format!("{}\n", user).as_str());
    }
    let text = Text::from(text).style(style);

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
        .map(|(i, m)| {
            let content = vec![Line::from(Span::raw(format!("{}: {}", i, m)))];
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
}
