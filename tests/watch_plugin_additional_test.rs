use bodo::plugins::watch_plugin::WatchPlugin;
use std::path::PathBuf;

#[test]
fn test_create_watcher_test() {
    let (watcher, rx) = WatchPlugin::create_watcher_test().expect("Failed to create watcher");
    // Expect timeout since no events occur.
    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => assert!(true),
        _ => panic!("Expected timeout when no events occur"),
    }
    drop(watcher);
}

#[test]
fn test_find_base_directory() {
    // For a pattern starting with **/ we always return "."
    let base = WatchPlugin::find_base_directory("**/foo/bar").unwrap();
    assert_eq!(base, PathBuf::from("."));
}

#[test]
fn test_find_base_directory_with_no_wildcard() {
    // If no wildcard is present and the given pattern does not resolve to an existing directory,
    // the implementation returns the parent (which for a single component yields ".").
    let base = WatchPlugin::find_base_directory("src").unwrap();
    assert_eq!(base, PathBuf::from("."));
}

#[test]
fn test_find_base_directory_with_wildcard_in_middle() {
    // With a wildcard in the middle, the base is the portion before the wildcard.
    let base = WatchPlugin::find_base_directory("src/*.rs").unwrap();
    assert_eq!(base, PathBuf::from("src"));
}

#[test]
fn test_find_base_directory_empty() {
    // If an empty string is provided, expect the result to be "."
    let base = WatchPlugin::find_base_directory("").unwrap();
    assert_eq!(base, PathBuf::from("."));
}

#[test]
fn test_filter_changed_paths() {
    use std::collections::HashSet;
    use std::env;
    use std::fs;
    use std::io::Write;

    // Create a temporary directory for testing.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Change current directory to temp_dir.
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    // Create "test_dir" and file "foo.txt" inside it.
    let test_dir = temp_path.join("test_dir");
    fs::create_dir_all(&test_dir).unwrap();
    let file_path = test_dir.join("foo.txt");
    {
        let mut file = fs::File::create(&file_path).expect("Failed to create foo.txt");
        writeln!(file, "Test content").expect("Failed to write to foo.txt");
    }

    // Build a glob set that matches "test_dir/foo.txt"
    let mut builder = globset::GlobSetBuilder::new();
    let glob = globset::Glob::new("test_dir/foo.txt").unwrap();
    builder.add(glob);
    let glob_set = builder.build().unwrap();

    // Create a dummy WatchEntry with directory to watch: "test_dir"
    let watch_entry = bodo::plugins::watch_plugin::WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut set = HashSet::new();
            set.insert(std::path::PathBuf::from("test_dir"));
            set
        },
        debounce_ms: 500,
    };

    // Prepare changed_paths using the absolute path of the file.
    let changed_path = env::current_dir().unwrap().join("test_dir").join("foo.txt");
    let changed_paths = vec![changed_path];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matched.len(), 1);

    // Restore original current directory.
    env::set_current_dir(original_dir).unwrap();
}
