use sui_mvr::prelude::*;
use tokio::time::Duration;

mod common;
use common::*;

/// Integration tests for the MVR resolver
///
/// These tests verify the complete functionality of the resolver,
/// including configuration, caching, error handling, and API interactions.

#[tokio::test]
async fn test_resolver_configuration() {
    // Test mainnet configuration
    let mainnet_resolver = MvrResolver::mainnet();
    assert!(mainnet_resolver.config().endpoint_url.contains("mainnet"));
    assert_eq!(
        mainnet_resolver.config().cache_ttl,
        Duration::from_secs(3600)
    );

    // Test testnet configuration
    let testnet_resolver = MvrResolver::testnet();
    assert!(testnet_resolver.config().endpoint_url.contains("testnet"));

    // Test custom configuration
    let custom_config = MvrConfig::testnet()
        .with_cache_ttl(Duration::from_secs(1800))
        .with_timeout(Duration::from_secs(60));

    let custom_resolver = MvrResolver::new(custom_config);
    assert_eq!(
        custom_resolver.config().cache_ttl,
        Duration::from_secs(1800)
    );
    assert_eq!(custom_resolver.config().timeout, Duration::from_secs(60));
}

#[tokio::test]
async fn test_static_overrides() {
    let resolver = create_test_resolver();

    // Package resolution should use override
    let result = resolver.resolve_package("@test/package").await.unwrap();
    assert_eq!(result, "0x111111111");
    assert_valid_address(&result);

    // Type resolution should use override
    let result = resolver
        .resolve_type("@test/package::module::TestType")
        .await
        .unwrap();
    assert_eq!(result, "0x111111111::module::TestType");
    assert_valid_type_signature(&result);
}

#[tokio::test]
async fn test_cache_functionality() {
    let resolver = MvrResolver::testnet();

    // Initially cache should be empty
    let stats = resolver.cache_stats().unwrap();
    assert_eq!(stats.total_entries, 0);

    // Test cache clearing
    resolver.clear_cache().unwrap();

    let stats_after_clear = resolver.cache_stats().unwrap();
    assert_eq!(stats_after_clear.total_entries, 0);
}

#[tokio::test]
async fn test_batch_operations() {
    let overrides = create_batch_test_overrides();
    let resolver = MvrResolver::testnet().with_overrides(overrides);

    // Test batch package resolution
    let package_names = vec!["@batch/pkg1", "@batch/pkg2", "@batch/pkg3"];
    let results = resolver.resolve_packages(&package_names).await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results.get("@batch/pkg1"), Some(&"0x111".to_string()));
    assert_eq!(results.get("@batch/pkg2"), Some(&"0x222".to_string()));
    assert_eq!(results.get("@batch/pkg3"), Some(&"0x333".to_string()));

    // Validate all addresses
    for address in results.values() {
        assert_valid_address(address);
    }

    // Test batch type resolution
    let type_names = vec!["@batch/pkg1::module::Type1", "@batch/pkg2::module::Type2"];
    let results = resolver.resolve_types(&type_names).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(
        results.get("@batch/pkg1::module::Type1"),
        Some(&"0x111::module::Type1".to_string())
    );
    assert_eq!(
        results.get("@batch/pkg2::module::Type2"),
        Some(&"0x222::module::Type2".to_string())
    );

    // Validate all type signatures
    for type_sig in results.values() {
        assert_valid_type_signature(type_sig);
    }

    // Test empty batch operations
    let empty_results = resolver.resolve_packages(&[]).await.unwrap();
    assert!(empty_results.is_empty());

    let empty_results = resolver.resolve_types(&[]).await.unwrap();
    assert!(empty_results.is_empty());
}

