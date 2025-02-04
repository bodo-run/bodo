/*
This is a dummy test file to simulate a dry-run mode test.
Currently, the ExecutionPlugin does not implement a real dry-run.
This test calls expand_env_vars on a sample command and ensures
the output is as expected.
*/
use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_execution_plugin_dry_run_stub() {
    let plugin = ExecutionPlugin::new();
    // In a real dry-run mode, commands would not execute.
    // For now, we simulate by checking that expand_env_vars returns expected value.
    let envs = HashMap::from([("TEST".to_string(), "value".to_string())]);
    let cmd = "echo $TEST";
    let expanded = plugin.expand_env_vars(cmd, &envs);
    assert_eq!(expanded, "echo value");
}
