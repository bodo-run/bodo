use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_expand_env_vars_no_var() {
    let plugin = ExecutionPlugin::new();
    let input = "echo $UNSET";
    let env = HashMap::new();
    let result = plugin.expand_env_vars(input, &env);
    // Variable not set remains unchanged.
    assert_eq!(result, "echo $UNSET");
}

#[test]
fn test_expand_env_vars_dollar_dollar() {
    let plugin = ExecutionPlugin::new();
    let input = "echo $$";
    let env = HashMap::new();
    let result = plugin.expand_env_vars(input, &env);
    assert_eq!(result, "echo $");
}

#[test]
fn test_expand_env_vars_braced() {
    let plugin = ExecutionPlugin::new();
    let mut env = HashMap::new();
    env.insert("VAR".to_string(), "value".to_string());
    let input = "echo ${VAR}";
    let result = plugin.expand_env_vars(input, &env);
    assert_eq!(result, "echo value");
}
