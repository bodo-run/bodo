// tests/errors_test.rs

use bodo::BodoError;

#[test]
fn test_bodo_error_display() {
    let err = BodoError::IoError(std::io::Error::new(std::io::ErrorKind::Other, "io_error"));
    assert_eq!(format!("{}", err), "io_error");

    let err = BodoError::WatcherError("watcher_error".to_string());
    assert_eq!(format!("{}", err), "watcher_error");

    let err = BodoError::TaskNotFound("missing_task".to_string());
    assert_eq!(format!("{}", err), "not found");

    let err = BodoError::PluginError("plugin failure".to_string());
    assert_eq!(format!("{}", err), "Plugin error: plugin failure");

    let err = BodoError::NoTaskSpecified;
    assert_eq!(
        format!("{}", err),
        "No task specified and no scripts/script.yaml found"
    );

    let err = BodoError::ValidationError("validation error".to_string());
    assert_eq!(format!("{}", err), "Validation error: validation error");
}

#[test]
fn test_bodo_error_from_io_error() {
    use std::io;
    let io_err = io::Error::new(io::ErrorKind::Other, "some io error");
    let bodo_err: BodoError = io_err.into();
    assert!(matches!(bodo_err, BodoError::IoError(_)));
}

#[test]
fn test_bodo_error_from_notify_error() {
    let notify_err = notify::Error::generic("notify error");
    let bodo_err: BodoError = notify_err.into();
    assert!(matches!(bodo_err, BodoError::WatcherError(_)));
}

#[test]
fn test_bodo_error_display_for_serialization_errors() {
    let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let bodo_err: BodoError = serde_err.into();
    assert!(matches!(bodo_err, BodoError::SerdeError(_)));
    assert!(format!("{}", bodo_err).contains("expected value at line 1 column 1"));

    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: :").unwrap_err();
    let bodo_err: BodoError = yaml_err.into();
    assert!(matches!(bodo_err, BodoError::YamlError(_)));
    // Print the error message
    println!("YAML error message: {}", format!("{}", bodo_err));
    assert!(
        !format!("{}", bodo_err).is_empty(),
        "YAML error message should not be empty"
    );
    assert!(
        format!("{}", bodo_err).contains("mapping values are not allowed"),
        "YAML error message should mention mapping values error"
    );
}

#[test]
fn test_bodo_error_from_validation_error() {
    use validator::ValidationError;
    let val_err = ValidationError::new("test_error");
    let bodo_err: BodoError = val_err.into();
    assert!(matches!(bodo_err, BodoError::ValidationError(_)));

    let val_errors = validator::ValidationErrors::new();
    let bodo_err: BodoError = val_errors.into();
    assert!(matches!(bodo_err, BodoError::ValidationError(_)));
}
