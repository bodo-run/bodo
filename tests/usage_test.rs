#[test]
fn test_usage_md_non_empty() {
    // Simply check that the USAGE.md file is non-empty.
    use std::fs;
    let content = fs::read_to_string("USAGE.md").expect("USAGE.md file not found");
    assert!(!content.trim().is_empty(), "USAGE.md should not be empty");
}

#[test]
fn test_design_md_non_empty() {
    use std::fs;
    let content = fs::read_to_string("DESIGN.md")
        .or_else(|_| fs::read_to_string("design.md"))
        .expect("DESIGN.md file not found");
    assert!(!content.trim().is_empty(), "DESIGN.md should not be empty");
}
