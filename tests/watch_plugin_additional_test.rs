extern crate globset;
use crate::plugins::watch_plugin::WatchPlugin;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use tempfile::tempdir;

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
    // Pattern starts with **/ should return "."
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
    builder.add(globset::Glob::new("test_dir/foo.txt").unwrap());
    let glob_set = builder.build().unwrap();

    // Create a dummy WatchEntry with a directory to watch: "test_dir"
    let watch_entry = crate::plugins::watch_plugin::WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut set = std::collections::HashSet::new();
            set.insert(Path::new("test_dir").to_path_buf());
            set
        },
        debounce_ms: 500,
    };

    // Use a temporary directory
    let temp_dir = tempdir().unwrap();
    env::set_current_dir(&temp_dir).unwrap();
    // Create "test_dir" and a file "foo.txt" inside it
    fs::create_dir_all("test_dir").unwrap();
    let file_path = Path::new("test_dir").join("foo.txt");
    fs::write(&file_path, "content").unwrap();

    // Build changed_paths using the absolute path of the file
    let current = env::current_dir().unwrap();
    let changed_path = current.join("test_dir").join("foo.txt");
    let changed_paths = vec![changed_path];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matched.len(), 1);
}
