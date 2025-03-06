use std::{io, time::Duration, sync::Arc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::{
    net::TcpStream,
    sync::mpsc,
};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message as WebSocketMessage,
    WebSocketStream,
    MaybeTlsStream,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame, Terminal,
};
use chrono::{DateTime, Local};
use prost::Message as ProstMessage;
use uuid::Uuid;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
use proto::*;

#[derive(Clone)]
enum UIMessageType {
    System(String),
    Sent(String),
    Received { from: String, content: String },
}

#[derive(Clone)]
struct UIMessage {
    content: UIMessageType,
    timestamp: DateTime<Local>,
}

impl UIMessage {
    fn new(content: UIMessageType) -> Self {
        Self {
            content,
            timestamp: Local::now(),
        }
    }

    fn format(&self) -> Spans {
        let (prefix, content, color) = match &self.content {
            UIMessageType::System(msg) => ("System", msg.as_str(), Color::Yellow),
            UIMessageType::Sent(msg) => ("You", msg.as_str(), Color::Cyan),
            UIMessageType::Received { from, content } => (from.as_str(), content.as_str(), Color::Green),
        };

        Spans::from(vec![
            Span::styled(
                format!("[{}] ", self.timestamp.format("%H:%M")),
                Style::default().fg(Color::DarkGray)
            ),
            Span::styled(
                format!("{}: ", prefix),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ),
            Span::raw(content),
        ])
    }
}

enum InputMode {
    Normal,
    Editing,
}

struct App {
    input: String,
    input_mode: InputMode,
    messages: Vec<UIMessage>,
    username: Option<String>,
    password: Option<String>,
    is_logged_in: bool,
    scroll_offset: usize,
    online_users: Vec<String>,
    ws_tx: Option<mpsc::UnboundedSender<WebSocketMessage>>,
    session_token: Option<String>,
    keypair: Option<SigningKey>,
}

impl Default for App {
    fn default() -> App {
        // Generate keypair on startup
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);

        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            username: None,
            password: None,
            is_logged_in: false,
            scroll_offset: 0,
            online_users: vec![],
            ws_tx: None,
            session_token: None,
            keypair: Some(signing_key),
        }
    }
}

async fn connect_websocket(messages: Arc<tokio::sync::Mutex<Vec<UIMessage>>>) -> Result<mpsc::UnboundedSender<WebSocketMessage>, Box<dyn std::error::Error>> {
    let (ws_stream, _) = connect_async("ws://localhost:8081/ws").await?;
    let (write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut write = write;
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write.send(msg).await {
                eprintln!("Error sending message: {}", e);
                break;
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(WebSocketMessage::Binary(data)) => {
                    if let Ok(ws_msg) = WsMessage::decode(&data[..]) {
                        match ws_msg.message {
                            Some(ws_message::Message::AuthResponse(resp)) => {
                                let mut messages = messages.lock().await;
                                match resp.status() {
                                    AuthStatus::Success => {
                                        messages.push(UIMessage::new(UIMessageType::System(
                                            "Authentication successful!".to_string()
                                        )));
                                    }
                                    _ => {
                                        messages.push(UIMessage::new(UIMessageType::System(
                                            format!("Authentication failed: {:?}", resp.status())
                                        )));
                                    }
                                }
                            }
                            Some(ws_message::Message::ChatMessage(chat_msg)) => {
                                let mut messages = messages.lock().await;
                                messages.push(UIMessage::new(UIMessageType::Received {
                                    from: chat_msg.sender,
                                    content: String::from_utf8_lossy(&chat_msg.content).to_string(),
                                }));
                            }
                            _ => {}
                        }
                    }
                }
                Ok(WebSocketMessage::Text(text)) => {
                    let mut messages = messages.lock().await;
                    messages.push(UIMessage::new(UIMessageType::System(text)));
                }
                Err(e) => {
                    let mut messages = messages.lock().await;
                    messages.push(UIMessage::new(UIMessageType::System(
                        format!("WebSocket error: {}", e)
                    )));
                }
                _ => {}
            }
        }
    });

    Ok(tx)
}

async fn handle_auth(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let (Some(username), Some(password), Some(tx), Some(keypair)) = (
        &app.username,
        &app.password,
        &app.ws_tx,
        &app.keypair,
    ) {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let message = format!("{}{}", timestamp, username);

        let signature = keypair.sign(message.as_bytes());
        let verifying_key = VerifyingKey::from(keypair);

        let handshake = HandshakeMessage {
            user_id: username.clone(),
            timestamp,
            signature: signature.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
        };

        let bytes = handshake.encode_to_vec();
        tx.send(WebSocketMessage::Binary(bytes.into()))?;

        let auth_msg = WsMessage {
            message: Some(ws_message::Message::AuthInitiate(AuthInitiate {
                identifier: Some(auth_initiate::Identifier::Username(username.clone())),
                password: password.clone(),
                app_hash: "cli_app".to_string(),
                auth_method: AuthMethod::Username as i32,
            })),
        };

        let bytes = auth_msg.encode_to_vec();
        tx.send(WebSocketMessage::Binary(bytes.into()))?;
    }
    Ok(())
}

