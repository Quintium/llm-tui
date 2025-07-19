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
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    // Helper function to create test messages
    fn create_test_message(role: MessageRole, content: &str, provisional: bool) -> Message {
        Message {
            role,
            content: content.to_string(),
            timestamp: Utc::now(),
            provisional,
            context_files: vec![],
        }
    }

    // Helper function to create test app display data
    fn create_test_app_data() -> AppDisplayData {
        AppDisplayData {
            messages: vec![
                create_test_message(MessageRole::User, "Hello", false),
                create_test_message(MessageRole::Assistant, "Hi there!", false),
                create_test_message(MessageRole::User, "Test provisional", true),
            ],
            provisional_mode: false,
            rag_enabled: true,
            current_status: "Ready".to_string(),
            streaming_response: None,
        }
    }

    #[test]
    fn test_tui_state_default() {
        let state = TuiState::default();
        
        assert_eq!(state.input_buffer, "");
        assert_eq!(state.scroll_position, 0);
        assert!(!state.command_mode);
        assert!(state.status_message.is_none());
        assert!(!state.show_help);
        // last_input_time should be recent
        assert!(state.last_input_time.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_app_display_data_default() {
        let data = AppDisplayData::default();
        
        assert!(data.messages.is_empty());
        assert!(!data.provisional_mode);
        assert!(!data.rag_enabled);
        assert_eq!(data.current_status, "");
        assert!(data.streaming_response.is_none());
    }

    #[test]
    fn test_command_parsing_basic_commands() {
        let renderer = create_mock_renderer();
        
        // Test basic commands without arguments
        assert!(matches!(renderer.parse_command("help"), Ok(Command::Help)));
        assert!(matches!(renderer.parse_command("config"), Ok(Command::Config)));
        assert!(matches!(renderer.parse_command("clear"), Ok(Command::Clear)));
        assert!(matches!(renderer.parse_command("toggle-rag"), Ok(Command::ToggleRag)));
        assert!(matches!(renderer.parse_command("toggle-prov"), Ok(Command::ToggleProvisional)));
        assert!(matches!(renderer.parse_command("toggle-provisional"), Ok(Command::ToggleProvisional)));
        assert!(matches!(renderer.parse_command("list-sources"), Ok(Command::ListSources)));
        assert!(matches!(renderer.parse_command("exit"), Ok(Command::Exit)));
        assert!(matches!(renderer.parse_command("quit"), Ok(Command::Exit)));
    }

    #[test]
    fn test_command_parsing_with_arguments() {
        let renderer = create_mock_renderer();
        
        // Test commands with arguments
        match renderer.parse_command("add-source /path/to/file") {
            Ok(Command::AddSource(path)) => {
                assert_eq!(path.to_string_lossy(), "/path/to/file");
            }
            _ => panic!("Expected AddSource command"),
        }

        match renderer.parse_command("remove-source /another/path") {
            Ok(Command::RemoveSource(path)) => {
                assert_eq!(path.to_string_lossy(), "/another/path");
            }
            _ => panic!("Expected RemoveSource command"),
        }
    }

    #[test]
    fn test_command_parsing_errors() {
        let renderer = create_mock_renderer();
        
        // Test empty command
        assert!(renderer.parse_command("").is_err());
        
        // Test unknown command
        assert!(renderer.parse_command("unknown-command").is_err());
        
        // Test commands missing required arguments
        assert!(renderer.parse_command("add-source").is_err());
        assert!(renderer.parse_command("remove-source").is_err());
    }

    #[test]
    fn test_command_parsing_whitespace_handling() {
        let renderer = create_mock_renderer();
        
        // Test commands with extra whitespace
        assert!(matches!(renderer.parse_command("  help  "), Ok(Command::Help)));
        assert!(matches!(renderer.parse_command("\tconfig\t"), Ok(Command::Config)));
        
        // Test arguments with spaces
        match renderer.parse_command("add-source /path with spaces/file.txt") {
            Ok(Command::AddSource(path)) => {
                assert_eq!(path.to_string_lossy(), "/path");
            }
            _ => panic!("Expected AddSource command"),
        }
    }

    #[test]
    fn test_input_buffer_operations() {
        let mut renderer = create_mock_renderer();
        
        // Test getting empty buffer
        assert_eq!(renderer.get_input_buffer(), "");
        
        // Test buffer manipulation through state
        renderer.state.input_buffer = "test input".to_string();
        assert_eq!(renderer.get_input_buffer(), "test input");
        
        // Test clearing buffer
        renderer.clear_input_buffer();
        assert_eq!(renderer.get_input_buffer(), "");
    }

    #[test]
    fn test_status_message_management() {
        let mut renderer = create_mock_renderer();
        
        // Test setting status message
        renderer.set_status_message(Some("Test status".to_string()));
        assert_eq!(renderer.state.status_message, Some("Test status".to_string()));
        
        // Test clearing status message
        renderer.set_status_message(None);
        assert!(renderer.state.status_message.is_none());
    }

    #[test]
    fn test_tui_state_mode_toggles() {
        let mut state = TuiState::default();
        
        // Test command mode toggle
        assert!(!state.command_mode);
        state.command_mode = !state.command_mode;
        assert!(state.command_mode);
        
        // Test help display toggle
        assert!(!state.show_help);
        state.show_help = !state.show_help;
        assert!(state.show_help);
    }

    #[test]
    fn test_message_role_display_properties() {
        // Test that message roles have expected properties for display
        let user_msg = create_test_message(MessageRole::User, "User message", false);
        let assistant_msg = create_test_message(MessageRole::Assistant, "Assistant message", false);
        let system_msg = create_test_message(MessageRole::System, "System message", false);
        
        assert!(matches!(user_msg.role, MessageRole::User));
        assert!(matches!(assistant_msg.role, MessageRole::Assistant));
        assert!(matches!(system_msg.role, MessageRole::System));
        
        // Test provisional flag
        let provisional_msg = create_test_message(MessageRole::User, "Provisional", true);
        assert!(provisional_msg.provisional);
    }

    #[test]
    fn test_app_display_data_with_streaming() {
        let mut data = create_test_app_data();
        
        // Test without streaming
        assert!(data.streaming_response.is_none());
        
        // Test with streaming response
        data.streaming_response = Some("Partial response...".to_string());
        assert_eq!(data.streaming_response, Some("Partial response...".to_string()));
    }

    #[test]
    fn test_app_display_data_status_indicators() {
        let mut data = create_test_app_data();
        
        // Test RAG enabled/disabled
        data.rag_enabled = true;
        assert!(data.rag_enabled);
        
        data.rag_enabled = false;
        assert!(!data.rag_enabled);
        
        // Test provisional mode
        data.provisional_mode = true;
        assert!(data.provisional_mode);
        
        data.provisional_mode = false;
        assert!(!data.provisional_mode);
        
        // Test status message
        data.current_status = "Processing...".to_string();
        assert_eq!(data.current_status, "Processing...");
    }

    #[test]
    fn test_message_timestamp_ordering() {
        let now = Utc::now();
        let msg1 = Message {
            role: MessageRole::User,
            content: "First message".to_string(),
            timestamp: now,
            provisional: false,
            context_files: vec![],
        };
        
        let msg2 = Message {
            role: MessageRole::Assistant,
            content: "Second message".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            provisional: false,
            context_files: vec![],
        };
        
        // Verify timestamp ordering
        assert!(msg1.timestamp < msg2.timestamp);
    }

    #[test]
    fn test_context_files_in_messages() {
        let context_files = vec![
            PathBuf::from("/path/to/file1.txt"),
            PathBuf::from("/path/to/file2.md"),
        ];
        
        let msg = Message {
            role: MessageRole::Assistant,
            content: "Response with context".to_string(),
            timestamp: Utc::now(),
            provisional: false,
            context_files: context_files.clone(),
        };
        
        assert_eq!(msg.context_files.len(), 2);
        assert_eq!(msg.context_files, context_files);
    }

    #[test]
    fn test_scroll_position_bounds() {
        let mut state = TuiState::default();
        
        // Test initial scroll position
        assert_eq!(state.scroll_position, 0);
        
        // Test setting scroll position
        state.scroll_position = 5;
        assert_eq!(state.scroll_position, 5);
        
        // Note: Actual bounds checking would be implemented in the rendering logic
        // based on the number of messages and terminal height
    }

    #[test]
    fn test_input_timing_tracking() {
        let mut state = TuiState::default();
        let initial_time = state.last_input_time;
        
        // Simulate input timing update
        std::thread::sleep(Duration::from_millis(10));
        state.last_input_time = Instant::now();
        
        assert!(state.last_input_time > initial_time);
    }

    // Mock renderer for testing that doesn't require terminal initialization
    struct MockRenderer {
        state: TuiState,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                state: TuiState::default(),
            }
        }

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

        fn get_input_buffer(&self) -> &str {
            &self.state.input_buffer
        }

        fn clear_input_buffer(&mut self) {
            self.state.input_buffer.clear();
        }

        fn set_status_message(&mut self, message: Option<String>) {
            self.state.status_message = message;
        }
    }

    // Helper function to create a mock renderer for testing
    fn create_mock_renderer() -> MockRenderer {
        MockRenderer::new()
    }

    // Integration-style tests for UI behavior
    mod integration_tests {
        use super::*;

        #[test]
        fn test_message_display_formatting() {
            let data = create_test_app_data();
            
            // Verify we have the expected test messages
            assert_eq!(data.messages.len(), 3);
            
            // Check message content and roles
            assert_eq!(data.messages[0].content, "Hello");
            assert!(matches!(data.messages[0].role, MessageRole::User));
            assert!(!data.messages[0].provisional);
            
            assert_eq!(data.messages[1].content, "Hi there!");
            assert!(matches!(data.messages[1].role, MessageRole::Assistant));
            assert!(!data.messages[1].provisional);
            
            assert_eq!(data.messages[2].content, "Test provisional");
            assert!(matches!(data.messages[2].role, MessageRole::User));
            assert!(data.messages[2].provisional);
        }

        #[test]
        fn test_status_bar_information() {
            let data = create_test_app_data();
            
            // Verify status information is available
            assert!(data.rag_enabled);
            assert!(!data.provisional_mode);
            assert_eq!(data.current_status, "Ready");
        }

        #[test]
        fn test_command_vs_message_mode() {
            let mut state = TuiState::default();
            
            // Test message mode (default)
            assert!(!state.command_mode);
            
            // Test switching to command mode
            state.command_mode = true;
            assert!(state.command_mode);
            
            // Test input buffer behavior in different modes
            state.input_buffer = "test input".to_string();
            assert_eq!(state.input_buffer, "test input");
            
            // Command mode should affect how input is interpreted
            // but the buffer itself works the same way
        }

        #[test]
        fn test_help_display_toggle() {
            let mut state = TuiState::default();
            
            // Test help initially hidden
            assert!(!state.show_help);
            
            // Test showing help
            state.show_help = true;
            assert!(state.show_help);
            
            // Test hiding help
            state.show_help = false;
            assert!(!state.show_help);
        }
    }

    // Performance and edge case tests
    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_empty_input_buffer_operations() {
            let mut renderer = create_mock_renderer();
            
            // Test operations on empty buffer
            assert_eq!(renderer.get_input_buffer(), "");
            renderer.clear_input_buffer(); // Should not panic
            assert_eq!(renderer.get_input_buffer(), "");
        }

        #[test]
        fn test_large_message_content() {
            let large_content = "x".repeat(10000);
            let msg = create_test_message(MessageRole::User, &large_content, false);
            
            assert_eq!(msg.content.len(), 10000);
            assert_eq!(msg.content, large_content);
        }

        #[test]
        fn test_many_messages_display_data() {
            let mut data = AppDisplayData::default();
            
            // Add many messages
            for i in 0..1000 {
                data.messages.push(create_test_message(
                    MessageRole::User,
                    &format!("Message {}", i),
                    i % 10 == 0, // Every 10th message is provisional
                ));
            }
            
            assert_eq!(data.messages.len(), 1000);
            
            // Check that provisional messages are correctly marked
            let provisional_count = data.messages.iter().filter(|m| m.provisional).count();
            assert_eq!(provisional_count, 100); // Every 10th message
        }

        #[test]
        fn test_special_characters_in_commands() {
            let renderer = create_mock_renderer();
            
            // Test commands with special characters in paths
            match renderer.parse_command("add-source /path/with-dashes/file_name.txt") {
                Ok(Command::AddSource(path)) => {
                    assert_eq!(path.to_string_lossy(), "/path/with-dashes/file_name.txt");
                }
                _ => panic!("Expected AddSource command"),
            }
        }

        #[test]
        fn test_unicode_in_input_buffer() {
            let mut renderer = create_mock_renderer();
            
            // Test unicode characters
            renderer.state.input_buffer = "Hello 世界 🌍".to_string();
            assert_eq!(renderer.get_input_buffer(), "Hello 世界 🌍");
            
            renderer.clear_input_buffer();
            assert_eq!(renderer.get_input_buffer(), "");
        }
    }
}