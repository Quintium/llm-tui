pub mod app;
pub mod config;
pub mod conversation;
pub mod filesystem;
pub mod llm;
pub mod rag;
pub mod ui;

pub use types::*;

pub mod types {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::PathBuf;

    // Core message structure for conversations
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub role: MessageRole,
        pub content: String,
        pub timestamp: DateTime<Utc>,
        pub provisional: bool,
        pub context_files: Vec<PathBuf>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MessageRole {
        User,
        Assistant,
        System,
    }

    // User input and actions
    #[derive(Debug, Clone)]
    pub enum UserInput {
        Message(String),
        Command(Command),
        KeyAction(KeyAction),
    }

    #[derive(Debug, Clone)]
    pub enum UserAction {
        SendMessage,
        ExecuteCommand(Command),
        ToggleMode,
        ScrollUp,
        ScrollDown,
        Exit,
    }

    #[derive(Debug, Clone)]
    pub enum KeyAction {
        Enter,
        Escape,
        Up,
        Down,
        PageUp,
        PageDown,
        Tab,
        Backspace,
        Delete,
        Char(char),
    }

    // Commands supported by the application
    #[derive(Debug, Clone)]
    pub enum Command {
        Help,
        Config,
        Clear,
        ToggleRag,
        ToggleProvisional,
        AddSource(PathBuf),
        RemoveSource(PathBuf),
        ListSources,
        Exit,
    }

    // Search and file system types
    #[derive(Debug, Clone)]
    pub struct SearchResult {
        pub file_path: PathBuf,
        pub relevance_score: f32,
        pub matching_lines: Vec<(usize, String)>,
        pub snippet: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FileInfo {
        pub path: PathBuf,
        pub size: u64,
        pub modified: DateTime<Utc>,
        pub file_type: FileType,
        pub indexable: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum FileType {
        Text,
        Markdown,
        Json,
        Config,
        Code(String), // Language extension
        Log,
        Binary, // Not indexable
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DataSource {
        pub path: PathBuf,
        pub source_type: SourceType,
        pub last_indexed: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SourceType {
        File,
        Directory,
    }

    // RAG workflow context
    #[derive(Debug, Clone)]
    pub struct RagContext {
        pub query: String,
        pub available_files: Vec<FileInfo>,
        pub keywords: Vec<String>,
        pub search_results: Vec<SearchResult>,
        pub selected_files: Vec<PathBuf>,
        pub file_contents: HashMap<PathBuf, String>,
    }

    // Configuration types
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LlmProvider {
        pub provider_type: ProviderType,
        pub api_key: String,
        pub model: String,
        pub base_url: Option<String>,
        pub max_tokens: Option<u32>,
        pub temperature: Option<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ProviderType {
        OpenAi,
        Anthropic,
        Local, // For future local model support
    }

    // Error types
    #[derive(Debug, thiserror::Error)]
    pub enum AppError {
        #[error("TUI error: {0}")]
        Tui(#[from] TuiError),
        
        #[error("LLM error: {0}")]
        Llm(#[from] LlmError),
        
        #[error("RAG error: {0}")]
        Rag(#[from] RagError),
        
        #[error("File system error: {0}")]
        FileSystem(#[from] FileSystemError),
        
        #[error("Configuration error: {0}")]
        Config(#[from] ConfigError),
        
        #[error("Conversation error: {0}")]
        Conversation(#[from] ConversationError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum TuiError {
        #[error("Terminal initialization failed: {0}")]
        TerminalInit(String),
        
        #[error("Input handling error: {0}")]
        InputHandling(String),
        
        #[error("Rendering error: {0}")]
        Rendering(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum LlmError {
        #[error("Network error: {0}")]
        Network(String),
        
        #[error("API error: {0}")]
        Api(String),
        
        #[error("Authentication error")]
        Authentication,
        
        #[error("Rate limit exceeded")]
        RateLimit,
        
        #[error("Context window exceeded")]
        ContextWindowExceeded,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum RagError {
        #[error("File processing error: {0}")]
        FileProcessing(String),
        
        #[error("Search error: {0}")]
        Search(String),
        
        #[error("Context preparation error: {0}")]
        ContextPreparation(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum FileSystemError {
        #[error("File access error: {0}")]
        FileAccess(String),
        
        #[error("Indexing error: {0}")]
        Indexing(String),
        
        #[error("Permission denied: {0}")]
        PermissionDenied(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ConfigError {
        #[error("Configuration file error: {0}")]
        FileError(String),
        
        #[error("Validation error: {0}")]
        Validation(String),
        
        #[error("Serialization error: {0}")]
        Serialization(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ConversationError {
        #[error("Message processing error: {0}")]
        MessageProcessing(String),
        
        #[error("Storage error: {0}")]
        Storage(String),
        
        #[error("History error: {0}")]
        History(String),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum CommandError {
        #[error("Invalid command: {0}")]
        InvalidCommand(String),
        
        #[error("Missing argument: {0}")]
        MissingArgument(String),
        
        #[error("Invalid argument: {0}")]
        InvalidArgument(String),
    }
}