use std::fs::{create_dir_all, write};
use tempfile::tempdir;

use bodo::{
    errors::PluginError,
    graph::{Graph, NodeKind},
    script_loader::{load_bodo_config, load_scripts_from_fs, BodoConfig},
};

// BodoConfig Tests
#[test]
fn test_default_config_when_no_bodo_toml() {
    let config = load_bodo_config::<&str>(None).unwrap();
    assert!(
        config.script_paths.is_none(),
        "Expected None for script_paths by default"
    );
}

#[test]
fn test_load_valid_toml_config() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("bodo.toml");

    let toml_content = r#"
script_paths = ["my-scripts/", "others/*.yaml"]
    "#;

    write(&config_path, toml_content).unwrap();

    let loaded = load_bodo_config(Some(config_path)).unwrap();

    assert_eq!(
        loaded.script_paths,
        Some(vec!["my-scripts/".to_string(), "others/*.yaml".to_string()])
    );
}

#[test]
fn test_load_invalid_toml_config() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("bodo.toml");

    let bad_toml = r#"
script_paths = ["scripts/]
"#;

    write(&config_path, bad_toml).unwrap();

    let result = load_bodo_config(Some(&config_path));
    match result {
        Err(PluginError::GenericError(msg)) => {
            assert!(
                msg.contains("bodo.toml parse error"),
                "Should mention a TOML parse error"
            );
        }
        _ => panic!("Expected GenericError for invalid TOML"),
    }
}

#[test]
fn test_file_missing_read_permission() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("bodo.toml");

    write(&config_path, "script_paths = [\"scripts/\"]").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o200); // Write-only
        std::fs::set_permissions(&config_path, perms).unwrap();

        let result = load_bodo_config(Some(&config_path));
        match result {
            Err(PluginError::GenericError(msg)) => {
                assert!(msg.contains("Cannot read bodo.toml"), "Expected read error");
            }
            _ => panic!("Expected error for unreadable file"),
        }
    }
}

#[test]
fn test_unknown_fields_in_toml_are_ignored() {
    let temp = tempdir().unwrap();
    let config_path = temp.path().join("bodo.toml");

    let extended_toml = r#"
script_paths = ["scripts/"]
some_extra_field = "Whatever"
another_one = 123
"#;
    write(&config_path, extended_toml).unwrap();

    let loaded = load_bodo_config(Some(&config_path)).unwrap();
    assert_eq!(loaded.script_paths, Some(vec!["scripts/".to_string()]));
}

#[test]
fn test_specify_config_path_non_existent() {
    let result = load_bodo_config(Some("nonexistent/bodo.toml"));
    let config = result.unwrap();
    assert!(config.script_paths.is_none());
}

// Script Loading Tests
#[test]
fn test_load_single_yaml_file() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("single.yaml");

    let yaml_content = r#"
name: "Single Test"
description: "Testing single-file load"

default_task:
  command: "echo default"
  description: "My default command"

tasks:
  build:
    command: "cargo build"
    description: "Build the project"
"#;

    write(&script_path, yaml_content).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);

    match &graph.nodes[0].kind {
        NodeKind::Command(cmd) => {
            assert_eq!(cmd.raw_command, "echo default");
            assert_eq!(cmd.description.as_deref(), Some("My default command"));
        }
        _ => panic!("Expected Command node for default_task"),
    }

    match &graph.nodes[1].kind {
        NodeKind::Task(task) => {
            assert_eq!(task.name, "build");
            assert_eq!(task.description.as_deref(), Some("Build the project"));
        }
        _ => panic!("Expected Task node for 'build'"),
    }
}

#[test]
fn test_load_multiple_files_in_directory() {
    let temp = tempdir().unwrap();
    let scripts_dir = temp.path().join("scripts");
    create_dir_all(&scripts_dir).unwrap();

    let script_a = scripts_dir.join("scriptA.yaml");
    let yaml_a = r#"
default_task:
  command: "echo from A"
tasks:
  foo:
    command: "echo Foo"
"#;
    write(&script_a, yaml_a).unwrap();

    let script_b = scripts_dir.join("scriptB.yaml");
    let yaml_b = r#"
default_task:
  command: "echo from B"
tasks:
  bar:
    command: "echo Bar"
"#;
    write(&script_b, yaml_b).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![scripts_dir.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 4);

    let commands = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Command(_)));
    let tasks = graph
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Task(_)));

    assert_eq!(commands.count(), 2);
    assert_eq!(tasks.count(), 2);
}

#[test]
fn test_load_with_glob_pattern() {
    let temp = tempdir().unwrap();
    let scripts_dir = temp.path().join("some_dir");
    create_dir_all(&scripts_dir).unwrap();

    let file1 = scripts_dir.join("script1.yaml");
    let file2 = scripts_dir.join("script2.yaml");

    write(&file1, "default_task:\n  command: \"echo 111\"").unwrap();
    write(&file2, "default_task:\n  command: \"echo 222\"").unwrap();

    let pattern = format!("{}/**/*.yaml", scripts_dir.display());

    let config = BodoConfig {
        script_paths: Some(vec![pattern]),
    };

    let mut graph = Graph::new();
    load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert!(graph
        .nodes
        .iter()
        .all(|n| matches!(n.kind, NodeKind::Command(_))));
}

#[test]
fn test_invalid_yaml() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("invalid.yaml");

    let bad_yaml = r#"
