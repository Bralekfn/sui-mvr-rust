use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;

/// Configuration for the MVR resolver
#[derive(Debug, Clone)]
pub struct MvrConfig {
    /// The MVR API endpoint URL
    pub endpoint_url: String,
    /// Cache time-to-live duration
    pub cache_ttl: Duration,
    /// Static overrides for packages and types
    pub overrides: Option<MvrOverrides>,
    /// HTTP request timeout
    pub timeout: Duration,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
}

impl Default for MvrConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "https://testnet.mvr.mystenlabs.com".to_string(),
            cache_ttl: Duration::from_secs(3600), // 1 hour
            overrides: None,
            timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
        }
    }
}

impl MvrConfig {
    /// Create a new configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            endpoint_url: "https://mainnet.mvr.mystenlabs.com".to_string(),
            ..Default::default()
        }
    }

    /// Create a new configuration for testnet
    pub fn testnet() -> Self {
        Self {
            endpoint_url: "https://testnet.mvr.mystenlabs.com".to_string(),
            ..Default::default()
        }
    }

    /// Set custom endpoint URL
    pub fn with_endpoint(mut self, endpoint_url: String) -> Self {
        self.endpoint_url = endpoint_url;
        self
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set static overrides
    pub fn with_overrides(mut self, overrides: MvrOverrides) -> Self {
        self.overrides = Some(overrides);
        self
    }
}

/// Static overrides for package addresses and types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MvrOverrides {
    /// Map of package names to their addresses
    pub packages: HashMap<String, String>,
    /// Map of type names to their full signatures
    pub types: HashMap<String, String>,
}

impl MvrOverrides {
    /// Create a new empty overrides instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a package override
    pub fn with_package(mut self, name: String, address: String) -> Self {
        self.packages.insert(name, address);
        self
    }

    /// Add a type override
    pub fn with_type(mut self, name: String, type_signature: String) -> Self {
        self.types.insert(name, type_signature);
        self
    }

    /// Load overrides from a JSON file
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save overrides to JSON format
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// MVR API response structure for package resolution
#[derive(Debug, Deserialize)]
pub(crate) struct MvrPackageResponse {
    pub package_id: Option<String>,
    pub address: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

/// MVR API response structure for type resolution
#[derive(Debug, Deserialize)]
pub(crate) struct MvrTypeResponse {
    pub type_signature: Option<String>,
    pub package_id: Option<String>,
    pub module: Option<String>,
    pub name: Option<String>,
}

/// Batch resolution request
#[derive(Debug, Serialize)]
pub(crate) struct BatchResolutionRequest {
    pub packages: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
}

/// Batch resolution response
#[derive(Debug, Deserialize)]
pub(crate) struct BatchResolutionResponse {
    pub packages: Option<HashMap<String, String>>,
    pub types: Option<HashMap<String, String>>,
    pub errors: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mvr_config_defaults() {
        let config = MvrConfig::default();
        assert_eq!(config.cache_ttl, Duration::from_secs(3600));
        assert!(config.endpoint_url.contains("testnet"));
    }

    #[test]
    fn test_mvr_config_mainnet() {
        let config = MvrConfig::mainnet();
        assert!(config.endpoint_url.contains("mainnet"));
    }

    #[test]
    fn test_mvr_config_builder() {
        let config = MvrConfig::testnet()
            .with_cache_ttl(Duration::from_secs(1800))
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(config.cache_ttl, Duration::from_secs(1800));
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_mvr_overrides() {
        let overrides = MvrOverrides::new()
            .with_package("@test/package".to_string(), "0x123".to_string())
            .with_type("@test/Type".to_string(), "0x123::module::Type".to_string());
        
        assert_eq!(overrides.packages.len(), 1);
        assert_eq!(overrides.types.len(), 1);
    }

    #[test]
    fn test_overrides_json_serialization() {
        let overrides = MvrOverrides::new()
            .with_package("@test/package".to_string(), "0x123".to_string());
        
        let json = overrides.to_json().unwrap();
        let deserialized = MvrOverrides::from_json(&json).unwrap();
        
        assert_eq!(overrides.packages, deserialized.packages);
    }
}