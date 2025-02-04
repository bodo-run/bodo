use bodo::config::BodoConfig;
use bodo::script_loader::ScriptLoader;

#[test]
fn test_build_graph_nonexistent_root_script() {
    let config = BodoConfig {
        root_script: Some("nonexistent.yaml".to_string()),
        ..Default::default()
    };
    let mut loader = ScriptLoader::new();
    let result = loader.build_graph(config);
    assert!(
        result.is_err(),
        "Expected error when root_script does not exist"
    );
}
