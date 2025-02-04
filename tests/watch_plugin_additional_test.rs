use bodo::plugins::watch_plugin::WatchPlugin;
use globset::{Glob, GlobSetBuilder};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

#[test]
fn test_find_base_directory_with_double_star() {
    // Pattern starts with **/, should return "."
    let base = WatchPlugin::find_base_directory("**/foo/bar").unwrap();
    assert_eq!(base, PathBuf::from("."));
}

#[test]
fn test_find_base_directory_with_no_wildcard() {
    // For a path without wildcards, return its parent (if file) or itself (if directory).
    // Since the function checks for wildcards, for "src", it returns "src" if not empty.
    let base = WatchPlugin::find_base_directory("src").unwrap();
    assert_eq!(base, PathBuf::from("src"));
}

#[test]
fn test_find_base_directory_with_wildcard_in_middle() {
    // "src/*.rs" should return "src"
    let base = WatchPlugin::find_base_directory("src/*.rs").unwrap();
    assert_eq!(base, PathBuf::from("src"));
}

#[test]
fn test_filter_changed_paths() {
    // Build a glob set that matches "foo.txt"
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("foo.txt").unwrap());
    let glob_set = builder.build().unwrap();

    // Create a dummy WatchEntry with a directory to watch: "test_dir"
    let watch_entry = bodo::plugins::watch_plugin::WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut set = HashSet::new();
            set.insert(PathBuf::from("test_dir"));
            set
        },
        debounce_ms: 500,
    };

    // Get the current working directory.
    let cwd = env::current_dir().unwrap();
    // Create a changed path that is within "test_dir"
    let changed_path = cwd.join("test_dir").join("foo.txt");
    let changed_paths = vec![changed_path.clone()];
    let matched = WatchPlugin::filter_changed_paths(&changed_paths, &watch_entry);
    // Should match since "foo.txt" matches and is under test_dir.
    assert_eq!(matched.len(), 1);
    // Now create a changed path outside the watched directory.
    let outside_path = cwd.join("other").join("foo.txt");
    let matched = WatchPlugin::filter_changed_paths(&[outside_path], &watch_entry);
    assert_eq!(matched.len(), 0);
}
