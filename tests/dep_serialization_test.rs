use bodo::config::Dependency;

#[test]
fn test_dependency_task_serialization() {
    let dep = Dependency::Task {
        task: "sample_task".to_string(),
    };
    // Serialize to YAML and JSON
    let yaml_str = serde_yaml::to_string(&dep).unwrap();
    assert!(yaml_str.contains("task: sample_task"));
    let json_str = serde_json::to_string(&dep).unwrap();
    assert!(json_str.contains("\"task\":\"sample_task\""));
    // Deserialize back from YAML
    let dep_from_yaml: Dependency = serde_yaml::from_str(&yaml_str).unwrap();
    match dep_from_yaml {
        Dependency::Task { task } => assert_eq!(task, "sample_task"),
        _ => panic!("Deserialized dependency is not the expected Task variant"),
    }
    // Deserialize back from JSON
    let dep_from_json: Dependency = serde_json::from_str(&json_str).unwrap();
    match dep_from_json {
        Dependency::Task { task } => assert_eq!(task, "sample_task"),
        _ => panic!("Deserialized dependency is not the expected Task variant from JSON"),
    }
}

#[test]
fn test_dependency_command_serialization() {
    let dep = Dependency::Command {
        command: "echo hi".to_string(),
    };
    // Serialize to YAML and JSON
    let yaml_str = serde_yaml::to_string(&dep).unwrap();
    assert!(yaml_str.contains("command: echo hi"));
    let json_str = serde_json::to_string(&dep).unwrap();
    assert!(json_str.contains("\"command\":\"echo hi\""));
    // Deserialize back from YAML
    let dep_from_yaml: Dependency = serde_yaml::from_str(&yaml_str).unwrap();
    match dep_from_yaml {
        Dependency::Command { command } => assert_eq!(command, "echo hi"),
        _ => panic!("Deserialized dependency is not the expected Command variant"),
    }
    // Deserialize back from JSON
    let dep_from_json: Dependency = serde_json::from_str(&json_str).unwrap();
    match dep_from_json {
        Dependency::Command { command } => assert_eq!(command, "echo hi"),
        _ => panic!("Deserialized dependency is not the expected Command variant from JSON"),
    }
}
