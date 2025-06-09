//! # Sui MVR - Move Registry Plugin for Rust
//!
//! This crate provides Move Registry (MVR) resolution for the Sui blockchain,
//! allowing developers to use human-readable package names instead of addresses.
//!
//! ## Quick Start
//!
//! ```rust
//! use sui_mvr::MvrResolver;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let resolver = MvrResolver::mainnet();
//!     let address = resolver.resolve_package("@suifrens/core").await?;
//!     println!("Package address: {}", address);
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Package Resolution**: Resolve MVR package names to their on-chain addresses
//! - **Type Resolution**: Resolve MVR type names to their full type signatures  
//! - **Caching**: Built-in memory cache with configurable TTL
//! - **Network Support**: Works with both mainnet and testnet
//! - **Override Support**: Define static overrides for local development and CI
//! - **Batch Operations**: Resolve multiple packages/types efficiently
//! - **Error Handling**: Comprehensive error types and fallback strategies

pub mod cache;
pub mod error;
pub mod resolver;
pub mod types;

pub use error::MvrError;
pub use resolver::MvrResolver;
pub use types::{MvrConfig, MvrOverrides};

/// Commonly used items for easy importing
pub mod prelude {
    pub use super::{MvrConfig, MvrError, MvrOverrides, MvrResolver};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
