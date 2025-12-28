#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

use airssys_wasm::core::config::{ComponentConfigToml, ConfigError};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_component_toml_file_parsing_valid() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("Component.toml");

    let toml_content = r#"
[component]
name = "integration-test-component"
version = "2.0.0"

[resources.memory]
max_bytes = 2097152

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let config = ComponentConfigToml::from_file(&toml_path).unwrap();

    assert_eq!(config.component.name, "integration-test-component");
    assert_eq!(config.component.version, "2.0.0");
    assert_eq!(
        config
            .resources
            .as_ref()
            .unwrap()
            .memory
            .as_ref()
            .unwrap()
            .max_bytes,
        2097152
    );

    let limits = config.to_resource_limits().unwrap();
    assert_eq!(limits.max_memory_bytes(), 2097152);
    assert_eq!(limits.max_fuel(), 10000);
    assert_eq!(limits.timeout_seconds(), 30);
}

#[test]
fn test_component_toml_file_parsing_missing_memory() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("Component.toml");

    let toml_content = r#"
[component]
name = "invalid-component"
version = "1.0.0"
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let result = ComponentConfigToml::from_file(&toml_path);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::MissingMemoryConfig));
}

#[test]
fn test_component_toml_file_parsing_invalid_memory_range() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("Component.toml");

    let toml_content = r#"
[component]
name = "too-large-component"
version = "1.0.0"

[resources.memory]
max_bytes = 10485760
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let result = ComponentConfigToml::from_file(&toml_path);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::InvalidResourceLimits(_)));
}

#[test]
fn test_component_toml_file_not_found() {
    let result = ComponentConfigToml::from_file("/nonexistent/Component.toml");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::IoError(_)));
}

#[test]
fn test_component_toml_invalid_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("Component.toml");

    let invalid_toml = r#"
[component
name = "broken"
"#;

    fs::write(&toml_path, invalid_toml).unwrap();

    let result = ComponentConfigToml::from_file(&toml_path);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(matches!(err, ConfigError::TomlParseError(_)));
}

#[test]
fn test_component_toml_with_cpu_config() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("Component.toml");

    let toml_content = r#"
[component]
name = "cpu-limited-component"
version = "1.0.0"

[resources.memory]
max_bytes = 1048576

[resources.cpu]
max_fuel = 50000
timeout_seconds = 120
"#;

    fs::write(&toml_path, toml_content).unwrap();

    let config = ComponentConfigToml::from_file(&toml_path).unwrap();

    assert_eq!(config.component.name, "cpu-limited-component");
    assert_eq!(
        config
            .resources
            .as_ref()
            .unwrap()
            .memory
            .as_ref()
            .unwrap()
            .max_bytes,
        1048576
    );
    assert_eq!(
        config
            .resources
            .as_ref()
            .unwrap()
            .cpu
            .as_ref()
            .unwrap()
            .max_fuel,
        Some(50000)
    );
    assert_eq!(
        config
            .resources
            .as_ref()
            .unwrap()
            .cpu
            .as_ref()
            .unwrap()
            .timeout_seconds,
        Some(120)
    );
}

#[test]
fn test_component_toml_boundary_values() {
    let temp_dir = TempDir::new().unwrap();

    let test_cases = vec![
        ("minimum", 524288),
        ("middle", 2097152),
        ("maximum", 4194304),
    ];

    for (name, memory_bytes) in test_cases {
        let toml_path = temp_dir.path().join(format!("Component_{name}.toml"));

        let toml_content = format!(
            r#"
[component]
name = "boundary-test-{name}"
version = "1.0.0"

[resources.memory]
max_bytes = {memory_bytes}

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
"#
        );

        fs::write(&toml_path, &toml_content).unwrap();

        let config = ComponentConfigToml::from_file(&toml_path).unwrap();
        let limits = config.to_resource_limits().unwrap();

        assert_eq!(
            limits.max_memory_bytes(),
            memory_bytes,
            "Failed for boundary case: {name}"
        );
    }
}

#[test]
fn test_component_toml_roundtrip_resource_limits() {
    let toml_content = r#"
[component]
name = "roundtrip-test"
version = "1.5.0"

[resources.memory]
max_bytes = 3145728

[resources.cpu]
max_fuel = 10000
timeout_seconds = 30
"#;

    let config = ComponentConfigToml::from_str(toml_content).unwrap();

    let limits = config.to_resource_limits().unwrap();
    assert_eq!(limits.max_memory_bytes(), 3145728);
    assert_eq!(limits.max_fuel(), 10000);
    assert_eq!(limits.timeout_seconds(), 30);

    assert_eq!(config.component.name, "roundtrip-test");
    assert_eq!(config.component.version, "1.5.0");
}

#[test]
fn test_component_toml_error_messages() {
    let toml_content = r#"
[component]
name = "no-resources"
version = "1.0.0"
"#;

    let result = ComponentConfigToml::from_str(toml_content);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let error_message = format!("{err}");

    assert!(error_message.contains("Missing [resources.memory]"));
    assert!(error_message.contains("ADR-WASM-002"));
    assert!(error_message.contains("max_bytes"));
}
