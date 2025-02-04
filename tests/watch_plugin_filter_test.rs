use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
use globset::{Glob, GlobSetBuilder};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_filter_changed_paths_with_nonexistent_file() {
    // Simulate a changed path that does not exist.
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("**/*.txt").unwrap());
    let glob_set = builder.build().unwrap();
    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut s = HashSet::new();
            s.insert(PathBuf::from("nonexistent_dir"));
            s
        },
        debounce_ms: 500,
    };
    // Provide a changed path which does not exist.
    let changed_paths = vec![PathBuf::from("nonexistent_dir/file.txt")];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    // Expect empty result since canonicalize should fail.
    assert!(
        matched.is_empty(),
        "Expected no files matched when file does not exist"
    );
}

#[test]
fn test_filter_changed_paths_matches_and_ignores() {
    // Set up temporary directory for file.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a subdirectory "watch_dir" and file "file.txt" inside it.
    let watch_dir = temp_path.join("watch_dir");
    fs::create_dir_all(&watch_dir).unwrap();
    let file_path = watch_dir.join("file.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
    }

    // Build a glob set that matches "watch_dir/file.txt"
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("watch_dir/file.txt").unwrap());
    let glob_set = builder.build().unwrap();

    // Build an ignore set that ignores "watch_dir/ignored.txt"
    let mut ignore_builder = GlobSetBuilder::new();
    ignore_builder.add(Glob::new("watch_dir/ignored.txt").unwrap());
    let ignore_set = ignore_builder.build().unwrap();

    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: Some(ignore_set),
        directories_to_watch: {
            let mut s = HashSet::new();
            s.insert(watch_dir.clone());
            s
        },
        debounce_ms: 500,
    };

    // Set current directory to temp_path so that relative path computation works.
    let orig_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    // Changed path that exists and should match.
    let changed_paths = vec![file_path.clone()];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matched.len(), 1, "Expected one matched file");

    // Now add an ignored file.
    let ignored_path = watch_dir.join("ignored.txt");
    {
        let mut f = File::create(&ignored_path).unwrap();
        writeln!(f, "ignore me").unwrap();
    }
    let changed_paths = vec![ignored_path.clone()];
    let matched_ignore = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    // Expect it to be filtered out.
    assert!(
        matched_ignore.is_empty(),
        "Expected ignored file not to match"
    );

    env::set_current_dir(orig_dir).unwrap();
}

#[test]
fn test_filter_changed_paths_not_under_watch() {
    // Setup temporary directory with a file not under a watched directory.
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create a file in "other_dir/file.txt"
    let other_dir = temp_path.join("other_dir");
    fs::create_dir_all(&other_dir).unwrap();
    let file_path = other_dir.join("file.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "content").unwrap();
    }

    // Build a glob set that matches "other_dir/file.txt"
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("other_dir/file.txt").unwrap());
    let glob_set = builder.build().unwrap();

    // Watch entry with a watch directory that does not include "other_dir"
    let watch_entry = WatchEntry {
        task_name: "dummy".to_string(),
        glob_set,
        ignore_set: None,
        directories_to_watch: {
            let mut s = HashSet::new();
            s.insert(PathBuf::from("watch_dir"));
            s
        },
        debounce_ms: 500,
    };

    // Set current directory to temp_path.
    let orig = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    let changed_paths = vec![file_path.clone()];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    // Expect no match because the file is not under the watched directory.
    assert!(
        matched.is_empty(),
        "Expected no match because file is not under watched directory"
    );

    env::set_current_dir(orig).unwrap();
}
