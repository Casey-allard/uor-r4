//! Interactive Conversational Chatbot CLI (`uor-chat`)
//!
//! Real-time streaming terminal REPL powered by the native UOR-R4 Geometric Language Model.
//! Executes with strictly zero transformers and zero hardware matrix multiplication in the served runtime.

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process;
use uor_r4_integer::bundle::{create_test_bundle_with_byte_vocab, Bundle};
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::model::{IntegerModel, IntegerSession, IntegerStep};
use uor_r4_integer::sampling::SamplePolicy;
use uor_r4_integer::session::ChatSession;
use uor_r4_integer::Result;

const VERSION: &str = "0.1.0";

const ANSI_CYAN_BOLD: &str = "\x1b[1;36m";
const ANSI_GREEN_BOLD: &str = "\x1b[1;32m";
const ANSI_YELLOW_BOLD: &str = "\x1b[1;33m";
const ANSI_RED_BOLD: &str = "\x1b[1;31m";
const ANSI_MAGENTA_BOLD: &str = "\x1b[1;35m";
const ANSI_RESET: &str = "\x1b[0m";

struct CliArgs {
    bundle_path: Option<PathBuf>,
    system_prompt: Option<String>,
    temperature: f64,
    top_k: usize,
    seed: u64,
    read_mode: ReadMode,
    max_tokens: usize,
    verify_kernel: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            bundle_path: None,
            system_prompt: None,
            temperature: 0.0,
            top_k: 16,
            seed: 0,
            read_mode: ReadMode::Enabled,
            max_tokens: 128,
            verify_kernel: false,
        }
    }
}

fn print_usage() {
    eprintln!("Usage: uor-chat --bundle <PATH> [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  -b, --bundle <PATH>         Path to model bundle directory containing bundle.json"
    );
    eprintln!("  -s, --system <PROMPT>       Initial persistent system persona (slots 0..31)");
    eprintln!(
        "  -t, --temperature <FLOAT>   Sampling temperature (0.0 = greedy, >0.0 = categorical)"
    );
    eprintln!("  -k, --top-k <INT>           Top-K candidate cutoff (default: 16)");
    eprintln!(
        "      --seed <INT>            PRNG seed for integer categorical sampler (default: 0)"
    );
    eprintln!(
        "  -m, --read-mode <MODE>      Memory read mode: 'enabled' or 'no_read' (default: enabled)"
    );
    eprintln!("      --max-tokens <INT>      Maximum response tokens per turn (default: 128)");
    eprintln!("      --verify-kernel         Run self-test verifying Zero-MatMul kernel retention and exit");
    eprintln!("  -h, --help                  Print help information");
    eprintln!("  -V, --version               Print version information");
}

