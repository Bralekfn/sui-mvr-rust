//! # Production Usage Example
//!
//! This example demonstrates production-ready patterns for using sui-mvr
//! in real applications, including:
//!
//! - Configuration management from environment variables
//! - Error handling and retry logic with exponential backoff
//! - Cache monitoring and maintenance
//! - Health checks and monitoring endpoints
//! - Graceful shutdown procedures
//! - Performance optimization strategies
//! - Integration with official Sui SDK
//!
//! ## Running
//!
//! ```bash
//! cargo run --example production_usage --features sui-integration
//! ```
//!
//! ## Environment Variables
//!
//! - `MVR_ENDPOINT` - Custom MVR endpoint (default: mainnet)
//! - `MVR_CACHE_TTL` - Cache TTL in seconds (default: 3600)
//! - `MVR_TIMEOUT` - Request timeout in seconds (default: 30)
//! - `MVR_OVERRIDES` - JSON string of package overrides (optional)
//! - `SUI_NETWORK` - Sui network to connect to (mainnet/testnet/devnet)

use anyhow::Result;
use std::time::Duration;
use sui_mvr::prelude::*;

#[cfg(feature = "sui-integration")]
use sui_mvr::sui_integration::{utils, MvrResolverExt};

#[cfg(feature = "sui-integration")]
use sui_sdk::{
    types::{
        base_types::{ObjectID, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, TransactionData},
        TypeTag,
    },
    SuiClient, SuiClientBuilder,
};

