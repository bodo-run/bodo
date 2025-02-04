use bodo::cli::Args;
use clap::Parser;

#[test]
fn test_args_default_values() {
    let args = Args::parse_from(["bodo"]);
    assert_eq!(args.task, None);
    assert_eq!(args.subtask, None);
    assert!(args.args.is_empty());
}

#[test]
fn test_args_with_all_options() {
    let args = Args::parse_from([
        "bodo",
        "--list",
        "--watch",
        "--auto-watch",
        "--debug",
        "taskname",
        "subtaskname",
        "--",
        "arg1",
        "arg2",
    ]);
    assert!(args.list);
    assert!(args.watch);
    assert!(args.auto_watch);
    assert!(args.debug);
    assert_eq!(args.task, Some("taskname".to_string()));
    assert_eq!(args.subtask, Some("subtaskname".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
}
