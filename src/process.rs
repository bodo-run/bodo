/* Existing imports and code... */
pub fn run_concurrently(&mut self) -> std::io::Result<()> {
    debug!("Running {} processes concurrently", self.children.len());
    let mut any_failed = false;

    let mut children = std::mem::take(&mut self.children);
    let len = children.len();

    for i in 0..len {
        let status = children[i].child.wait()?;
        if !status.success() {
            let code = status.code().unwrap_or(-1);
            warn!(
                "Process '{}' failed with exit code {}",
                children[i].name, code
            );
            any_failed = true;
            if self.fail_fast {
                debug!("Fail-fast enabled, killing remaining processes");
                for child in children.iter_mut().skip(i + 1) {
                    let _ = child.child.kill();
                }
                break;
            }
        } else {
            debug!("Process '{}' completed successfully", children[i].name);
        }
    }

    for mut child_info in children {
        if let Some(handle) = child_info.stdout_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = child_info.stderr_handle.take() {
            let _ = handle.join();
        }
    }

    if any_failed {
        debug!("One or more processes failed");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "One or more processes failed",
        ));
    }

    Ok(())
}
