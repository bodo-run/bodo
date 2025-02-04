use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_find_base_directory() {
    // When pattern starts with **/, expect "."
    let base = WatchPlugin::find_base_directory("**/foo/bar").unwrap();
    assert_eq!(base, PathBuf::from("."));
}

#[test]
fn test_find_base_directory_with_no_wildcard() {
    let base = WatchPlugin::find_base_directory("src").unwrap();
    assert_eq!(base, PathBuf::from("src"));
}

#[test]
fn test_find_base_directory_with_wildcard_in_middle() {
    let base = WatchPlugin::find_base_directory("src/*.rs").unwrap();
    assert_eq!(base, PathBuf::from("src"));
}

#[test]
fn test_filter_changed_paths() {
    // Create a temporary directory structure and file.
    let temp_dir = tempfile::tempdir().unwrap();
    let watch_dir = temp_dir.path().join("watch_dir");
    fs::create_dir_all(&watch_dir).unwrap();
    let file_path = watch_dir.join("foo.txt");
    fs::write(&file_path, "dummy").unwrap();

    let mut directories_to_watch = HashSet::new();
    directories_to_watch.insert(watch_dir.clone());

    let mut glob_builder = globset::GlobSetBuilder::new();
    let glob = globset::Glob::new("foo.txt").unwrap();
    glob_builder.add(glob);
    let glob_set = glob_builder.build().unwrap();

    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch,
        debounce_ms: 500,
    };

    // Set current directory to temp_dir
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let changed_paths = vec![PathBuf::from("watch_dir/foo.txt")];
    let matches = WatchPlugin::new(false, false).filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matches.len(), 1);

    // Restore original directory
    env::set_current_dir(&original_dir).unwrap();
}

#[test]
fn test_filter_changed_paths_ignore() {
    // Create a temporary directory structure and file.
    let temp_dir = tempfile::tempdir().unwrap();
    let watch_dir = temp_dir.path().join("watch_dir");
    fs::create_dir_all(&watch_dir).unwrap();
    let file_path = watch_dir.join("ignore.txt");
    fs::write(&file_path, "content").unwrap();

    let mut glob_builder = globset::GlobSetBuilder::new();
    glob_builder.add(globset::Glob::new("*.txt").unwrap());
    let glob_set = glob_builder.build().unwrap();

    let mut ignore_builder = globset::GlobSetBuilder::new();
    ignore_builder.add(globset::Glob::new("ignore.txt").unwrap());
    let ignore_set = Some(ignore_builder.build().unwrap());

    let mut directories_to_watch = HashSet::new();
    directories_to_watch.insert(watch_dir.clone());

    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set,
        directories_to_watch,
        debounce_ms: 500,
    };

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let changed_paths = vec![PathBuf::from("watch_dir/ignore.txt")];
    let matches = WatchPlugin::new(false, false).filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matches.len(), 0);

    env::set_current_dir(&original_dir).unwrap();
}

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
