use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use regex::Regex;

// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm_provider: Option<LlmProvider>,
    pub global_system_prompt: Option<String>,
    pub rag_enabled_default: bool,
    pub provisional_mode_default: bool,
    pub data_sources: Vec<PathBuf>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub conversation_storage_path: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm_provider: None,
            global_system_prompt: None,
            rag_enabled_default: false,
            provisional_mode_default: false,
            data_sources: Vec::new(),
            include_patterns: vec![
                r"\.txt$".to_string(),
                r"\.md$".to_string(),
                r"\.json$".to_string(),
                r"\.toml$".to_string(),
                r"\.yaml$".to_string(),
                r"\.yml$".to_string(),
            ],
            exclude_patterns: vec![
                r"\.git/".to_string(),
                r"target/".to_string(),
                r"node_modules/".to_string(),
                r"\.DS_Store$".to_string(),
            ],
            conversation_storage_path: PathBuf::from("conversations"),
        }
    }
}

// Manages application configuration loading and saving
pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path();
        let mut config = Self::load_config_from_file(&config_path)?;
        
        // Validate the loaded configuration
        Self::validate_config(&mut config)?;

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn load_config() -> Result<AppConfig, ConfigError> {
        let config_path = Self::get_config_path();
        let mut config = Self::load_config_from_file(&config_path)?;
        Self::validate_config(&mut config)?;
        Ok(config)
    }

    fn get_config_path() -> PathBuf {
        // Try to use XDG config directory, fallback to current directory
        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_dir).join("llm-tui-assistant").join("config.toml")
        } else if let Ok(home_dir) = std::env::var("HOME") {
            PathBuf::from(home_dir).join(".config").join("llm-tui-assistant").join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    fn load_config_from_file(path: &PathBuf) -> Result<AppConfig, ConfigError> {
        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ConfigError::FileError(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            ConfigError::Serialization(format!("Failed to parse config file: {}", e))
        })
    }

    pub fn save_config(&self) -> Result<(), ConfigError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::FileError(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(&self.config).map_err(|e| {
            ConfigError::Serialization(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(&self.config_path, content).map_err(|e| {
            ConfigError::FileError(format!("Failed to write config file: {}", e))
        })
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn update_llm_provider(&mut self, provider: LlmProvider) -> Result<(), ConfigError> {
        self.config.llm_provider = Some(provider);
        self.save_config()
    }

    pub fn update_system_prompt(&mut self, prompt: Option<String>) -> Result<(), ConfigError> {
        self.config.global_system_prompt = prompt;
        self.save_config()
    }

    pub fn add_data_source(&mut self, path: PathBuf) -> Result<(), ConfigError> {
        if !self.config.data_sources.contains(&path) {
            self.config.data_sources.push(path);
            self.save_config()?;
        }
        Ok(())
    }

    pub fn remove_data_source(&mut self, path: &PathBuf) -> Result<(), ConfigError> {
        self.config.data_sources.retain(|p| p != path);
        self.save_config()
    }

    pub fn update_rag_default(&mut self, enabled: bool) -> Result<(), ConfigError> {
        self.config.rag_enabled_default = enabled;
        self.save_config()
    }

    pub fn update_provisional_default(&mut self, enabled: bool) -> Result<(), ConfigError> {
        self.config.provisional_mode_default = enabled;
        self.save_config()
    }

    pub fn update_conversation_storage_path(&mut self, path: PathBuf) -> Result<(), ConfigError> {
        self.config.conversation_storage_path = path;
        self.save_config()
    }

    pub fn add_include_pattern(&mut self, pattern: String) -> Result<(), ConfigError> {
        // Validate regex pattern
        Regex::new(&pattern).map_err(|e| {
            ConfigError::Validation(format!("Invalid include pattern '{}': {}", pattern, e))
        })?;
        
        if !self.config.include_patterns.contains(&pattern) {
            self.config.include_patterns.push(pattern);
            self.save_config()?;
        }
        Ok(())
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) -> Result<(), ConfigError> {
        // Validate regex pattern
        Regex::new(&pattern).map_err(|e| {
            ConfigError::Validation(format!("Invalid exclude pattern '{}': {}", pattern, e))
        })?;
        
        if !self.config.exclude_patterns.contains(&pattern) {
            self.config.exclude_patterns.push(pattern);
            self.save_config()?;
        }
        Ok(())
    }

    pub fn remove_include_pattern(&mut self, pattern: &str) -> Result<(), ConfigError> {
        self.config.include_patterns.retain(|p| p != pattern);
        self.save_config()
    }

    pub fn remove_exclude_pattern(&mut self, pattern: &str) -> Result<(), ConfigError> {
        self.config.exclude_patterns.retain(|p| p != pattern);
        self.save_config()
    }

    /// Validates the configuration and applies fixes where possible
    fn validate_config(config: &mut AppConfig) -> Result<(), ConfigError> {
        // Validate regex patterns
        for pattern in &config.include_patterns {
            Regex::new(pattern).map_err(|e| {
                ConfigError::Validation(format!("Invalid include pattern '{}': {}", pattern, e))
            })?;
        }

        for pattern in &config.exclude_patterns {
            Regex::new(pattern).map_err(|e| {
                ConfigError::Validation(format!("Invalid exclude pattern '{}': {}", pattern, e))
            })?;
        }

        // Validate data sources exist and are accessible
        let mut valid_sources = Vec::new();
        for source in &config.data_sources {
            if source.exists() {
                valid_sources.push(source.clone());
            }
        }
        config.data_sources = valid_sources;

        // Validate LLM provider configuration if present
        if let Some(ref provider) = config.llm_provider {
            Self::validate_llm_provider(provider)?;
        }

        // Ensure conversation storage path is valid
        if config.conversation_storage_path.to_string_lossy().is_empty() {
            config.conversation_storage_path = PathBuf::from("conversations");
        }

        Ok(())
    }

    fn validate_llm_provider(provider: &LlmProvider) -> Result<(), ConfigError> {
        // Validate API key is not empty
        if provider.api_key.trim().is_empty() {
            return Err(ConfigError::Validation(
                "LLM provider API key cannot be empty".to_string()
            ));
        }

        // Validate model name is not empty
        if provider.model.trim().is_empty() {
            return Err(ConfigError::Validation(
                "LLM provider model name cannot be empty".to_string()
            ));
        }

        // Validate base URL if provided
        if let Some(ref base_url) = provider.base_url {
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                return Err(ConfigError::Validation(
                    "LLM provider base URL must start with http:// or https://".to_string()
                ));
            }
        }

        // Validate temperature range
        if let Some(temp) = provider.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err(ConfigError::Validation(
                    "LLM provider temperature must be between 0.0 and 2.0".to_string()
                ));
            }
        }

        // Validate max_tokens
        if let Some(max_tokens) = provider.max_tokens {
            if max_tokens == 0 {
                return Err(ConfigError::Validation(
                    "LLM provider max_tokens must be greater than 0".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Creates a new configuration with default values
    pub fn create_default_config() -> AppConfig {
        AppConfig::default()
    }

    /// Resets configuration to defaults
    pub fn reset_to_defaults(&mut self) -> Result<(), ConfigError> {
        self.config = AppConfig::default();
        self.save_config()
    }

    /// Validates the current configuration without modifying it
    pub fn validate_current_config(&self) -> Result<(), ConfigError> {
        let mut config_copy = self.config.clone();
        Self::validate_config(&mut config_copy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_config() -> AppConfig {
        AppConfig {
            llm_provider: Some(LlmProvider {
                provider_type: ProviderType::OpenAi,
                api_key: "test-api-key".to_string(),
                model: "gpt-4".to_string(),
                base_url: None,
                max_tokens: Some(4000),
                temperature: Some(0.7),
            }),
            global_system_prompt: Some("You are a helpful assistant.".to_string()),
            rag_enabled_default: true,
            provisional_mode_default: false,
            data_sources: vec![PathBuf::from("/tmp/test")],
            include_patterns: vec![r"\.txt$".to_string(), r"\.md$".to_string()],
            exclude_patterns: vec![r"\.git/".to_string()],
            conversation_storage_path: PathBuf::from("test_conversations"),
        }
    }

    fn create_invalid_llm_provider() -> LlmProvider {
        LlmProvider {
            provider_type: ProviderType::OpenAi,
            api_key: "".to_string(), // Invalid: empty API key
            model: "gpt-4".to_string(),
            base_url: Some("invalid-url".to_string()), // Invalid: not http/https
            max_tokens: Some(0), // Invalid: zero tokens
            temperature: Some(3.0), // Invalid: out of range
        }
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        assert!(config.llm_provider.is_none());
        assert!(config.global_system_prompt.is_none());
        assert!(!config.rag_enabled_default);
        assert!(!config.provisional_mode_default);
        assert!(config.data_sources.is_empty());
        assert!(!config.include_patterns.is_empty());
        assert!(!config.exclude_patterns.is_empty());
        assert_eq!(config.conversation_storage_path, PathBuf::from("conversations"));
    }

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        
        // Test serialization to TOML
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize config");
        assert!(toml_str.contains("rag_enabled_default = true"));
        assert!(toml_str.contains("test-api-key"));
        
        // Test deserialization from TOML
        let deserialized: AppConfig = toml::from_str(&toml_str).expect("Failed to deserialize config");
        assert_eq!(config.rag_enabled_default, deserialized.rag_enabled_default);
        assert_eq!(config.llm_provider.as_ref().unwrap().api_key, 
                   deserialized.llm_provider.as_ref().unwrap().api_key);
    }

    #[test]
    fn test_config_manager_new_with_nonexistent_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("nonexistent").join("config.toml");
        
        // Mock the config path by setting environment variable
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let manager = ConfigManager::new().expect("Failed to create ConfigManager");
        let config = manager.get_config();
        
        // Should use default configuration
        assert!(config.llm_provider.is_none());
        assert!(!config.rag_enabled_default);
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_config_manager_load_existing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join("llm-tui-assistant");
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        
        let config_path = config_dir.join("config.toml");
        let test_config = create_test_config();
        let toml_content = toml::to_string_pretty(&test_config).expect("Failed to serialize");
        fs::write(&config_path, toml_content).expect("Failed to write config file");
        
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let manager = ConfigManager::new().expect("Failed to create ConfigManager");
        let config = manager.get_config();
        
        assert!(config.llm_provider.is_some());
        assert_eq!(config.llm_provider.as_ref().unwrap().model, "gpt-4");
        assert!(config.rag_enabled_default);
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_config_manager_save_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        // Update configuration
        manager.update_rag_default(true).expect("Failed to update RAG default");
        manager.update_system_prompt(Some("Test prompt".to_string()))
            .expect("Failed to update system prompt");
        
        // Verify the file was saved
        let config_path = temp_dir.path().join("llm-tui-assistant").join("config.toml");
        assert!(config_path.exists());
        
        let content = fs::read_to_string(&config_path).expect("Failed to read config file");
        assert!(content.contains("rag_enabled_default = true"));
        assert!(content.contains("Test prompt"));
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_llm_provider_validation_valid() {
        let provider = LlmProvider {
            provider_type: ProviderType::OpenAi,
            api_key: "valid-key".to_string(),
            model: "gpt-4".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
            max_tokens: Some(4000),
            temperature: Some(0.7),
        };
        
        assert!(ConfigManager::validate_llm_provider(&provider).is_ok());
    }

    #[test]
    fn test_llm_provider_validation_invalid_api_key() {
        let mut provider = create_invalid_llm_provider();
        provider.api_key = "".to_string();
        
        let result = ConfigManager::validate_llm_provider(&provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key cannot be empty"));
    }

    #[test]
    fn test_llm_provider_validation_invalid_model() {
        let mut provider = create_invalid_llm_provider();
        provider.api_key = "valid-key".to_string();
        provider.model = "".to_string();
        
        let result = ConfigManager::validate_llm_provider(&provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("model name cannot be empty"));
    }

    #[test]
    fn test_llm_provider_validation_invalid_base_url() {
        let mut provider = create_invalid_llm_provider();
        provider.api_key = "valid-key".to_string();
        provider.model = "gpt-4".to_string();
        provider.base_url = Some("invalid-url".to_string());
        
        let result = ConfigManager::validate_llm_provider(&provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with http"));
    }

    #[test]
    fn test_llm_provider_validation_invalid_temperature() {
        let mut provider = create_invalid_llm_provider();
        provider.api_key = "valid-key".to_string();
        provider.model = "gpt-4".to_string();
        provider.base_url = None;
        provider.max_tokens = Some(1000);
        provider.temperature = Some(3.0);
        
        let result = ConfigManager::validate_llm_provider(&provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("temperature must be between"));
    }

    #[test]
    fn test_llm_provider_validation_invalid_max_tokens() {
        let mut provider = create_invalid_llm_provider();
        provider.api_key = "valid-key".to_string();
        provider.model = "gpt-4".to_string();
        provider.base_url = None;
        provider.temperature = Some(0.7);
        provider.max_tokens = Some(0);
        
        let result = ConfigManager::validate_llm_provider(&provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_tokens must be greater than 0"));
    }

    #[test]
    fn test_regex_pattern_validation_valid() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        assert!(manager.add_include_pattern(r"\.rs$".to_string()).is_ok());
        assert!(manager.add_exclude_pattern(r"target/".to_string()).is_ok());
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_regex_pattern_validation_invalid() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        // Invalid regex pattern
        let result = manager.add_include_pattern("[invalid".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid include pattern"));
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_data_source_management() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        let test_path = PathBuf::from("/tmp/test");
        
        // Add data source
        manager.add_data_source(test_path.clone()).expect("Failed to add data source");
        assert!(manager.get_config().data_sources.contains(&test_path));
        
        // Remove data source
        manager.remove_data_source(&test_path).expect("Failed to remove data source");
        assert!(!manager.get_config().data_sources.contains(&test_path));
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_pattern_management() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        let test_pattern = r"\.test$".to_string();
        
        // Add include pattern
        manager.add_include_pattern(test_pattern.clone()).expect("Failed to add include pattern");
        assert!(manager.get_config().include_patterns.contains(&test_pattern));
        
        // Remove include pattern
        manager.remove_include_pattern(&test_pattern).expect("Failed to remove include pattern");
        assert!(!manager.get_config().include_patterns.contains(&test_pattern));
        
        // Add exclude pattern
        manager.add_exclude_pattern(test_pattern.clone()).expect("Failed to add exclude pattern");
        assert!(manager.get_config().exclude_patterns.contains(&test_pattern));
        
        // Remove exclude pattern
        manager.remove_exclude_pattern(&test_pattern).expect("Failed to remove exclude pattern");
        assert!(!manager.get_config().exclude_patterns.contains(&test_pattern));
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_config_validation_with_invalid_patterns() {
        let mut config = AppConfig::default();
        config.include_patterns.push("[invalid".to_string());
        
        let result = ConfigManager::validate_config(&mut config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid include pattern"));
    }

    #[test]
    fn test_config_validation_removes_nonexistent_sources() {
        let mut config = AppConfig::default();
        config.data_sources.push(PathBuf::from("/nonexistent/path"));
        config.data_sources.push(PathBuf::from("/tmp")); // This should exist on most systems
        
        let result = ConfigManager::validate_config(&mut config);
        assert!(result.is_ok());
        
        // Should have removed the nonexistent path but kept /tmp if it exists
        assert!(!config.data_sources.contains(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_reset_to_defaults() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let mut manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        // Modify configuration
        manager.update_rag_default(true).expect("Failed to update RAG default");
        manager.update_system_prompt(Some("Test".to_string())).expect("Failed to update prompt");
        
        // Reset to defaults
        manager.reset_to_defaults().expect("Failed to reset to defaults");
        
        let config = manager.get_config();
        assert!(!config.rag_enabled_default);
        assert!(config.global_system_prompt.is_none());
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_validate_current_config() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let manager = ConfigManager::new().expect("Failed to create ConfigManager");
        
        // Should validate successfully with default config
        assert!(manager.validate_current_config().is_ok());
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_malformed_config_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path().join("llm-tui-assistant");
        fs::create_dir_all(&config_dir).expect("Failed to create config dir");
        
        let config_path = config_dir.join("config.toml");
        fs::write(&config_path, "invalid toml content [[[").expect("Failed to write invalid config");
        
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        
        let result = ConfigManager::new();
        assert!(result.is_err());

        let err = result.map(|_| "Expected error, got ok").unwrap_err();
        assert!(err.to_string().contains("Failed to parse config file"));
        
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}