//! Integration utilities for the official Sui Rust SDK
//!
//! This module provides helper functions and utilities to seamlessly integrate
//! MVR resolution with the official Sui SDK transaction building.

#[cfg(feature = "sui-integration")]
use crate::{MvrResolver, MvrResult};

#[cfg(feature = "sui-integration")]
use sui_sdk::types::{
    base_types::ObjectID,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ProgrammableMoveCall},
    Identifier,
};

#[cfg(feature = "sui-integration")]
/// Extension trait for MvrResolver to provide Sui SDK integration helpers
pub trait MvrResolverExt {
    /// Resolve an MVR target format for move calls
    /// 
    /// Converts "@package::module::function" format into components
    /// suitable for building Sui transactions.
    /// 
    /// # Arguments
    /// * `target` - MVR target in format "@namespace/package::module::function"
    /// 
    /// # Returns
    /// * `Ok((package_id, module, function))` - Resolved components
    /// * `Err(MvrError)` - If resolution or parsing fails
    /// 
    /// # Example
    /// ```rust,no_run
    /// use sui_mvr::prelude::*;
    /// use sui_mvr::sui_integration::MvrResolverExt;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = MvrResolver::testnet();
    /// let (package_id, module, function) = resolver
    ///     .resolve_mvr_target("@suifrens/core::mint::new_suifren")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn resolve_mvr_target(&self, target: &str) -> MvrResult<(ObjectID, String, String)>;

    /// Build a simple move call transaction using MVR package resolution
    /// 
    /// # Arguments
    /// * `package_name` - MVR package name (e.g., "@suifrens/core")
    /// * `module` - Module name
    /// * `function` - Function name
    /// * `type_arguments` - Type arguments for the move call
    /// * `arguments` - Function arguments
    /// 
    /// # Returns
    /// * `Ok(ProgrammableTransactionBuilder)` - Transaction builder with the move call
    /// * `Err(MvrError)` - If resolution fails
    async fn build_move_call_transaction(
        &self,
        package_name: &str,
        module: &str,
        function: &str,
        type_arguments: Vec<sui_sdk::types::TypeTag>,
        arguments: Vec<Argument>,
    ) -> MvrResult<ProgrammableTransactionBuilder>;

    /// Resolve multiple packages and return them as ObjectIDs for transaction building
    /// 
    /// # Arguments
    /// * `package_names` - Vector of MVR package names
    /// 
    /// # Returns
    /// * `Ok(Vec<(String, ObjectID)>)` - Pairs of package names and resolved ObjectIDs
    /// * `Err(MvrError)` - If any resolution fails
    async fn resolve_packages_as_object_ids(
        &self,
        package_names: &[&str],
    ) -> MvrResult<Vec<(String, ObjectID)>>;
}

#[cfg(feature = "sui-integration")]
impl MvrResolverExt for MvrResolver {
    async fn resolve_mvr_target(&self, target: &str) -> MvrResult<(ObjectID, String, String)> {
        if !target.starts_with('@') {
            return Err(crate::MvrError::InvalidPackageName(format!(
                "MVR target must start with '@': {}",
                target
            )));
        }

        // Parse format: @package::module::function
        let parts: Vec<&str> = target.splitn(2, "::").collect();
        if parts.len() != 2 {
            return Err(crate::MvrError::InvalidPackageName(format!(
                "Invalid MVR target format. Expected '@package::module::function', got: {}",
                target
            )));
        }

        let package_part = parts[0];
        let module_function = parts[1];

        // Split module::function
        let module_parts: Vec<&str> = module_function.splitn(2, "::").collect();
        if module_parts.len() != 2 {
            return Err(crate::MvrError::InvalidPackageName(format!(
                "Invalid module::function format. Expected 'module::function', got: {}",
                module_function
            )));
        }

        // Resolve the package address
        let package_address = self.resolve_package(package_part).await?;
        let package_id = ObjectID::from_hex_literal(&package_address).map_err(|e| {
            crate::MvrError::ConfigError(format!("Invalid package address '{}': {}", package_address, e))
        })?;

        Ok((
            package_id,
            module_parts[0].to_string(),
            module_parts[1].to_string(),
        ))
    }