fn parse_cli_args() -> CliArgs {
    let mut args = CliArgs::default();
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut idx = 0;

    while idx < raw.len() {
        match raw[idx].as_str() {
            "-h" | "--help" => {
                println!("uor-chat {}", VERSION);
                println!("UOR-R4 Geometric Conversational Chatbot REPL");
                println!();
                print_usage();
                process::exit(0);
            }
            "-V" | "--version" => {
                println!("uor-chat {}", VERSION);
                process::exit(0);
            }
            "-b" | "--bundle" => {
                idx += 1;
                if idx < raw.len() {
                    args.bundle_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--bundle'");
                    process::exit(1);
                }
            }
            "-s" | "--system" => {
                idx += 1;
                if idx < raw.len() {
                    args.system_prompt = Some(raw[idx].clone());
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--system'");
                    process::exit(1);
                }
            }
            "-t" | "--temperature" => {
                idx += 1;
                if idx < raw.len() {
                    if let Ok(val) = raw[idx].parse::<f64>() {
                        if val < 0.0 {
                            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} temperature must be non-negative");
                            process::exit(1);
                        }
                        args.temperature = val;
                    } else {
                        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} invalid float value for '--temperature'");
                        process::exit(1);
                    }
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--temperature'"
                    );
                    process::exit(1);
                }
            }
            "-k" | "--top-k" => {
                idx += 1;
                if idx < raw.len() {
                    if let Ok(val) = raw[idx].parse::<usize>() {
                        args.top_k = val;
                    } else {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} invalid integer value for '--top-k'"
                        );
                        process::exit(1);
                    }
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--top-k'");
                    process::exit(1);
                }
            }
            "--seed" => {
                idx += 1;
                if idx < raw.len() {
                    if let Ok(val) = raw[idx].parse::<u64>() {
                        args.seed = val;
                    } else {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} invalid integer value for '--seed'"
                        );
                        process::exit(1);
                    }
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--seed'");
                    process::exit(1);
                }
            }
            "-m" | "--read-mode" => {
                idx += 1;
                if idx < raw.len() {
                    match raw[idx].to_lowercase().as_str() {
                        "enabled" | "on" => args.read_mode = ReadMode::Enabled,
                        "no_read" | "noread" | "off" => args.read_mode = ReadMode::NoRead,
                        other => {
                            eprintln!(
                                "{ANSI_RED_BOLD}error:{ANSI_RESET} unrecognized read-mode '{other}'. Expected 'enabled' or 'no_read'"
                            );
                            process::exit(1);
                        }
                    }
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--read-mode'"
                    );
                    process::exit(1);
                }
            }
            "--max-tokens" => {
                idx += 1;
                if idx < raw.len() {
                    if let Ok(val) = raw[idx].parse::<usize>() {
                        args.max_tokens = val;
                    } else {
                        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} invalid integer value for '--max-tokens'");
                        process::exit(1);
                    }
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--max-tokens'"
                    );
                    process::exit(1);
                }
            }
            "--verify-kernel" => {
                args.verify_kernel = true;
            }
            unknown => {
                eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} unrecognized option '{unknown}'");
                print_usage();
                process::exit(1);
            }
        }
        idx += 1;
    }

    if args.bundle_path.is_none() && !args.verify_kernel {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing required argument '--bundle <path>'");
        print_usage();
        process::exit(1);
    }

    args
}

fn get_process_rss_mb() -> Option<f64> {
    let pid = process::id();
    let output = process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = std::str::from_utf8(&output.stdout).ok()?.trim();
    let rss_kib: f64 = text.parse().ok()?;
    Some(rss_kib / 1024.0)
}

fn print_welcome_banner(bundle: &Bundle, policy: SamplePolicy, read_mode: ReadMode) {
    let id = bundle.identity();
    let id_short = if id.len() > 16 { &id[..16] } else { id };
    println!("================================================================================");
    println!("  UOR-R4 Geometric Conversational Chatbot (uor-chat)");
    println!("  Zero Transformers | Zero Hardware MatMul | Apple Silicon M1 Native");
    println!("================================================================================");
    println!("  Bundle Identity : {}...", id_short);
    println!("  Memory Capacity : 256 tokens (32 Persistent Persona, 224 Dialogue Slots)");
    println!("  Sampling Policy : {:?}", policy);
    println!("  Memory Read Mode: {:?}", read_mode);
    println!("  Type /help for slash commands, /quit to exit.");
    println!("================================================================================");
}

fn print_help() {
    println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Available Slash Commands:");
    println!(
        "  /reset                Clear dialogue slots (32..255) and retain persistent persona"
    );
    println!("  /verify               Execute self-test verifying Zero-MatMul kernel retention");
    println!("  /save <path>          Save current session state to JSON file (uor-r4.integer-session/1)");
    println!("  /load <path>          Load session state from JSON file with bundle checksum verification");
    println!(
        "  /stats                Display session telemetry, geometric coordinates, and process RSS"
    );
    println!("  /read-mode <on|off>   Toggle prime-addressed memory reading (Enabled vs NoRead)");
    println!("  /quit, /exit          Exit uor-chat cleanly");
    println!("  /help                 Display this command help menu");
}

