use bodo::config::WatchConfig;
use validator::Validate;

#[test]
fn test_valid_watch_config() {
    let config = WatchConfig {
        patterns: vec!["src/**/*.rs".to_string()],
        debounce_ms: 500,
        ignore_patterns: vec!["target/**".to_string()],
        auto_watch: true,
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_watch_config_empty_patterns() {
    let config = WatchConfig {
        patterns: vec![],
        debounce_ms: 500,
        ignore_patterns: vec![],
        auto_watch: false,
    };
    // Validate should fail because patterns must have at least one element.
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_watch_config_debounce_too_low() {
    let config = WatchConfig {
        patterns: vec!["src/**/*.rs".to_string()],
        debounce_ms: 0,
        ignore_patterns: vec![],
        auto_watch: false,
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_watch_config_debounce_too_high() {
    let config = WatchConfig {
        patterns: vec!["src/**/*.rs".to_string()],
        debounce_ms: 70000,
        ignore_patterns: vec![],
        auto_watch: false,
    };
    assert!(config.validate().is_err());
}
