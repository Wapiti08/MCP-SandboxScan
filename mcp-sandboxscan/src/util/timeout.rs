use std::process::{Command, ExitStatus};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Result, bail};

pub fn command_status_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<ExitStatus> {
    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("failed to spawn command: {e}"))?;
    let pid = child.id();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(status)) => Ok(status),
        Ok(Err(e)) => Err(e.into()),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = Command::new("kill").args(["-9", &pid.to_string()]).status();
            match rx.recv_timeout(Duration::from_secs(5)) {
                Ok(Ok(status)) => Ok(status),
                Ok(Err(e)) => Err(e.into()),
                _ => bail!("command timed out after {}s", timeout.as_secs()),
            }
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            bail!("command worker exited unexpectedly");
        }
    }
}
