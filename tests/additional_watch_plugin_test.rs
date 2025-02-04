use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
use globset::{Glob, GlobSetBuilder};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

#[test]
fn test_find_base_directory_starts_with_double_star() {
    // When pattern starts with "**/", expect "."
    let res = WatchPlugin::find_base_directory("**/foo/bar");
    assert_eq!(res, Some(PathBuf::from(".")));
}

#[test]
fn test_find_base_directory_no_wildcard() {
    // When no wildcard is present, simply return the directory or the file parent.
    let res = WatchPlugin::find_base_directory("src");
    // Since we are not verifying file-system, we assume "src" is returned.
    assert_eq!(res, Some(PathBuf::from("src")));
}

#[test]
fn test_find_base_directory_wildcard_in_middle() {
    let res = WatchPlugin::find_base_directory("src/*.rs");
    assert_eq!(res, Some(PathBuf::from("src")));
}

#[test]
fn test_filter_changed_paths_match() {
    // Create a temporary directory structure.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create subdirectory "watch_dir" inside temp_dir.
    let watch_dir = temp_path.join("watch_dir");
    std::fs::create_dir_all(&watch_dir).unwrap();

    // Create a file "watch_dir/test.txt".
    let file_path = watch_dir.join("test.txt");
    std::fs::write(&file_path, "dummy content").unwrap();

    // Build a glob set that matches "test.txt".
    let mut glob_builder = GlobSetBuilder::new();
    let glob = Glob::new("test.txt").unwrap();
    glob_builder.add(glob);
    let glob_set = glob_builder.build().unwrap();

    // No ignore set.
    let ignore_set = None;

    let mut directories_to_watch = HashSet::new();
    directories_to_watch.insert(watch_dir.clone());

    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set: glob_set.clone(),
        ignore_set,
        directories_to_watch,
        debounce_ms: 500,
    };

    // Set current directory to the temporary directory.
    let original_cwd = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    // Provide relative changed path.
    let changed_paths = vec![PathBuf::from("watch_dir/test.txt")];
    let matches = WatchPlugin::new(false, false).filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matches.len(), 1);

    // Restore original current directory.
    env::set_current_dir(&original_cwd).unwrap();
}

#[test]
fn test_filter_changed_paths_ignore() {
    // Test that a file matching the ignore pattern is skipped.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create subdirectory "watch_dir"
    let watch_dir = temp_path.join("watch_dir");
    std::fs::create_dir_all(&watch_dir).unwrap();

    // Create file "watch_dir/ignore.txt"
    let file_path = watch_dir.join("ignore.txt");
    std::fs::write(&file_path, "content").unwrap();

    // Build a glob set that matches "*.txt"
    let mut glob_builder = GlobSetBuilder::new();
    glob_builder.add(Glob::new("*.txt").unwrap());
    let glob_set = glob_builder.build().unwrap();

    // Build an ignore set that matches exactly "ignore.txt"
    let mut ignore_builder = GlobSetBuilder::new();
    ignore_builder.add(Glob::new("ignore.txt").unwrap());
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

    let original_cwd = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    let changed_paths = vec![PathBuf::from("watch_dir/ignore.txt")];
    let matches = WatchPlugin::new(false, false).filter_changed_paths(&changed_paths, &watch_entry);
    // Should be ignored.
    assert_eq!(matches.len(), 0);

    env::set_current_dir(&original_cwd).unwrap();
}
