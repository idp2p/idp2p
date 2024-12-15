use futures::channel::mpsc;
use idp2p_p2p::handler::IdHandlerInboundEvent;
use layout::Flex;
/// This example is taken from https://raw.githubusercontent.com/fdehau/tui-rs/master/examples/user_input.rs
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
    name: String,
    /// Current value of the input box
    input: Input,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    // Show help popup
    show_help_popup: bool,
    // Event receiver
    event_receiver: mpsc::Receiver<IdAppEvent>,
}

impl App {
    fn new(name: String, event_receiver: mpsc::Receiver<IdAppEvent>) -> Self {
        Self {
            name: name,
            input: Input::default(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            show_help_popup: false,
            event_receiver: event_receiver,
        }
    }

    fn handle_event(&mut self, event: IdAppEvent) {
        match event {
            IdAppEvent::ListenOn(addr) => {
                self.name = format!("Listening on {} as {}", addr, self.name);
            },
            IdAppEvent::Resolved { id, peer, name } => todo!(),
            IdAppEvent::GotMessage(_) => todo!(),
        }
    }
}
pub(crate) async fn run(
    name: String,
    handler_event_sender: mpsc::Sender<IdHandlerInboundEvent>,
    app_event_receiver: mpsc::Receiver<IdAppEvent>,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(name, app_event_receiver);

    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;
        while let Ok(Some(event)) = app.event_receiver.try_next() {
            app.handle_event(event);
        }
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
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
                            app.messages.push(app.input.value().into());
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
            .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
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

    // Render header
    let header = Paragraph::new(Span::styled(
        &app.name,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
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
    let text = Text::from(Line::from(msg)).style(style);
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
