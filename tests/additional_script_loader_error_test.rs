use bodo::config::BodoConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // YAML duplicate keys: duplicate_task appears twice.
    let script_content = r#"tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;
    fs::write(&script_path, script_content).unwrap();

    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    // Expect error due to duplicate keys.
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(res.is_err(), "Expected error due to duplicate task keys");
}

#[test]
fn test_load_script_with_invalid_dependency() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Invalid dependency format: using a number instead of a map.
    let script_content = r#"tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - 123
"#;
    fs::write(&script_path, script_content).unwrap();

    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(
        res.is_err(),
        "Expected error due to invalid dependency format"
    );
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Task name contains invalid characters.
    let script_content = r#"tasks:
  "invalid/task.name":
    command: echo "Invalid task name"
"#;
    fs::write(&script_path, script_content).unwrap();

    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(
        res.is_err(),
        "Expected error due to invalid characters in task name"
    );
}

#[test]
fn test_load_script_with_reserved_task_name() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // Using reserved task name "watch"
    let script_content = r#"tasks:
  watch:
    command: echo "This should fail"
"#;
    fs::write(&script_path, script_content).unwrap();

    let config_yaml = format!("tasks: {}", fs::read_to_string(&script_path).unwrap());
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(res.is_err(), "Expected error due to reserved task name");
}
