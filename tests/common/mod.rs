use std::collections::HashMap;
use sui_mvr::prelude::*;

/// Create a test resolver with common overrides for testing
pub fn create_test_resolver() -> MvrResolver {
    let overrides = MvrOverrides::new()
        .with_package("@suifrens/core".to_string(), "0x123456789".to_string())
        .with_package(
            "@suifrens/accessories".to_string(),
            "0x987654321".to_string(),
        )
        .with_package("@test/package".to_string(), "0x111111111".to_string())
        .with_type(
            "@suifrens/core::suifren::SuiFren".to_string(),
            "0x123456789::suifren::SuiFren".to_string(),
        )
        .with_type(
            "@suifrens/core::bullshark::Bullshark".to_string(),
            "0x123456789::bullshark::Bullshark".to_string(),
        )
        .with_type(
            "@test/package::TestType".to_string(),
            "0x111111111::module::TestType".to_string(),
        );

    MvrResolver::testnet().with_overrides(overrides)
}

/// Create overrides for batch testing
pub fn create_batch_test_overrides() -> MvrOverrides {
    let mut packages = HashMap::new();
    packages.insert("@batch/pkg1".to_string(), "0x111".to_string());
    packages.insert("@batch/pkg2".to_string(), "0x222".to_string());
    packages.insert("@batch/pkg3".to_string(), "0x333".to_string());
    packages.insert("@batch/pkg4".to_string(), "0x444".to_string());
    packages.insert("@batch/pkg5".to_string(), "0x555".to_string());

    let mut types = HashMap::new();
    types.insert(
        "@batch/pkg1::Type1".to_string(),
        "0x111::module::Type1".to_string(),
    );
    types.insert(
        "@batch/pkg2::Type2".to_string(),
        "0x222::module::Type2".to_string(),
    );
    types.insert(
        "@batch/pkg3::Type3".to_string(),
        "0x333::module::Type3".to_string(),
    );

    MvrOverrides { packages, types }
}

/// Test package names for validation testing
pub fn invalid_package_names() -> Vec<&'static str> {
    vec![
        "invalid-name",  // Missing @
        "@incomplete",   // Missing /
        "@ns/",          // Empty package name
        "@/pkg",         // Empty namespace
        "",              // Empty string
        "@ns/pkg/extra", // Too many parts
        "@",             // Just @
        "/pkg",          // Missing @
    ]
}

/// Test type names for validation testing
pub fn invalid_type_names() -> Vec<&'static str> {
    vec![
        "invalid-type",         // Missing @
        "@ns/pkg",              // Missing ::
        "@ns/pkg::Type",        // Not enough parts
        "ns/pkg::module::Type", // Missing @
        "@ns/pkg:Type",         // Wrong separator
        "@ns/pkg::module:",     // Empty type name
        "",                     // Empty string
    ]
}

/// Valid test package names
pub fn valid_package_names() -> Vec<&'static str> {
    vec![
        "@suifrens/core",
        "@suifrens/accessories",
        "@namespace/package",
        "@test/pkg",
        "@a/b",
    ]
}

/// Valid test type names
pub fn valid_type_names() -> Vec<&'static str> {
    vec![
        "@suifrens/core::suifren::SuiFren",
        "@suifrens/core::bullshark::Bullshark",
        "@namespace/package::module::Type",
        "@test/pkg::mod::T",
        "@a/b::c::D",
        "@ns/pkg::module::Type<T>",
        "@ns/pkg::module::Generic<A, B>",
    ]
}

/// Assert that a package address is valid (starts with 0x and has appropriate length)
pub fn assert_valid_address(address: &str) {
    assert!(
        address.starts_with("0x"),
        "Address should start with 0x: {}",
        address
    );
    assert!(
        address.len() >= 3,
        "Address should be longer than just 0x: {}",
        address
    );

    // Check that the rest are valid hex characters
    let hex_part = &address[2..];
    for c in hex_part.chars() {
        assert!(
            c.is_ascii_hexdigit(),
            "Invalid hex character in address: {}",
            address
        );
    }
}

/// Assert that a type signature is valid
pub fn assert_valid_type_signature(type_sig: &str) {
    assert!(
        type_sig.contains("::"),
        "Type signature should contain :: separator: {}",
        type_sig
    );

    // Should start with 0x address
    if !type_sig.starts_with("0x") {
        panic!("Type signature should start with address: {}", type_sig);
    }

    // Should have at least module::Type format after address
    let parts: Vec<&str> = type_sig.split("::").collect();
    assert!(
        parts.len() >= 3,
        "Type signature should have at least address::module::Type: {}",
        type_sig
    );
}

/// Utility for testing error properties
pub fn test_error_properties(
    error: &MvrError,
    expected_retryable: bool,
    expected_client_error: bool,
) {
    assert_eq!(
        error.is_retryable(),
        expected_retryable,
        "Error retryable property mismatch for: {:?}",
        error
    );
    assert_eq!(
        error.is_client_error(),
        expected_client_error,
        "Error client_error property mismatch for: {:?}",
        error
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_functions() {
        // Test address validation
        assert_valid_address("0x123456");
        assert_valid_address("0xabcdef");

        // Test type signature validation
        assert_valid_type_signature("0x123::module::Type");
        assert_valid_type_signature("0x456::test::Generic<T>");
    }

    #[test]
    #[should_panic]
    fn test_invalid_address() {
        assert_valid_address("123456"); // Missing 0x
    }

    #[test]
    #[should_panic]
    fn test_invalid_type_signature() {
        assert_valid_type_signature("invalid"); // No :: separator
    }

    #[test]
    fn test_test_data() {
        assert!(!invalid_package_names().is_empty());
        assert!(!invalid_type_names().is_empty());
        assert!(!valid_package_names().is_empty());
        assert!(!valid_type_names().is_empty());
    }

    #[tokio::test]
    async fn test_create_test_resolver() {
        let resolver = create_test_resolver();

        // Should be able to resolve test packages
        let result = resolver.resolve_package("@test/package").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x111111111");
    }
}
