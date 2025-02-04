use std::fs;

#[test]
fn test_design_md_contains_content() {
    let content = fs::read_to_string("DESIGN.md").expect("DESIGN.md not found");
    assert!(!content.trim().is_empty(), "DESIGN.md should not be empty");
}

#[test]
fn test_usage_md_contains_content() {
    let content = fs::read_to_string("USAGE.md").expect("USAGE.md not found");
    assert!(!content.trim().is_empty(), "USAGE.md should not be empty");
}

#[test]
fn test_readme_contains_logo_info() {
    let content = fs::read_to_string("README.md").expect("README.md not found");
    assert!(
        content.contains("bodo logo"),
        "README.md should reference the logo"
    );
}

#[test]
fn test_cargo_toml_exists() {
    assert!(
        fs::metadata("Cargo.toml").is_ok(),
        "Cargo.toml should exist in the repository"
    );
}
