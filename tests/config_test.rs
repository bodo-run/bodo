use bodo::config::{load_config_from_path, TaskConfig};

#[test]
fn test_load_config() {
    let task_config = load_config_from_path("path/to/config").unwrap();
    assert_eq!(task_config.some_field, "expected_value");
}
