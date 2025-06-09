//! Example showing batch resolution operations
//!
//! Run with: cargo run --example batch_operations

use std::time::Instant;
use sui_mvr::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Sui MVR Rust Plugin - Batch Operations Example\n");

    let resolver = MvrResolver::mainnet();

    // Prepare test data
    let package_names = vec![
        "@suifrens/core",
        "@suifrens/accessories",
        "@suifrens/bullshark",
        "@suifrens/capy",
    ];

    let type_names = vec![
        "@suifrens/core::suifren::SuiFren",
        "@suifrens/core::bullshark::Bullshark",
        "@suifrens/accessories::hat::Hat",
        "@suifrens/accessories::shirt::Shirt",
    ];

    println!("📦 Batch resolving {} packages...", package_names.len());
    let start = Instant::now();

    match resolver.resolve_packages(&package_names).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!("✓ Resolved {} packages in {:?}", results.len(), duration);

            for (name, address) in &results {
                println!("   {} -> {}", name, address);
            }
        }
        Err(e) => println!("✗ Batch package resolution failed: {}", e),
    }

    println!("\n🏷️ Batch resolving {} types...", type_names.len());
    let start = Instant::now();

    match resolver.resolve_types(&type_names).await {
        Ok(results) => {
            let duration = start.elapsed();
            println!("✓ Resolved {} types in {:?}", results.len(), duration);

            for (name, type_sig) in &results {
                println!("   {} -> {}", name, type_sig);
            }
        }
        Err(e) => println!("✗ Batch type resolution failed: {}", e),
    }

    // Compare with individual resolution
    println!("\n⚡ Performance comparison: Individual vs Batch");

    // Individual resolution
    let start = Instant::now();
    let mut individual_results = Vec::new();
    for &name in &package_names {
        match resolver.resolve_package(name).await {
            Ok(address) => individual_results.push((name, address)),
            Err(e) => println!("✗ Failed to resolve {}: {}", name, e),
        }
    }
    let individual_duration = start.elapsed();

    // Since packages are now cached, try with fresh resolver for fair comparison
    let fresh_resolver = MvrResolver::mainnet();
    let start = Instant::now();
    match fresh_resolver.resolve_packages(&package_names).await {
        Ok(_batch_results) => {
            let batch_duration = start.elapsed();

            println!(
                "   Individual resolution: {:?} ({} requests)",
                individual_duration,
                package_names.len()
            );
            println!("   Batch resolution: {:?} (1 request)", batch_duration);

            if batch_duration < individual_duration {
                let speedup =
                    individual_duration.as_millis() as f64 / batch_duration.as_millis() as f64;
                println!("   🚀 Batch is {:.1}x faster!", speedup);
            }
        }
        Err(e) => println!("✗ Fresh batch resolution failed: {}", e),
    }

    // Cache statistics
    println!("\n📊 Cache performance:");
    match resolver.cache_stats() {
        Ok(stats) => {
            println!("   Total entries: {}", stats.total_entries);
            println!("   Cache hits: {}", stats.total_hits);
            println!("   Hit rate: {:.1}%", stats.hit_rate() * 100.0);
            println!("   Utilization: {:.1}%", stats.utilization() * 100.0);
        }
        Err(e) => println!("✗ Failed to get cache stats: {}", e),
    }

    // Demonstrate cache cleanup
    println!("\n🧹 Cache maintenance:");
    match resolver.cleanup_expired_cache() {
        Ok(removed) => println!("   Cleaned up {} expired entries", removed),
        Err(e) => println!("✗ Cache cleanup failed: {}", e),
    }

    // Test error handling with invalid names
    println!("\n❌ Error handling examples:");
    let invalid_names = vec!["invalid-name", "@incomplete", "@ns/pkg/too/many/parts"];

    for &invalid in &invalid_names {
        match resolver.resolve_package(invalid).await {
            Ok(_) => println!("   Unexpected success for: {}", invalid),
            Err(e) => println!("   ✓ Correctly rejected '{}': {}", invalid, e),
        }
    }

    println!("\n🎉 Batch operations example completed!");
    Ok(())
}