    async fn build_move_call_transaction(
        &self,
        package_name: &str,
        module: &str,
        function: &str,
        type_arguments: Vec<sui_sdk::types::TypeTag>,
        arguments: Vec<Argument>,
    ) -> MvrResult<ProgrammableTransactionBuilder> {
        // Resolve the package address
        let package_address = self.resolve_package(package_name).await?;
        let package_id = ObjectID::from_hex_literal(&package_address).map_err(|e| {
            crate::MvrError::ConfigError(format!("Invalid package address '{}': {}", package_address, e))
        })?;

        // Create the transaction builder
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Create the move call
        let move_call = ProgrammableMoveCall {
            package: package_id,
            module: Identifier::new(module).map_err(|e| {
                crate::MvrError::ConfigError(format!("Invalid module name '{}': {}", module, e))
            })?,
            function: Identifier::new(function).map_err(|e| {
                crate::MvrError::ConfigError(format!("Invalid function name '{}': {}", function, e))
            })?,
            type_arguments,
            arguments,
        };

        // Add the move call to the transaction
        ptb.command(Command::MoveCall(Box::new(move_call)));

        Ok(ptb)
    }

    async fn resolve_packages_as_object_ids(
        &self,
        package_names: &[&str],
    ) -> MvrResult<Vec<(String, ObjectID)>> {
        // Use batch resolution for performance
        let resolved_packages = self.resolve_packages(package_names).await?;

        let mut result = Vec::new();
        for (name, address) in resolved_packages {
            let package_id = ObjectID::from_hex_literal(&address).map_err(|e| {
                crate::MvrError::ConfigError(format!("Invalid package address '{}': {}", address, e))
            })?;
            result.push((name, package_id));
        }

        Ok(result)
    }
}

#[cfg(feature = "sui-integration")]
/// Utility functions for common Sui transaction patterns with MVR
pub mod utils {
    use super::*;
    use crate::{MvrResolver, MvrResult};
    use sui_sdk::types::{
        base_types::SuiAddress,
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, CallArg, Command},
    };

    /// Create a transaction that transfers objects using MVR-resolved package calls
    /// 
    /// # Example
    /// ```rust,no_run
    /// use sui_mvr::prelude::*;
    /// use sui_mvr::sui_integration::utils;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = MvrResolver::testnet();
    /// let recipient = "0x123...".parse()?;
    /// 
    /// let ptb = utils::create_transfer_transaction(
    ///     &resolver,
    ///     "@myapp/nft",
    ///     "transfer",
    ///     "transfer_nft",
    ///     vec![], // type arguments
    ///     vec![], // additional arguments
    ///     recipient,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_transfer_transaction(
        resolver: &MvrResolver,
        package_name: &str,
        module: &str,
        function: &str,
        type_arguments: Vec<sui_sdk::types::TypeTag>,
        mut arguments: Vec<Argument>,
        recipient: SuiAddress,
    ) -> MvrResult<ProgrammableTransactionBuilder> {
        use super::MvrResolverExt;

        // Add recipient as the last argument
        let mut ptb = ProgrammableTransactionBuilder::new();
        
        // Create pure argument for recipient address
        let recipient_arg = CallArg::Pure(bcs::to_bytes(&recipient).map_err(|e| {
            crate::MvrError::ConfigError(format!("Failed to serialize recipient address: {}", e))
        })?);
        ptb.input(recipient_arg)?;
        arguments.push(Argument::Input(arguments.len() as u16));

        // Build the move call transaction
        let mut move_call_ptb = resolver
            .build_move_call_transaction(package_name, module, function, type_arguments, arguments)
            .await?;

        // Merge the transactions
        let programmable_tx = move_call_ptb.finish();
        for input in programmable_tx.inputs {
            ptb.input(input)?;
        }
        for command in programmable_tx.commands {
            ptb.command(command);
        }

        Ok(ptb)
    }

    /// Create a batch transaction that calls multiple MVR-resolved packages
    /// 
    /// # Example
    /// ```rust,no_run
    /// use sui_mvr::prelude::*;
    /// use sui_mvr::sui_integration::utils;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = MvrResolver::testnet();
    /// 
    /// let calls = vec![
    ///     ("@defi/pool", "liquidity", "add_liquidity"),
    ///     ("@nft/marketplace", "trading", "list_item"),
    /// ];
    /// 
    /// let ptb = utils::create_batch_transaction(&resolver, &calls).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_batch_transaction(
        resolver: &MvrResolver,
        calls: &[(&str, &str, &str)], // (package_name, module, function)
    ) -> MvrResult<ProgrammableTransactionBuilder> {
        use super::MvrResolverExt;

        let mut ptb = ProgrammableTransactionBuilder::new();

        for (package_name, module, function) in calls {
            // Build individual move call transaction
            let individual_ptb = resolver
                .build_move_call_transaction(
                    package_name,
                    module,
                    function,
                    vec![], // No type arguments for this example
                    vec![], // No arguments for this example
                )
                .await?;

            // Add its commands to the batch transaction
            let programmable_tx = individual_ptb.finish();
            for command in programmable_tx.commands {
                ptb.command(command);
            }
        }

        Ok(ptb)
    }

    /// Helper to create pure arguments for common types
    pub fn create_pure_arg<T: serde::Serialize>(value: &T) -> MvrResult<CallArg> {
        let bytes = bcs::to_bytes(value).map_err(|e| {
            crate::MvrError::ConfigError(format!("Failed to serialize argument: {}", e))
        })?;
        Ok(CallArg::Pure(bytes))
    }
}

