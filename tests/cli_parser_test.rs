use bodo::cli::Args;
use clap::Parser;

#[test]
fn test_cli_parser() {
    let args = Args::parse_from(["bodo", "--debug", "-l", "mytask", "subtask", "arg1", "arg2"]);
    assert_eq!(args.task, Some("mytask".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(args.debug);
    assert!(args.list);

    // Test default no-argument invocation.
    let default_args = Args::parse_from(["bodo"]);
    assert_eq!(default_args.task, None);
    assert_eq!(default_args.subtask, None);
    assert!(default_args.args.is_empty());
}
