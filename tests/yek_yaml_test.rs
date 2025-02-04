use serde_yaml::Value;
use std::fs;
use std::path::Path;

#[test]
fn test_yek_yaml_valid() {
    let path = Path::new("yek.yaml");
    assert!(path.exists(), "yek.yaml should exist in the repository");
    let content = fs::read_to_string(path).expect("Failed to read yek.yaml");
    assert!(!content.trim().is_empty(), "yek.yaml should not be empty");
    let yaml: Value = serde_yaml::from_str(&content).expect("yek.yaml should be valid YAML");
    // Validate that certain keys exist
    assert!(
        yaml.get("output_dir").is_some(),
        "output_dir key missing in yek.yaml"
    );
    assert!(
        yaml.get("ignore_patterns").is_some(),
        "ignore_patterns key missing in yek.yaml"
    );
}
