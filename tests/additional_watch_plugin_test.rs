use bodo::plugins::watch_plugin::WatchPlugin;
use globset::Glob;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

#[test]
fn test_create_watcher_test() {
    let (watcher, rx) = WatchPlugin::create_watcher_test().expect("Failed to create watcher");
    // Expect timeout since no events occur.
    match rx.recv_timeout(Duration::from_millis(100)) {
        Err(RecvTimeoutError::Timeout) => assert!(true),
        _ => panic!("Expected timeout when no events occur"),
    }
    drop(watcher);
}

#[test]
fn test_find_base_directory() {
    // Pattern starts with **/, should return "."
    let base = WatchPlugin::find_base_directory("**/foo/bar").unwrap();
    assert_eq!(base, std::path::PathBuf::from("."));
}

#[test]
fn test_find_base_directory_with_no_wildcard() {
    let base = WatchPlugin::find_base_directory("src").unwrap();
    assert_eq!(base, std::path::PathBuf::from("src"));
}

#[test]
fn test_find_base_directory_with_wildcard_in_middle() {
    let base = WatchPlugin::find_base_directory("src/*.rs").unwrap();
    assert_eq!(base, std::path::PathBuf::from("src"));
}

#[test]
fn test_filter_changed_paths() {
    // Build a glob set that matches "test_dir/foo.txt"
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(Glob::new("test_dir/foo.txt").unwrap());
    let glob_set = builder.build().unwrap();

    // Create a dummy WatchEntry with a directory to watch: "test_dir"
    let watch_entry = bodo::plugins::watch_plugin::WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut set = std::collections::HashSet::new();
            set.insert(std::path::PathBuf::from("test_dir"));
            set
        },
        debounce_ms: 500,
    };

    // Get the current working directory.
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => return,
    };
    // Create a changed path that is within "test_dir"
    let changed_path = cwd.join("test_dir").join("foo.txt");
    let changed_paths = vec![changed_path.clone()];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    // Should match since "test_dir/foo.txt" matches and is under test_dir.
    assert_eq!(matched.len(), 1);
}