#[tokio::test]
async fn test_package_name_validation() {
    let resolver = MvrResolver::testnet();

    // Test invalid package names
    for invalid_name in invalid_package_names() {
        let result = resolver.resolve_package(invalid_name).await;
        assert!(
            result.is_err(),
            "Should reject invalid package name: {}",
            invalid_name
        );

        if let Err(e) = result {
            assert!(matches!(e, MvrError::InvalidPackageName(_)));
            test_error_properties(&e, false, true);
        }
    }

    // Test valid package name formats (these will fail resolution but pass validation)
    let resolver_with_overrides = create_test_resolver();
    for valid_name in valid_package_names() {
        // If it's in our test overrides, it should succeed
        if valid_name == "@suifrens/core" || valid_name == "@test/package" {
            let result = resolver_with_overrides.resolve_package(valid_name).await;
            assert!(
                result.is_ok(),
                "Should accept valid package name: {}",
                valid_name
            );
        } else {
            // For names not in overrides, just test that validation passes
            // by checking that the error is NOT InvalidPackageName
            let result = resolver_with_overrides.resolve_package(valid_name).await;
            if let Err(e) = result {
                assert!(
                    !matches!(e, MvrError::InvalidPackageName(_)),
                    "Should not reject valid package name format: {}",
                    valid_name
                );
            }
        }
    }
}

#[tokio::test]
async fn test_type_name_validation() {
    let resolver = MvrResolver::testnet();

    // Test invalid type names
    for invalid_type in invalid_type_names() {
        let result = resolver.resolve_type(invalid_type).await;
        assert!(
            result.is_err(),
            "Should reject invalid type name: {}",
            invalid_type
        );

        if let Err(e) = result {
            assert!(matches!(e, MvrError::InvalidTypeName(_)));
            test_error_properties(&e, false, true);
        }
    }

    // Test valid type name formats with overrides
    let resolver_with_overrides = create_test_resolver();
    let result = resolver_with_overrides
        .resolve_type("@suifrens/core::suifren::SuiFren")
        .await;
    assert!(result.is_ok(), "Should accept valid type name");
}