/// Production configuration loader
fn load_production_config() -> Result<MvrConfig> {
    let endpoint = std::env::var("MVR_ENDPOINT")
        .unwrap_or_else(|_| "https://mainnet.mvr.mystenlabs.com".to_string());

    let cache_ttl_secs = std::env::var("MVR_CACHE_TTL")
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<u64>()
        .unwrap_or(3600);

    let timeout_secs = std::env::var("MVR_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .unwrap_or(30);

    let mut config = MvrConfig::default()
        .with_endpoint(endpoint)
        .with_cache_ttl(Duration::from_secs(cache_ttl_secs))
        .with_timeout(Duration::from_secs(timeout_secs));

    // Load static overrides from environment if provided
    if let Ok(overrides_json) = std::env::var("MVR_OVERRIDES") {
        if let Ok(overrides) = MvrOverrides::from_json(&overrides_json) {
            config = config.with_overrides(overrides);
            println!(
                "📝 Loaded {} package overrides from environment",
                overrides.packages.len()
            );
        }
    }

    Ok(config)
}

/// Create production MVR resolver with environment configuration
fn create_production_resolver() -> Result<MvrResolver> {
    let config = load_production_config()?;
    println!("🔧 Production configuration:");
    println!("   Endpoint: {}", config.endpoint_url);
    println!("   Cache TTL: {:?}", config.cache_ttl);
    println!("   Timeout: {:?}", config.timeout);

    Ok(MvrResolver::new(config))
}

/// Robust package resolution with retry logic
async fn resolve_package_with_retry(
    resolver: &MvrResolver,
    package_name: &str,
    max_retries: u32,
) -> Result<String> {
    let mut attempts = 0;

    loop {
        match resolver.resolve_package(package_name).await {
            Ok(address) => {
                if attempts > 0 {
                    println!("✅ Resolved '{}' after {} retries", package_name, attempts);
                }
                return Ok(address);
            }
            Err(e) if attempts < max_retries && e.is_retryable() => {
                attempts += 1;

                if let Some(delay) = e.retry_delay() {
                    println!(
                        "⏳ Retrying '{}' in {:?} (attempt {}/{})",
                        package_name, delay, attempts, max_retries
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    // Exponential backoff for other retryable errors
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    println!(
                        "⏳ Retrying '{}' in {:?} (attempt {}/{})",
                        package_name, delay, attempts, max_retries
                    );
                    tokio::time::sleep(delay).await;
                }
            }
            Err(e) => {
                println!(
                    "❌ Failed to resolve '{}' after {} attempts: {}",
                    package_name, attempts, e
                );
                return Err(e.into());
            }
        }
    }
}

/// Health status for monitoring
#[derive(Debug, Default)]
pub struct HealthStatus {
    pub mvr_responsive: bool,
    pub cache_utilization: f64,
    pub cache_hit_rate: f64,
    pub total_cache_entries: usize,
    #[cfg(feature = "sui-integration")]
    pub sui_connectivity: bool,
}

/// Production application structure
pub struct ProductionApp {
    resolver: MvrResolver,
    #[cfg(feature = "sui-integration")]
    sui_client: SuiClient,
}

impl ProductionApp {
    /// Initialize production application
    pub async fn new() -> Result<Self> {
        println!("🚀 Initializing production Sui MVR application...");

        let resolver = create_production_resolver()?;

        #[cfg(feature = "sui-integration")]
        let sui_client = {
            let network = std::env::var("SUI_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
            match network.as_str() {
                "mainnet" => SuiClientBuilder::default().build_mainnet().await?,
                "testnet" => SuiClientBuilder::default().build_testnet().await?,
                "devnet" => SuiClientBuilder::default().build_devnet().await?,
                url if url.starts_with("http") => SuiClientBuilder::default().build(url).await?,
                _ => return Err(anyhow::anyhow!("Invalid SUI_NETWORK: {}", network)),
            }
        };

        #[cfg(feature = "sui-integration")]
        println!("✅ Connected to Sui network: {}", sui_client.api_version());

        Ok(Self {
            resolver,
            #[cfg(feature = "sui-integration")]
            sui_client,
        })
    }

    /// Execute a batch DeFi operation using MVR resolution
    #[cfg(feature = "sui-integration")]
    pub async fn execute_defi_batch(&self, operations: Vec<DefiOperation>) -> Result<Vec<String>> {
        println!(
            "🔄 Executing batch DeFi operations ({} operations)...",
            operations.len()
        );

        let mut results = Vec::new();

        for (i, operation) in operations.into_iter().enumerate() {
            println!(
                "  [{}/{}] Processing operation: {:?}",
                i + 1,
                results.len() + 1,
                operation
            );

            let result = match operation {
                DefiOperation::AddLiquidity {
                    pool_package,
                    amount_a,
                    amount_b,
                } => {
                    let package_address =
                        resolve_package_with_retry(&self.resolver, &pool_package, 3).await?;
                    let package_id = ObjectID::from_hex_literal(&package_address)?;

                    // Build add liquidity transaction
                    let ptb = self
                        .resolver
                        .build_move_call_transaction(
                            &pool_package,
                            "pool",
                            "add_liquidity",
                            vec![], // Type arguments would be specified based on pool type
                            vec![], // Arguments would include amounts and pool references
                        )
                        .await?;

                    format!(
                        "Add liquidity: {} + {} to pool {}",
                        amount_a, amount_b, package_id
                    )
                }

                DefiOperation::Swap {
                    dex_package,
                    from_amount,
                    to_token_type,
                } => {
                    let package_address =
                        resolve_package_with_retry(&self.resolver, &dex_package, 3).await?;
                    let package_id = ObjectID::from_hex_literal(&package_address)?;

                    // Build swap transaction
                    let ptb = self
                        .resolver
                        .build_move_call_transaction(
                            &dex_package,
                            "trading",
                            "swap",
                            vec![], // Token type arguments
                            vec![], // Swap parameters
                        )
                        .await?;

                    format!(
                        "Swap {} for {} via DEX {}",
                        from_amount, to_token_type, package_id
                    )
                }
            };

            results.push(result);
        }

        println!("✅ Completed {} DeFi operations", results.len());
        Ok(results)
    }

    /// Health check for production monitoring
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus::default();

        // Check MVR resolver responsiveness
        let test_resolution = tokio::time::timeout(
            Duration::from_secs(5),
            self.resolver.resolve_package("@test/health"),
        )
        .await;
        status.mvr_responsive = test_resolution.is_ok();

        // Check cache health
        if let Ok(stats) = self.resolver.cache_stats() {
            status.cache_utilization = stats.utilization();
            status.cache_hit_rate = stats.hit_rate();
            status.total_cache_entries = stats.total_entries;
        }

        #[cfg(feature = "sui-integration")]
        {
            // Check Sui client connectivity
            status.sui_connectivity = !self.sui_client.api_version().is_empty();
        }

        Ok(status)
    }

    /// Monitor cache performance continuously
    pub async fn start_cache_monitoring(&self) -> Result<()> {
        println!("📊 Starting cache performance monitoring...");

        let resolver = self.resolver.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await; // Check every 5 minutes

                if let Ok(stats) = resolver.cache_stats() {
                    println!("📊 Cache Stats:");
                    println!("   Utilization: {:.1}%", stats.utilization() * 100.0);
                    println!("   Hit Rate: {:.1}%", stats.hit_rate() * 100.0);
                    println!("   Total Entries: {}", stats.total_entries);

                    // Clean up expired entries if utilization is high
                    if stats.utilization() > 0.8 {
                        if let Ok(cleaned) = resolver.cleanup_expired_cache() {
                            println!("   🧹 Cleaned {} expired entries", cleaned);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Graceful shutdown with cleanup
    pub async fn shutdown(&self) -> Result<()> {
        println!("🔄 Shutting down production application...");

        // Save final cache statistics
        if let Ok(stats) = self.resolver.cache_stats() {
            println!("📊 Final cache statistics:");
            println!("   Total entries processed: {}", stats.total_entries);
            println!("   Final hit rate: {:.1}%", stats.hit_rate() * 100.0);
            println!("   Final utilization: {:.1}%", stats.utilization() * 100.0);
        }

        // In a real application, you might:
        // 1. Save important cache entries to persistent storage
        // 2. Close database connections
        // 3. Flush any pending operations
        // 4. Send shutdown notifications

        println!("✅ Production application shutdown complete");
        Ok(())
    }
}

/// DeFi operations for the production example
#[derive(Debug, Clone)]
pub enum DefiOperation {
    AddLiquidity {
        pool_package: String,
        amount_a: u64,
        amount_b: u64,
    },
    Swap {
        dex_package: String,
        from_amount: u64,
        to_token_type: String,
    },
}

/// Error recovery strategies for production
pub async fn handle_mvr_error(error: &MvrError, package_name: &str) -> Option<String> {
    match error {
        MvrError::PackageNotFound(_) => {
            println!(
                "🔄 Package '{}' not found in MVR, checking fallback mappings...",
                package_name
            );

            // In production, you might have a fallback registry
            match package_name {
                "@suifrens/core" => {
                    let fallback =
                        "0xee496a0cc04d06a345982ba6697c90c619020de9e274408c7819f787ff66e1a1";
                    println!(
                        "✅ Using fallback address for {}: {}",
                        package_name, fallback
                    );
                    Some(fallback.to_string())
                }
                "@common/utils" => {
                    let fallback = "0x1234567890abcdef1234567890abcdef12345678";
                    println!(
                        "✅ Using fallback address for {}: {}",
                        package_name, fallback
                    );
                    Some(fallback.to_string())
                }
                _ => {
                    println!("❌ No fallback available for {}", package_name);
                    None
                }
            }
        }
        MvrError::RateLimitExceeded { retry_after_secs } => {
            println!(
                "⏳ Rate limited, waiting {} seconds before retry...",
                retry_after_secs
            );
            tokio::time::sleep(Duration::from_secs(*retry_after_secs)).await;
            None // Caller should retry
        }
        MvrError::Timeout { timeout_secs } => {
            println!(
                "⏰ Request timeout after {}s, implementing exponential backoff...",
                timeout_secs
            );
            None // Caller should retry
        }
        _ => {
            println!("❌ Non-recoverable error: {}", error);
            None
        }
    }
}

/// Main production application entry point
#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Sui MVR Production Usage Example");
    println!("=====================================\n");

    // Load configuration from environment
    println!("🔧 Loading production configuration...");

    // Initialize application
    let app = ProductionApp::new().await?;

    // Start monitoring
    app.start_cache_monitoring().await?;

    // Run health check
    println!("\n🏥 Running health check...");
    let health = app.health_check().await?;

    println!(
        "   MVR Responsive: {}",
        if health.mvr_responsive { "✅" } else { "❌" }
    );
    #[cfg(feature = "sui-integration")]
    println!(
        "   Sui Connectivity: {}",
        if health.sui_connectivity {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Cache Utilization: {:.1}%",
        health.cache_utilization * 100.0
    );
    println!("   Cache Hit Rate: {:.1}%", health.cache_hit_rate * 100.0);

    #[cfg(not(feature = "sui-integration"))]
    if !health.mvr_responsive {
        println!("⚠️  Health check warnings detected, but continuing...");
    }

    #[cfg(feature = "sui-integration")]
    if !health.mvr_responsive || !health.sui_connectivity {
        return Err(anyhow::anyhow!("❌ Health check failed - cannot continue"));
    }

    // Demonstrate production usage patterns
    println!("\n📦 Demonstrating production package resolution...");

    // Add production overrides for demo
    let demo_overrides = MvrOverrides::new()
        .with_package(
            "@myapp/defi".to_string(),
            "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string(),
        )
        .with_package(
            "@myapp/nft".to_string(),
            "0xabcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef".to_string(),
        )
        .with_package(
            "@common/utils".to_string(),
            "0x987654321fedcba987654321fedcba987654321fedcba987654321fedcba9876".to_string(),
        );

    let demo_resolver = app.resolver.clone().with_overrides(demo_overrides);

    // Batch resolve critical packages
    let critical_packages = vec!["@myapp/defi", "@myapp/nft", "@common/utils"];
    let start = std::time::Instant::now();
    let resolved = demo_resolver.resolve_packages(&critical_packages).await?;
    let duration = start.elapsed();

    println!(
        "✅ Resolved {} critical packages in {:?}",
        resolved.len(),
        duration
    );
    for (name, address) in &resolved {
        println!("   {} -> {}", name, address);
    }

    #[cfg(feature = "sui-integration")]
    {
        // Demonstrate DeFi operations
        println!("\n💰 Executing sample DeFi operations...");

        let operations = vec![
            DefiOperation::AddLiquidity {
                pool_package: "@myapp/defi".to_string(),
                amount_a: 1_000_000_000, // 1 SUI
                amount_b: 2_000_000_000, // 2 tokens
            },
            DefiOperation::Swap {
                dex_package: "@myapp/defi".to_string(),
                from_amount: 500_000_000, // 0.5 SUI
                to_token_type: "USDC".to_string(),
            },
        ];

        let results = app.execute_defi_batch(operations).await?;
        for (i, result) in results.iter().enumerate() {
            println!("   [{}] {}", i + 1, result);
        }
    }

    // Demonstrate error handling
    println!("\n❌ Demonstrating error handling...");

    let error_cases = vec![
        "@nonexistent/package",
        "invalid-package-name",
        "@missing/namespace",
    ];

    for error_case in error_cases {
        match demo_resolver.resolve_package(error_case).await {
            Ok(_) => println!("   Unexpected success for: {}", error_case),
            Err(e) => {
                println!("   ✅ Correctly handled error for '{}': {}", error_case, e);

                // Demonstrate error recovery
                if let Some(fallback) = handle_mvr_error(&e, error_case).await {
                    println!("      🔄 Fallback address: {}", fallback);
                }
            }
        }
    }

    // Cache performance demonstration
    println!("\n📊 Final cache performance:");
    if let Ok(stats) = demo_resolver.cache_stats() {
        println!("   Total entries: {}", stats.total_entries);
        println!("   Valid entries: {}", stats.valid_entries);
        println!("   Expired entries: {}", stats.expired_entries);
        println!("   Hit rate: {:.1}%", stats.hit_rate() * 100.0);
        println!("   Utilization: {:.1}%", stats.utilization() * 100.0);
    }

    // Simulate running for a short time
    println!("\n🔄 Simulating production runtime (5 seconds)...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Graceful shutdown
    app.shutdown().await?;

    println!("\n🎉 Production usage example completed successfully!");
    println!("\n💡 Key takeaways for production:");
    println!("   • Use environment variables for configuration");
    println!("   • Implement retry logic with exponential backoff");
    println!("   • Monitor cache performance and cleanup regularly");
    println!("   • Have fallback strategies for critical packages");
    println!("   • Use batch operations for better performance");
    println!("   • Implement proper health checks for monitoring");
    println!("   • Plan for graceful shutdown procedures");

    Ok(())
}
