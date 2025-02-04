use bodo::errors::BodoError;
use bodo::plugins::timeout_plugin::TimeoutPlugin;

#[test]
fn test_parse_timeout_valid() {
    let secs = TimeoutPlugin::parse_timeout("30s").unwrap();
    assert_eq!(secs, 30);
    let secs = TimeoutPlugin::parse_timeout("1m").unwrap();
    assert_eq!(secs, 60);
}

#[test]
fn test_parse_timeout_invalid() {
    let result = TimeoutPlugin::parse_timeout("invalid");
    assert!(result.is_err());
    if let Err(BodoError::PluginError(msg)) = result {
        assert!(
            msg.contains("Invalid timeout duration"),
            "Expected timeout error message"
        );
    } else {
        panic!("Expected PluginError for invalid timeout duration");
    }
}
