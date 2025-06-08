# API Documentation

This document provides detailed documentation for all public APIs in the `sui-mvr` crate.

## Table of Contents

- [MvrResolver](#mvrresolver)
- [MvrConfig](#mvrconfig)
- [MvrOverrides](#mvroverrides)
- [MvrError](#mvrerror)
- [Utility Functions](#utility-functions)
- [Traits](#traits)

## MvrResolver

The main resolver for MVR operations.

### Constructors

#### `new(config: MvrConfig) -> Self`

Creates a new resolver with custom configuration.

```rust
use sui_mvr::*;
use tokio::time::Duration;

let config = MvrConfig::mainnet()
    .with_cache_ttl(Duration::from_secs(1800));
let resolver = MvrResolver::new(config);
```

#### `mainnet() -> Self`

Creates a resolver configured for Sui mainnet.

```rust
let resolver = MvrResolver::mainnet();
```

#### `testnet() -> Self`

Creates a resolver configured for Sui testnet.

```rust
let resolver = MvrResolver::testnet();
```

#### `with_overrides(self, overrides: MvrOverrides) -> Self`

Adds static overrides to the resolver.

```rust
let overrides = MvrOverrides::new()
    .with_package("@test/pkg".to_string(), "0x123".to_string());
let resolver = MvrResolver::testnet().with_overrides(overrides);
```

### Resolution Methods

#### `resolve_package(&self, package_name: &str) -> MvrResult<String>`

Resolves a package name to its on-chain address.

**Arguments:**
- `package_name` - Package name in format `@namespace/package`

**Returns:**
- `Ok(String)` - The resolved package address
- `Err(MvrError)` - Resolution error

**Example:**
```rust
let address = resolver.resolve_package("@suifrens/core").await?;
```

**Validation:**
- Package name must start with `@`
- Must contain exactly one `/` separator
- Namespace and package parts must be non-empty

#### `resolve_type(&self, type_name: &str) -> MvrResult<String>`

Resolves a type name to its full type signature.

**Arguments:**
- `type_name` - Type name in format `@namespace/package::module::Type`

**Returns:**
- `Ok(String)` - The resolved type signature
- `Err(MvrError)` - Resolution error

**Example:**
```rust
let type_sig = resolver.resolve_type("@suifrens/core::suifren::SuiFren").await?;
```

#### `resolve_packages(&self, package_names: &[&str]) -> MvrResult<HashMap<String, String>>`

Batch resolves multiple package names.

**Arguments:**
- `package_names` - Array of package names

**Returns:**
- `Ok(HashMap<String, String>)` - Map of package names to addresses
- `Err(MvrError)` - Resolution error

**Example:**
```rust
let packages = vec!["@suifrens/core", "@suifrens/accessories"];
let results = resolver.resolve_packages(&packages).await?;
```

**Performance:** More efficient than individual calls for multiple packages.

#### `resolve_types(&self, type_names: &[&str]) -> MvrResult<HashMap<String, String>>`

Batch resolves multiple type names.

**Arguments:**
- `type_names` - Array of type names

**Returns:**
- `Ok(HashMap<String, String>)` - Map of type names to signatures
- `Err(MvrError)` - Resolution error

**Example:**
```rust
let types = vec!["@suifrens/core::suifren::SuiFren", "@suifrens/core::bullshark::Bullshark"];
let results = resolver.resolve_types(&types).await?;
```

### Cache Management

#### `clear_cache(&self) -> MvrResult<()>`

Clears all cached entries.

```rust
resolver.clear_cache()?;
```

#### `cache_stats(&self) -> MvrResult<CacheStats>`

Returns cache statistics.

```rust
let stats = resolver.cache_stats()?;
println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
```

#### `cleanup_expired_cache(&self) -> MvrResult<usize>`

Removes expired entries from cache.

**Returns:** Number of entries removed.

```rust
let removed = resolver.cleanup_expired_cache()?;
```

### Configuration Access

#### `config(&self) -> &MvrConfig`

Returns the resolver's configuration.

```rust
let endpoint = resolver.config().endpoint_url;
```

## MvrConfig

Configuration for MVR resolvers.

### Fields

```rust
pub struct MvrConfig {
    pub endpoint_url: String,               // MVR API endpoint
    pub cache_ttl: Duration,                // Cache time-to-live
    pub overrides: Option<MvrOverrides>,    // Static overrides
    pub timeout: Duration,                  // HTTP request timeout
    pub max_concurrent_requests: usize,     // Concurrency limit
}
```

### Constructors

#### `default() -> Self`

Creates default configuration (testnet).

#### `mainnet() -> Self`

Creates mainnet configuration.

#### `testnet() -> Self`

Creates testnet configuration.

### Builder Methods

#### `with_endpoint(mut self, endpoint_url: String) -> Self`

Sets custom endpoint URL.

#### `with_cache_ttl(mut self, ttl: Duration) -> Self`

Sets cache TTL.

#### `with_timeout(mut self, timeout: Duration) -> Self`

Sets HTTP timeout.

#### `with_overrides(mut self, overrides: MvrOverrides) -> Self`

Sets static overrides.

**Example:**
```rust
let config = MvrConfig::mainnet()
    .with_cache_ttl(Duration::from_secs(1800))
    .with_timeout(Duration::from_secs(30));
```

## MvrOverrides

Static overrides for packages and types.

### Fields

```rust
pub struct MvrOverrides {
    pub packages: HashMap<String, String>,  // Package overrides
    pub types: HashMap<String, String>,     // Type overrides
}
```

### Constructors

#### `new() -> Self`

Creates empty overrides.

#### `default() -> Self`

Same as `new()`.

### Builder Methods

#### `with_package(mut self, name: String, address: String) -> Self`

Adds package override.

#### `with_type(mut self, name: String, type_signature: String) -> Self`

Adds type override.

### Serialization

#### `from_json(json: &str) -> Result<Self, serde_json::Error>`

Loads overrides from JSON string.

#### `to_json(&self) -> Result<String, serde_json::Error>`

Serializes overrides to JSON string.

**Example:**
```rust
let overrides = MvrOverrides::new()
    .with_package("@test/pkg".to_string(), "0x123".to_string());

let json = overrides.to_json()?;
let loaded = MvrOverrides::from_json(&json)?;
```

## MvrError

Comprehensive error type for MVR operations.

### Variants

```rust
pub enum MvrError {
    HttpError(reqwest::Error),                           // Network errors
    JsonError(serde_json::Error),                        // JSON parsing
    PackageNotFound(String),                             // 404 for packages
    TypeNotFound(String),                                // 404 for types
    CacheError(String),                                  // Cache operations
    InvalidPackageName(String),                          // Validation errors
    InvalidTypeName(String),                             // Format errors
    Timeout { timeout_secs: u64 },                       // Request timeouts
    RateLimitExceeded { retry_after_secs: u64 },        // 429 responses
    ServerError { status_code: u16, message: String },  // 5xx errors
    ConfigError(String),                                 // Configuration
    TooManyConcurrentRequests { max_concurrent: usize }, // Concurrency
}
```

### Methods

#### `is_retryable(&self) -> bool`

Returns `true` if the error suggests a retry might succeed.

```rust
if error.is_retryable() {
    // Implement retry logic
}
```

#### `is_rate_limited(&self) -> bool`

Returns `true` for rate limiting errors.

#### `is_client_error(&self) -> bool`

Returns `true` for 4xx HTTP errors.

#### `retry_delay(&self) -> Option<Duration>`

Returns suggested retry delay.

```rust
if let Some(delay) = error.retry_delay() {
    tokio::time::sleep(delay).await;
}
```

## CacheStats

Cache performance statistics.

### Fields

```rust
pub struct CacheStats {
    pub total_entries: usize,    // Total cache entries
    pub expired_entries: usize,  // Expired entries
    pub valid_entries: usize,    // Valid entries
    pub total_hits: u64,         // Total cache hits
    pub max_size: usize,         // Maximum cache size
}
```

### Methods

#### `utilization(&self) -> f64`

Returns cache utilization (0.0 to 1.0).

#### `hit_rate(&self) -> f64`

Returns cache hit rate (0.0 to 1.0).

## Utility Functions

### `resolve_mvr_target(resolver: &MvrResolver, target: &str) -> MvrResult<String>`

Resolves MVR target format for move calls.

**Arguments:**
- `resolver` - MVR resolver instance
- `target` - Target in format `@package/module::function`

**Returns:**
- Resolved target with package address

**Example:**
```rust
let target = resolve_mvr_target(&resolver, "@suifrens/core/suifren::mint").await?;
// Returns: "0x123456::suifren::mint"
```

## Traits

### MvrTransactionExt

Trait for extending transaction builders with MVR support.

#### `move_call_mvr(...) -> MvrResult<()>`

Creates move call using MVR names.

**Note:** This is a conceptual trait for future integration with Sui SDK.

## Type Aliases

### `MvrResult<T>`

Convenience alias for `Result<T, MvrError>`.

```rust
pub type MvrResult<T> = Result<T, MvrError>;
```

## Constants

### `VERSION`

Crate version string.

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

## Feature Flags

### `tracing`

Enables detailed logging and tracing.

```toml
[dependencies]
sui-mvr = { version = "0.1", features = ["tracing"] }
```

### `static-resolution`

Enables static resolution capabilities.

### `metrics`

Enables additional metrics and monitoring.

### `wasm`

Enables WebAssembly support.

## Error Handling Best Practices

1. **Check error type** before deciding on action:
   ```rust
   match error {
       MvrError::PackageNotFound(_) => /* use fallback */,
       MvrError::RateLimitExceeded { .. } => /* wait and retry */,
       e if e.is_retryable() => /* retry with backoff */,
       _ => /* permanent failure */,
   }
   ```

2. **Use retry logic** for transient errors:
   ```rust
   for attempt in 1..=3 {
       match resolver.resolve_package(name).await {
           Ok(address) => return Ok(address),
           Err(e) if e.is_retryable() => {
               if let Some(delay) = e.retry_delay() {
                   tokio::time::sleep(delay).await;
               }
           }
           Err(e) => return Err(e),
       }
   }
   ```

3. **Handle rate limiting** gracefully:
   ```rust
   if let Err(MvrError::RateLimitExceeded { retry_after_secs }) = result {
       tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
       // Retry the operation
   }
   ```

## Performance Optimization

1. **Use batch operations** for multiple resolutions
2. **Monitor cache hit rates** and adjust TTL accordingly
3. **Use static overrides** for known packages in production
4. **Configure appropriate timeouts** for your network conditions
5. **Implement exponential backoff** for retries