default_task: {
  command: "echo BAD
  description: 'unclosed quote
  invalid: [1, 2,
"#;

    write(&script_path, bad_yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();

    let _result = load_scripts_from_fs(&config, &mut graph);

    match _result {
        Err(PluginError::GenericError(msg)) => {
            assert!(
                msg.contains("YAML parse error"),
                "Should mention parse error"
            );
        }
        _ => panic!("Expected a GenericError due to invalid YAML"),
    }
}

#[test]
fn test_non_existent_path() {
    let config = BodoConfig {
        script_paths: Some(vec!["this/path/does/not/exist".to_string()]),
    };
    let mut graph = Graph::new();

    let _result = load_scripts_from_fs(&config, &mut graph);
    assert!(
        _result.is_ok(),
        "We skip non-existent directories by default"
    );
    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_empty_scripts() {
    let temp = tempdir().unwrap();
    let empty_dir = temp.path().join("scripts_empty");
    create_dir_all(&empty_dir).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![empty_dir.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();
    load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_complex_task_unused_fields() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("complex.yaml");

    let complex_yaml = r#"
default_task:
  command: "./do_something.sh"
  description: "Complex default"
  concurrently:
    - command: "echo one"
    - command: "echo two"
tasks:
  alpha:
    command: "echo ALPHA"
"#;

    write(&script_path, complex_yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();
    load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);

    match &graph.nodes[0].kind {
        NodeKind::Command(cmd) => {
            assert_eq!(cmd.raw_command, "./do_something.sh");
            assert_eq!(cmd.description.as_deref(), Some("Complex default"));
        }
        _ => panic!("Expected Command node"),
    }

    match &graph.nodes[1].kind {
        NodeKind::Task(td) => {
            assert_eq!(td.name, "alpha");
        }
        _ => panic!("Expected Task node named 'alpha'"),
    }
}

#[test]
fn test_tasks_with_same_name_in_one_file() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("duplicate.yaml");

    let yaml = r#"
default_task:
  command: "echo 'Default'"
tasks:
  build:
    command: "echo 'Build #1'"
  build:
    command: "echo 'Build #2'"
"#;

    write(&script_path, yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph);

    match _result {
        Err(PluginError::GenericError(msg)) => {
            assert!(
                msg.contains("duplicate"),
                "Should mention duplicate task name"
            );
        }
        _ => panic!("Expected an error for duplicate tasks"),
    }
}

#[test]
fn test_multiple_default_tasks_in_one_file() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("multiple_defaults.yaml");

    let yaml = r#"
default_task:
  command: "echo 'Default #1'"
default_task:
  command: "echo 'Default #2'"
"#;

    write(&script_path, yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph);

    match _result {
        Err(PluginError::GenericError(msg)) => {
            assert!(
                msg.contains("multiple default_task"),
                "Should mention multiple defaults"
            );
        }
        _ => panic!("Expected an error for multiple default tasks"),
    }
}

#[test]
fn test_pre_deps_are_ignored_or_stored_for_future() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("pre_deps.yaml");

    let yaml = r#"
default_task:
  pre_deps:
    - task: test
    - task: lint
  command: "echo 'Default with deps'"

tasks:
  test:
    command: "cargo test"
  lint:
    command: "cargo clippy"
"#;

    write(&script_path, yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 3);

    // Verify the nodes exist but don't check edges yet since pre_deps isn't implemented
    let node_names: Vec<String> = graph
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Task(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();

    assert!(node_names.contains(&"test".to_string()));
    assert!(node_names.contains(&"lint".to_string()));
}

#[test]
fn test_env_and_exec_paths_ignored_for_now() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("env_exec.yaml");

    let yaml = r#"
default_task:
  command: "echo 'Building...'"
  env:
    RUST_BACKTRACE: "1"
  exec_paths:
    - "./node_modules/.bin"

tasks:
  release:
    command: "cargo build --release"
    env:
      RUST_LOG: "debug"
"#;

    write(&script_path, yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };

    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph).unwrap();

    assert_eq!(graph.nodes.len(), 2);
}

#[test]
fn test_large_number_of_scripts() {
    let temp = tempdir().unwrap();
    let scripts_dir = temp.path().join("scripts");
    create_dir_all(&scripts_dir).unwrap();

    for i in 0..100 {
        let filename = format!("script_{}.yaml", i);
        let path = scripts_dir.join(filename);

        let content = format!(
            r#"
default_task:
  command: "echo 'Default {i}'"
tasks:
  taskA:
    command: "echo 'A{i}'"
  taskB:
    command: "echo 'B{i}'"
"#
        );
        write(path, content).unwrap();
    }

    let config = BodoConfig {
        script_paths: Some(vec![scripts_dir.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph);
    assert!(_result.is_ok());

    assert_eq!(graph.nodes.len(), 300);
}

#[test]
fn test_task_with_no_command() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("no_command.yaml");

    let yaml = r#"
tasks:
  weird:
    description: "No command here"
"#;
    write(&script_path, yaml).unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph);

    assert!(_result.is_ok());
    assert_eq!(graph.nodes.len(), 1);
    match &graph.nodes[0].kind {
        NodeKind::Task(td) => {
            assert_eq!(td.name, "weird");
        }
        _ => panic!("Expected a Task node"),
    }
}

#[test]
fn test_minimal_empty_yaml() {
    let temp = tempdir().unwrap();
    let script_path = temp.path().join("empty.yaml");
    write(&script_path, "").unwrap();

    let config = BodoConfig {
        script_paths: Some(vec![script_path.to_string_lossy().into_owned()]),
    };
    let mut graph = Graph::new();
    let _result = load_scripts_from_fs(&config, &mut graph);
    assert!(_result.is_ok());
    assert_eq!(graph.nodes.len(), 0);
}
