use bodo::plugins::concurrent_plugin::ConcurrentPlugin;
use bodo::plugins::env_plugin::EnvPlugin;
use bodo::plugins::execution_plugin::ExecutionPlugin;
use bodo::plugins::path_plugin::PathPlugin;
use bodo::plugins::prefix_plugin::PrefixPlugin;
use bodo::plugins::print_list_plugin::PrintListPlugin;
use bodo::plugins::timeout_plugin::TimeoutPlugin;
use bodo::plugins::watch_plugin::WatchPlugin;
use bodo::Plugin;

#[test]
fn test_as_any_methods() {
    let cp = ConcurrentPlugin::new();
    let ep = EnvPlugin::new();
    let exp = ExecutionPlugin::new();
    let pp = PathPlugin::new();
    let prp = PrefixPlugin::new();
    let plp = PrintListPlugin;
    let tp = TimeoutPlugin::new();
    let wp = WatchPlugin::new(false, false);

    // Test downcasting using as_any.
    let _cp_down: &ConcurrentPlugin = cp.as_any().downcast_ref().unwrap();
    let _ep_down: &EnvPlugin = ep.as_any().downcast_ref().unwrap();
    let _exp_down: &ExecutionPlugin = exp.as_any().downcast_ref().unwrap();
    let _pp_down: &PathPlugin = pp.as_any().downcast_ref().unwrap();
    let _prp_down: &PrefixPlugin = prp.as_any().downcast_ref().unwrap();
    let _plp_down: &PrintListPlugin = plp.as_any().downcast_ref().unwrap();
    let _tp_down: &TimeoutPlugin = tp.as_any().downcast_ref().unwrap();
    let _wp_down: &WatchPlugin = wp.as_any().downcast_ref().unwrap();
}
