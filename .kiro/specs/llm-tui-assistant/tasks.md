# Implementation Plan

- [ ] 1. Set up project structure and core data models
  - Create Rust project with Cargo.toml
  - Define core data structures (Message, UserAction, AppError types)
  - Create module structure (ui, app, conversation, rag, filesystem, config, llm)
  - _Requirements: 1.1, 1.2_

- [ ] 2. Implement configuration management system
  - Create AppConfig struct with all configuration fields (LLM provider, system prompt, RAG settings, file patterns)
  - Implement ConfigManager with load_config() and save_config() methods
  - Add configuration file validation and default value handling
  - Write unit tests for configuration loading and saving
  - _Requirements: 2.1, 2.3, 7.1, 7.4, 7.5, 12.1_

- [ ] 3. Create basic TUI foundation with ratatui
  - Implement TuiState struct for UI display state
  - Create RatatuiRenderer implementing TuiRenderer trait
  - Set up basic terminal initialization and cleanup
  - Implement basic input handling and display rendering
  - Add help display showing available commands
  - Write tests for UI state management
  - _Requirements: 1.1, 1.4, 3.1_

- [ ] 4. Implement command parsing system
  - Create Command enum with all supported commands (/help, /config, /clear, /toggle-rag, etc.)
  - Implement parse_command() function with comprehensive command syntax support
  - Add command validation and error handling
  - Write unit tests for all command parsing scenarios
  - _Requirements: 1.3, 7.3, 12.2_

- [ ] 5. Build LLM client abstraction and implementations
  - Define LlmClient trait with send_message() and stream_message() methods
  - Implement OpenAiClient with API key authentication and streaming support
  - Implement AnthropicClient with API key authentication and streaming support
  - Add error handling for network issues, API errors, and rate limiting
  - Write unit tests with mock HTTP responses
  - _Requirements: 2.2, 2.4, 6.1, 6.3_

- [ ] 6. Create conversation management system
  - Implement Conversation struct with message history and metadata
  - Create ConversationManager with message handling and LLM communication
  - Add support for provisional messages that don't get stored in history
  - Implement conversation persistence to .txt/.md files with timestamps
  - Write tests for conversation state management and persistence
  - _Requirements: 5.1, 5.2, 5.3, 8.1, 8.4, 10.1, 10.2, 10.3, 10.4_

- [ ] 7. Implement file system management and indexing
  - Create FileSystemManager with file indexing capabilities
  - Implement add_source(), index_sources(), and file validation methods
  - Add support for regex-based include/exclude patterns for file filtering
  - Implement file size checking to only index files suitable for LLM context
  - Support multiple file formats (text, markdown, JSON, config, code, logs)
  - Write tests for file indexing and pattern matching
  - _Requirements: 3.1, 3.2, 3.3, 3.5, 4.5, 4.6, 4.7_

- [ ] 8. Build keyword search functionality
  - Implement search_files() method in FileSystemManager
  - Create SearchResult struct with relevance scoring and snippet extraction
  - Add keyword matching across indexed file contents
  - Implement result ranking by relevance score
  - Write tests for search accuracy and performance
  - _Requirements: 11.3, 11.5_

- [ ] 9. Create structured RAG workflow engine
  - Implement RagEngine with the 6-step RAG process
  - Create process_query() method that orchestrates: query+files → keywords → search → file selection → content → response
  - Add RagContext struct to track workflow state between steps
  - Implement file content retrieval and context preparation for LLM
  - Write integration tests for complete RAG workflow
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 11.1, 11.2, 11.3, 11.4, 11.5, 11.6_

- [ ] 10. Implement streaming response handling
  - Add streaming support to ConversationManager using stream_response() method
  - Create ResponseStream handling for real-time display updates
  - Implement partial response display in TUI during streaming
  - Add streaming cancellation and error recovery
  - Write tests for streaming behavior and error scenarios
  - _Requirements: 1.5, 6.5_

- [ ] 11. Build application controller and orchestration
  - Create AppController that coordinates all components
  - Implement process_user_input() for handling messages and commands
  - Add RAG toggle functionality with runtime state management
  - Implement provisional mode toggle with visual feedback
  - Add status display showing current RAG and provisional mode states
  - Write integration tests for complete user interaction flows
  - _Requirements: 8.2, 8.3, 8.5, 12.2, 12.3, 12.4, 12.5_

- [ ] 12. Add multi-instance safety and file locking
  - Implement safe concurrent access to configuration and conversation files
  - Add file locking mechanisms to prevent data corruption
  - Handle multiple instances accessing shared resources gracefully
  - Write tests for concurrent access scenarios
  - _Requirements: 9.1, 9.2, 9.3_

- [ ] 13. Implement comprehensive error handling
  - Add graceful error recovery for network failures with retry logic
  - Implement user-friendly error messages for all error scenarios
  - Add detailed error logging for debugging purposes
  - Create error handling for file access failures and permission issues
  - Write tests for all error scenarios and recovery mechanisms
  - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [ ] 14. Create main application loop and integration
  - Implement main() function with proper async runtime setup
  - Create application initialization sequence with configuration loading
  - Add graceful shutdown handling and resource cleanup
  - Integrate all components into working application
  - Write end-to-end integration tests
  - _Requirements: 1.1, 2.1, 7.5_

- [ ] 15. Add final polish and user experience features
  - Implement conversation history scrolling and navigation
  - Add clear visual indicators for RAG and provisional mode states
  - Create comprehensive help system with command documentation
  - Add progress indicators for long-running operations
  - Optimize performance for responsive user experience
  - _Requirements: 1.3, 5.2, 6.5, 8.3, 12.5_