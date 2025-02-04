use bodo::cli::Args;
use clap::Parser;

#[test]
fn test_cli_default_args() {
    let args = Args::parse_from(["bodo"]);
    assert_eq!(args.task, None);
    assert_eq!(args.subtask, None);
    assert!(args.args.is_empty());
    assert!(!args.no_watch);
}

#[test]
fn test_cli_with_task_and_options() {
    let args = Args::parse_from([
        "bodo", "--debug", "-l", "mytask", "subtask", "--", "arg1", "arg2",
    ]);
    assert_eq!(args.task, Some("mytask".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(args.debug);
    assert!(args.list);
    assert!(!args.no_watch);
}
