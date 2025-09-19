use bodo::plugins::watch_plugin::WatchPlugin;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

#[test]
fn test_create_watcher_test() {
    let (watcher, rx) = WatchPlugin::create_watcher_test().expect("Failed to create watcher");
    // Expect timeout since no events occur.
    match rx.recv_timeout(Duration::from_millis(100)) {
        Err(RecvTimeoutError::Timeout) => {}
        _ => panic!("Expected timeout when no events occur"),
    }
    drop(watcher);
}
