use crate::cache::{MvrCache, CacheStats};
use crate::error::{MvrError, MvrResult, validate_package_name, validate_type_name};
use crate::types::{MvrConfig, MvrOverrides, BatchResolutionRequest, BatchResolutionResponse};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::Duration;

/// Main MVR resolver for Rust Sui SDK
pub struct MvrResolver {
    config: MvrConfig,
    client: Client,
    cache: MvrCache,
    semaphore: Arc<Semaphore>,
}

impl MvrResolver {
    /// Create a new MVR resolver with the given configuration
    pub fn new(config: MvrConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(format!("sui-mvr-rust/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        let cache = MvrCache::new(config.cache_ttl, 1000); // Default max 1000 entries
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        Self {
            config,
            client,
            cache,
            semaphore,
        }
    }

    /// Create a resolver for mainnet
    pub fn mainnet() -> Self {
        Self::new(MvrConfig::mainnet())
    }

    /// Create a resolver for testnet
    pub fn testnet() -> Self {
        Self::new(MvrConfig::testnet())
    }

    /// Create a resolver with custom overrides
    pub fn with_overrides(mut self, overrides: MvrOverrides) -> Self {
        self.config.overrides = Some(overrides);
        self
    }

    /// Resolve a package name to its address
    pub async fn resolve_package(&self, package_name: &str) -> MvrResult<String> {
        validate_package_name(package_name)?;

        // Check static overrides first
        if let Some(overrides) = &self.config.overrides {
            if let Some(address) = overrides.packages.get(package_name) {
                return Ok(address.clone());
            }
        }

        // Check cache
        let cache_key = MvrCache::package_key(package_name);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        // Fetch from API
        let address = self.fetch_package_from_api(package_name).await?;
        
        // Store in cache
        self.cache.insert(cache_key, address.clone())?;
        
        Ok(address)
    }

    /// Resolve a type name to its full type signature
    pub async fn resolve_type(&self, type_name: &str) -> MvrResult<String> {
        validate_type_name(type_name)?;

        // Check static overrides first
        if let Some(overrides) = &self.config.overrides {
            if let Some(type_sig) = overrides.types.get(type_name) {
                return Ok(type_sig.clone());
            }
        }

        // Check cache
        let cache_key = MvrCache::type_key(type_name);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        // Fetch from API
        let type_sig = self.fetch_type_from_api(type_name).await?;
        
        // Store in cache
        self.cache.insert(cache_key, type_sig.clone())?;
        
        Ok(type_sig)
    }

    /// Batch resolve multiple packages
    pub async fn resolve_packages(&self, package_names: &[&str]) -> MvrResult<HashMap<String, String>> {
        let mut results = HashMap::new();
        let mut to_fetch = Vec::new();

        // Check overrides and cache first
        for &name in package_names {
            validate_package_name(name)?;

            // Check overrides
            if let Some(overrides) = &self.config.overrides {
                if let Some(address) = overrides.packages.get(name) {
                    results.insert(name.to_string(), address.clone());
                    continue;
                }
            }

            // Check cache
            let cache_key = MvrCache::package_key(name);
            if let Some(cached) = self.cache.get(&cache_key) {
                results.insert(name.to_string(), cached);
                continue;
            }

            to_fetch.push(name);
        }

        // Fetch remaining packages from API
        if !to_fetch.is_empty() {
            let fetched = self.batch_fetch_packages(&to_fetch).await?;
            
            // Store in cache and add to results
            for (name, address) in fetched {
                let cache_key = MvrCache::package_key(&name);
                self.cache.insert(cache_key, address.clone())?;
                results.insert(name, address);
            }
        }

        Ok(results)
    }

    /// Batch resolve multiple types
    pub async fn resolve_types(&self, type_names: &[&str]) -> MvrResult<HashMap<String, String>> {
        let mut results = HashMap::new();
        let mut to_fetch = Vec::new();

        // Check overrides and cache first
        for &name in type_names {
            validate_type_name(name)?;

            // Check overrides
            if let Some(overrides) = &self.config.overrides {
                if let Some(type_sig) = overrides.types.get(name) {
                    results.insert(name.to_string(), type_sig.clone());
                    continue;
                }
            }

            // Check cache
            let cache_key = MvrCache::type_key(name);
            if let Some(cached) = self.cache.get(&cache_key) {
                results.insert(name.to_string(), cached);
                continue;
            }

            to_fetch.push(name);
        }

        // Fetch remaining types from API
        if !to_fetch.is_empty() {
            let fetched = self.batch_fetch_types(&to_fetch).await?;
            
            // Store in cache and add to results
            for (name, type_sig) in fetched {
                let cache_key = MvrCache::type_key(&name);
                self.cache.insert(cache_key, type_sig.clone())?;
                results.insert(name, type_sig);
            }
        }

        Ok(results)
    }

    /// Clear the cache
    pub fn clear_cache(&self) -> MvrResult<()> {
        self.cache.clear()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> MvrResult<CacheStats> {
        self.cache.stats()
    }

    /// Cleanup expired cache entries
    pub fn cleanup_expired_cache(&self) -> MvrResult<usize> {
        self.cache.cleanup_expired()
    }

    /// Get resolver configuration
    pub fn config(&self) -> &MvrConfig {
        &self.config
    }

    // Private helper methods

    async fn fetch_package_from_api(&self, package_name: &str) -> MvrResult<String> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| MvrError::TooManyConcurrentRequests { 
                max_concurrent: self.config.max_concurrent_requests 
            })?;

        let url = format!("{}/resolve/package/{}", self.config.endpoint_url, package_name);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let text = response.text().await?;
                // Simple extraction - in real implementation, parse proper JSON response
                self.extract_package_address(&text, package_name)
            }
            404 => Err(MvrError::PackageNotFound(package_name.to_string())),
            429 => {
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60);
                Err(MvrError::RateLimitExceeded { retry_after_secs: retry_after })
            }
            status => {
                let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(MvrError::ServerError { 
                    status_code: status, 
                    message 
                })
            }
        }
    }

    async fn fetch_type_from_api(&self, type_name: &str) -> MvrResult<String> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| MvrError::TooManyConcurrentRequests { 
                max_concurrent: self.config.max_concurrent_requests 
            })?;

        let url = format!("{}/resolve/type/{}", self.config.endpoint_url, type_name);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let text = response.text().await?;
                self.extract_type_signature(&text, type_name)
            }
            404 => Err(MvrError::TypeNotFound(type_name.to_string())),
            429 => {
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60);
                Err(MvrError::RateLimitExceeded { retry_after_secs: retry_after })
            }
            status => {
                let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(MvrError::ServerError { 
                    status_code: status, 
                    message 
                })
            }
        }
    }

    async fn batch_fetch_packages(&self, package_names: &[&str]) -> MvrResult<HashMap<String, String>> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| MvrError::TooManyConcurrentRequests { 
                max_concurrent: self.config.max_concurrent_requests 
            })?;

        let request = BatchResolutionRequest {
            packages: Some(package_names.iter().map(|s| s.to_string()).collect()),
            types: None,
        };

        let url = format!("{}/resolve/batch", self.config.endpoint_url);
        
        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let batch_response: BatchResolutionResponse = response.json().await?;
                Ok(batch_response.packages.unwrap_or_default())
            }
            status => {
                let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(MvrError::ServerError { 
                    status_code: status, 
                    message 
                })
            }
        }
    }

    async fn batch_fetch_types(&self, type_names: &[&str]) -> MvrResult<HashMap<String, String>> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| MvrError::TooManyConcurrentRequests { 
                max_concurrent: self.config.max_concurrent_requests 
            })?;

        let request = BatchResolutionRequest {
            packages: None,
            types: Some(type_names.iter().map(|s| s.to_string()).collect()),
        };

        let url = format!("{}/resolve/batch", self.config.endpoint_url);
        
        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let batch_response: BatchResolutionResponse = response.json().await?;
                Ok(batch_response.types.unwrap_or_default())
            }
            status => {
                let message = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(MvrError::ServerError { 
                    status_code: status, 
                    message 
                })
            }
        }
    }

    fn extract_package_address(&self, response_text: &str, _package_name: &str) -> MvrResult<String> {
        // This is a simplified extraction - in reality you'd parse the JSON response properly
        // For now, assuming the response contains the address directly
        if response_text.starts_with("0x") && response_text.len() >= 42 {
            Ok(response_text.trim().to_string())
        } else {
            // Try to parse as JSON and extract address field
            let json: serde_json::Value = serde_json::from_str(response_text)?;
            json.get("address")
                .or_else(|| json.get("package_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| MvrError::JsonError(serde_json::Error::custom("Address not found in response")))
        }
    }

    fn extract_type_signature(&self, response_text: &str, _type_name: &str) -> MvrResult<String> {
        // This is a simplified extraction - in reality you'd parse the JSON response properly
        let json: serde_json::Value = serde_json::from_str(response_text)?;
        json.get("type_signature")
            .or_else(|| json.get("signature"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| MvrError::JsonError(serde_json::Error::custom("Type signature not found in response")))
    }
}

/// Helper trait to extend transaction builders with MVR support
pub trait MvrTransactionExt {
    /// Create a move call using MVR package names
    fn move_call_mvr<'a>(
        &'a mut self,
        resolver: &'a MvrResolver,
        target: &'a str,
        arguments: Vec<serde_json::Value>,
        type_arguments: Vec<&'a str>,
    ) -> impl std::future::Future<Output = MvrResult<()>> + 'a;
}

// Note: This would be implemented for the actual Sui SDK transaction builder
// For now, this is a conceptual implementation
impl MvrTransactionExt for serde_json::Value {
    async fn move_call_mvr(
        &mut self,
        resolver: &MvrResolver,
        target: &str,
        _arguments: Vec<serde_json::Value>,
        type_arguments: Vec<&str>,
    ) -> MvrResult<()> {
        // Parse target to extract package name if it starts with @
        let resolved_target = if target.starts_with('@') {
            resolve_mvr_target(resolver, target).await?
        } else {
            target.to_string()
        };

        // Resolve type arguments
        let mut resolved_type_args = Vec::new();
        for type_arg in type_arguments {
            if type_arg.starts_with('@') {
                let resolved = resolver.resolve_type(type_arg).await?;
                resolved_type_args.push(resolved);
            } else {
                resolved_type_args.push(type_arg.to_string());
            }
        }

        // Store resolved values (in a real implementation, this would call the actual Sui SDK)
        *self = serde_json::json!({
            "target": resolved_target,
            "type_arguments": resolved_type_args
        });

        Ok(())
    }
}

/// Helper function to resolve MVR target format
pub async fn resolve_mvr_target(resolver: &MvrResolver, target: &str) -> MvrResult<String> {
    if !target.starts_with('@') {
        return Ok(target.to_string());
    }

    // Parse MVR target format: @package/module::function
    let parts: Vec<&str> = target.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(MvrError::InvalidPackageName(target.to_string()));
    }

    let package_part = parts[0];
    let module_function = parts[1];

    let package_address = resolver.resolve_package(package_part).await?;
    Ok(format!("{}::{}", package_address, module_function))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[test]
    fn test_resolver_creation() {
        let resolver = MvrResolver::mainnet();
        assert!(resolver.config().endpoint_url.contains("mainnet"));

        let resolver = MvrResolver::testnet();
        assert!(resolver.config().endpoint_url.contains("testnet"));
    }

    #[test]
    fn test_resolver_with_overrides() {
        let overrides = MvrOverrides::new()
            .with_package("@test/package".to_string(), "0x123".to_string());
        
        let resolver = MvrResolver::testnet().with_overrides(overrides);
        assert!(resolver.config().overrides.is_some());
    }

    #[tokio::test]
    async fn test_resolve_mvr_target() {
        let resolver = MvrResolver::testnet();
        
        // Test non-MVR target (should pass through unchanged)
        let normal_target = "0x123::module::function";
        let result = resolve_mvr_target(&resolver, normal_target).await.unwrap();
        assert_eq!(result, normal_target);
        
        // Test invalid MVR target format
        let invalid_target = "@invalid-format";
        assert!(resolve_mvr_target(&resolver, invalid_target).await.is_err());
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let resolver = MvrResolver::testnet();
        
        // Test cache stats on empty cache
        let stats = resolver.cache_stats().unwrap();
        assert_eq!(stats.total_entries, 0);
        
        // Test cache clearing
        resolver.clear_cache().unwrap();
    }

    #[tokio::test]
    async fn test_batch_resolution_empty() {
        let resolver = MvrResolver::testnet();
        
        // Test empty batch resolution
        let results = resolver.resolve_packages(&[]).await.unwrap();
        assert!(results.is_empty());
        
        let results = resolver.resolve_types(&[]).await.unwrap();
        assert!(results.is_empty());
    }
}