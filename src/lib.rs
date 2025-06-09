//! # Sui MVR - Move Registry Plugin for Rust
//!
//! This crate provides Move Registry (MVR) resolution for the Sui blockchain,
//! allowing developers to use human-readable package names instead of addresses.
//!
//! ## Quick Start
//!
//! ```rust
//! use sui_mvr::{MvrResolver, MvrOverrides};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // For this example, we'll use static overrides instead of real API calls
//!     let overrides = MvrOverrides::new()
//!         .with_package("@suifrens/core".to_string(), "0x123456789".to_string());
//!     
//!     let resolver = MvrResolver::mainnet().with_overrides(overrides);
//!     let address = resolver.resolve_package("@suifrens/core").await?;
//!     println!("Package address: {}", address);
//!     Ok(())
//! }
//! ```
//!
//! ## Official Sui SDK Integration
//!
//! When building transactions with the official Sui SDK:
//!
//! ```rust,no_run
//! use sui_mvr::prelude::*;
//! # #[cfg(feature = "sui-integration")]
//! use sui_mvr::sui_integration::MvrResolverExt;
//! # #[cfg(feature = "sui-integration")]
//! use sui_sdk::{SuiClientBuilder, types::base_types::ObjectID};
//!
//! # #[cfg(feature = "sui-integration")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize Sui client
//!     let sui_client = SuiClientBuilder::default().build_testnet().await?;
//!     
//!     // Create MVR resolver
//!     let resolver = MvrResolver::testnet();
//!     
//!     // Resolve and build transaction
//!     let ptb = resolver.build_move_call_transaction(
//!         "@suifrens/core",  // Human-readable package name
//!         "mint",
//!         "new_suifren",
//!         vec![],            // Type arguments
//!         vec![],            // Function arguments
//!     ).await?;
//!     
//!     println!("Transaction built with MVR-resolved package");
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
//! - **Sui SDK Integration**: Seamless integration with official Sui Rust SDK

pub mod cache;
pub mod error;
pub mod resolver;
pub mod types;

// Conditional compilation for Sui integration
#[cfg(feature = "sui-integration")]
pub mod sui_integration;

pub use error::MvrError;
pub use resolver::MvrResolver;
pub use types::{MvrConfig, MvrOverrides};

// Re-export cache stats for public API
pub use cache::CacheStats;

/// Commonly used items for easy importing
pub mod prelude {
    pub use super::{MvrConfig, MvrError, MvrOverrides, MvrResolver, CacheStats};
    
    // Re-export Sui integration when feature is enabled
    #[cfg(feature = "sui-integration")]
    pub use super::sui_integration::MvrResolverExt;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type alias for MVR operations
pub type MvrResult<T> = Result<T, MvrError>;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_resolver_functionality() {
        // Test that basic resolver creation works
        let resolver = MvrResolver::testnet();
        assert!(resolver.config().endpoint_url.contains("testnet"));
        
        let resolver = MvrResolver::mainnet();
        assert!(resolver.config().endpoint_url.contains("mainnet"));
    }

    #[tokio::test]
    async fn test_overrides_integration() {
        let overrides = MvrOverrides::new()
            .with_package("@test/pkg".to_string(), "0x123".to_string());
        
        let resolver = MvrResolver::testnet().with_overrides(overrides);
        
        let result = resolver.resolve_package("@test/pkg").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x123");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let resolver = MvrResolver::testnet();
        
        // Test invalid package name
        let result = resolver.resolve_package("invalid-name").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MvrError::InvalidPackageName(_)));
    }

    #[cfg(feature = "sui-integration")]
    #[tokio::test]
    async fn test_sui_integration_compilation() {
        use crate::sui_integration::MvrResolverExt;
        
        // Test that Sui integration compiles correctly
        let overrides = MvrOverrides::new()
            .with_package("@test/pkg".to_string(), "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string());
        
        let resolver = MvrResolver::testnet().with_overrides(overrides);
        
        // Test MVR target resolution
        let result = resolver.resolve_mvr_target("@test/pkg::module::function").await;
        assert!(result.is_ok());
        
        let (package_id, module, function) = result.unwrap();
        assert_eq!(module, "module");
        assert_eq!(function, "function");
        assert_eq!(package_id.to_hex_literal(), "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234");
    }
}