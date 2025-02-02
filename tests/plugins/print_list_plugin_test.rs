use super::*;
use bodo::plugin::Plugin;

#[test]
fn test_print_list_plugin_implementation() {
    let plugin = PrintListPlugin;
    assert_eq!(plugin.name(), "PrintListPlugin");
    assert_eq!(plugin.priority(), 0);
}