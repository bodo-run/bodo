use bodo::plugins::watch_plugin::{WatchEntry, WatchPlugin};
use globset::{Glob, GlobSetBuilder};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_filter_changed_paths_with_existing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_path).unwrap();

    let test_dir = temp_path.join("test_dir");
    fs::create_dir_all(&test_dir).unwrap();

    let file_path = test_dir.join("foo.txt");
    {
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
    }

    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("test_dir/foo.txt").unwrap());
    let glob_set = builder.build().unwrap();

    let watch_entry = WatchEntry {
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

    let changed_paths = vec![file_path.clone()];
    let plugin = WatchPlugin::new(false, false);
    let matched = plugin.filter_changed_paths(&changed_paths, &watch_entry);
    assert_eq!(matched.len(), 1);

    env::set_current_dir(original_dir).unwrap();
}
