use crate::types::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

// UI state - only display-related information
#[derive(Debug)]
pub struct TuiState {
    pub input_buffer: String,
    pub scroll_position: usize,
    pub command_mode: bool,
    pub status_message: Option<String>,
    pub show_help: bool,
    pub last_input_time: Instant,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            input_buffer: String::new(),
            scroll_position: 0,
            command_mode: false,
            status_message: None,
            show_help: false,
            last_input_time: Instant::now(),
        }
    }
}

// Data passed from app controller to UI for rendering
#[derive(Debug, Default)]
pub struct AppDisplayData {
    pub messages: Vec<Message>,
    pub provisional_mode: bool,
    pub rag_enabled: bool,
    pub current_status: String,
    pub streaming_response: Option<String>, // Partial response being streamed
}

// TUI renderer trait for abstraction
pub trait TuiRenderer {
    fn render(&mut self, app_data: &AppDisplayData) -> Result<(), TuiError>;
    fn handle_input(&mut self) -> Result<Option<UserAction>, TuiError>;
    fn cleanup(&mut self) -> Result<(), TuiError>;
    fn initialize(&mut self) -> Result<(), TuiError>;
}

// Ratatui-based implementation
pub struct RatatuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: TuiState,
}

impl RatatuiRenderer {
    pub fn new() -> Result<Self, TuiError> {
        // Set up terminal
        enable_raw_mode().map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| TuiError::TerminalInit(e.to_string()))?;

        Ok(Self {
            terminal,
            state: TuiState::default(),
        })
    }

    fn render_help_static(f: &mut Frame) {
        let help_text = vec![
            Line::from(vec![
                Span::styled("LLM TUI Assistant - Help", Style::default().add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from("Available Commands:"),
            Line::from("  /help          - Show this help message"),
            Line::from("  /config        - Open configuration"),
            Line::from("  /clear         - Clear conversation history"),
            Line::from("  /toggle-rag    - Toggle RAG functionality"),
            Line::from("  /toggle-prov   - Toggle provisional mode"),
            Line::from("  /add-source    - Add file/directory source"),
            Line::from("  /remove-source - Remove file/directory source"),
            Line::from("  /list-sources  - List configured sources"),
            Line::from("  /exit          - Exit application"),
            Line::from(""),
            Line::from("Keyboard Shortcuts:"),
            Line::from("  Enter          - Send message"),
            Line::from("  Escape         - Close help/cancel input"),
            Line::from("  Ctrl+C         - Exit application"),
            Line::from("  Page Up/Down   - Scroll conversation"),
            Line::from("  Tab            - Toggle command mode"),
            Line::from(""),
            Line::from("Status Indicators:"),
            Line::from("  RAG: ON/OFF    - Retrieval-Augmented Generation"),
            Line::from("  PROV: ON/OFF   - Provisional mode (messages not saved)"),
            Line::from(""),
            Line::from("Press Escape to close this help"),
        ];

        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        let area = f.size();
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(area)[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(popup_area)[1];

        f.render_widget(Clear, popup_area);
        f.render_widget(help_paragraph, popup_area);
    }

    fn render_main_ui_static(f: &mut Frame, app_data: &AppDisplayData, state: &TuiState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),     // Messages area
                Constraint::Length(3),  // Input area
                Constraint::Length(1),  // Status bar
            ])
            .split(f.size());

        // Render messages area
        Self::render_messages_static(f, chunks[0], app_data);

        // Render input area
        Self::render_input_static(f, chunks[1], state);

        // Render status bar
        Self::render_status_bar_static(f, chunks[2], app_data);
    }

    fn render_messages_static(f: &mut Frame, area: ratatui::layout::Rect, app_data: &AppDisplayData) {
        let mut items = Vec::new();

        // Add conversation messages
        for message in &app_data.messages {
            let role_style = match message.role {
                MessageRole::User => Style::default().fg(Color::Cyan),
                MessageRole::Assistant => Style::default().fg(Color::Green),
                MessageRole::System => Style::default().fg(Color::Yellow),
            };

            let timestamp = message.timestamp.format("%H:%M:%S");
            let role_prefix = match message.role {
                MessageRole::User => "You",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
            };

            let provisional_indicator = if message.provisional { " [PROV]" } else { "" };
            
            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("[{}] {}{}: ", timestamp, role_prefix, provisional_indicator),
                        role_style.add_modifier(Modifier::BOLD)
                    )
                ]),
                Line::from(Span::raw(&message.content)),
                Line::from(""), // Empty line for spacing
            ]));
        }

        // Add streaming response if present
        if let Some(streaming_content) = &app_data.streaming_response {
            items.push(ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        "Assistant (streaming): ",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    )
                ]),
                Line::from(Span::raw(streaming_content)),
                Line::from(""),
            ]));
        }

        let messages_list = List::new(items)
            .block(Block::default().title("Conversation").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(messages_list, area);
    }

    fn render_input_static(f: &mut Frame, area: ratatui::layout::Rect, state: &TuiState) {
        let input_style = if state.command_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let mode_indicator = if state.command_mode { "CMD" } else { "MSG" };
        let title = format!("Input [{}]", mode_indicator);

        let input = Paragraph::new(state.input_buffer.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(input, area);

        // Set cursor position
        f.set_cursor(
            area.x + state.input_buffer.len() as u16 + 1,
            area.y + 1,
        );
    }

    fn render_status_bar_static(f: &mut Frame, area: ratatui::layout::Rect, app_data: &AppDisplayData) {
        let rag_status = if app_data.rag_enabled { "RAG: ON" } else { "RAG: OFF" };
        let prov_status = if app_data.provisional_mode { "PROV: ON" } else { "PROV: OFF" };
        
        let status_text = format!(
            " {} | {} | {} | Press Tab for command mode, F1 for help",
            rag_status,
            prov_status,
            app_data.current_status
        );

        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_widget(status_paragraph, area);
    }
}