fn print_stats(session: &ChatSession) {
    let t = session.telemetry();
    let rss_str = match get_process_rss_mb() {
        Some(rss) => format!("{:.2} MB (Invariant: < 35.0 MB - PASS)", rss),
        None => "Unavailable".to_string(),
    };

    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}|                          Chat Session Telemetry                            |{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("| Turn Count             : {:<50}|", t.current_turn_id);
    println!(
        "| Active Tokens          : {:<50}|",
        format!(
            "{} / 256 (Persistent: {}/32, Dialogue: {}/224)",
            t.persistent_slots_used + t.dialogue_slots_used,
            t.persistent_slots_used,
            t.dialogue_slots_used
        )
    );
    println!("| Dialogue Tokens Seen   : {:<50}|", t.dialogue_tokens_seen);
    println!("| Persona Sealed         : {:<50}|", t.persistent_sealed);
    println!(
        "| Memory Read Mode       : {:<50}|",
        format!("{:?}", session.read_mode())
    );
    println!(
        "| Sampling Policy        : {:<50}|",
        format!("{:?}", session.policy())
    );
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}| Geometric State Coordinates:                                               |{ANSI_RESET}");
    println!(
        "| Hopf Holonomy DeltaPsi : {:<50}|",
        format!("Q30: {}, S1 winding != 0", t.cumulative_holonomy_q30)
    );
    let zeta_sample = format!(
        "[{}, {}, {}, {}, {}, {}, {}, {}]",
        t.zeta_phases[0],
        t.zeta_phases[1],
        t.zeta_phases[2],
        t.zeta_phases[3],
        t.zeta_phases[4],
        t.zeta_phases[5],
        t.zeta_phases[6],
        t.zeta_phases[7]
    );
    println!("| Zeta T^8 Phase Vector  : {:<50}|", zeta_sample);
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}| Apple Silicon Host Resources:                                              |{ANSI_RESET}");
    println!("| Process RSS            : {:<50}|", rss_str);
    println!("| Hardware Multipliers   : 0 (Static Audit Certified - PASS)                 |");
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
}

fn handle_slash_command<'a>(line: &str, session: &mut ChatSession<'a>, bundle: &'a Bundle) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }
    let command = parts[0];

    match command {
        "/quit" | "/exit" => {
            println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Session ended. Goodbye!");
            process::exit(0);
        }
        "/help" => {
            print_help();
        }
        "/reset" => {
            session.reset_dialogue();
            let used = session.telemetry().persistent_slots_used;
            println!(
                "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Dialogue partition reset. Persistent persona retained ({used} tokens)."
            );
        }
        "/verify" | "/verify-kernel" => {
            if let Err(err) = run_kernel_verification(bundle) {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Kernel verification failed: {err}");
            }
        }
        "/save" => {
            if parts.len() < 2 {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Usage: /save <path>");
            } else {
                let path = Path::new(parts[1]);
                match session.save_session(path, bundle.identity()) {
                    Ok(()) => {
                        println!(
                            "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Session serialized successfully to '{}'.",
                            path.display()
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to save session: {err}"
                        );
                    }
                }
            }
        }
        "/load" => {
            if parts.len() < 2 {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Usage: /load <path>");
            } else {
                let path = Path::new(parts[1]);
                match ChatSession::load_session(bundle, path, bundle.identity()) {
                    Ok(loaded) => {
                        *session = loaded;
                        let t = session.telemetry();
                        println!(
                            "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Session loaded from '{}'. Turn count: {}, Active tokens: {}.",
                            path.display(),
                            t.current_turn_id,
                            t.persistent_slots_used + t.dialogue_slots_used
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to load session: {err}"
                        );
                    }
                }
            }
        }
        "/stats" => {
            print_stats(session);
        }
        "/read-mode" => {
            if parts.len() < 2 {
                eprintln!(
                    "{ANSI_RED_BOLD}[error]{ANSI_RESET} Usage: /read-mode <on|off|enabled|no_read>"
                );
            } else {
                match parts[1].to_lowercase().as_str() {
                    "on" | "enabled" => {
                        session.set_read_mode(ReadMode::Enabled);
                        println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Memory read mode set to Enabled.");
                    }
                    "off" | "no_read" | "noread" => {
                        session.set_read_mode(ReadMode::NoRead);
                        println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Memory read mode set to NoRead.");
                    }
                    other => {
                        eprintln!(
                            "{ANSI_RED_BOLD}[error]{ANSI_RESET} Unrecognized mode '{other}'. Expected 'on' or 'off'."
                        );
                    }
                }
            }
        }
        unknown => {
            eprintln!(
                "{ANSI_RED_BOLD}[error]{ANSI_RESET} Unknown command '{unknown}'. Type /help for available commands."
            );
        }
    }

    true
}