impl App {
    async fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.input.starts_with('@') {
            // Handle direct message
            let parts: Vec<&str> = self.input[1..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                let receiver = parts[0].to_string();
                let content = parts[1].to_string();

                if let Some(tx) = &self.ws_tx {
                    let timestamp = chrono::Utc::now().timestamp() as u64;
                    let msg_id = Uuid::new_v4().to_string();
                    let content_bytes = content.as_bytes().to_vec();

                    // Sign the message using the exact format the server expects
                    let message_to_sign = format!("{}{}{:?}",
                        timestamp,
                        self.username.clone().unwrap_or_default(),
                        content_bytes
                    );

                    // Sign the message
                    let signature = if let Some(keypair) = &self.keypair {
                        keypair.sign(message_to_sign.as_bytes()).to_bytes().to_vec()
                    } else {
                        vec![]
                    };

                    let chat_msg = ChatMessage {
                        id: msg_id,
                        sender: self.username.clone().unwrap_or_default(),
                        receiver: receiver.clone(),
                        content: content_bytes,
                        timestamp,
                        message_type: MessageType::Direct as i32,
                        version_vector: "".to_string(),
                        signature,
                        encryption_type: EncryptionType::None as i32,
                        metadata: None,
                    };

                    let ws_msg = WsMessage {
                        message: Some(ws_message::Message::ChatMessage(chat_msg)),
                    };

                    let bytes = ws_msg.encode_to_vec();
                    tx.send(WebSocketMessage::Binary(bytes.into()))?;

                    // Add message to UI
                    self.messages.push(UIMessage::new(UIMessageType::Sent(
                        format!("@{} {}", receiver, content)
                    )));
                }
            } else {
                self.messages.push(UIMessage::new(UIMessageType::System(
                    "Invalid direct message format. Use: @username message".to_string()
                )));
            }
        } else {
            if !self.is_logged_in {
                if self.username.is_none() {
                    self.username = Some(self.input.clone());
                    self.messages.push(UIMessage::new(UIMessageType::System(
                        "Please enter your password".to_string()
                    )));
                } else if self.password.is_none() {
                    self.password = Some(self.input.clone());

                    if let Err(e) = handle_auth(self).await {
                        self.messages.push(UIMessage::new(UIMessageType::System(
                            format!("Authentication error: {}", e)
                        )));
                    }
                }
            } else {
                if let Some(tx) = &self.ws_tx {
                    let chat_msg = ChatMessage {
                        id: Uuid::new_v4().to_string(),
                        sender: self.username.clone().unwrap_or_default(),
                        receiver: "".to_string(), // Broadcast
                        content: self.input.as_bytes().to_vec(),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        message_type: MessageType::Group as i32,
                        version_vector: "".to_string(),
                        signature: vec![],
                        encryption_type: EncryptionType::None as i32,
                        metadata: None,
                    };

                    let ws_msg = WsMessage {
                        message: Some(ws_message::Message::ChatMessage(chat_msg)),
                    };

                    let bytes = ws_msg.encode_to_vec();
                    if let Err(e) = tx.send(WebSocketMessage::Binary(bytes.into())) {
                        self.messages.push(UIMessage::new(UIMessageType::System(
                            format!("Failed to send message: {}", e)
                        )));
                    } else {
                        self.messages.push(UIMessage::new(UIMessageType::Sent(
                            self.input.clone()
                        )));
                    }
                }
            }
        }
        self.input.clear();
        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let messages = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    match connect_websocket(messages.clone()).await {
        Ok(tx) => {
            app.ws_tx = Some(tx);
            app.messages.push(UIMessage::new(UIMessageType::System(
                "Connected to server".to_string()
            )));
        }
        Err(e) => {
            app.messages.push(UIMessage::new(UIMessageType::System(
                format!("Failed to connect: {}", e)
            )));
        }
    }

    let res = run_app(&mut terminal, app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('i') => {
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            if app.scroll_offset > 0 {
                                app.scroll_offset -= 1;
                            }
                        }
                        KeyCode::Down => {
                            app.scroll_offset += 1;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                if let Err(e) = app.handle_input().await {
                                    app.messages.push(UIMessage::new(UIMessageType::System(
                                        format!("Error: {}", e)
                                    )));
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing, "),
                Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to scroll."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to send message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, main_chunks[0]);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| ListItem::new(m.format()))
        .collect();
    let messages = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Messages"))
        .style(Style::default().fg(Color::White));
    f.render_widget(messages, main_chunks[1]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title(if !app.is_logged_in {
            if app.username.is_none() {
                "Enter username"
            } else {
                "Enter password"
            }
        } else {
            "Message"
        }));
    f.render_widget(input, main_chunks[2]);

    let online_users: Vec<ListItem> = app
        .online_users
        .iter()
        .map(|name| {
            ListItem::new(Spans::from(vec![
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::raw(name),
            ]))
        })
        .collect();
    let online_users = List::new(online_users)
        .block(Block::default().borders(Borders::ALL).title("Online Users"))
        .style(Style::default().fg(Color::White));
    f.render_widget(online_users, chunks[1]);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => {
            f.set_cursor(
                main_chunks[2].x + app.input.len() as u16 + 1,
                main_chunks[2].y + 1,
            )
        }
    }
}
