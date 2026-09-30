//! `lab-runner` CLI. Manual argv parsing, following the uor-r4-integer
//! binary pattern. Global flags `--root PATH` / `--ledger-dir PATH` may
//! appear before or after the subcommand; `UOR_RUNNER_ROOT` overrides the
//! default queue root.

use lab_runner::daemon::{self, DaemonConfig};
use lab_runner::{agent, coord, delivery, github_sync, jobs, ledger, RunnerError};
use std::path::PathBuf;
use std::time::Duration;

fn invalid(message: impl Into<String>) -> RunnerError {
    RunnerError::Invalid(message.into())
}

const USAGE: &str = "usage: lab-runner [--root PATH] [--ledger-dir PATH] <command>
  submit <spec.json>        validate and queue a job
  daemon [--poll-ms N] [--monitor-ms N] [--stop-file PATH]
                            run the foreground daemon (launchd KeepAlive owns restarts)
  status [id]               JSON summary of queue/running/done
  tail <id>                 last ~50 lines of the job's logs
  cancel <id>               remove a queued job or kill a running one
  reconcile-stopped <id>    preserve UNKNOWN result and prove stopped ownership
  install-agent             write the LaunchAgent plist and print the bootstrap command
  ledger rebuild            recompute from immutable charges and explicit baseline
  ledger migrate|extend|import-legacy|charge-resource|observe-legacy-snapshot RECORD_JSON
  coord init STORE REMOTE OWNER/REPO POLICY_SHA
  coord status STORE | apply STORE EVENT_JSON
  delivery check|enqueue RECEIPT_JSON
  github-sync OWNER/REPO OUTPUT_ROOT
  maintenance check REQUEST_JSON COORD_STORE
  outbox send STORE EVENT_JSON | replay STORE
  outbox retire STORE EVENT_ID REASON
  daemon --test-mode        only bounded inert fixtures under temporary roots";

struct Globals {
    root: PathBuf,
    ledger_dir: PathBuf,
}

fn parse_globals(args: &[String]) -> Result<(Globals, Vec<String>), RunnerError> {
    let mut root = std::env::var_os("UOR_RUNNER_ROOT")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(lab_runner::default_root)?;
    let mut ledger_dir: Option<PathBuf> = None;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                root = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| invalid("--root requires a path"))?,
                );
            }
            "--ledger-dir" => {
                index += 1;
                ledger_dir = Some(PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| invalid("--ledger-dir requires a path"))?,
                ));
            }
            other => rest.push(other.to_string()),
        }
        index += 1;
    }
    let ledger_dir = match ledger_dir {
        Some(dir) => dir,
        None => lab_runner::default_ledger_dir()?,
    };
    Ok((Globals { root, ledger_dir }, rest))
}

fn flag_value(args: &[String], name: &str) -> Result<Option<String>, RunnerError> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == name {
            return Ok(Some(
                args.get(index + 1)
                    .ok_or_else(|| invalid(format!("{name} requires a value")))?
                    .clone(),
            ));
        }
        index += 1;
    }
    Ok(None)
}

