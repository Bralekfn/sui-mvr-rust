//! Example showing how to use static overrides
//!
//! Run with: cargo run --example with_overrides

use sui_mvr::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Sui MVR Rust Plugin - Overrides Example\n");

    // Create overrides for local development or CI
    let overrides = MvrOverrides::new()
        .with_package("@myapp/core".to_string(), "0x123456789abcdef".to_string())
        .with_package("@myapp/utils".to_string(), "0xfedcba987654321".to_string())
        .with_type(
            "@myapp/core::token::MyToken".to_string(),
            "0x123456789abcdef::token::MyToken".to_string(),
        )
        .with_type(
            "@myapp/core::nft::MyNFT".to_string(),
            "0x123456789abcdef::nft::MyNFT<T>".to_string(),
        );

    println!("📝 Created overrides:");
    println!("   Packages: {}", overrides.packages.len());
    println!("   Types: {}", overrides.types.len());

    // Create resolver with overrides
    let resolver = MvrResolver::testnet().with_overrides(overrides);

    println!("\n📦 Resolving packages (using overrides)...");

    // These will use the overrides instead of making API calls
    match resolver.resolve_package("@myapp/core").await {
        Ok(address) => println!("✓ MyApp core package: {address} (from override)"),
        Err(e) => println!("✗ Failed to resolve MyApp core: {e}"),
    }

    match resolver.resolve_package("@myapp/utils").await {
        Ok(address) => println!("✓ MyApp utils package: {address} (from override)"),
        Err(e) => println!("✗ Failed to resolve MyApp utils: {e}"),
    }

    println!("\n🏷️ Resolving types (using overrides)...");

    match resolver.resolve_type("@myapp/core::token::MyToken").await {
        Ok(type_sig) => println!("✓ MyToken type: {type_sig} (from override)"),
        Err(e) => println!("✗ Failed to resolve MyToken type: {e}"),
    }

    match resolver.resolve_type("@myapp/core::nft::MyNFT").await {
        Ok(type_sig) => println!("✓ MyNFT type: {type_sig} (from override)"),
        Err(e) => println!("✗ Failed to resolve MyNFT type: {e}"),
    }

    // This will try to fetch from API since it's not in overrides
    println!("\n🌐 Resolving from API (not in overrides)...");
    match resolver.resolve_package("@suifrens/core").await {
        Ok(address) => println!("✓ SuiFrens core package: {address} (from API)"),
        Err(e) => println!("✗ Failed to resolve SuiFrens core: {e}"),
    }

    println!("\n💾 Saving overrides to JSON:");
    match resolver.config().overrides.as_ref().unwrap().to_json() {
        Ok(json) => {
            println!("{json}");

            // Example of loading from JSON
            println!("\n📖 Loading overrides from JSON:");
            match MvrOverrides::from_json(&json) {
                Ok(loaded) => println!(
                    "✓ Successfully loaded {} packages and {} types",
                    loaded.packages.len(),
                    loaded.types.len()
                ),
                Err(e) => println!("✗ Failed to load from JSON: {e}"),
            }
        }
        Err(e) => println!("✗ Failed to serialize to JSON: {e}"),
    }

    println!("\n🎉 Overrides example completed!");
    Ok(())
}
