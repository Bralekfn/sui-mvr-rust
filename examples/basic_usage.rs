//! Basic usage example for sui-mvr
//!
//! Run with: cargo run --example basic_usage

use sui_mvr::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Sui MVR Rust Plugin - Basic Usage Example\n");

    // Create a resolver for mainnet
    let resolver = MvrResolver::mainnet();

    println!("📦 Resolving package addresses...");

    // Resolve a package name to address
    match resolver.resolve_package("@suifrens/core").await {
        Ok(address) => println!("✓ SuiFrens core package: {}", address),
        Err(e) => println!("✗ Failed to resolve SuiFrens core: {}", e),
    }

    // Try another package
    match resolver.resolve_package("@suifrens/accessories").await {
        Ok(address) => println!("✓ SuiFrens accessories package: {}", address),
        Err(e) => println!("✗ Failed to resolve SuiFrens accessories: {}", e),
    }

    println!("\n🏷️ Resolving type signatures...");

    // Resolve type names
    match resolver
        .resolve_type("@suifrens/core::suifren::SuiFren")
        .await
    {
        Ok(type_sig) => println!("✓ SuiFren type: {}", type_sig),
        Err(e) => println!("✗ Failed to resolve SuiFren type: {}", e),
    }

    match resolver
        .resolve_type("@suifrens/core::bullshark::Bullshark")
        .await
    {
        Ok(type_sig) => println!("✓ Bullshark type: {}", type_sig),
        Err(e) => println!("✗ Failed to resolve Bullshark type: {}", e),
    }

    println!("\n📊 Cache statistics:");
    match resolver.cache_stats() {
        Ok(stats) => {
            println!("   Total entries: {}", stats.total_entries);
            println!("   Valid entries: {}", stats.valid_entries);
            println!("   Expired entries: {}", stats.expired_entries);
            println!("   Cache utilization: {:.1}%", stats.utilization() * 100.0);
            println!("   Total hits: {}", stats.total_hits);
        }
        Err(e) => println!("✗ Failed to get cache stats: {}", e),
    }

    println!("\n🎉 Example completed!");
    Ok(())
}
