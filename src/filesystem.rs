use crate::types::*;
use chrono::Utc;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

// Manages file system operations, indexing, and searching
pub struct FileSystemManager {
    indexed_sources: Vec<DataSource>,
    file_index: HashMap<PathBuf, FileInfo>,
    include_patterns: Vec<Regex>,
    exclude_patterns: Vec<Regex>,
}

impl FileSystemManager {
    pub fn new() -> Self {
        Self {
            indexed_sources: Vec::new(),
            file_index: HashMap::new(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    pub fn add_source(&mut self, path: PathBuf) -> Result<(), FileSystemError> {
        if !path.exists() {
            return Err(FileSystemError::FileAccess(format!(
                "Path does not exist: {:?}",
                path
            )));
        }

        let source_type = if path.is_file() {
            SourceType::File
        } else {
            SourceType::Directory
        };

        let data_source = DataSource {
            path,
            source_type,
            last_indexed: Utc::now(),
        };

        self.indexed_sources.push(data_source);
        Ok(())
    }

    pub fn remove_source(&mut self, path: &PathBuf) -> Result<(), FileSystemError> {
        self.indexed_sources.retain(|source| &source.path != path);
        
        // Remove files from index that belong to this source
        self.file_index.retain(|file_path, _| {
            !file_path.starts_with(path)
        });
        
        Ok(())
    }

    pub fn list_sources(&self) -> &[DataSource] {
        &self.indexed_sources
    }

    pub fn index_sources(&mut self) -> Result<(), FileSystemError> {
        // TODO: Implement file indexing logic
        // - Walk through all data sources
        // - Apply include/exclude patterns
        // - Determine file types
        // - Check file sizes for LLM context suitability
        Ok(())
    }

    pub fn search_files(&self, keywords: &[String]) -> Result<Vec<SearchResult>, FileSystemError> {
        // TODO: Implement keyword search across indexed files
        let _keywords = keywords; // Suppress unused warning
        Ok(Vec::new())
    }

    pub fn read_file_content(&self, path: &PathBuf) -> Result<String, FileSystemError> {
        std::fs::read_to_string(path).map_err(|e| {
            FileSystemError::FileAccess(format!("Failed to read file {:?}: {}", path, e))
        })
    }

    pub fn set_include_patterns(&mut self, patterns: Vec<String>) -> Result<(), FileSystemError> {
        let mut compiled_patterns = Vec::new();
        for pattern in patterns {
            let regex = Regex::new(&pattern).map_err(|e| {
                FileSystemError::Indexing(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;
            compiled_patterns.push(regex);
        }
        self.include_patterns = compiled_patterns;
        Ok(())
    }

    pub fn set_exclude_patterns(&mut self, patterns: Vec<String>) -> Result<(), FileSystemError> {
        let mut compiled_patterns = Vec::new();
        for pattern in patterns {
            let regex = Regex::new(&pattern).map_err(|e| {
                FileSystemError::Indexing(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;
            compiled_patterns.push(regex);
        }
        self.exclude_patterns = compiled_patterns;
        Ok(())
    }

    pub fn get_indexed_files(&self) -> Vec<&FileInfo> {
        self.file_index.values().collect()
    }
}