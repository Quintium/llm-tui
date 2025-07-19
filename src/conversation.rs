use crate::types::*;
use crate::llm::LlmClient;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use uuid::Uuid;

// Conversation structure to hold message history and metadata
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub provisional_mode: bool,
}

impl Conversation {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: Utc::now(),
            provisional_mode: false,
        }
    }
}

// Manages conversation state and LLM communication
pub struct ConversationManager {
    current_conversation: Conversation,
    storage_path: PathBuf,
}

impl ConversationManager {
    pub fn new() -> Result<Self, ConversationError> {
        Ok(Self {
            current_conversation: Conversation::new(),
            storage_path: PathBuf::from("conversations"),
        })
    }

    pub async fn send_message(
        &mut self,
        content: String,
        provisional: bool,
        _llm_client: &dyn LlmClient,
    ) -> Result<String, ConversationError> {
        let message = Message {
            role: MessageRole::User,
            content: content.clone(),
            timestamp: Utc::now(),
            provisional,
            context_files: Vec::new(),
        };

        if !provisional {
            self.current_conversation.messages.push(message);
        }

        // TODO: Send to LLM and get response
        Ok(format!("Echo: {}", content))
    }

    pub fn save_conversation(&self) -> Result<(), ConversationError> {
        // TODO: Implement conversation persistence
        Ok(())
    }

    pub fn clear_conversation(&mut self) {
        self.current_conversation = Conversation::new();
    }

    pub fn toggle_provisional_mode(&mut self) {
        self.current_conversation.provisional_mode = !self.current_conversation.provisional_mode;
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.current_conversation.messages
    }

    pub fn is_provisional_mode(&self) -> bool {
        self.current_conversation.provisional_mode
    }
}