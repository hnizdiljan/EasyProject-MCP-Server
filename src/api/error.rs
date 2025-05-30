use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

/// EasyProject API Error Response podle Swagger dokumentace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<ApiErrorResponse> for ApiError {
    fn from(error_response: ApiErrorResponse) -> Self {
        let message = error_response.message
            .or(error_response.error)
            .or_else(|| error_response.errors.as_ref().map(|e| e.join(", ")))
            .unwrap_or_else(|| "Neznámá chyba API".to_string());
        
        ApiError::Api { status: 400, message }
    }
}

pub type ApiResult<T> = Result<T, ApiError>; 