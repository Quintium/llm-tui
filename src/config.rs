use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        let config = Self::load_config_from_file(&config_path)?;

        Ok(Self {
            config_path,
            config,
        })
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
}