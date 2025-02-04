use bodo::plugins::execution_plugin::ExecutionPlugin;
use std::collections::HashMap;

#[test]
fn test_expand_env_vars_complex_scenarios() {
    let plugin = ExecutionPlugin::new();

    // Test with multiple variables and adjacent text
    let input = "Path:$HOME/bin:$PATH end";
    let mut env_vars = HashMap::new();
    env_vars.insert("HOME".to_string(), "/home/alice".to_string());
    env_vars.insert("PATH".to_string(), "/usr/bin".to_string());
    let result = plugin.expand_env_vars(input, &env_vars);
    assert_eq!(result, "Path:/home/alice/bin:/usr/bin end");

    // Test with adjacent variable syntax: "$$VAR" should become "$VAR"
    let input2 = "Cost: $$PRICE dollars";
    let result2 = plugin.expand_env_vars(input2, &HashMap::new());
    assert_eq!(result2, "Cost: $PRICE dollars");
}