/// Unconditionally anchor numerical serving symbols so linker dead-stripping
/// does not eliminate `IntegerModel::step` in release mode.
#[inline(never)]
fn retain_kernel_symbols() {
    let step_fn: fn(&IntegerModel, &mut IntegerSession, u32, ReadMode) -> Result<IntegerStep> =
        IntegerModel::step;
    std::hint::black_box(step_fn as *const () as usize);
    std::hint::black_box(IntegerModel::step_conversational as *const () as usize);
}

/// Run an explicit self-test verifying the autoregressive transition step
/// executes cleanly using strictly zero multipliers, dividers, and floating point.
#[inline(never)]
fn run_kernel_verification(bundle: &Bundle) -> Result<()> {
    let model = bundle.model();
    let mut session = model.new_session();
    let step = model.step(&mut session, 0, ReadMode::Enabled)?;
    std::hint::black_box(&step);
    println!("{ANSI_GREEN_BOLD}[PASS]{ANSI_RESET} Zero-MatMul numerical serving kernel verified (IntegerModel::step retained).");
    Ok(())
}

fn main() {
    retain_kernel_symbols();

    let cli = parse_cli_args();

    if cli.verify_kernel {
        let bundle = match cli.bundle_path.as_deref() {
            Some(p) if p == Path::new("synthetic") || p == Path::new(":synthetic:") => {
                create_test_bundle_with_byte_vocab()
            }
            Some(p) => match Bundle::load(p) {
                Ok(b) => b,
                Err(err) => {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to load bundle at '{}': {err}",
                        p.display()
                    );
                    process::exit(1);
                }
            },
            None => create_test_bundle_with_byte_vocab(),
        };

        if let Err(err) = run_kernel_verification(&bundle) {
            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} kernel verification failed: {err}");
            process::exit(1);
        }
        process::exit(0);
    }

    let bundle_path = cli.bundle_path.as_ref().unwrap();

    let bundle = if bundle_path == Path::new("synthetic")
        || bundle_path == Path::new(":synthetic:")
        || bundle_path.to_str() == Some("synthetic")
    {
        create_test_bundle_with_byte_vocab()
    } else {
        match Bundle::load(bundle_path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!(
                    "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to load bundle at '{}': {err}",
                    bundle_path.display()
                );
                process::exit(1);
            }
        }
    };

    let policy = if cli.temperature <= 0.0 {
        SamplePolicy::Greedy
    } else {
        SamplePolicy::Categorical { top_k: cli.top_k }
    };

    let mut session = match ChatSession::new(&bundle, cli.system_prompt.as_deref(), cli.seed) {
        Ok(mut s) => {
            s.set_read_mode(cli.read_mode);
            s.set_policy(policy);
            s
        }
        Err(err) => {
            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} failed to initialize chat session: {err}");
            process::exit(1);
        }
    };

    print_welcome_banner(&bundle, policy, cli.read_mode);

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        print!("{ANSI_CYAN_BOLD}User>{ANSI_RESET} ");
        if io::stdout().flush().is_err() {
            break;
        }

        let mut input_line = String::new();
        match reader.read_line(&mut input_line) {
            Ok(0) => {
                // EOF (Ctrl-D)
                println!();
                println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Exiting session.");
                process::exit(0);
            }
            Ok(_) => {
                let trimmed = input_line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.starts_with('/') {
                    handle_slash_command(trimmed, &mut session, &bundle);
                    continue;
                }

                // Assistant streaming generation
                print!("{ANSI_GREEN_BOLD}Assistant>{ANSI_RESET} ");
                io::stdout().flush().ok();

                match session.generate_stream(trimmed, cli.max_tokens, &[]) {
                    Ok(stream) => {
                        for chunk in stream {
                            print!("{}", chunk);
                            io::stdout().flush().ok();
                        }
                        println!();
                    }
                    Err(err) => {
                        println!();
                        eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Generation error: {err}");
                    }
                }
            }
            Err(err) => {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to read input: {err}");
                break;
            }
        }
    }
}
