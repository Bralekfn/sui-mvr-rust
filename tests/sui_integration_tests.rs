//! Integration tests for Sui SDK compatibility
//!
//! These tests verify that sui-mvr works correctly with the official Sui SDK

#[cfg(feature = "sui-integration")]
mod sui_integration_tests {
    use anyhow::Result;
    use std::str::FromStr;
    use sui_mvr::prelude::*;
    use sui_mvr::sui_integration::{utils, MvrResolverExt};
    use sui_sdk::{
        types::{
            base_types::{ObjectID, SuiAddress},
            programmable_transaction_builder::ProgrammableTransactionBuilder,
            transaction::{Argument, Command, ProgrammableMoveCall},
            Identifier, TypeTag,
        },
        SuiClientBuilder,
    };

    fn create_test_resolver_with_real_addresses() -> MvrResolver {
        // Use realistic Sui mainnet addresses for testing
        let overrides = MvrOverrides::new()
            .with_package(
                "@sui/framework".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .with_package(
                "@test/package".to_string(),
                "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string(),
            )
            .with_package(
                "@defi/pool".to_string(),
                "0xabcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef".to_string(),
            )
            .with_type(
                "@sui/framework::coin::Coin".to_string(),
                "0x2::coin::Coin<T>".to_string(),
            );

        MvrResolver::testnet().with_overrides(overrides)
    }

    #[tokio::test]
    async fn test_sui_client_integration() -> Result<()> {
        // Test that we can connect to Sui testnet
        let sui_client = SuiClientBuilder::default().build_testnet().await?;

        // Verify connection
        let version = sui_client.api_version();
        assert!(
            !version.is_empty(),
            "Should get API version from Sui client"
        );

        println!("✅ Connected to Sui testnet, API version: {}", version);
        Ok(())
    }

    #[tokio::test]
    async fn test_mvr_target_resolution() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Test valid MVR target resolution
        let (package_id, module, function) = resolver
            .resolve_mvr_target("@test/package::transfer::send")
            .await?;

        assert_eq!(module, "transfer");
        assert_eq!(function, "send");
        assert_eq!(
            package_id.to_hex_literal(),
            "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234"
        );

        println!("✅ MVR target resolution successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_move_call_transaction_building() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Build a move call transaction
        let ptb = resolver
            .build_move_call_transaction(
                "@sui/framework",
                "coin",
                "mint",
                vec![], // Type arguments
                vec![], // Function arguments
            )
            .await?;

        let tx = ptb.finish();
        assert_eq!(tx.commands.len(), 1);

        // Verify the move call was built correctly
        if let Command::MoveCall(move_call) = &tx.commands[0] {
            assert_eq!(
                move_call.package.to_hex_literal(),
                "0x0000000000000000000000000000000000000000000000000000000000000002"
            );
            assert_eq!(move_call.module.as_str(), "coin");
            assert_eq!(move_call.function.as_str(), "mint");
        } else {
            panic!("Expected MoveCall command");
        }

        println!("✅ Move call transaction building successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_batch_package_resolution_as_object_ids() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        let package_names = vec!["@sui/framework", "@test/package", "@defi/pool"];
        let results = resolver
            .resolve_packages_as_object_ids(&package_names)
            .await?;

        assert_eq!(results.len(), 3);

        // Verify all packages were resolved correctly
        let mut found_sui = false;
        let mut found_test = false;
        let mut found_defi = false;

        for (name, object_id) in &results {
            match name.as_str() {
                "@sui/framework" => {
                    assert_eq!(
                        object_id.to_hex_literal(),
                        "0x0000000000000000000000000000000000000000000000000000000000000002"
                    );
                    found_sui = true;
                }
                "@test/package" => {
                    assert_eq!(
                        object_id.to_hex_literal(),
                        "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234"
                    );
                    found_test = true;
                }
                "@defi/pool" => {
                    assert_eq!(
                        object_id.to_hex_literal(),
                        "0xabcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef"
                    );
                    found_defi = true;
                }
                _ => panic!("Unexpected package name: {}", name),
            }
        }

        assert!(found_sui, "Should find @sui/framework");
        assert!(found_test, "Should find @test/package");
        assert!(found_defi, "Should find @defi/pool");

        println!("✅ Batch package resolution as ObjectIDs successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_complex_transaction_building() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Build a transaction with multiple move calls
        let mut ptb = ProgrammableTransactionBuilder::new();

        // First call: get coins
        let (coin_package, coin_module, coin_function) = resolver
            .resolve_mvr_target("@sui/framework::coin::mint")
            .await?;

        let mint_call = ProgrammableMoveCall {
            package: coin_package,
            module: Identifier::new(&coin_module)?,
            function: Identifier::new(&coin_function)?,
            type_arguments: vec![],
            arguments: vec![],
        };
        ptb.command(Command::MoveCall(Box::new(mint_call)));

        // Second call: transfer to DeFi pool
        let (defi_package, defi_module, defi_function) = resolver
            .resolve_mvr_target("@defi/pool::liquidity::add")
            .await?;

        let add_liquidity_call = ProgrammableMoveCall {
            package: defi_package,
            module: Identifier::new(&defi_module)?,
            function: Identifier::new(&defi_function)?,
            type_arguments: vec![],
            arguments: vec![
                Argument::Result(0), // Use result from first call
            ],
        };
        ptb.command(Command::MoveCall(Box::new(add_liquidity_call)));

        let tx = ptb.finish();
        assert_eq!(tx.commands.len(), 2);

        println!("✅ Complex transaction building successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_utility_functions() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Test create_pure_arg utility
        let amount = 1000u64;
        let pure_arg = utils::create_pure_arg(&amount)?;

        // Verify it's a Pure CallArg
        match pure_arg {
            sui_sdk::types::transaction::CallArg::Pure(bytes) => {
                // Verify the bytes can be deserialized back to the original value
                let deserialized: u64 = bcs::from_bytes(&bytes)?;
                assert_eq!(deserialized, amount);
            }
            _ => panic!("Expected Pure CallArg"),
        }

        // Test batch transaction creation
        let calls = vec![
            ("@sui/framework", "coin", "mint"),
            ("@test/package", "transfer", "send"),
        ];

        let batch_ptb = utils::create_batch_transaction(&resolver, &calls).await?;
        let batch_tx = batch_ptb.finish();

        assert_eq!(batch_tx.commands.len(), 2);
        for command in &batch_tx.commands {
            assert!(matches!(command, Command::MoveCall(_)));
        }

        println!("✅ Utility functions testing successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_integration() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Test invalid MVR target formats
        let invalid_targets = vec![
            "@test/package",           // Missing module::function
            "test/package::mod::func", // Missing @
            "@test/package::func",     // Missing module
        ];

        for invalid_target in invalid_targets {
            let result = resolver.resolve_mvr_target(invalid_target).await;
            assert!(
                result.is_err(),
                "Should reject invalid target: {}",
                invalid_target
            );

            if let Err(e) = result {
                assert!(matches!(e, MvrError::InvalidPackageName(_)));
            }
        }

        // Test non-existent package
        let result = resolver
            .build_move_call_transaction(
                "@nonexistent/package",
                "module",
                "function",
                vec![],
                vec![],
            )
            .await;

        assert!(result.is_err(), "Should fail for non-existent package");

        println!("✅ Error handling integration successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_type_compatibility() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Test that ObjectID conversion works correctly
        let package_address = resolver.resolve_package("@sui/framework").await?;
        let object_id = ObjectID::from_hex_literal(&package_address)?;

        // Verify round-trip conversion
        let hex_literal = object_id.to_hex_literal();
        assert_eq!(hex_literal, package_address);

        // Test SuiAddress compatibility
        let test_address = "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234";
        let sui_address = SuiAddress::from_str(test_address)?;
        let address_bytes = bcs::to_bytes(&sui_address)?;

        // Verify the address can be used in CallArg::Pure
        let _pure_arg = sui_sdk::types::transaction::CallArg::Pure(address_bytes);

        println!("✅ Type compatibility testing successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_with_sui_integration() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Measure individual resolution time
        let start = std::time::Instant::now();
        let _address1 = resolver.resolve_package("@sui/framework").await?;
        let _address2 = resolver.resolve_package("@test/package").await?;
        let _address3 = resolver.resolve_package("@defi/pool").await?;
        let individual_duration = start.elapsed();

        // Measure batch resolution time
        let packages = vec!["@sui/framework", "@test/package", "@defi/pool"];
        let start = std::time::Instant::now();
        let _batch_results = resolver.resolve_packages_as_object_ids(&packages).await?;
        let batch_duration = start.elapsed();

        println!("⚡ Performance comparison:");
        println!("   Individual: {:?}", individual_duration);
        println!("   Batch: {:?}", batch_duration);

        // Batch should typically be faster for multiple resolutions
        // (though with overrides/cache, the difference might be minimal)

        println!("✅ Performance testing with Sui integration successful");
        Ok(())
    }

    #[tokio::test]
    async fn test_real_world_transaction_pattern() -> Result<()> {
        let resolver = create_test_resolver_with_real_addresses();

        // Simulate a real-world DeFi transaction pattern:
        // 1. Resolve multiple DeFi packages
        // 2. Build a complex transaction
        // 3. Verify transaction structure

        // Step 1: Resolve packages
        let packages = vec!["@sui/framework", "@defi/pool"];
        let resolved = resolver.resolve_packages_as_object_ids(&packages).await?;

        // Step 2: Build transaction using resolved packages
        let mut ptb = ProgrammableTransactionBuilder::new();

        for (package_name, package_id) in resolved {
            let (module, function) = match package_name.as_str() {
                "@sui/framework" => ("coin", "mint"),
                "@defi/pool" => ("liquidity", "add"),
                _ => continue,
            };

            let move_call = ProgrammableMoveCall {
                package: package_id,
                module: Identifier::new(module)?,
                function: Identifier::new(function)?,
                type_arguments: vec![],
                arguments: vec![],
            };
            ptb.command(Command::MoveCall(Box::new(move_call)));
        }

        // Step 3: Verify transaction
        let tx = ptb.finish();
        assert_eq!(tx.commands.len(), 2);

        // Verify each command is a MoveCall
        for command in &tx.commands {
            assert!(matches!(command, Command::MoveCall(_)));
        }

        println!("✅ Real-world transaction pattern testing successful");
        Ok(())
    }
}

// Tests that run without sui-integration feature
#[cfg(not(feature = "sui-integration"))]
mod basic_tests {
    use super::*;
    use sui_mvr::prelude::*;

    #[tokio::test]
    async fn test_basic_functionality_without_sui_integration() {
        let overrides =
            MvrOverrides::new().with_package("@test/package".to_string(), "0x123456".to_string());

        let resolver = MvrResolver::testnet().with_overrides(overrides);

        let result = resolver.resolve_package("@test/package").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x123456");

        println!("✅ Basic functionality works without sui-integration feature");
    }
}
