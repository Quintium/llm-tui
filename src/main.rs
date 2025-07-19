use llm_tui_assistant::app::AppController;
use llm_tui_assistant::types::*;
use llm_tui_assistant::ui::{RatatuiRenderer, TuiRenderer};
use std::io;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting LLM TUI Assistant");

    // Initialize application controller
    let mut app = match AppController::new() {
        Ok(app) => app,
        Err(e) => {
            error!("Failed to initialize application: {}", e);
            return Err(Box::new(e));
        }
    };

    // Initialize TUI renderer
    let mut renderer = match RatatuiRenderer::new() {
        Ok(renderer) => renderer,
        Err(e) => {
            error!("Failed to initialize TUI: {}", e);
            return Err(Box::new(e));
        }
    };

    info!("Application initialized successfully");

    // TODO: Implement main application loop
    // For now, just show a placeholder message
    println!("LLM TUI Assistant - Project structure created successfully!");
    println!("This is a placeholder main function. The full TUI implementation will be added in subsequent tasks.");
    
    // Test basic functionality
    let test_input = UserInput::Message("Hello, world!".to_string());
    match app.process_user_input(test_input).await {
        Ok(response) => println!("App response: {}", response),
        Err(e) => error!("Error processing input: {}", e),
    }

    // Test help command
    let help_command = UserInput::Command(Command::Help);
    match app.process_user_input(help_command).await {
        Ok(response) => println!("Help: {}", response),
        Err(e) => error!("Error processing help command: {}", e),
    }

    // Cleanup
    if let Err(e) = renderer.cleanup() {
        error!("Error during cleanup: {}", e);
    }

    info!("Application shutdown complete");
    Ok(())
}