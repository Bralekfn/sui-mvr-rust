//! Complete integration example with official Sui Rust SDK
//!
//! This example demonstrates how to use sui-mvr with the official Sui SDK
//! to build and execute transactions using human-readable package names.
//!
//! Run with: cargo run --example sui_integration --features sui-integration

use anyhow::Result;
use std::str::FromStr;
use sui_mvr::prelude::*;
use sui_sdk::{
    types::{
        base_types::{ObjectID, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, CallArg, Command, ProgrammableMoveCall, TransactionData},
        Identifier,
    },
    SuiClient, SuiClientBuilder,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Sui MVR + Official Sui SDK Integration Example\n");

    // Step 1: Initialize Sui client using official SDK patterns
    println!("📡 Connecting to Sui network...");
    let sui_client: SuiClient = SuiClientBuilder::default()
        .build_testnet()
        .await?;
    
    println!("✅ Connected to Sui testnet");
    println!("   RPC version: {}", sui_client.api_version());
    println!("   Chain identifier: {:?}", sui_client.read_api().get_chain_identifier().await?);

    // Step 2: Create MVR resolver with static overrides for demo
    println!("\n📦 Setting up MVR resolver...");
    
    // In production, these would be real package addresses from MVR
    let overrides = MvrOverrides::new()
        .with_package(
            "@suifrens/core".to_string(), 
            "0xee496a0cc04d06a345982ba6697c90c619020de9e274408c7819f787ff66e1a1".to_string()
        )
        .with_package(
            "@example/defi".to_string(),
            "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string()
        )
        .with_type(
            "@suifrens/core::suifren::SuiFren".to_string(),
            "0xee496a0cc04d06a345982ba6697c90c619020de9e274408c7819f787ff66e1a1::suifren::SuiFren".to_string(),
        );
    
    let mvr_resolver = MvrResolver::testnet().with_overrides(overrides);
    println!("✅ MVR resolver configured with test overrides");

    // Step 3: Package Resolution Demo
    println!("\n📦 Resolving package addresses...");
    
    let package_address = mvr_resolver.resolve_package("@suifrens/core").await?;
    println!("✅ '@suifrens/core' -> {}", package_address);
    
    let defi_address = mvr_resolver.resolve_package("@example/defi").await?;
    println!("✅ '@example/defi' -> {}", defi_address);

    // Step 4: Type Resolution Demo
    println!("\n🏷️  Resolving type signatures...");
    
    let type_signature = mvr_resolver.resolve_type("@suifrens/core::suifren::SuiFren").await?;
    println!("✅ SuiFren type: {}", type_signature);

    // Step 5: Building Programmable Transaction Blocks with MVR
    println!("\n🔧 Building PTB with MVR-resolved addresses...");
    
    let ptb = build_mvr_transaction(
        &mvr_resolver,
        "@suifrens/core",
        "mint", 
        "new_suifren"
    ).await?;
    
    println!("✅ Transaction block built using resolved package address");

    // Step 6: Advanced Transaction Building Example
    println!("\n⚙️  Advanced transaction building...");
    
    // Build a more complex transaction that uses multiple resolved packages
    let complex_ptb = build_complex_mvr_transaction(&mvr_resolver, &sui_client).await?;
    println!("✅ Complex transaction built with multiple MVR resolutions");

    // Step 7: Batch Resolution Performance Demo
    println!("\n⚡ Batch resolution performance...");
    
    let package_names = vec![
        "@suifrens/core",
        "@example/defi",
    ];
    
    let start = std::time::Instant::now();
    let batch_results = mvr_resolver.resolve_packages(&package_names).await?;
    let duration = start.elapsed();
    
    println!("✅ Batch resolved {} packages in {:?}", batch_results.len(), duration);
    for (name, address) in batch_results {
        println!("   {} -> {}", name, address);
    }

    // Step 8: Cache Statistics
    println!("\n📊 Cache performance:");
    if let Ok(stats) = mvr_resolver.cache_stats() {
        println!("   Total entries: {}", stats.total_entries);
        println!("   Valid entries: {}", stats.valid_entries);
        println!("   Cache utilization: {:.1}%", stats.utilization() * 100.0);
    }

    // Step 9: Error Handling Demonstration
    println!("\n❌ Error handling examples:");
    
    // Try to resolve a non-existent package
    match mvr_resolver.resolve_package("@nonexistent/package").await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✅ Correctly handled error: {}", e),
    }
    
    // Try an invalid package name
    match mvr_resolver.resolve_package("invalid-name").await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✅ Correctly rejected invalid name: {}", e),
    }

    println!("\n🎉 Integration example completed successfully!");
    println!("\n💡 Key takeaways:");
    println!("   • MVR resolver integrates seamlessly with official Sui SDK");
    println!("   • Use resolved addresses in ProgrammableTransactionBuilder");
    println!("   • Batch resolution improves performance for multiple packages");
    println!("   • Static overrides enable testing and local development");
    
    Ok(())
}

