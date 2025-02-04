use bodo::cli::Args;
use clap::Parser;

#[test]
fn test_args_separator_handling() {
    let args = Args::parse_from(["bodo", "task", "subtask", "--", "extra1", "extra2"]);
    assert_eq!(args.task, Some("task".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["extra1".to_string(), "extra2".to_string()]);
    // Ensure default no_watch is false.
    assert!(!args.no_watch);
}

#[test]
fn test_default_no_arg_invocation() {
    let default_args = Args::parse_from(["bodo"]);
    assert_eq!(default_args.task, None);
    assert_eq!(default_args.subtask, None);
    assert!(default_args.args.is_empty());
    assert!(!default_args.no_watch);
}
