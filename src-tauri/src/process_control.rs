use std::process::{Child, Command};

#[cfg(unix)]
pub(crate) fn prepare(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
pub(crate) fn prepare(_command: &mut Command) {}

pub(crate) fn terminate(child: &mut Child) -> bool {
    if !matches!(child.try_wait(), Ok(None)) {
        return false;
    }
    #[cfg(unix)]
    {
        let group = format!("-{}", child.id());
        let _ = Command::new("kill")
            .args(["-TERM", "--", &group])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = Command::new("kill")
            .args(["-KILL", "--", &group])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_termination_reports_no_second_signal() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "sleep 30"])
            .spawn()
            .expect("spawn termination fixture");
        assert!(terminate(&mut child));
        assert!(!terminate(&mut child));
    }
}
