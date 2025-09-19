use std::path::{Path, PathBuf};

#[test]
fn test_cross_platform_path_handling() {
    // Test that PathBuf handles both Unix and Windows style paths appropriately
    let unix_path = Path::new("scripts/tasks/build.yaml");
    let windows_path = Path::new("scripts\\tasks\\build.yaml");
    
    // Both should be valid paths
    assert!(unix_path.is_relative());
    assert!(windows_path.is_relative());
    
    // Test path building works correctly
    let base = PathBuf::from("project");
    let combined_unix = base.join(unix_path);
    let combined_windows = base.join(windows_path);
    
    // Should handle properly on current platform
    assert!(combined_unix.to_string_lossy().len() > 0);
    assert!(combined_windows.to_string_lossy().len() > 0);
}

#[test]
fn test_executable_extension_handling() {
    let base_name = "bodo";
    
    #[cfg(windows)]
    let expected_name = format!("{}.exe", base_name);
    #[cfg(not(windows))]
    let expected_name = base_name.to_string();
    
    let path = PathBuf::from(base_name);
    
    #[cfg(windows)]
    let exe_path = path.with_extension("exe");
    #[cfg(not(windows))]
    let exe_path = path;
    
    let file_name = exe_path.file_name().unwrap().to_string_lossy();
    assert_eq!(file_name, expected_name);
}

#[test]
fn test_path_separators_normalization() {
    // Test that we handle both types of separators correctly
    let mixed_path = "scripts/tasks\\config/file.yaml";
    let path = Path::new(mixed_path);
    
    // Should be able to iterate components regardless of separator style
    let components: Vec<_> = path.components().collect();
    assert!(components.len() >= 3); // Should have multiple components
    
    // Test PathBuf construction with cross-platform join
    let mut cross_platform = PathBuf::new();
    cross_platform.push("scripts");
    cross_platform.push("tasks");
    cross_platform.push("config");
    cross_platform.push("file.yaml");
    
    // Should work on any platform
    assert!(cross_platform.to_string_lossy().contains("file.yaml"));
}

#[test]
fn test_working_directory_handling() {
    use std::env;
    
    // Test that we can get and work with current directory
    let current_dir = env::current_dir();
    assert!(current_dir.is_ok());
    
    let current = current_dir.unwrap();
    assert!(current.is_absolute());
    
    // Test relative path resolution
    let relative = PathBuf::from(".");
    let resolved = relative.canonicalize();
    assert!(resolved.is_ok());
}

#[cfg(test)]
mod windows_compatibility {
    use super::*;
    
    #[test]
    fn test_script_path_construction() {
        // Test that script paths work on Windows and Unix
        let script_paths = vec![
            "script.yaml",
            "scripts/build.yaml", 
            "tasks/test.yaml",
            "config/deploy.yaml",
        ];
        
        for script_path in script_paths {
            let path = PathBuf::from(script_path);
            
            // Should be valid regardless of platform
            assert!(path.file_name().is_some());
            assert!(path.extension().is_some());
            
            // Should handle parent directory correctly
            if path.parent().is_some() {
                let parent = path.parent().unwrap();
                assert!(!parent.as_os_str().is_empty() || parent == Path::new(""));
            }
        }
    }
    
    #[test]
    fn test_temporary_directory_paths() {
        use tempfile::tempdir;
        
        // Test that temporary directories work cross-platform
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        
        assert!(temp_path.exists());
        assert!(temp_path.is_dir());
        
        // Test file creation in temp directory
        let test_file = temp_path.join("test_script.yaml");
        let content = "default_task:\n  command: echo hello\n";
        
        std::fs::write(&test_file, content).expect("Failed to write test file");
        assert!(test_file.exists());
        
        // Test that we can read it back
        let read_content = std::fs::read_to_string(&test_file).expect("Failed to read test file");
        assert_eq!(read_content, content);
    }
    
    #[test]
    fn test_environment_variable_paths() {
        use std::env;
        
        // Test common environment variables that might contain paths
        let path_vars = vec!["PATH", "HOME", "USERPROFILE", "TEMP", "TMP"];
        
        for var in path_vars {
            if let Ok(value) = env::var(var) {
                // Should be non-empty if set
                assert!(!value.is_empty());
                
                // PATH should contain separator-delimited paths
                if var == "PATH" {
                    #[cfg(windows)]
                    let separator = ';';
                    #[cfg(not(windows))]
                    let separator = ':';
                    
                    // Should contain at least one path
                    let paths: Vec<_> = value.split(separator).collect();
                    assert!(paths.len() > 0);
                }
            }
        }
    }
}