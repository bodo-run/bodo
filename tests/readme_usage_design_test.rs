use std::fs;

#[test]
fn test_readme_usage_design_files_exist_and_non_empty() {
    let readme = fs::read_to_string("README.md").expect("README.md not found");
    let usage = fs::read_to_string("USAGE.md").expect("USAGE.md not found");
    let design = fs::read_to_string("DESIGN.md").expect("DESIGN.md not found");
    assert!(!readme.trim().is_empty(), "README.md is empty");
    assert!(!usage.trim().is_empty(), "USAGE.md is empty");
    assert!(!design.trim().is_empty(), "DESIGN.md is empty");
}
