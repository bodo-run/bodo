use bodo::config::validate_task_name;

#[test]
fn test_validate_task_name_success() {
    assert!(validate_task_name("valid_name").is_ok());
}

#[test]
fn test_validate_task_name_reserved_failure() {
    let reserved = [
        "watch",
        "default_task",
        "pre_deps",
        "post_deps",
        "concurrently",
    ];
    for name in reserved.iter() {
        assert!(
            validate_task_name(name).is_err(),
            "Reserved name '{}' should be invalid",
            name
        );
    }
}

#[test]
fn test_validate_task_name_invalid_characters() {
    let invalid_names = ["invalid/name", "..", ".", "in.valid", "invalid..name"];
    for name in &invalid_names {
        assert!(
            validate_task_name(name).is_err(),
            "Name '{}' should be invalid",
            name
        );
    }
}

#[test]
fn test_validate_task_name_length() {
    assert!(validate_task_name("").is_err());
    let long_name = "a".repeat(101);
    assert!(validate_task_name(&long_name).is_err());
    let valid = "a".repeat(50);
    assert!(validate_task_name(&valid).is_ok());
}
