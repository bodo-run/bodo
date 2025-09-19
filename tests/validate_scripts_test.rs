use serde_yaml::Value;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_validate_all_scripts_yaml() {
    let scripts_dir = Path::new("scripts");
    assert!(
        scripts_dir.exists(),
        "scripts directory must exist in the repository"
    );

    let mut count = 0;
    for entry in WalkDir::new(scripts_dir) {
        let entry = entry.expect("Failed to read directory entry");
        if entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "yaml") {
            count += 1;
            let content = fs::read_to_string(entry.path()).expect("Failed to read YAML file");
            let _doc: Value = serde_yaml::from_str(&content)
                .unwrap_or_else(|_| panic!("Invalid YAML in file {:?}", entry.path()));
        }
    }
    assert!(
        count > 0,
        "There should be at least one YAML file in the scripts directory"
    );
}
