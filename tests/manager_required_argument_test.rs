use bodo::config::{BodoConfig, TaskArgument, TaskConfig};
use bodo::errors::BodoError;
use bodo::manager::GraphManager;
use std::collections::HashMap;

#[test]
fn test_missing_required_argument() {
    let task_config = TaskConfig {
        command: Some("echo $ARG".to_string()),
        arguments: vec![TaskArgument {
            name: "ARG".to_string(),
            description: Some("A required argument".to_string()),
            required: true,
            default: None,
        }],
        ..Default::default()
    };
    let mut tasks = HashMap::new();
    tasks.insert("test".to_string(), task_config);
    let config = BodoConfig {
        tasks,
        ..Default::default()
    };
    let mut manager = GraphManager::new();
    manager.build_graph(config).unwrap();
    let result = manager.apply_task_arguments("test", &[]);
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert!(
            msg.contains("Missing required argument"),
            "Message should mention missing required argument"
        );
    } else {
        panic!("Expected PluginError for missing required argument");
    }
}
