// tests/execution_plugin_test.rs

use bodo::plugins::execution_plugin::expand_env_vars;
use std::collections::HashMap;

#[test]
fn test_expand_env_vars_basic() {
    let env_map = HashMap::from([
        ("VAR1".to_string(), "value1".to_string()),
        ("VAR2".to_string(), "value2".to_string()),
    ]);
    let input = "echo $VAR1 and $VAR2";
    let expected = "echo value1 and value2";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_no_match() {
    let env_map = HashMap::from([("VAR1".to_string(), "value1".to_string())]);
    let input = "echo $VAR2 and ${VAR3}";
    let expected = "echo $VAR2 and ${VAR3}";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_partial() {
    let env_map = HashMap::from([("HOME".to_string(), "/home/user".to_string())]);
    let input = "cd $HOME/projects";
    let expected = "cd /home/user/projects";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_special_chars() {
    let env_map = HashMap::from([("VAR".to_string(), "value".to_string())]);
    let input = "echo $$VAR $VAR$ $VAR text";
    let expected = "echo $VAR value$ value text";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}

#[test]
fn test_expand_env_vars_empty_var() {
    let env_map = HashMap::new();
    let input = "echo $";
    let expected = "echo $";
    let result = expand_env_vars(input, &env_map);
    assert_eq!(result, expected);
}
