//! # CLI Bridge
//! 
//! A bridge layer connecting Python CLI tools to the Rust agent runtime.
//! Provides typed APIs for spawning subprocesses and handling JSON I/O.

pub mod codexify;
pub mod ritual_engine;
pub mod manifest;
pub mod subprocess;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Process timeout after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("Process exited with non-zero status: {0}")]
    ProcessFailed(i32),
    
    #[error("Invalid output format: {0}")]
    InvalidOutput(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

pub type BridgeResult<T> = Result<T, BridgeError>;

/// Common response wrapper for all CLI tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub tool_name: String,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

impl<T> CapabilityResult<T> {
    pub fn success(data: T, tool_name: &str, duration: Duration, version: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                tool_name: tool_name.to_string(),
                duration_ms: duration.as_millis() as u64,
                timestamp: chrono::Utc::now(),
                version: version.to_string(),
            },
        }
    }

    pub fn error(error: String, tool_name: &str, duration: Duration, version: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: ResponseMetadata {
                tool_name: tool_name.to_string(),
                duration_ms: duration.as_millis() as u64,
                timestamp: chrono::Utc::now(),
                version: version.to_string(),
            },
        }
    }
}

/// Common configuration for subprocess execution
#[derive(Debug, Clone)]
pub struct SubprocessConfig {
    pub timeout: Duration,
    pub working_dir: Option<String>,
    pub env_vars: std::collections::HashMap<String, String>,
    pub log_io: bool,
}

impl Default for SubprocessConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
            log_io: true,
        }
    }
}

/// Re-export commonly used items
pub use subprocess::{spawn_subprocess, SubprocessConfig};
pub use manifest::{Manifest, load_manifest};
