use bodo::config::BodoConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    // Duplicate keys in YAML should produce an error.
    let config_yaml = r#"
tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;
    let res = serde_yaml::from_str::<BodoConfig>(config_yaml);
    // Expect error due to duplicate keys.
    assert!(res.is_err(), "Expected error due to duplicate task keys");
}

#[test]
fn test_load_script_with_invalid_dependency() {
    // An invalid dependency format should cause an error.
    let config_yaml = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps: [123]
"#;
    let res = serde_yaml::from_str::<BodoConfig>(config_yaml);
    // Expect error because pre_deps is not in expected format.
    assert!(
        res.is_err(),
        "Expected error due to invalid dependency format"
    );
}

#[test]
fn test_load_script_with_invalid_yaml() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");
    // Invalid YAML: missing closing bracket
    let script_content = r#"tasks:
  task1:
    command: echo "Task 1"
    pre_deps: [task2
"#;
    fs::write(&script_path, script_content).unwrap();
    let yaml_str = fs::read_to_string(&script_path).unwrap();
    let res = serde_yaml::from_str::<BodoConfig>(&yaml_str);
    assert!(res.is_err(), "Expected error due to invalid YAML syntax");
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let config_yaml = r#"
tasks:
  "invalid/task.name":
    command: echo "Invalid task name"
"#;
    let res = serde_yaml::from_str::<BodoConfig>(config_yaml);
    assert!(
        res.is_err(),
        "Expected ValidationError due to invalid characters in task name"
    );
}

#[test]
fn test_load_script_with_reserved_task_name() {
    let config_yaml = r#"
tasks:
  watch:
    command: echo "This should fail"
"#;
    let res = serde_yaml::from_str::<BodoConfig>(config_yaml);
    assert!(
        res.is_err(),
        "Expected ValidationError due to reserved task name"
    );
}
