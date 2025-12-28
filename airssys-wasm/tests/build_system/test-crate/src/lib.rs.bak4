// Test crate to validate wit-bindgen multi-package binding generation

// Generate bindings for the test-world defined in wit/test-component/component.wit
wit_bindgen::generate!({
    world: "test-world",
    path: "wit/test-component",
});

// Define the type implementing the exports
struct TestComponent;

impl Guest for TestComponent {
    fn execute() -> test::types::TestResult {
        test::types::TestResult {
            success: true,
            message: "Binding generation successful!".to_string(),
        }
    }
}

// Export the implementation
export!(TestComponent);