#[cfg(all(test, feature = "sui-integration"))]
mod tests {
    use super::*;
    use crate::{MvrOverrides, MvrResolver};

    fn create_test_resolver() -> MvrResolver {
        let overrides = MvrOverrides::new()
            .with_package(
                "@test/package".to_string(),
                "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234".to_string(),
            )
            .with_package(
                "@defi/pool".to_string(),
                "0xabcdef123456789abcdef123456789abcdef123456789abcdef123456789abcdef".to_string(),
            );

        MvrResolver::testnet().with_overrides(overrides)
    }

    #[tokio::test]
    async fn test_resolve_mvr_target() {
        let resolver = create_test_resolver();

        let (package_id, module, function) = resolver
            .resolve_mvr_target("@test/package::module::function")
            .await
            .unwrap();

        assert_eq!(module, "module");
        assert_eq!(function, "function");
        assert_eq!(
            package_id.to_hex_literal(),
            "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234"
        );
    }

    #[tokio::test]
    async fn test_resolve_mvr_target_invalid_format() {
        let resolver = create_test_resolver();

        // Missing module::function
        let result = resolver.resolve_mvr_target("@test/package").await;
        assert!(result.is_err());

        // Missing @
        let result = resolver.resolve_mvr_target("test/package::module::function").await;
        assert!(result.is_err());

        // Invalid module::function format
        let result = resolver.resolve_mvr_target("@test/package::function").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_build_move_call_transaction() {
        let resolver = create_test_resolver();

        let ptb = resolver
            .build_move_call_transaction(
                "@test/package",
                "module",
                "function",
                vec![],
                vec![],
            )
            .await
            .unwrap();

        let tx = ptb.finish();
        assert_eq!(tx.commands.len(), 1);

        if let Command::MoveCall(move_call) = &tx.commands[0] {
            assert_eq!(
                move_call.package.to_hex_literal(),
                "0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234"
            );
            assert_eq!(move_call.module.as_str(), "module");
            assert_eq!(move_call.function.as_str(), "function");
        } else {
            panic!("Expected MoveCall command");
        }
    }

    #[tokio::test]
    async fn test_resolve_packages_as_object_ids() {
        let resolver = create_test_resolver();

        let package_names = vec!["@test/package", "@defi/pool"];
        let results = resolver.resolve_packages_as_object_ids(&package_names).await.unwrap();

        assert_eq!(results.len(), 2);
        
        let (name1, id1) = &results[0];
        let (name2, id2) = &results[1];

        assert!(name1 == "@test/package" || name1 == "@defi/pool");
        assert!(name2 == "@test/package" || name2 == "@defi/pool");
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_create_batch_transaction() {
        let resolver = create_test_resolver();

        let calls = vec![
            ("@test/package", "module1", "function1"),
            ("@defi/pool", "module2", "function2"),
        ];

        let ptb = utils::create_batch_transaction(&resolver, &calls).await.unwrap();
        let tx = ptb.finish();

        assert_eq!(tx.commands.len(), 2);
        for command in &tx.commands {
            assert!(matches!(command, Command::MoveCall(_)));
        }
    }
}