//! Native desktop client CLI / JSON-RPC & loopback Studio host runner (`uor-desktop-client`).
//!
//! Supports:
//! - `uor-desktop-client --self-test`: runs an end-to-end verification of the synthetic stand-in
//!   and durable memory store and prints a JSON receipt.
//! - `uor-desktop-client [--grant <dir> ...] --serve-studio <127.0.0.1:port>`: serves `web/studio/`
//!   over loopback HTTP with a per-launch 256-bit token and `Host`/`Origin` enforcement, and
//!   answers `POST /__tauri_invoke` (`window.__TAURI__.core.invoke`) against [`DesktopBackend`].
//! - `uor-desktop-client [--grant <dir> ...] [--stdio]`: runs a newline-delimited JSON-RPC loop
//!   driving [`DesktopBackend`].
#![forbid(unsafe_code)]

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use uor_r4_desktop_client::{
    agent_host::AgentHost, agent_server::StudioServer, generate_launch_token,
    handle_rpc_line_with_host, run_self_test_receipt, DesktopBackend, PinRegistry,
};

fn apply_cli_grants(args: &[String], backend: &mut DesktopBackend) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--grant" {
            let dir = args
                .get(i + 1)
                .ok_or_else(|| "missing directory path after --grant".to_string())?;
            backend
                .native_grant_folder(Path::new(dir))
                .map_err(|e| format!("--grant {dir}: {e}"))?;
            i += 2;
        } else if args[i] == "--enable-terminal" {
            backend.native_set_terminal_enabled(true);
            i += 1;
        } else if args[i] == "--terminal-home" {
            let home = args
                .get(i + 1)
                .ok_or_else(|| "missing directory after --terminal-home".to_string())?;
            backend
                .set_terminal_home(Path::new(home))
                .map_err(|error| format!("--terminal-home {home}: {error}"))?;
            i += 2;
        } else if args[i] == "--approve-shell" {
            let cwd = args
                .get(i + 1)
                .ok_or_else(|| "missing working directory after --approve-shell".to_string())?;
            let shell = args
                .get(i + 2)
                .ok_or_else(|| "missing shell after --approve-shell".to_string())?;
            backend
                .host_approve_terminal_shell(Path::new(cwd), Path::new(shell))
                .map_err(|error| format!("--approve-shell {cwd} {shell}: {error}"))?;
            i += 3;
        } else if args[i] == "--approve-command" {
            let cwd = args
                .get(i + 1)
                .ok_or_else(|| "missing working directory after --approve-command".to_string())?;
            let command = args
                .get(i + 2)
                .ok_or_else(|| "missing command after --approve-command".to_string())?;
            backend
                .host_approve_terminal_command(Path::new(cwd), command)
                .map_err(|error| format!("--approve-command {cwd} {command}: {error}"))?;
            i += 3;
        } else {
            i += 1;
        }
    }
    Ok(())
}

fn studio_root(args: &[String]) -> Result<std::path::PathBuf, String> {
    let root = match args.iter().position(|arg| arg == "--studio-root") {
        Some(index) => args
            .get(index + 1)
            .filter(|path| !path.starts_with("--"))
            .map(std::path::PathBuf::from)
            .ok_or_else(|| "--studio-root requires a directory value".to_string())?,
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/studio"),
    };
    let canonical = std::fs::canonicalize(&root)
        .map_err(|error| format!("invalid studio root {}: {error}", root.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "studio root is not a directory: {}",
            root.display()
        ));
    }
    Ok(canonical)
}

