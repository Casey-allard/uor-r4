//! LaunchAgent installer. Writes the plist only; the human runs the printed
//! `launchctl bootstrap` command. Tests exercise `write_plist` against a
//! temporary path and never touch launchd.

use std::path::{Path, PathBuf};

use crate::{invalid, Result};

pub const AGENT_LABEL: &str = "org.uor.lab-runner";

/// Resolve all existing prefixes, including dangling symlinks (which fail),
/// then bind future suffixes to the real internal filesystem location.
fn internal_path(path: &Path) -> Result<PathBuf> {
    use std::path::Component;
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(invalid(
            "supervisor paths must be absolute without parent traversal",
        ));
    }
    let ancestor = path
        .ancestors()
        .find(|p| std::fs::symlink_metadata(p).is_ok())
        .ok_or_else(|| invalid("supervisor path has no observable ancestor"))?;
    let canonical = std::fs::canonicalize(ancestor)?;
    let suffix = path
        .strip_prefix(ancestor)
        .map_err(|_| invalid("invalid path suffix"))?;
    let resolved = if suffix.as_os_str().is_empty() {
        canonical.clone()
    } else {
        canonical.join(suffix)
    };
    if resolved.starts_with("/Volumes") {
        return Err(invalid(
            "supervisor and control state must remain on internal storage",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let device = std::fs::metadata(&canonical)?.dev();
        let internal = ["/", "/System/Volumes/Data"]
            .iter()
            .filter_map(|path| std::fs::metadata(path).ok())
            .any(|m| m.dev() == device);
        if !internal {
            return Err(invalid(
                "supervisor path is not on an internal system filesystem",
            ));
        }
    }
    Ok(resolved)
}

pub fn default_plist_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| invalid("HOME is not set; cannot locate LaunchAgents"))?;
    Ok(PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("{AGENT_LABEL}.plist")))
}

pub fn plist_content(program: &Path, root: &Path, ledger_dir: &Path) -> String {
    let escape = |path: &Path| {
        path.to_string_lossy()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    };
    let stdout = root.join("daemon.out.log");
    let stderr = root.join("daemon.err.log");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{AGENT_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{program}</string>
        <string>--root</string>
        <string>{root}</string>
        <string>--ledger-dir</string>
        <string>{ledger}</string>
        <string>daemon</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>ThrottleInterval</key>
    <integer>30</integer>
    <key>RunAtLoad</key>
    <true/>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
    </dict>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
</dict>
</plist>
"#,
        program = escape(program),
        root = escape(root),
        ledger = escape(ledger_dir),
        stdout = escape(&stdout),
        stderr = escape(&stderr),
    )
}

/// Write the plist to `plist_path` and return the bootstrap command for a
/// human to run. Does not invoke launchctl.
pub fn write_plist(
    plist_path: &Path,
    program: &Path,
    root: &Path,
    ledger_dir: &Path,
) -> Result<String> {
    let program = internal_path(program)?;
    let metadata = std::fs::metadata(&program)?;
    if !metadata.is_file() {
        return Err(invalid("supervisor executable is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(invalid("supervisor program is not executable"));
        }
    }
    let root = internal_path(root)?;
    let ledger_dir = internal_path(ledger_dir)?;
    let plist_path = internal_path(plist_path)?;
    for name in [
        "queue",
        "running",
        "done",
        "locks",
        "daemon.out.log",
        "daemon.err.log",
        "daemon.lock",
        "daemon.json",
        "host-policy.json",
        "admissions-held.json",
    ] {
        internal_path(&root.join(name))?;
    }
    crate::jobs::ensure_layout(&root)?;
    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::atomic_write(
        &plist_path,
        plist_content(&program, &root, &ledger_dir).as_bytes(),
    )?;
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|uid| !uid.is_empty())
        .unwrap_or_else(|| "$(id -u)".to_string());
    Ok(format!(
        "launchctl bootstrap gui/{uid} '{}'",
        plist_path.to_string_lossy().replace('\'', "'\\''")
    ))
}

use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_mentions_label_paths_and_keepalive() {
        let content = plist_content(
            Path::new("/usr/local/bin/lab-runner"),
            Path::new("/Volumes/UOR-Workspace/runner"),
            Path::new("/home/u/.uor-models/native-joint-learning-2026-09-04"),
        );
        assert!(content.contains("<key>EnvironmentVariables</key>"));
        assert!(content.contains("/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"));
        assert!(content.contains("<string>org.uor.lab-runner</string>"));
        assert!(content.contains("<key>KeepAlive</key>"));
        assert!(content.contains("<key>RunAtLoad</key>"));
        assert!(content.contains("/usr/local/bin/lab-runner"));
        assert!(content.contains("<string>daemon</string>"));
        assert!(content.contains("daemon.out.log"));
        assert!(content.contains("daemon.err.log"));
    }

    #[test]
    fn write_plist_targets_a_tempdir_and_prints_bootstrap() {
        let dir = std::env::temp_dir().join(format!(
            "lab-runner-agent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let plist = dir.join("org.uor.lab-runner.plist");
        let command = write_plist(
            &plist,
            &std::env::current_exe().unwrap(),
            &dir.join("runner"),
            &dir.join("ledger"),
        )
        .unwrap();
        assert!(plist.is_file());
        assert!(command.starts_with("launchctl bootstrap gui/"));
        assert!(command.contains(std::fs::canonicalize(&plist).unwrap().to_str().unwrap()));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn installer_rejects_missing_program_and_dangling_external_aliases() {
        use std::os::unix::fs::symlink;
        let dir = std::env::temp_dir().join(format!(
            "uor-agent-alias-{}-{}",
            std::process::id(),
            crate::jobs::now_ms()
        ));
        std::fs::create_dir(&dir).unwrap();
        let plist = dir.join("agent.plist");
        assert!(write_plist(
            &plist,
            &dir.join("missing"),
            &dir.join("runner"),
            &dir.join("ledger")
        )
        .is_err());
        let alias = dir.join("ledger-alias");
        symlink("/Volumes/uor-missing-install-fixture", &alias).unwrap();
        assert!(write_plist(
            &plist,
            &std::env::current_exe().unwrap(),
            &dir.join("runner"),
            &alias.join("ledger")
        )
        .is_err());
        let root = dir.join("runner");
        std::fs::create_dir(&root).unwrap();
        symlink(
            "/Volumes/uor-missing-install-fixture",
            root.join("daemon.out.log"),
        )
        .unwrap();
        assert!(write_plist(
            &plist,
            &std::env::current_exe().unwrap(),
            &root,
            &dir.join("ledger")
        )
        .is_err());
        assert!(!plist.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
