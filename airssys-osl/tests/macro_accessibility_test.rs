//! Test to verify #[executor] macro is accessible via prelude

#![cfg(feature = "macros")]

#[test]
fn test_macro_is_accessible_via_prelude() {
    // This test verifies that the #[executor] macro can be imported via the prelude
    // If this compiles, the macro is accessible
    #[allow(unused_imports)]
    use airssys_osl::prelude::executor;

    // The macro attribute is now in scope and can be used
    // No assertion needed - compilation success proves accessibility
}

// Note: Full integration tests with actual executor implementations
// will be added in Phase 2 of MACROS-TASK-003
