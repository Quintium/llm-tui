use crate::types::*;
use async_trait::async_trait;

// Response stream for handling streaming LLM responses
pub type ResponseStream = Box<dyn futures::Stream<Item = Result<String, LlmError>> + Unpin + Send>;

// LLM client trait for abstraction over different providers
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn send_message(&self, messages: &[Message]) -> Result<String, LlmError>;
    async fn stream_message(&self, messages: &[Message]) -> Result<ResponseStream, LlmError>;
}

// OpenAI client implementation
pub struct OpenAiClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn send_message(&self, _messages: &[Message]) -> Result<String, LlmError> {
        // TODO: Implement OpenAI API call
        Ok("OpenAI response placeholder".to_string())
    }

    async fn stream_message(&self, _messages: &[Message]) -> Result<ResponseStream, LlmError> {
        // TODO: Implement OpenAI streaming API call
        Err(LlmError::Api("Streaming not yet implemented".to_string()))
    }
}

// Anthropic client implementation
pub struct AnthropicClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn send_message(&self, _messages: &[Message]) -> Result<String, LlmError> {
        // TODO: Implement Anthropic API call
        Ok("Anthropic response placeholder".to_string())
    }

    async fn stream_message(&self, _messages: &[Message]) -> Result<ResponseStream, LlmError> {
        // TODO: Implement Anthropic streaming API call
        Err(LlmError::Api("Streaming not yet implemented".to_string()))
    }
}

// Factory function to create LLM clients based on provider configuration
pub fn create_llm_client(provider: &LlmProvider) -> Result<Box<dyn LlmClient>, LlmError> {
    match provider.provider_type {
        ProviderType::OpenAi => {
            let mut client = OpenAiClient::new(provider.api_key.clone(), provider.model.clone());
            if let Some(base_url) = &provider.base_url {
                client = client.with_base_url(base_url.clone());
            }
            Ok(Box::new(client))
        }
        ProviderType::Anthropic => {
            let client = AnthropicClient::new(provider.api_key.clone(), provider.model.clone());
            Ok(Box::new(client))
        }
        ProviderType::Local => {
            // TODO: Implement local model support
            Err(LlmError::Api("Local models not yet supported".to_string()))
        }
    }
}