#[tokio::test]
async fn test_mvr_target_resolution() {
    use sui_mvr::resolver::resolve_mvr_target;

    let resolver = create_test_resolver();

    // Test non-MVR target (should pass through)
    let normal_target = "0x123::module::function";
    let result = resolve_mvr_target(&resolver, normal_target).await.unwrap();
    assert_eq!(result, normal_target);

    // Test MVR target resolution
    let mvr_target = "@test/package::module::function";
    let result = resolve_mvr_target(&resolver, mvr_target).await.unwrap();
    assert_eq!(result, "0x111111111::module::function");

    // Test invalid MVR target format
    let invalid_target = "@invalid-format";
    let result = resolve_mvr_target(&resolver, invalid_target).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_overrides_serialization() {
    let original_overrides = create_batch_test_overrides();

    // Test JSON serialization
    let json = original_overrides.to_json().unwrap();
    assert!(json.contains("@batch/pkg1"));
    assert!(json.contains("0x111"));

    // Test JSON deserialization
    let deserialized_overrides = MvrOverrides::from_json(&json).unwrap();
    assert_eq!(original_overrides.packages, deserialized_overrides.packages);
    assert_eq!(original_overrides.types, deserialized_overrides.types);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let resolver = create_test_resolver();

    // Spawn multiple concurrent resolution tasks
    let tasks = vec![
        tokio::spawn({
            let resolver = resolver.clone();
            async move { resolver.resolve_package("@suifrens/core").await }
        }),
        tokio::spawn({
            let resolver = resolver.clone();
            async move { resolver.resolve_package("@suifrens/accessories").await }
        }),
        tokio::spawn({
            let resolver = resolver.clone();
            async move { resolver.resolve_package("@test/package").await }
        }),
    ];

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all tasks completed successfully
    for result in results {
        let resolution_result = result.unwrap();
        assert!(
            resolution_result.is_ok(),
            "Concurrent resolution should succeed"
        );

        if let Ok(address) = resolution_result {
            assert_valid_address(&address);
        }
    }
}

#[tokio::test]
async fn test_error_types_and_properties() {
    // Test different error types and their properties
    let package_not_found = MvrError::PackageNotFound("test".to_string());
    test_error_properties(&package_not_found, false, true);
    assert!(!package_not_found.is_rate_limited());

    let rate_limited = MvrError::RateLimitExceeded {
        retry_after_secs: 60,
    };
    test_error_properties(&rate_limited, true, false); // Rate limits are retryable
    assert!(rate_limited.is_rate_limited());
    assert_eq!(rate_limited.retry_delay(), Some(Duration::from_secs(60)));

    let timeout = MvrError::Timeout { timeout_secs: 30 };
    test_error_properties(&timeout, true, false);
    assert!(!timeout.is_rate_limited());

    let invalid_name = MvrError::InvalidPackageName("bad-name".to_string());
    test_error_properties(&invalid_name, false, true);
}

#[tokio::test]
async fn test_cache_statistics() {
    let resolver = create_test_resolver();

    // Initially empty cache
    let initial_stats = resolver.cache_stats().unwrap();
    assert_eq!(initial_stats.total_entries, 0);
    assert_eq!(initial_stats.valid_entries, 0);

    // Perform some resolutions to populate cache
    let _ = resolver.resolve_package("@suifrens/core").await.unwrap();
    let _ = resolver
        .resolve_package("@suifrens/accessories")
        .await
        .unwrap();

    // Note: Since we're using overrides, these don't actually go to cache in this implementation
    // but in a real implementation, this would test cache population

    // Test cache cleanup - just verify it doesn't error
    let _cleaned = resolver.cleanup_expired_cache().unwrap();
}

#[tokio::test]
async fn test_comprehensive_workflow() {
    let resolver = create_test_resolver();

    // Test individual package resolution
    let core_pkg = resolver.resolve_package("@suifrens/core").await.unwrap();
    assert_eq!(core_pkg, "0x123456789");
    assert_valid_address(&core_pkg);

    // Test individual type resolution
    let suifren_type = resolver
        .resolve_type("@suifrens/core::suifren::SuiFren")
        .await
        .unwrap();
    assert_eq!(suifren_type, "0x123456789::suifren::SuiFren");
    assert_valid_type_signature(&suifren_type);

    // Test batch operations
    let packages = vec!["@suifrens/core", "@suifrens/accessories"];
    let batch_results = resolver.resolve_packages(&packages).await.unwrap();
    assert_eq!(batch_results.len(), 2);

    // Test cache statistics - just verify it doesn't error
    let _stats = resolver.cache_stats().unwrap();

    // Test cache cleanup - just verify it doesn't error
    let _cleaned = resolver.cleanup_expired_cache().unwrap();
}

#[tokio::test]
async fn test_performance_comparison() {
    let resolver = create_test_resolver();
    let packages = vec!["@suifrens/core", "@suifrens/accessories", "@test/package"];

    // Test individual resolution timing
    let start = std::time::Instant::now();
    for &pkg in &packages {
        let _ = resolver.resolve_package(pkg).await.unwrap();
    }
    let individual_duration = start.elapsed();

    // Test batch resolution timing
    let start = std::time::Instant::now();
    let batch_results = resolver.resolve_packages(&packages).await.unwrap();
    let batch_duration = start.elapsed();

    // Batch should resolve all packages
    assert_eq!(batch_results.len(), packages.len());

    // In a real implementation with network calls, batch would be faster
    // For now, just ensure both methods work
    println!(
        "Individual: {:?}, Batch: {:?}",
        individual_duration, batch_duration
    );
}

// Note: These tests use overrides to avoid making real network calls.
// To test with a real MVR server, you would:
// 1. Set up environment variables for test endpoints
// 2. Use actual package names that exist in the registry
// 3. Handle rate limiting and network timeouts
// 4. Mock the HTTP client for deterministic testing
