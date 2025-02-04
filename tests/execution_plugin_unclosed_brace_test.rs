use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_expand_env_vars_unclosed_brace() {
    let plugin = ExecutionPlugin::new();
    let env = HashMap::new();
    let input = "echo ${VAR";
    let result = plugin.expand_env_vars(input, &env);
    // When no closing brace is found, the implementation should return the variable wrapped with a closing brace.
    // Expected result: "echo ${VAR}"
    assert_eq!(result, "echo ${VAR}");
}
