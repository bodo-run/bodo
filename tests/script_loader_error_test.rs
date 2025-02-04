use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    // YAML with duplicate keys.
    let script_content = r#"tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;
    fs::write(&script_path, script_content).unwrap();

    let _loader = ScriptLoader::new();
    // Read the file and attempt to deserialize into BodoConfig.
    let config_yaml = fs::read_to_string(&script_path).unwrap();
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(res.is_err(), "Expected error due to duplicate task keys");
}

#[test]
fn test_load_script_with_invalid_dependency() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - 123
"#;
    fs::write(&script_path, script_content).unwrap();

    let _loader = ScriptLoader::new();
    let config_yaml = fs::read_to_string(&script_path).unwrap();
    let res = serde_yaml::from_str::<BodoConfig>(&config_yaml);
    assert!(
        res.is_err(),
        "Expected error due to invalid dependency format"
    );
}
