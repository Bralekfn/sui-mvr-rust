//! Integration utilities for the official Sui Rust SDK
//!
//! This module provides documentation and examples for integrating
//! MVR resolution with the official Sui SDK transaction building.
//!
//! ## Setup Required
//!
//! To use these integration features, you must manually add the Sui SDK:
//!
//! ```toml
//! [dependencies]
//! sui-mvr = { version = "0.1.0", features = ["sui-integration"] }
//! sui-sdk = { git = "https://github.com/mystenlabs/sui", package = "sui-sdk" }
//! ```

#[cfg(feature = "sui-integration")]
pub mod docs {
    //! Documentation and examples for Sui SDK integration
    //!
    //! The actual integration code requires the Sui SDK to be manually added
    //! as shown in the crate documentation.
}
