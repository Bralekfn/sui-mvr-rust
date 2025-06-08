use std::fmt;

/// Error types for MVR operations
#[derive(Debug, thiserror::Error)]
pub enum MvrError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// Failed to parse JSON response
    #[error("Failed to parse JSON response: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Package not found in MVR
    #[error("Package '{0}' not found in MVR")]
    PackageNotFound(String),
    
    /// Type not found in MVR
    #[error("Type '{0}' not found in MVR")]
    TypeNotFound(String),
    
    /// Cache operation failed
    #[error("Cache error: {0}")]
    CacheError(String),
    
    /// Invalid package name format
    #[error("Invalid package name format: '{0}'. Expected format: @namespace/package")]
    InvalidPackageName(String),
    
    /// Invalid type name format
    #[error("Invalid type name format: '{0}'. Expected format: @namespace/package::module::Type")]
    InvalidTypeName(String),
    
    /// Network timeout
    #[error("Request timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded. Try again in {retry_after_secs} seconds")]
    RateLimitExceeded { retry_after_secs: u64 },
    
    /// Server error
    #[error("Server error: {status_code} - {message}")]
    ServerError { status_code: u16, message: String },
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    /// Concurrent request limit exceeded
    #[error("Too many concurrent requests. Maximum allowed: {max_concurrent}")]
    TooManyConcurrentRequests { max_concurrent: usize },
}

impl MvrError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MvrError::HttpError(_) | 
            MvrError::Timeout { .. } | 
            MvrError::ServerError { status_code, .. } if *status_code >= 500
        )
    }

    /// Check if the error is due to rate limiting
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, MvrError::RateLimitExceeded { .. })
    }

    /// Check if the error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            MvrError::PackageNotFound(_) |
            MvrError::TypeNotFound(_) |
            MvrError::InvalidPackageName(_) |
            MvrError::InvalidTypeName(_) |
            MvrError::ServerError { status_code, .. } if *status_code >= 400 && *status_code < 500
        )
    }

    /// Get retry delay for retryable errors
    pub fn retry_delay(&self) -> Option<std::time::Duration> {
        match self {
            MvrError::RateLimitExceeded { retry_after_secs } => {
                Some(std::time::Duration::from_secs(*retry_after_secs))
            }
            MvrError::HttpError(_) | MvrError::Timeout { .. } => {
                Some(std::time::Duration::from_secs(1))
            }
            MvrError::ServerError { status_code, .. } if *status_code >= 500 => {
                Some(std::time::Duration::from_secs(2))
            }
            _ => None,
        }
    }
}

/// Result type alias for MVR operations
pub type MvrResult<T> = Result<T, MvrError>;

/// Helper function to validate package name format
pub(crate) fn validate_package_name(name: &str) -> MvrResult<()> {
    if !name.starts_with('@') {
        return Err(MvrError::InvalidPackageName(name.to_string()));
    }
    
    let without_at = &name[1..];
    if !without_at.contains('/') {
        return Err(MvrError::InvalidPackageName(name.to_string()));
    }
    
    let parts: Vec<&str> = without_at.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(MvrError::InvalidPackageName(name.to_string()));
    }
    
    Ok(())
}

/// Helper function to validate type name format
pub(crate) fn validate_type_name(name: &str) -> MvrResult<()> {
    if !name.starts_with('@') {
        return Err(MvrError::InvalidTypeName(name.to_string()));
    }
    
    if !name.contains("::") {
        return Err(MvrError::InvalidTypeName(name.to_string()));
    }
    
    // Basic validation - could be more sophisticated
    let parts: Vec<&str> = name.split("::").collect();
    if parts.len() < 3 {
        return Err(MvrError::InvalidTypeName(name.to_string()));
    }
    
    // First part should be @namespace/package
    validate_package_name(parts[0])?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        // Valid names
        assert!(validate_package_name("@suifrens/core").is_ok());
        assert!(validate_package_name("@namespace/package").is_ok());
        
        // Invalid names
        assert!(validate_package_name("suifrens/core").is_err()); // Missing @
        assert!(validate_package_name("@suifrens").is_err()); // Missing /
        assert!(validate_package_name("@/core").is_err()); // Empty namespace
        assert!(validate_package_name("@suifrens/").is_err()); // Empty package
    }

    #[test]
    fn test_validate_type_name() {
        // Valid names
        assert!(validate_type_name("@suifrens/core::module::Type").is_ok());
        assert!(validate_type_name("@ns/pkg::mod::Type<T>").is_ok());
        
        // Invalid names
        assert!(validate_type_name("@suifrens/core").is_err()); // Missing ::
        assert!(validate_type_name("suifrens/core::Type").is_err()); // Missing @
        assert!(validate_type_name("@ns/pkg::Type").is_err()); // Not enough parts
    }

    #[test]
    fn test_error_properties() {
        let error = MvrError::PackageNotFound("test".to_string());
        assert!(error.is_client_error());
        assert!(!error.is_retryable());
        
        let error = MvrError::Timeout { timeout_secs: 30 };
        assert!(error.is_retryable());
        assert!(!error.is_client_error());
        
        let error = MvrError::RateLimitExceeded { retry_after_secs: 60 };
        assert!(error.is_rate_limited());
        assert_eq!(error.retry_delay(), Some(std::time::Duration::from_secs(60)));
    }
}