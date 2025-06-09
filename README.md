# sui-mvr 🦀

[![Crates.io](https://img.shields.io/crates/v/sui-mvr.svg)](https://crates.io/crates/sui-mvr)
[![Documentation](https://docs.rs/sui-mvr/badge.svg)](https://docs.rs/sui-mvr)
[![License](https://img.shields.io/github/license/Bralekfn/sui-mvr-rust)](https://github.com/Bralekfn/sui-mvr-rust/blob/main/LICENSE)
[![Build Status](https://github.com/Bralekfn/sui-mvr-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Bralekfn/sui-mvr-rust/actions)
[![Coverage](https://codecov.io/gh/Bralekfn/sui-mvr-rust/branch/main/graph/badge.svg)](https://codecov.io/gh/Bralekfn/sui-mvr-rust)

**The first Rust implementation of the Move Registry (MVR) plugin for the Sui blockchain.**

Transform cryptic package addresses into human-readable names in your Rust applications. No more copy-pasting `0x80d7de9c4a56194087e0ba0bf59492aa8e6a5ee881606226930827085ddf2332` - just use `@suifrens/core`!

## 🚧 Current Status

**Early Release (v0.1.0)** - This is the first Rust implementation of MVR for Sui. Core functionality is stable and tested.

### ✅ **What's Ready:**
- ✓ Package and type resolution with caching
- ✓ Batch operations for performance  
- ✓ Static overrides for development
- ✓ Comprehensive error handling
- ✓ Three working examples

### 🔄 **Ongoing Improvements:**
- Expanding test coverage (currently ~50%)
- Additional edge case handling  
- Performance optimizations
- WASM and static resolution features

## ✨ Features

- 🔍 **Package Resolution**: `@suifrens/core` → `0x123...`
- 🏷️ **Type Resolution**: Full type signatures with generics
- ⚡ **Smart Caching**: TTL + LRU with configurable size limits
- 🌐 **Network Support**: Mainnet, testnet, and custom endpoints
- 🔧 **Static Overrides**: Perfect for local development and CI
- 📦 **Batch Operations**: Resolve multiple packages/types efficiently
- 🚨 **Comprehensive Errors**: Detailed error types with retry logic
- 🔄 **Async/Await**: Non-blocking operations with tokio
- 📊 **Performance Metrics**: Cache statistics and monitoring

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
sui-mvr = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

Basic usage:

```rust
use sui_mvr::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a resolver for mainnet
    let resolver = MvrResolver::mainnet();
    
    // Resolve a package name to its address
    let address = resolver.resolve_package("@suifrens/core").await?;
    println!("SuiFrens core package: {}", address);
    
    // Resolve a type name
    let type_sig = resolver.resolve_type("@suifrens/core::suifren::SuiFren").await?;
    println!("SuiFren type: {}", type_sig);
    
    Ok(())
}
```

## 🛠️ Advanced Usage

### Configuration

```rust
use sui_mvr::*;
use tokio::time::Duration;

// Custom configuration
let config = MvrConfig::mainnet()
    .with_cache_ttl(Duration::from_secs(1800))  // 30 min cache
    .with_timeout(Duration::from_secs(30));     // 30 sec timeout

let resolver = MvrResolver::new(config);
```

### Static Overrides for Development

```rust
use sui_mvr::*;
use std::collections::HashMap;

// Perfect for local development and CI
let overrides = MvrOverrides::new()
    .with_package("@myapp/core".to_string(), "0x123456".to_string())
    .with_type("@myapp/core::Token".to_string(), "0x123456::token::Token".to_string());

let resolver = MvrResolver::testnet().with_overrides(overrides);

// This uses the override instead of making an API call
let address = resolver.resolve_package("@myapp/core").await?;
```

### Batch Operations for Performance

```rust
// Resolve multiple packages at once
let package_names = vec!["@suifrens/core", "@suifrens/accessories"];
let results = resolver.resolve_packages(&package_names).await?;

for (name, address) in results {
    println!("{} -> {}", name, address);
}

// Batch type resolution
let type_names = vec![
    "@suifrens/core::suifren::SuiFren",
    "@suifrens/core::bullshark::Bullshark"
];
let type_results = resolver.resolve_types(&type_names).await?;
```

### Error Handling with Fallbacks

```rust
use sui_mvr::MvrError;

match resolver.resolve_package("@myapp/core").await {
    Ok(address) => println!("✓ Resolved: {}", address),
    Err(MvrError::PackageNotFound(name)) => {
        println!("Package {} not found, using fallback", name);
        // Use fallback address
    }
    Err(MvrError::RateLimitExceeded { retry_after_secs }) => {
        println!("Rate limited, retry in {} seconds", retry_after_secs);
        tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
        // Retry logic
    }
    Err(e) if e.is_retryable() => {
        println!("Retryable error: {}", e);
        // Implement retry with exponential backoff
    }
    Err(e) => println!("Permanent error: {}", e),
}
```

### Cache Management

```rust
// Get cache statistics
let stats = resolver.cache_stats()?;
println!("Cache utilization: {:.1}%", stats.utilization() * 100.0);
println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);

// Cleanup expired entries
let removed = resolver.cleanup_expired_cache()?;
println!("Cleaned up {} expired entries", removed);

// Clear entire cache
resolver.clear_cache()?;
```

## 📊 Performance Comparison

| Operation | Individual Requests | Batch Request | Improvement |
|-----------|-------------------|---------------|-------------|
| 4 packages | ~400ms | ~120ms | **3.3x faster** |
| 8 packages | ~800ms | ~150ms | **5.3x faster** |
| Cache hits | ~0.1ms | ~0.1ms | Instant ⚡ |

## 🆚 Comparison with TypeScript SDK

| Feature | TypeScript SDK | sui-mvr (Rust) |
|---------|----------------|-----------------|
| Package Resolution | ✅ | ✅ |
| Type Resolution | ✅ | ✅ |
| Basic Caching | ✅ | ✅ |
| Static Overrides | ✅ | ✅ |
| Batch Resolution | ❌ | ✅ |
| Cache Statistics | ❌ | ✅ |
| Configurable TTL | ❌ | ✅ |
| Error Fallbacks | ❌ | ✅ |
| Retry Logic | ❌ | ✅ |
| Rate Limiting | ❌ | ✅ |
| Performance Metrics | ❌ | ✅ |

## 🔧 Configuration Options

### Networks

```rust
// Pre-configured endpoints
let mainnet = MvrResolver::mainnet();  // https://mainnet.mvr.mystenlabs.com
let testnet = MvrResolver::testnet();  // https://testnet.mvr.mystenlabs.com

// Custom endpoint
let custom = MvrConfig::default()
    .with_endpoint("https://my-mvr-endpoint.com".to_string());
```

### Cache Settings

```rust
let config = MvrConfig::mainnet()
    .with_cache_ttl(Duration::from_secs(3600))  // 1 hour
    .with_timeout(Duration::from_secs(30));     // 30 seconds
```

### Overrides from JSON

```rust
// Save overrides
let json = overrides.to_json()?;
std::fs::write("overrides.json", json)?;

// Load overrides
let json = std::fs::read_to_string("overrides.json")?;
let overrides = MvrOverrides::from_json(&json)?;
let resolver = MvrResolver::testnet().with_overrides(overrides);
```

## 📚 Examples

Check out the [examples directory](./examples/) for complete working examples:

- **[basic_usage.rs](./examples/basic_usage.rs)** - Simple resolver usage
- **[with_overrides.rs](./examples/with_overrides.rs)** - Static overrides for local development
- **[batch_operations.rs](./examples/batch_operations.rs)** - Batch resolution and performance testing

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example with_overrides
cargo run --example batch_operations
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --all-features

# Test examples
cargo run --example basic_usage
```

## 🚀 Performance Tips

1. **Use batch operations** when resolving multiple items
2. **Enable caching** for repeated resolutions
3. **Use static overrides** for known packages in production
4. **Configure appropriate timeouts** for your network conditions
5. **Monitor cache hit rates** to optimize cache size

## 🛡️ Error Handling

The crate provides comprehensive error types:

```rust
pub enum MvrError {
    PackageNotFound(String),           // 404 errors
    TypeNotFound(String),              // Type resolution failures
    RateLimitExceeded { retry_after_secs: u64 }, // 429 errors
    Timeout { timeout_secs: u64 },     // Network timeouts
    ServerError { status_code: u16, message: String }, // 5xx errors
    InvalidPackageName(String),        // Validation errors
    InvalidTypeName(String),           // Format errors
    HttpError(reqwest::Error),         // Network errors
    JsonError(serde_json::Error),      // Parsing errors
    CacheError(String),                // Cache operations
    ConfigError(String),               // Configuration issues
}
```

Each error type provides:
- **Retry logic**: `error.is_retryable()`
- **Rate limiting**: `error.is_rate_limited()`
- **Retry delays**: `error.retry_delay()`

## 🗺️ Roadmap

- [ ] **Static Resolution** - Compile-time package resolution (like @mysten/mvr-static)
- [ ] **Official Sui SDK Integration** - Direct integration with sui-sdk
- [ ] **WebAssembly Support** - Run in browsers and edge environments
- [ ] **Metrics & Observability** - Prometheus metrics and tracing
- [ ] **CLI Tool** - Command-line MVR operations
- [ ] **Custom Cache Backends** - Redis, file-based caching

## 🤝 Contributing

We love contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for details on:

- 🐛 Reporting bugs
- ✨ Requesting features  
- 🔧 Setting up development environment
- 📝 Code style and testing guidelines
- 🚀 Release process

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Inspired by** the [TypeScript MVR plugin](https://docs.suins.io/move-registry/tooling/typescript-sdk) from Mysten Labs
- **Built for** the [Sui blockchain](https://sui.io/) ecosystem  
- **Thanks to** the Move Registry team for creating MVR

## 📞 Support & Community

- 📖 [Documentation](https://docs.rs/sui-mvr)
- 🐛 [Issue Tracker](https://github.com/Bralekfn/sui-mvr-rust/issues)
- 💬 [Discussions](https://github.com/Bralekfn/sui-mvr-rust/discussions)
- 🚀 [Sui Developer Discord](https://discord.gg/sui)

---

**Made with ❤️ for the Sui community** • **Star ⭐ this repo if it helped you!**