fn run() -> Result<(), RunnerError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (globals, rest) = parse_globals(&args)?;
    match rest.first().map(String::as_str) {
        Some("outbox") if rest.len() >= 3 => {
            let store = PathBuf::from(&rest[2]);
            match rest[1].as_str() {
                "send" if rest.len() == 4 => {
                    let event = serde_json::from_slice(&std::fs::read(&rest[3])?)?;
                    println!("{}", lab_runner::outbox::send(&store, &event)?);
                }
                "replay" if rest.len() == 3 => println!(
                    "{}",
                    serde_json::to_string_pretty(&lab_runner::outbox::replay(&store)?)?
                ),
                "retire" if rest.len() == 5 => println!(
                    "{}",
                    serde_json::to_string_pretty(&lab_runner::outbox::retire(
                        &store, &rest[3], &rest[4]
                    )?)?
                ),
                _ => return Err(invalid("outbox send STORE EVENT_JSON | replay STORE")),
            }
            Ok(())
        }
        Some("host-id") if rest.len() == 1 => {
            println!("{}", lab_runner::process::host_id()?);
            Ok(())
        }
        Some("maintenance") if rest.len() == 4 && rest[1] == "check" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&lab_runner::maintenance::check(
                    PathBuf::from(&rest[2]).as_path(),
                    PathBuf::from(&rest[3]).as_path()
                )?)?
            );
            Ok(())
        }
        // Internal read-only child command. It verifies the exact canonical
        // spec admitted by the parent; it cannot queue, launch or finalize work.
        Some("verify-inputs") if rest.len() == 3 => {
            let spec = lab_runner::spec::JobSpec::parse(&std::fs::read(&rest[1])?)?;
            if coord::digest(&serde_json::to_vec(&spec)?) != rest[2] {
                return Err(invalid("input verifier spec digest mismatch"));
            }
            spec.verify_provenance()
        }
        Some("submit") if rest.len() == 2 => {
            let id = jobs::submit(&globals.root, PathBuf::from(&rest[1]).as_path())?;
            println!("{id}");
            Ok(())
        }
        Some("daemon") => {
            let poll_ms = flag_value(&rest, "--poll-ms")?
                .map(|v| v.parse::<u64>())
                .transpose()
                .map_err(|_| invalid("--poll-ms must be a positive integer"))?
                .unwrap_or(1000);
            let monitor_ms = flag_value(&rest, "--monitor-ms")?
                .map(|v| v.parse::<u64>())
                .transpose()
                .map_err(|_| invalid("--monitor-ms must be a positive integer"))?
                .unwrap_or(2000);
            let stop_file = flag_value(&rest, "--stop-file")?.map(PathBuf::from);
            daemon::run(&DaemonConfig {
                root: globals.root,
                ledger_dir: globals.ledger_dir,
                poll_interval: Duration::from_millis(poll_ms),
                monitor_interval: Duration::from_millis(monitor_ms),
                stop_file,
                require_host_policy: !rest.iter().any(|arg| arg == "--test-mode"),
            })
        }
        Some("status") if rest.len() <= 2 => {
            let summary = jobs::status(&globals.root, rest.get(1).map(String::as_str))?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            Ok(())
        }
        Some("tail") if rest.len() == 2 => {
            print!("{}", jobs::tail(&globals.root, &rest[1], 50)?);
            Ok(())
        }
        Some("cancel") if rest.len() == 2 => {
            jobs::cancel(&globals.root, &rest[1], &globals.ledger_dir)
        }
        Some("reconcile-stopped") if rest.len() == 2 => {
            println!(
                "{}",
                jobs::reconcile_stopped(&globals.root, &rest[1], &globals.ledger_dir)?.display()
            );
            Ok(())
        }
        Some("install-agent") if rest.len() == 1 => {
            let program = std::env::current_exe()?;
            let plist = agent::default_plist_path()?;
            let command = agent::write_plist(&plist, &program, &globals.root, &globals.ledger_dir)?;
            println!("wrote {}", plist.display());
            println!("run: {command}");
            Ok(())
        }
        Some("ledger") if rest.len() == 2 && rest[1] == "rebuild" => {
            let state = ledger::rebuild(&globals.ledger_dir)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "cumulative_ms": state.cumulative_ms,
                    "limit_ms": state.limit_ms,
                }))?
            );
            Ok(())
        }
        Some("coord") => {
            match rest.get(1).map(String::as_str) {
                Some("init") if rest.len()==6 => println!("{}",coord::initialize(PathBuf::from(&rest[2]).as_path(),&rest[3],&rest[4],&rest[5])?),
                Some("status") if rest.len()==3 => println!("{}",serde_json::to_string_pretty(&coord::status(PathBuf::from(&rest[2]).as_path())?)?),
                Some("apply") if rest.len()==4 => {
                    let event=serde_json::from_slice(&std::fs::read(&rest[3])?)?;
                    println!("{}",coord::apply(PathBuf::from(&rest[2]).as_path(),&event)?);
                }
                _=>return Err(invalid("coord init STORE REMOTE OWNER/REPO POLICY_SHA | status STORE | apply STORE EVENT_JSON")),
            }
            Ok(())
        }
        Some("delivery") if rest.len() == 3 => {
            let path = PathBuf::from(&rest[2]);
            let decision = match rest[1].as_str() {
                "check" => delivery::check(&path)?,
                "enqueue" => delivery::enqueue(&path)?,
                _ => return Err(invalid("delivery check|enqueue RECEIPT_JSON")),
            };
            println!("{}", serde_json::to_string_pretty(&decision)?);
            Ok(())
        }
        Some("github-sync") if rest.len() == 3 => {
            println!(
                "{}",
                serde_json::to_string_pretty(&github_sync::sync_github(
                    &rest[1],
                    PathBuf::from(&rest[2]).as_path()
                )?)?
            );
            Ok(())
        }
        Some("ledger") if rest.len() == 3 => {
            let bytes = std::fs::read(&rest[2])?;
            match rest[1].as_str() {
                "migrate" => {
                    ledger::migrate(&globals.ledger_dir, &serde_json::from_slice(&bytes)?)?;
                }
                "extend" => {
                    ledger::record_extension(
                        &globals.ledger_dir,
                        &serde_json::from_slice(&bytes)?,
                    )?;
                }
                "charge-resource" => {
                    ledger::record_resource_charge(
                        &globals.ledger_dir,
                        &serde_json::from_slice(&bytes)?,
                    )?;
                }
                "import-legacy" => {
                    ledger::import_legacy(&globals.ledger_dir, &serde_json::from_slice(&bytes)?)?;
                }
                "observe-legacy-snapshot" => {
                    ledger::record_legacy_snapshot_observation(
                        &globals.ledger_dir,
                        &serde_json::from_slice(&bytes)?,
                    )?;
                }
                _ => {
                    return Err(invalid(
                        "ledger migrate|extend|import-legacy|charge-resource|observe-legacy-snapshot RECORD_JSON",
                    ))
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&ledger::rebuild(&globals.ledger_dir)?)?
            );
            Ok(())
        }
        _ => Err(invalid(USAGE)),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
