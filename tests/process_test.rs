use bodo::process::ProcessManager;

#[test]
fn test_process_spawning() -> std::io::Result<()> {
    let mut pm = ProcessManager::new(false);
    pm.spawn_command("test", "echo hello", false, None, None)?;
    pm.run_concurrently()?;
    Ok(())
}

#[test]
fn test_fail_fast() -> std::io::Result<()> {
    let mut pm = ProcessManager::new(true);
    pm.spawn_command("failing", "exit 1", false, None, None)?;
    let result = pm.run_concurrently();
    assert!(result.is_err());
    Ok(())
}
