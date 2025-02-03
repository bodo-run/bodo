// tests/config_test.rs

use bodo::config::{BodoConfig, TaskConfig};
use validator::Validate;

use validator::ValidationErrors;

#[test]
fn test_validate_task_name_reserved() {
    let mut config = TaskConfig::default();
    config._name_check = Some("default_task".to_string());
    let result = config.validate();
    assert!(matches!(result, Err(ValidationErrors { .. })));
}

// ... existing tests ...

#[test]
fn test_generate_schema() {
    let schema = BodoConfig::generate_schema();
    assert!(!schema.is_empty(), "Schema should not be empty");
    // Optionally, verify that the schema contains certain expected strings
    assert!(
        schema.contains("\"title\": \"BodoConfig\""),
        "Schema should contain BodoConfig title"
    );
}
