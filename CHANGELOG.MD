# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Static resolution feature (similar to @mysten/mvr-static)
- Performance benchmarks
- WebAssembly support
- Metrics and monitoring capabilities

### Changed
- TBD

### Deprecated
- TBD

### Removed
- TBD

### Fixed
- TBD

### Security
- TBD

## [0.1.0] - 2025-06-08

### Added
- Initial release of sui-mvr Rust plugin
- **Core Features:**
  - Package name resolution (`@namespace/package` → `0x123...`)
  - Type name resolution with full type signatures
  - Support for both mainnet and testnet networks
- **Caching System:**
  - Built-in memory cache with configurable TTL
  - LRU eviction policy for memory management
  - Cache statistics and cleanup operations
- **Configuration:**
  - Static overrides for local development and CI
  - Configurable endpoints, timeouts, and cache settings
  - Support for custom MVR endpoints
- **Performance Features:**
  - Batch resolution for multiple packages/types
  - Concurrent request handling with rate limiting
  - Connection pooling and keep-alive
- **Developer Experience:**
  - Comprehensive error handling with retry logic
  - Input validation for package and type names
  - Detailed logging and tracing support (optional)
- **Documentation:**
  - Complete API documentation with examples
  - Three comprehensive usage examples
  - Integration test suite
  - Performance comparison demonstrations
- **Examples:**
  - `basic_usage.rs` - Simple resolver usage
  - `with_overrides.rs` - Static overrides for local development
  - `batch_operations.rs` - Batch resolution and performance testing

### Technical Details
- **Dependencies:**
  - reqwest 0.11 for HTTP client functionality
  - tokio 1.0 for async runtime
  - serde 1.0 for JSON serialization
  - thiserror 1.0 for error handling
  - futures 0.3 for concurrent operations
- **Rust Version:** 
  - MSRV: 1.70.0
  - Edition: 2021
- **Features:**
  - Default: Core functionality only
  - `tracing`: Detailed logging and tracing
  - `static-resolution`: Static resolution capabilities
  - `metrics`: Additional metrics and monitoring
  - `wasm`: WebAssembly support

### Comparison with TypeScript SDK

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

### Known Limitations
- API endpoints are currently mock implementations for testing
- WebAssembly support requires additional testing
- Integration with official Sui Rust SDK is planned for future releases

### Migration Guide
This is the initial release, so no migration is required. For users coming from the TypeScript MVR plugin:

1. Install: `cargo add sui-mvr`
2. Replace TypeScript resolver creation with:
   ```rust
   use sui_mvr::MvrResolver;
   let resolver = MvrResolver::mainnet();
   ```
3. Use async/await for all resolution operations
4. Take advantage of batch operations for better performance

### Contributors
- @Bralekfn - Initial implementation and design

### Acknowledgments
- Inspired by the TypeScript MVR plugin from Mysten Labs
- Built for the Sui blockchain ecosystem
- Thanks to the Move Registry team for creating MVR