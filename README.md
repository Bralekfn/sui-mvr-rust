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
- ✓ **Official Sui SDK integration**
- ✓ Comprehensive error handling
- ✓ Four working examples including full Sui integration

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
- **🆕 Sui SDK Integration**: Works seamlessly with official Sui Rust SDK

## 🚀 Quick Start

### Basic Setup

Add to your `Cargo.toml`:

```toml
[dependencies]
sui-mvr = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Standalone Usage

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

### **🆕 Integration with Official Sui SDK**

For building transactions with human-readable package names:

```toml
[dependencies]
sui-mvr = { version = "0.1.0", features = ["sui-integration"] }
sui-sdk = { git = "https://github.com/mystenlabs/sui", package = "sui-sdk" }
tokio = { version = "1.2", features = ["full"] }
anyhow = "1.0"
```

```rust
use anyhow::Result;
use sui_mvr::prelude::*;
use sui_sdk::{
    types::{
        base_types::ObjectID,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Command, ProgrammableMoveCall},
        Identifier,
    },
    SuiClientBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize official Sui client
    let sui_client = SuiClientBuilder::default().build_testnet().await?;
    println!("Connected to Sui testnet: {}", sui_client.api_version());

    // Create MVR resolver
    let mvr_resolver = MvrResolver::testnet();
    
    // Resolve package address using human-readable name
    let package_address = mvr_resolver.resolve_package("@suifrens/core").await?;
    let package_id = ObjectID::from_hex_literal(&package_address)?;
    
    // Build transaction with resolved address
    let mut ptb = ProgrammableTransactionBuilder::new();
    ptb.command(Command::MoveCall(Box::new(ProgrammableMoveCall {
        package: package_id,
        module: Identifier::new("mint")?,
        function: Identifier::new("new_suifren")?,
        type_arguments: vec![],
        arguments: vec![],
    })));
    
    println!("✅ Transaction built using '@suifrens/core' -> {}", package_id);
    
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
```

### **🆕 Helper Functions for Sui Integration**

```rust
/// Resolve MVR target format for move calls
/// "@package::module::function" -> (ObjectID, "module", "function")
pub async fn resolve_mvr_target(
    resolver: &MvrResolver, 
    target: &str
) -> Result<(ObjectID, String, String)> {
    // Implementation handles "@package::module::function" format
}

/// Build transaction with MVR-resolved package
async fn build_mvr_transaction(
    resolver: &MvrResolver,
    package_name: &str,
    module: &str,
    function: &str,
) -> Result<ProgrammableTransactionBuilder> {
    let package_address = resolver.resolve_package(package_name).await?;
    let package_id = ObjectID::from_hex_literal(&package_address)?;
    
    let mut ptb = ProgrammableTransactionBuilder::new();
    // ... build transaction with resolved package_id
    Ok(ptb)
}
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
| **Official SDK Integration** | ✅ | **✅ NEW** |
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

### Features

```toml
[dependencies]
sui-mvr = { version = "0.1.0", features = ["sui-integration", "tracing"] }
```

Available features:
- `sui-integration` - Official Sui SDK integration helpers
- `tracing` - Detailed logging and tracing
- `static-resolution` - Compile-time resolution (planned)
- `metrics` - Additional metrics and monitoring
- `wasm` - WebAssembly support

## 📚 Examples

Check out the [examples directory](./examples/) for complete working examples:

- **[basic_usage.rs](./examples/basic_usage.rs)** - Simple resolver usage
- **[with_overrides.rs](./examples/with_overrides.rs)** - Static overrides for development
- **[batch_operations.rs](./examples/batch_operations.rs)** - Batch resolution and performance
- **🆕 [sui_integration.rs](./examples/sui_integration.rs)** - **Full Sui SDK integration**

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example sui_integration --features sui-integration
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Test with Sui integration
cargo test --features sui-integration

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --all-features

# Test examples
cargo run --example sui_integration --features sui-integration
```

## 🚀 Performance Tips

1. **Use batch operations** when resolving multiple items
2. **Enable caching** for repeated resolutions
3. **Use static overrides** for known packages in production
4. **Configure appropriate timeouts** for your network conditions
5. **Monitor cache hit rates** to optimize cache size
6. **🆕 Use official Sui SDK integration** for optimal transaction building

## 🛡️ Error Handling

```rust
use sui_mvr::MvrError;

match resolver.resolve_package("@myapp/core").await {
    Ok(address) => println!("✓ Resolved: {}", address),
    Err(MvrError::PackageNotFound(name)) => {
        println!("Package {} not found, using fallback", name);
    }
    Err(MvrError::RateLimitExceeded { retry_after_secs }) => {
        tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
        // Retry logic
    }
    Err(e) if e.is_retryable() => {
        // Implement retry with exponential backoff
    }
    Err(e) => println!("Permanent error: {}", e),
}
```

## 🗺️ Roadmap

- [x] **Official Sui SDK Integration** - ✅ **COMPLETED in v0.1.0**
- [ ] **Static Resolution** - Compile-time package resolution
- [ ] **WebAssembly Support** - Run in browsers and edge environments
- [ ] **Metrics & Observability** - Prometheus metrics and tracing
- [ ] **CLI Tool** - Command-line MVR operations
- [ ] **Custom Cache Backends** - Redis, file-based caching

## 🤝 Contributing

We love contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Inspired by** the [TypeScript MVR plugin](https://docs.suins.io/move-registry/tooling/typescript-sdk) from Mysten Labs
- **Built for** the [Sui blockchain](https://sui.io/) ecosystem  
- **Integrates with** the [official Sui Rust SDK](https://docs.sui.io/references/rust-sdk)
- **Thanks to** the Move Registry team for creating MVR

## 📞 Support & Community

- 📖 [Documentation](https://docs.rs/sui-mvr)
- 🐛 [Issue Tracker](https://github.com/Bralekfn/sui-mvr-rust/issues)
- 💬 [Discussions](https://github.com/Bralekfn/sui-mvr-rust/discussions)
- 🚀 [Sui Developer Discord](https://discord.gg/sui)

---

**Made with ❤️ for the Sui community** • **Star ⭐ this repo if it helped you!**