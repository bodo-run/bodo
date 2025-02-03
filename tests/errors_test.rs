use bodo::errors::BodoError;
use std::io;

#[test]
fn test_bodo_error_display() {
    let err = BodoError::IoError(io::Error::new(io::ErrorKind::Other, "io error"));
    assert_eq!(format!("{}", err), "io error");

    let err = BodoError::TaskNotFound("task".to_string());
    assert_eq!(format!("{}", err), "not found");

    let err = BodoError::PluginError("plugin error".to_string());
    assert_eq!(format!("{}", err), "Plugin error: plugin error");

    let err = BodoError::NoTaskSpecified;
    assert_eq!(
        format!("{}", err),
        "No task specified and no scripts/script.yaml found"
    );
}