impl TuiRenderer for RatatuiRenderer {
    fn initialize(&mut self) -> Result<(), TuiError> {
        // Terminal is already initialized in new(), but we can add any additional setup here
        self.terminal.clear().map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        Ok(())
    }

    fn render(&mut self, app_data: &AppDisplayData) -> Result<(), TuiError> {
        let show_help = self.state.show_help;
        let state = &self.state;
        
        self.terminal
            .draw(|f| {
                if show_help {
                    Self::render_help_static(f);
                } else {
                    Self::render_main_ui_static(f, app_data, state);
                }
            })
            .map_err(|e| TuiError::Rendering(e.to_string()))?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<Option<UserAction>, TuiError> {
        // Check for input with a timeout to avoid blocking
        if event::poll(Duration::from_millis(100))
            .map_err(|e| TuiError::InputHandling(e.to_string()))?
        {
            if let Event::Key(key) = event::read()
                .map_err(|e| TuiError::InputHandling(e.to_string()))?
            {
                // Only handle key press events, not release
                if key.kind != KeyEventKind::Press {
                    return Ok(None);
                }

                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        return Ok(Some(UserAction::Exit));
                    }
                    KeyCode::F(1) => {
                        self.state.show_help = !self.state.show_help;
                        return Ok(None);
                    }
                    KeyCode::Esc => {
                        if self.state.show_help {
                            self.state.show_help = false;
                        } else if !self.state.input_buffer.is_empty() {
                            self.state.input_buffer.clear();
                        } else {
                            return Ok(Some(UserAction::Exit));
                        }
                        return Ok(None);
                    }
                    KeyCode::Tab => {
                        self.state.command_mode = !self.state.command_mode;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        if !self.state.input_buffer.is_empty() {
                            let input = self.state.input_buffer.clone();
                            self.state.input_buffer.clear();
                            
                            if self.state.command_mode || input.starts_with('/') {
                                // Parse as command
                                let command_str = if input.starts_with('/') {
                                    &input[1..]
                                } else {
                                    &input
                                };
                                
                                let command = self.parse_command(command_str)?;
                                return Ok(Some(UserAction::ExecuteCommand(command)));
                            } else {
                                // Regular message
                                return Ok(Some(UserAction::SendMessage));
                            }
                        }
                        return Ok(None);
                    }
                    KeyCode::Backspace => {
                        self.state.input_buffer.pop();
                        return Ok(None);
                    }
                    KeyCode::PageUp => {
                        return Ok(Some(UserAction::ScrollUp));
                    }
                    KeyCode::PageDown => {
                        return Ok(Some(UserAction::ScrollDown));
                    }
                    KeyCode::Char(c) => {
                        self.state.input_buffer.push(c);
                        self.state.last_input_time = Instant::now();
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    fn cleanup(&mut self) -> Result<(), TuiError> {
        disable_raw_mode().map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        self.terminal.show_cursor().map_err(|e| TuiError::TerminalInit(e.to_string()))?;
        Ok(())
    }
}

impl RatatuiRenderer {
    fn parse_command(&self, command_str: &str) -> Result<Command, TuiError> {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        if parts.is_empty() {
            return Err(TuiError::InputHandling("Empty command".to_string()));
        }

        match parts[0] {
            "help" => Ok(Command::Help),
            "config" => Ok(Command::Config),
            "clear" => Ok(Command::Clear),
            "toggle-rag" => Ok(Command::ToggleRag),
            "toggle-prov" | "toggle-provisional" => Ok(Command::ToggleProvisional),
            "add-source" => {
                if parts.len() < 2 {
                    return Err(TuiError::InputHandling("add-source requires a path argument".to_string()));
                }
                Ok(Command::AddSource(parts[1].into()))
            }
            "remove-source" => {
                if parts.len() < 2 {
                    return Err(TuiError::InputHandling("remove-source requires a path argument".to_string()));
                }
                Ok(Command::RemoveSource(parts[1].into()))
            }
            "list-sources" => Ok(Command::ListSources),
            "exit" | "quit" => Ok(Command::Exit),
            _ => Err(TuiError::InputHandling(format!("Unknown command: {}", parts[0]))),
        }
    }

    pub fn get_input_buffer(&self) -> &str {
        &self.state.input_buffer
    }

    pub fn clear_input_buffer(&mut self) {
        self.state.input_buffer.clear();
    }

    pub fn set_status_message(&mut self, message: Option<String>) {
        self.state.status_message = message;
    }
}