/// Helper function to build a transaction using MVR-resolved package address
async fn build_mvr_transaction(
    resolver: &MvrResolver,
    package_name: &str,
    module: &str,
    function: &str,
) -> Result<ProgrammableTransactionBuilder> {
    // Step 1: Resolve the package address using MVR
    let package_address = resolver.resolve_package(package_name).await?;
    
    // Step 2: Convert to ObjectID for use in Sui transactions
    let package_id = ObjectID::from_hex_literal(&package_address)?;
    
    // Step 3: Build the programmable transaction block
    let mut ptb = ProgrammableTransactionBuilder::new();
    
    // Step 4: Create the move call with resolved package ID
    let move_call = ProgrammableMoveCall {
        package: package_id,
        module: Identifier::new(module)?,
        function: Identifier::new(function)?,
        type_arguments: vec![],
        arguments: vec![
            // In a real example, you'd add actual arguments here
            // For example: Argument::Input(0) for first input
        ],
    };
    
    // Step 5: Add the move call to the transaction block
    ptb.command(Command::MoveCall(Box::new(move_call)));
    
    println!("   📦 Resolved '{}' -> {}", package_name, package_id);
    println!("   🔧 Built move call: {}::{}::{}", package_id, module, function);
    
    Ok(ptb)
}

/// Build a more complex transaction using multiple MVR resolutions
async fn build_complex_mvr_transaction(
    resolver: &MvrResolver,
    _sui_client: &SuiClient,
) -> Result<ProgrammableTransactionBuilder> {
    // Resolve multiple packages
    let packages = vec!["@suifrens/core", "@example/defi"];
    let resolved_packages = resolver.resolve_packages(&packages).await?;
    
    let mut ptb = ProgrammableTransactionBuilder::new();
    
    // Example: Build a transaction that interacts with multiple packages
    for (package_name, package_address) in resolved_packages {
        let package_id = ObjectID::from_hex_literal(&package_address)?;
        
        // Add a move call for each resolved package
        let move_call = ProgrammableMoveCall {
            package: package_id,
            module: Identifier::new("example")?,
            function: Identifier::new("function")?,
            type_arguments: vec![],
            arguments: vec![],
        };
        
        ptb.command(Command::MoveCall(Box::new(move_call)));
        println!("   📦 Added call to {}: {}", package_name, package_id);
    }
    
    // In a real application, you might:
    // 1. Add coin splits: ptb.command(Command::SplitCoins(...));
    // 2. Add transfers: ptb.command(Command::TransferObjects(...));
    // 3. Use results from one call as input to another
    
    Ok(ptb)
}

/// Utility function to resolve MVR target format for transaction building
pub async fn resolve_mvr_target(
    resolver: &MvrResolver, 
    target: &str
) -> Result<(ObjectID, String, String)> {
    if !target.starts_with('@') {
        return Err(anyhow::anyhow!("Target must start with '@': {}", target));
    }

    // Parse format: @package::module::function
    let parts: Vec<&str> = target.splitn(2, "::").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid target format: {}", target));
    }

    let package_part = parts[0];
    let module_function = parts[1];
    
    // Split module::function
    let module_parts: Vec<&str> = module_function.splitn(2, "::").collect();
    if module_parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid module::function format: {}", module_function));
    }

    let package_address = resolver.resolve_package(package_part).await?;
    let package_id = ObjectID::from_hex_literal(&package_address)?;
    
    Ok((package_id, module_parts[0].to_string(), module_parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mvr_target_resolution() {
        let resolver = MvrResolver::testnet().with_overrides(
            MvrOverrides::new().with_package(
                "@test/package".to_string(),
                "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string()
            )
        );
        
        let (package_id, module, function) = resolve_mvr_target(
            &resolver,
            "@test/package::module::function"
        ).await.unwrap();
        
        assert_eq!(module, "module");
        assert_eq!(function, "function");
        assert_eq!(
            package_id.to_hex_literal(),
            "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234"
        );
    }
}