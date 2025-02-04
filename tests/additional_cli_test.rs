use bodo::cli::Args;
use clap::Parser;

#[test]
fn test_args_separator_handling() {
    // Ensure that the "--" separator works as expected.
    let args = Args::parse_from(["bodo", "task", "subtask", "--", "extra1", "extra2"]);
    assert_eq!(args.task, Some("task".to_string()));
    assert_eq!(args.subtask, Some("subtask".to_string()));
    assert_eq!(args.args, vec!["extra1".to_string(), "extra2".to_string()]);
}
