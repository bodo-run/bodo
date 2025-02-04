use bodo::config::BodoConfig;
use bodo::errors::BodoError;
use bodo::manager::GraphManager;

#[test]
fn test_initialize_no_script() {
    // Create a configuration with a nonexistent root_script.
    let config = BodoConfig {
        root_script: Some("nonexistent.yaml".to_string()),
        ..Default::default()
    };
    let mut manager = GraphManager::new();
    let result = manager.build_graph(config);
    assert!(
        result.is_err(),
        "Expected error when root_script does not exist"
    );
    if let Err(err) = result {
        match err {
            BodoError::IoError(_) | BodoError::PluginError(_) => { /* expected */ }
            _ => panic!("Unexpected error type: {}", err),
        }
    }
}