fn run_serve_studio(addr: &str, studio_root: &Path, args: &[String]) -> Result<(), String> {
    let mut host = AgentHost::new(DesktopBackend::new(PinRegistry::with_synthetic_standin()))?;
    apply_cli_grants(args, host.backend_mut())?;
    let launch_token = generate_launch_token();
    let server = StudioServer::start(addr, studio_root, Arc::new(Mutex::new(host)), launch_token)?;
    println!(
        "UOR-R4 Desktop Studio loopback host listening on http://{}/",
        server.bound_host()
    );
    server.serve();
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--self-test") {
        match run_self_test_receipt() {
            Ok(receipt) => println!("{receipt}"),
            Err(err) => {
                eprintln!("self-test error: {err}");
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(pos) = args.iter().position(|a| a == "--serve-studio") {
        let addr = args
            .get(pos + 1)
            .filter(|s| !s.starts_with("--"))
            .map(String::as_str)
            .unwrap_or("127.0.0.1:1420");
        let studio_root = studio_root(&args).unwrap_or_else(|error| {
            eprintln!("studio-root error: {error}");
            std::process::exit(2);
        });
        if let Err(err) = run_serve_studio(addr, &studio_root, &args) {
            eprintln!("serve-studio error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
    if let Err(err) = apply_cli_grants(&args, &mut backend) {
        eprintln!("grant error: {err}");
        std::process::exit(1);
    }

    #[cfg(feature = "gui")]
    if !args.iter().any(|a| a == "--stdio") {
        if let Err(err) = uor_r4_desktop_client::tauri_gui::run_tauri_gui(backend) {
            eprintln!("tauri gui error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut host = match AgentHost::new(backend) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("host startup error: {error}");
            std::process::exit(1);
        }
    };
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let resp = handle_rpc_line_with_host(&mut host, &line);
        let _ = writeln!(stdout, "{resp}");
        let _ = stdout.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_desktop_client::TerminalCommandRequest;

    #[test]
    fn studio_root_honors_the_explicit_dist_and_preserves_the_default() {
        assert!(studio_root(&[])
            .expect("default root")
            .ends_with("web/studio"));
        let parent = std::env::temp_dir().join(format!("uor-explicit-dist-{}", std::process::id()));
        let root = parent.join("dist");
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&root).expect("create explicit dist");
        let args = vec!["--studio-root".to_string(), root.display().to_string()];
        assert_eq!(
            studio_root(&args).expect("explicit dist"),
            root.canonicalize().expect("canonical dist")
        );
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn studio_root_rejects_missing_invalid_and_noncanonical_values() {
        for args in [
            vec!["--studio-root".to_string()],
            vec!["--studio-root".to_string(), "--enable-terminal".to_string()],
            vec![
                "--studio-root".to_string(),
                "missing-studio-root".to_string(),
            ],
        ] {
            assert!(
                studio_root(&args).is_err(),
                "accepted invalid root: {args:?}"
            );
        }
        let root = std::env::temp_dir().join(format!("uor-studio-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create non-dist root");
        let file = root.join("not-a-directory");
        std::fs::write(&file, "not a directory").expect("create regular-file root");
        assert!(studio_root(&["--studio-root".to_string(), file.display().to_string()]).is_err());
        let args = vec!["--studio-root".to_string(), root.display().to_string()];
        assert_eq!(
            studio_root(&args).expect("non-dist root"),
            root.canonicalize().expect("canonical root")
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cli_controls_preauthorize_exact_command_once() {
        let root = std::env::temp_dir().join(format!("uor-cli-approval-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create CLI approval workspace");
        let args = vec![
            "--grant".to_string(),
            root.display().to_string(),
            "--enable-terminal".to_string(),
            "--approve-command".to_string(),
            root.display().to_string(),
            "pwd".to_string(),
        ];
        let mut backend = DesktopBackend::new(PinRegistry::with_synthetic_standin());
        apply_cli_grants(&args, &mut backend).expect("apply host CLI controls");
        let request = TerminalCommandRequest {
            cwd: root.clone(),
            command: "pwd".to_string(),
            user_confirmed: true,
            confirmed_command: "pwd".to_string(),
        };
        let output = backend
            .native_run_terminal_command(&request)
            .expect("host-approved command runs once");
        assert!(output.contains(root.to_string_lossy().as_ref()));
        assert!(backend.native_run_terminal_command(&request).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
