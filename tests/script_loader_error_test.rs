use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::script_loader::ScriptLoader;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_script_with_duplicate_tasks() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  duplicate_task:
    command: echo "First definition"
  duplicate_task:
    command: echo "Second definition"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::PluginError(_))),
        "Expected PluginError due to duplicate task names"
    );
}

#[test]
fn test_load_script_with_reserved_task_name() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  watch:
    command: echo "This should fail"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::ValidationError(_))),
        "Expected ValidationError due to reserved task name"
    );
}

#[test]
fn test_load_script_with_invalid_dependency() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps:
      - task: non_existent_task
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::PluginError(_))),
        "Expected PluginError due to invalid dependency"
    );
}

#[test]
fn test_load_script_with_invalid_task_name_chars() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  invalid/task.name:
    command: echo "Invalid task name"
"#;

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::ValidationError(_))),
        "Expected ValidationError due to invalid characters in task name"
    );
}

#[test]
fn test_load_script_with_invalid_yaml() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("script.yaml");

    let script_content = r#"
tasks:
  task1:
    command: echo "Task 1"
    pre_deps: [task2
"#; // Missing closing bracket

    fs::write(&script_path, script_content).unwrap();

    let mut loader = ScriptLoader::new();
    let mut config = BodoConfig::default();
    config.scripts_dirs = Some(vec![temp_dir.path().to_string_lossy().to_string()]);

    let result = loader.build_graph(config);
    assert!(
        matches!(result, Err(BodoError::YamlError(_))),
        "Expected YamlError due to invalid YAML syntax"
    );
}
