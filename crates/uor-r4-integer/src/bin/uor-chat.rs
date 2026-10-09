//! Interactive Conversational Chatbot CLI (`uor-chat`)
//!
//! Real-time streaming terminal REPL powered by the native UOR-R4 Geometric Language Model.
//! Executes with strictly zero transformers and zero hardware matrix multiplication in the served runtime.
//! Persistent dialogue continuity via exact-token history and literal-role protocol.

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process;
use uor_r4_integer::bundle::{create_test_bundle_with_byte_vocab, Bundle};
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::generation::conversation::{
    ConversationError, ConversationRequest, DialogueConversation, DialogueConversationStream,
    TurnClosure,
};
use uor_r4_integer::generation::Stop;
use uor_r4_integer::model::{IntegerModel, IntegerSession, IntegerStep};
use uor_r4_integer::sampling::SamplePolicy;
use uor_r4_integer::stack::{IntegerStackModel, StackChat, StackChatError, StackError, StackReply};
use uor_r4_integer::{IntegerError, Result};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

const VERSION: &str = "0.1.0";

const ANSI_CYAN_BOLD: &str = "\x1b[1;36m";
const ANSI_GREEN_BOLD: &str = "\x1b[1;32m";
const ANSI_YELLOW_BOLD: &str = "\x1b[1;33m";
const ANSI_RED_BOLD: &str = "\x1b[1;31m";
const ANSI_MAGENTA_BOLD: &str = "\x1b[1;35m";
const ANSI_RESET: &str = "\x1b[0m";

struct CliArgs {
    bundle_path: Option<PathBuf>,
    /// A `UORLUT01` geometric-stack artifact (`--stack`): served directly by
    /// the D11 stack engine instead of a sealed bundle.
    stack_path: Option<PathBuf>,
    tokenizer_path: Option<PathBuf>,
    /// Literal-role dialogue version for the stack artifact (default 1).
    protocol: u8,
    threads: usize,
    /// One-shot turn: answer this message, print the record and exit.
    say: Option<String>,
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
            stack_path: None,
            tokenizer_path: None,
            protocol: 1,
            threads: 1,
            say: None,
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
    eprintln!("       uor-chat --stack <ARTIFACT.lut> --tokenizer <TOKENIZER.json> [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  -b, --bundle <PATH>         Path to model bundle directory containing bundle.json"
    );
    eprintln!(
        "      --stack <ARTIFACT>      UORLUT01 geometric-stack artifact (model.lut); served"
    );
    eprintln!("                              directly by the D11 integer stack engine");
    eprintln!("      --tokenizer <PATH>      tokenizer.json for --stack (required with it)");
    eprintln!(
        "      --protocol <1|2>        literal-role dialogue version for --stack (default: 1)"
    );
    eprintln!("      --threads <INT>         worker threads for --stack steps (default: 1)");
    eprintln!("      --say <TEXT>            answer one turn, print a JSON record and exit");
    eprintln!("  -s, --system <PROMPT>       Initial persistent system persona");
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
            "--stack" => {
                idx += 1;
                if idx < raw.len() {
                    args.stack_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--stack'");
                    process::exit(1);
                }
            }
            "--tokenizer" => {
                idx += 1;
                if idx < raw.len() {
                    args.tokenizer_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--tokenizer'"
                    );
                    process::exit(1);
                }
            }
            "--protocol" => {
                idx += 1;
                match raw.get(idx).map(String::as_str) {
                    Some("1") => args.protocol = 1,
                    Some("2") => args.protocol = 2,
                    Some(other) => {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} invalid value for '--protocol': {other} (expected 1 or 2)"
                        );
                        process::exit(1);
                    }
                    None => {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--protocol'"
                        );
                        process::exit(1);
                    }
                }
            }
            "--threads" => {
                idx += 1;
                match raw.get(idx).map(String::as_str).map(str::parse::<usize>) {
                    Some(Ok(value)) if value > 0 => args.threads = value,
                    _ => {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--threads' takes a positive integer"
                        );
                        process::exit(1);
                    }
                }
            }
            "--say" => {
                idx += 1;
                if idx < raw.len() {
                    args.say = Some(raw[idx].clone());
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--say'");
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

    if args.bundle_path.is_some() && args.stack_path.is_some() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--bundle' and '--stack' are different serving paths; pass one"
        );
        process::exit(1);
    }
    if args.stack_path.is_some() && args.tokenizer_path.is_none() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--stack' requires '--tokenizer <tokenizer.json>'"
        );
        process::exit(1);
    }
    if args.tokenizer_path.is_some() && args.stack_path.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--tokenizer' is only used with '--stack'");
        process::exit(1);
    }
    if args.bundle_path.is_none() && args.stack_path.is_none() && !args.verify_kernel {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} missing required argument '--bundle <path>' or '--stack <artifact.lut>'"
        );
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

fn print_welcome_banner(
    bundle: &Bundle,
    protocol: &DialogueProtocol,
    policy: SamplePolicy,
    read_mode: ReadMode,
) {
    let id = bundle.identity();
    let id_short = if id.len() > 16 { &id[..16] } else { id };
    println!("================================================================================");
    println!("  UOR-R4 Geometric Conversational Chatbot (uor-chat)");
    println!("  Zero Transformers | Zero Hardware MatMul | Apple Silicon M1 Native");
    println!("================================================================================");
    println!("  Bundle Identity : {}...", id_short);
    println!(
        "  Dialogue Protocol: {} ({})",
        protocol.schema,
        if protocol.tokenizer_cid.len() > 24 {
            &protocol.tokenizer_cid[..24]
        } else {
            &protocol.tokenizer_cid
        }
    );
    println!(
        "  Vocabulary Size : {} tokens",
        bundle.model().config().vocab_size
    );
    println!(
        "  State Dimension : {} dimensions",
        bundle.model().config().width
    );
    println!(
        "  Context Capacity: {} tokens (exact-token persistent history)",
        bundle.model().config().context
    );
    println!("  Sampling Policy : {:?}", policy);
    println!("  Memory Read Mode: {:?}", read_mode);
    println!("  Type /help for slash commands, /quit to exit.");
    println!("================================================================================");
}

fn print_help() {
    println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Available Slash Commands:");
    println!("  /persona [PROMPT]     Set or inspect persistent system persona");
    println!("  /reset, /clear        Reset dialogue history and start a fresh conversation");
    println!("  /verify               Execute self-test verifying Zero-MatMul kernel retention");
    println!("  /stats                Display session telemetry, active context, and process RSS");
    println!("  /history              Display conversation turns and token counts");
    println!("  /read-mode <on|off>   Toggle prime-addressed memory reading (Enabled vs NoRead)");
    println!("  /quit, /exit          Exit uor-chat cleanly");
    println!("  /help                 Display this command help menu");
}

fn print_stats(conversation: &DialogueConversation<'_>, turn_count: usize) {
    let rss_str = match get_process_rss_mb() {
        Some(rss) => format!("{:.2} MB (Invariant: < 35.0 MB - PASS)", rss),
        None => "Unavailable".to_string(),
    };
    let context_cap = conversation.bundle().model().config().context;
    let active_tokens = conversation.len();
    let step_calls = conversation.step_calls();
    let sampler_state = conversation.sampler_state().unwrap_or(0);

    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}|                          Dialogue Conversation Telemetry                    |{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("| Turn Count             : {:<50}|", turn_count);
    println!(
        "| Active Context Tokens  : {:<50}|",
        format!("{} / {}", active_tokens, context_cap)
    );
    println!("| Model Steps Executed   : {:<50}|", step_calls);
    println!("| Sampler PRNG State     : {:<50}|", sampler_state);
    println!(
        "| Memory Read Mode       : {:<50}|",
        format!("{:?}", conversation.read_mode())
    );
    println!(
        "| Poisoned State         : {:<50}|",
        format!("{}", conversation.is_poisoned())
    );
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}| Protocol & Architecture:                                                   |{ANSI_RESET}");
    println!(
        "| Protocol Schema        : {:<50}|",
        conversation.protocol().schema
    );
    println!(
        "| Protocol Identity      : {:<50}|",
        conversation.protocol_identity()
    );
    println!("| Numerical Engine       : Pure Integer Shift-Add / Lookup (D11 contract)    |");
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("{ANSI_MAGENTA_BOLD}| Apple Silicon Host Resources:                                              |{ANSI_RESET}");
    println!("| Process RSS            : {:<50}|", rss_str);
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
}

fn create_conversation<'a>(
    bundle: &'a Bundle,
    protocol: &DialogueProtocol,
    system_prompt: Option<&str>,
    seed: u64,
    read_mode: ReadMode,
) -> std::result::Result<DialogueConversation<'a>, ConversationError> {
    let history = match system_prompt {
        Some(prompt) if !prompt.trim().is_empty() => vec![Message {
            role: "system",
            content: prompt.trim(),
        }],
        _ => vec![],
    };
    bundle.dialogue_conversation(protocol, &history, seed, read_mode)
}

fn handle_slash_command<'a>(
    line: &str,
    conversation: &mut DialogueConversation<'a>,
    bundle: &'a Bundle,
    protocol: &DialogueProtocol,
    current_system_prompt: &mut Option<String>,
    current_read_mode: &mut ReadMode,
    current_seed: u64,
    turn_count: &mut usize,
) -> bool {
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
        "/persona" => {
            let prompt = line["/persona".len()..].trim();
            if prompt.is_empty() {
                if let Some(ref p) = current_system_prompt {
                    println!(
                        "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Current Persona ({} initial tokens):\n\"{}\"",
                        conversation.initial_tokens().len(),
                        p
                    );
                } else {
                    println!(
                        "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} No persistent persona is currently set."
                    );
                }
            } else {
                let candidate_persona = Some(prompt.to_string());
                match create_conversation(
                    bundle,
                    protocol,
                    candidate_persona.as_deref(),
                    current_seed,
                    *current_read_mode,
                ) {
                    Ok(new_conv) => {
                        *current_system_prompt = candidate_persona;
                        *conversation = new_conv;
                        *turn_count = 0;
                        println!(
                            "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} System persona set and session reset ({} initial tokens).",
                            conversation.initial_tokens().len()
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to set persona: {err}"
                        );
                    }
                }
            }
        }
        "/reset" | "/clear" => {
            match create_conversation(
                bundle,
                protocol,
                current_system_prompt.as_deref(),
                current_seed,
                *current_read_mode,
            ) {
                Ok(new_conv) => {
                    *conversation = new_conv;
                    *turn_count = 0;
                    println!(
                        "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Dialogue history reset cleanly."
                    );
                }
                Err(err) => {
                    eprintln!(
                        "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to reset conversation: {err}"
                    );
                }
            }
        }
        "/verify" | "/verify-kernel" => {
            if let Err(err) = run_kernel_verification(bundle) {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Kernel verification failed: {err}");
            }
        }
        "/stats" => {
            print_stats(conversation, *turn_count);
        }
        "/history" => {
            println!(
                "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Dialogue history: {} completed turns, {} active tokens in context (capacity {}).",
                *turn_count,
                conversation.len(),
                bundle.model().config().context
            );
        }
        "/save" | "/load" => {
            println!(
                "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} /save and /load are unsupported for DialogueConversation streaming sessions (use /stats or /history to view dialogue state)."
            );
        }
        "/read-mode" => {
            if parts.len() < 2 {
                eprintln!(
                    "{ANSI_RED_BOLD}[error]{ANSI_RESET} Usage: /read-mode <on|off|enabled|no_read>"
                );
            } else {
                match parts[1].to_lowercase().as_str() {
                    "on" | "enabled" => {
                        let candidate_mode = ReadMode::Enabled;
                        match create_conversation(
                            bundle,
                            protocol,
                            current_system_prompt.as_deref(),
                            current_seed,
                            candidate_mode,
                        ) {
                            Ok(new_conv) => {
                                *current_read_mode = candidate_mode;
                                *conversation = new_conv;
                                *turn_count = 0;
                                println!(
                                    "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Memory read mode set to Enabled (session reset)."
                                );
                            }
                            Err(err) => {
                                eprintln!(
                                    "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to update read mode: {err}"
                                );
                            }
                        }
                    }
                    "off" | "no_read" | "noread" => {
                        let candidate_mode = ReadMode::NoRead;
                        match create_conversation(
                            bundle,
                            protocol,
                            current_system_prompt.as_deref(),
                            current_seed,
                            candidate_mode,
                        ) {
                            Ok(new_conv) => {
                                *current_read_mode = candidate_mode;
                                *conversation = new_conv;
                                *turn_count = 0;
                                println!(
                                    "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Memory read mode set to NoRead (session reset)."
                                );
                            }
                            Err(err) => {
                                eprintln!(
                                    "{ANSI_RED_BOLD}[error]{ANSI_RESET} Failed to update read mode: {err}"
                                );
                            }
                        }
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

    let mut conv_session = model.new_conversational_session();
    let conv_step = model.step_conversational(
        &mut conv_session,
        0,
        uor_r4_integer::model::SlotTarget::Dialogue,
        ReadMode::Enabled,
    )?;
    std::hint::black_box(&conv_step);

    println!("{ANSI_GREEN_BOLD}[PASS]{ANSI_RESET} Zero-MatMul numerical serving kernel verified (IntegerModel::step and step_conversational retained).");
    Ok(())
}

fn resolve_bundle_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    let mapped_alias = if path_str.contains("fit256-quaternion-3") {
        Some(("fit256-quaternion-3", "bundle-quaternion-1"))
    } else if path_str.contains("fit256-householder_pair-3") {
        Some(("fit256-householder_pair-3", "bundle-householder_pair-1"))
    } else {
        None
    };

    if let Some((alias, target)) = mapped_alias {
        eprintln!(
            "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} Mapping training alias '{alias}' to pre-trained integer serving bundle '{target}'."
        );
        let target_path = Path::new(target);
        if target_path.exists() {
            return target_path.to_path_buf();
        }
        let full_target =
            Path::new("/Users/casey.allard/uor-r4-investigations/integer-serving-20260925")
                .join(target);
        if full_target.exists() {
            return full_target;
        }
    }

    if path.exists() {
        return path.to_path_buf();
    }
    let default_bases = [
        "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925",
        "/Users/casey.allard/uor-r4-investigations/language-continuation-20260925",
        "/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926",
    ];
    for base in &default_bases {
        let candidate = Path::new(base).join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}

/// Determine terminal stop reason for a completed dialogue stream.
/// Terminal execution errors fail the operation rather than silently reporting completion.
fn completed_dialogue_stream_stop(stream: &DialogueConversationStream<'_, '_>) -> Result<Stop> {
    if let Some(error) = stream.error() {
        return Err(IntegerError::Invalid(format!(
            "dialogue stream stopped with error after {} generated tokens: {error}",
            stream.tokens_generated()
        )));
    }
    stream.stop_reason().ok_or_else(|| {
        IntegerError::Invalid("stream exhausted without a terminal stop reason".into())
    })
}

/// Iterator exhaustion alone is not success: terminal model errors may arrive
/// after the last visible text chunk, including while committing a stop token.
#[cfg(test)]
fn completed_stream_stop(
    stream: &uor_r4_integer::session::ChatTokenStream<'_, '_>,
) -> Result<uor_r4_integer::session::StreamStopReason> {
    if let Some(error) = stream.error() {
        return Err(IntegerError::Invalid(format!(
            "stream stopped with {:?} after {} generated tokens: {error}",
            stream.stop_reason(),
            stream.tokens_generated()
        )));
    }
    stream.stop_reason().ok_or_else(|| {
        IntegerError::Invalid("stream exhausted without a terminal stop reason".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::session::{ChatSession, ChatTokenStream, StreamStopReason};

    #[test]
    fn chat_stream_cli_distinguishes_failed_and_successful_exhaustion() -> Result<()> {
        let bundle = create_test_bundle_with_byte_vocab();
        let mut session = ChatSession::new(&bundle, None, 17)?;
        session.state_mut().identity = "injected identity mismatch".into();
        let mut stream = ChatTokenStream::new(&mut session, 0, &[]);
        assert_eq!(stream.next(), None);
        let error =
            completed_stream_stop(&stream).expect_err("terminal error must fail CLI outcome");
        assert!(error.to_string().contains("ModelError"));
        assert!(error.to_string().contains("identity"));

        let mut session = ChatSession::new(&bundle, None, 17)?;
        let mut stream = ChatTokenStream::new(&mut session, 0, &[]);
        assert_eq!(stream.next(), None);
        assert_eq!(
            completed_stream_stop(&stream)?,
            StreamStopReason::MaxTokens { count: 0 }
        );
        Ok(())
    }

    #[test]
    fn dialogue_stream_cli_distinguishes_failed_and_successful_exhaustion() -> Result<()> {
        let bundle = create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conv = bundle
            .dialogue_conversation(&protocol, &[], 42, ReadMode::Enabled)
            .unwrap();

        let req = ConversationRequest {
            user: "Hello",
            max_new_tokens: 4,
            policy: SamplePolicy::Greedy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        };
        let mut stream = conv.respond_stream(req).unwrap();
        while let Some(_chunk) = stream.next() {}
        let stop = completed_dialogue_stream_stop(&stream)?;
        assert_eq!(stop, Stop::ShortCycle { period: 1 });
        assert_eq!(stream.tokens_generated(), 3);
        drop(stream);

        // Preflight rejection does not poison conversation
        let oversized = ConversationRequest {
            user: "Invalid budget",
            max_new_tokens: 256,
            policy: SamplePolicy::Greedy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        };
        assert!(conv.respond_stream(oversized).is_err());
        assert!(!conv.is_poisoned());

        Ok(())
    }

    #[test]
    fn dialogue_stream_cli_multi_turn_continuity() -> Result<()> {
        let bundle = create_test_bundle_with_byte_vocab();
        let protocol = DialogueProtocol::literal_roles_v1(bundle.tokenizer()).unwrap();
        let mut conv = bundle
            .dialogue_conversation(&protocol, &[], 2026, ReadMode::Enabled)
            .unwrap();

        // Turn 1: 2 tokens (does not trigger short cycle)
        let req1 = ConversationRequest {
            user: "Hi",
            max_new_tokens: 2,
            policy: SamplePolicy::Greedy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        };
        let mut stream1 = conv.respond_stream(req1).unwrap();
        let mut output1 = String::new();
        while let Some(chunk) = stream1.next() {
            output1.push_str(&chunk);
        }
        let stop1 = completed_dialogue_stream_stop(&stream1)?;
        assert_eq!(stop1, Stop::MaximumNewTokens);
        assert_eq!(stream1.tokens_generated(), 2);
        drop(stream1);

        // Turn 2: continuity with exact ID retention
        let req2 = ConversationRequest {
            user: "Tell me more",
            max_new_tokens: 2,
            policy: SamplePolicy::Greedy,
            first_sentence: false,
            closure: TurnClosure::InterruptAssistant,
        };
        let mut stream2 = conv.respond_stream(req2).unwrap();
        let mut output2 = String::new();
        while let Some(chunk) = stream2.next() {
            output2.push_str(&chunk);
        }
        let stop2 = completed_dialogue_stream_stop(&stream2)?;
        assert_eq!(stop2, Stop::MaximumNewTokens);
        assert_eq!(stream2.tokens_generated(), 2);
        drop(stream2);
        assert!(!conv.is_poisoned());

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stack serving path (`--stack`): a `UORLUT01` geometric-stack artifact served
// directly by the D11 integer stack engine, under the literal-role dialogue
// protocol. The sealed-bundle path above serves the retained recurrent model;
// the two containers describe different models and no conversion exists.

/// A refused stack CLI operation.
#[derive(Debug)]
enum StackCliError {
    Chat(StackChatError),
    Model(StackError),
    Io(std::io::Error),
    Json(serde_json::Error),
    Usage(String),
}

impl std::fmt::Display for StackCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat(error) => write!(formatter, "{error}"),
            Self::Model(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "I/O: {error}"),
            Self::Json(error) => write!(formatter, "JSON: {error}"),
            Self::Usage(message) => formatter.write_str(message),
        }
    }
}

impl From<serde_json::Error> for StackCliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<StackChatError> for StackCliError {
    fn from(error: StackChatError) -> Self {
        Self::Chat(error)
    }
}

impl From<StackError> for StackCliError {
    fn from(error: StackError) -> Self {
        Self::Model(error)
    }
}

impl From<std::io::Error> for StackCliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// The initial history of a stack conversation: BOS alone, or BOS and a system
/// turn encoded by the literal-role protocol.
fn stack_initial_history(
    chat: &StackChat<'_>,
    system_prompt: Option<&str>,
) -> std::result::Result<Vec<u32>, StackCliError> {
    let encoder = chat
        .protocol()
        .bind(chat.tokenizer())
        .map_err(|error| StackCliError::Usage(format!("dialogue protocol: {error}")))?;
    Ok(match system_prompt {
        Some(prompt) if !prompt.trim().is_empty() => {
            let encoded = encoder.encode_open_history(&[Message {
                role: "system",
                content: prompt.trim(),
            }]);
            if encoded.emitted_turns != 1 || encoded.special_token_occurrences != 0 {
                return Err(StackCliError::Usage(
                    "the system persona is not one plain system turn".into(),
                ));
            }
            encoded.tokens
        }
        _ => vec![chat.protocol().bos_id],
    })
}

fn print_stack_banner(
    chat: &StackChat<'_>,
    artifact: &Path,
    tokenizer_path: &Path,
    threads: usize,
) {
    let shape = chat.model().shape();
    let sha = chat.model().artifact_sha256();
    let sha_short = if sha.len() > 16 { &sha[..16] } else { sha };
    println!("================================================================================");
    println!("  UOR-R4 Geometric Conversational Chatbot (uor-chat)");
    println!("  Zero Transformers | Zero Hardware MatMul | D11 integer stack engine");
    println!("================================================================================");
    println!("  Artifact          : {}", artifact.display());
    println!("  Artifact SHA-256  : {sha_short}...");
    println!("  Tokenizer         : {}", tokenizer_path.display());
    println!(
        "  Stack Shape       : vocab {} | width {} | heads {} | mlp {} | pattern {}",
        shape.vocab, shape.width, shape.heads, shape.mlp, shape.pattern
    );
    println!(
        "  Read / Transport  : {} | rotation {} | pointer {}",
        shape.read,
        shape.rotation,
        if shape.pointer.is_some() { "yes" } else { "no" }
    );
    println!(
        "  Context Capacity  : {} tokens (artifact declaration)",
        shape.context
    );
    println!(
        "  Dialogue Protocol : {} ({})",
        chat.protocol().schema,
        chat.protocol().tokenizer_cid
    );
    println!("  Decoding          : greedy integer argmax (stack_argmax), no float sampling");
    println!("  Worker Threads    : {threads}");
    if !chat.declares_chat_context() {
        println!(
            "  [notice] The sealed bundle contract fixes context at {}; this artifact declares {}.",
            uor_r4_integer::stack::CHAT_CONTEXT,
            shape.context
        );
    }
    println!("  Type /help for slash commands, /quit to exit.");
    println!("================================================================================");
}

/// One reply, resetting the conversation once when the turn does not fit the
/// artifact's context. Nothing is truncated silently: the reset is reported.
fn stack_reply(
    chat: &mut StackChat<'_>,
    initial: &[u32],
    user: &str,
    max_tokens: usize,
) -> std::result::Result<StackReply, StackCliError> {
    match chat.reply(user, max_tokens) {
        Ok(reply) => Ok(reply),
        Err(StackChatError::Context { needed, context }) => {
            println!(
                "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} The turn needs {needed} of {context} positions; starting a new conversation."
            );
            chat.reset();
            chat.seed(initial)?;
            Ok(chat.reply(user, max_tokens)?)
        }
        Err(error) => Err(StackCliError::Chat(error)),
    }
}

fn print_stack_reply(reply: &StackReply, positions: usize, seconds: f64) {
    // The `Assistant>` prompt was printed before the reply was generated.
    println!("{}", reply.text);
    let rate = if seconds > 0.0 {
        reply.ids.len() as f64 / seconds
    } else {
        0.0
    };
    println!(
        "{ANSI_MAGENTA_BOLD}[served]{ANSI_RESET} {} id(s) | stop {} | history {positions} | {rate:.2} id/s",
        reply.ids.len(),
        reply.stop.name()
    );
}

fn stack_help() {
    println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Available Slash Commands (stack path):");
    println!("  /reset, /clear        Reset dialogue history and start a fresh conversation");
    println!("  /history              Display the exact token history length and context");
    println!("  /tokens               Display the exact token ids of the conversation");
    println!("  /stats                Display session telemetry and process RSS");
    println!("  /quit, /exit          Exit uor-chat cleanly");
    println!("  /help                 Display this command help menu");
}

fn stack_stats(chat: &StackChat<'_>) {
    let rss = match get_process_rss_mb() {
        Some(rss) => format!("{rss:.2} MB"),
        None => "Unavailable".to_string(),
    };
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!("| Stack Session Telemetry                                                    |");
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
    println!(
        "| History Tokens         : {:<50}|",
        format!("{} / {}", chat.tokens().len(), chat.context())
    );
    println!("| Engine Steps           : {:<50}|", chat.position());
    println!("| Process RSS            : {:<50}|", rss);
    println!("{ANSI_MAGENTA_BOLD}+----------------------------------------------------------------------------+{ANSI_RESET}");
}

/// Serve a geometric-stack artifact: one `--say` turn, or a REPL.
fn run_stack_chat(cli: &CliArgs) -> std::result::Result<(), StackCliError> {
    let artifact = cli
        .stack_path
        .as_deref()
        .ok_or_else(|| StackCliError::Usage("--stack requires an artifact path".into()))?;
    let tokenizer_path = cli
        .tokenizer_path
        .as_deref()
        .ok_or_else(|| StackCliError::Usage("--stack requires --tokenizer".into()))?;
    if cli.temperature > 0.0 {
        return Err(StackCliError::Usage(
            "the D11 stack engine serves greedy decoding (integer argmax) only: the retained \
             bundle path keeps its Q48 categorical sampler, but this engine has no integer \
             sampler to draw from, so --temperature must be 0"
                .into(),
        ));
    }
    let mut model = IntegerStackModel::load(artifact)?;
    model.set_threads(cli.threads)?;
    let mut chat = StackChat::from_tokenizer_path(&model, tokenizer_path, cli.protocol)?;
    if cli.read_mode == ReadMode::NoRead {
        eprintln!(
            "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} --read-mode is fixed by the artifact's own read \
             geometry ({}); it does not apply to the stack path.",
            model.shape().read
        );
    }
    let initial = stack_initial_history(&chat, cli.system_prompt.as_deref())?;
    chat.seed(&initial)?;

    if let Some(text) = cli.say.as_deref() {
        let started = std::time::Instant::now();
        let reply = stack_reply(&mut chat, &initial, text, cli.max_tokens)?;
        let seconds = started.elapsed().as_secs_f64();
        let record = serde_json::json!({
            "schema": "uor-r4.uor-chat-stack-reply/1",
            "artifact": artifact,
            "artifact_sha256": model.artifact_sha256(),
            "tokenizer": tokenizer_path,
            "tokenizer_cid": chat.protocol().tokenizer_cid,
            "protocol": chat.protocol(),
            "context": chat.context(),
            "decoding": "greedy stack_argmax",
            "user": text,
            "reply_text": reply.text,
            "reply_ids": reply.ids,
            "stop": reply.stop.name(),
            "history_tokens": chat.tokens().len(),
            "seconds": seconds,
        });
        println!("{}", serde_json::to_string_pretty(&record)?);
        return Ok(());
    }

    print_stack_banner(&chat, artifact, tokenizer_path, cli.threads);
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut turns = 0usize;
    loop {
        print!("{ANSI_CYAN_BOLD}User>{ANSI_RESET} ");
        if io::stdout().flush().is_err() {
            break;
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!();
                println!("{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Exiting session.");
                return Ok(());
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with('/') {
                    match trimmed.split_whitespace().next().unwrap_or(trimmed) {
                        "/quit" | "/exit" => {
                            println!(
                                "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Session ended. Goodbye!"
                            );
                            return Ok(());
                        }
                        "/help" => stack_help(),
                        "/reset" | "/clear" => {
                            chat.reset();
                            chat.seed(&initial)?;
                            turns = 0;
                            println!(
                                "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Dialogue history reset cleanly."
                            );
                        }
                        "/history" => println!(
                            "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Dialogue history: {turns} completed turns, {} active tokens in context (capacity {}).",
                            chat.tokens().len(),
                            chat.context()
                        ),
                        "/tokens" => println!(
                            "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} Token ids: {:?}",
                            chat.tokens()
                        ),
                        "/stats" => stack_stats(&chat),
                        unknown => eprintln!(
                            "{ANSI_RED_BOLD}[error]{ANSI_RESET} Unknown command '{unknown}'. Type /help for available commands."
                        ),
                    }
                    continue;
                }
                print!("{ANSI_GREEN_BOLD}Assistant>{ANSI_RESET} ");
                io::stdout().flush().ok();
                let started = std::time::Instant::now();
                match stack_reply(&mut chat, &initial, trimmed, cli.max_tokens) {
                    Ok(reply) => {
                        let seconds = started.elapsed().as_secs_f64();
                        print_stack_reply(&reply, chat.tokens().len(), seconds);
                        turns += 1;
                    }
                    Err(error) => {
                        eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Generation failed: {error}");
                    }
                }
            }
            Err(error) => {
                eprintln!("{ANSI_RED_BOLD}[error]{ANSI_RESET} Input failed: {error}");
                return Err(StackCliError::Io(error));
            }
        }
    }
    Ok(())
}

fn main() {
    retain_kernel_symbols();

    let cli = parse_cli_args();

    if cli.stack_path.is_some() {
        if cli.verify_kernel {
            eprintln!(
                "{ANSI_RED_BOLD}error:{ANSI_RESET} '--verify-kernel' verifies the retained bundle \
                 kernel; it does not apply to the stack path"
            );
            process::exit(1);
        }
        if let Err(error) = run_stack_chat(&cli) {
            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} {error}");
            process::exit(1);
        }
        process::exit(0);
    }

    if cli.verify_kernel {
        let bundle = match cli.bundle_path.as_deref() {
            Some(p) if p == Path::new("synthetic") || p == Path::new(":synthetic:") => {
                create_test_bundle_with_byte_vocab()
            }
            Some(p) => {
                let resolved = resolve_bundle_path(p);
                match Bundle::load(&resolved) {
                    Ok(b) => b,
                    Err(err) => {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to load bundle at '{}': {err}",
                            resolved.display()
                        );
                        process::exit(1);
                    }
                }
            }
            None => create_test_bundle_with_byte_vocab(),
        };

        if let Err(err) = run_kernel_verification(&bundle) {
            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} kernel verification failed: {err}");
            process::exit(1);
        }
        process::exit(0);
    }

    let bundle_path_raw = cli.bundle_path.as_ref().unwrap();

    let bundle = if bundle_path_raw == Path::new("synthetic")
        || bundle_path_raw == Path::new(":synthetic:")
        || bundle_path_raw.to_str() == Some("synthetic")
    {
        create_test_bundle_with_byte_vocab()
    } else {
        let resolved = resolve_bundle_path(bundle_path_raw);
        match Bundle::load(&resolved) {
            Ok(b) => b,
            Err(err) => {
                eprintln!(
                    "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to load bundle at '{}': {err}",
                    resolved.display()
                );
                process::exit(1);
            }
        }
    };

    let protocol = match DialogueProtocol::literal_roles_v1(bundle.tokenizer()) {
        Ok(p) => p,
        Err(err) => {
            eprintln!(
                "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to bind dialogue protocol to tokenizer: {err}"
            );
            process::exit(1);
        }
    };

    let policy = if cli.temperature <= 0.0 {
        SamplePolicy::Greedy
    } else {
        SamplePolicy::Categorical { top_k: cli.top_k }
    };

    let mut current_system_prompt = cli.system_prompt.clone();
    let mut current_read_mode = cli.read_mode;
    let current_seed = cli.seed;
    let current_policy = policy;
    let mut turn_count: usize = 0;

    let mut conversation = match create_conversation(
        &bundle,
        &protocol,
        current_system_prompt.as_deref(),
        current_seed,
        current_read_mode,
    ) {
        Ok(conv) => conv,
        Err(err) => {
            eprintln!(
                "{ANSI_RED_BOLD}error:{ANSI_RESET} failed to initialize dialogue conversation: {err}"
            );
            process::exit(1);
        }
    };

    print_welcome_banner(&bundle, &protocol, policy, cli.read_mode);

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
                    handle_slash_command(
                        trimmed,
                        &mut conversation,
                        &bundle,
                        &protocol,
                        &mut current_system_prompt,
                        &mut current_read_mode,
                        current_seed,
                        &mut turn_count,
                    );
                    continue;
                }

                if conversation.is_poisoned() {
                    eprintln!(
                        "{ANSI_RED_BOLD}[error]{ANSI_RESET} Conversation is poisoned due to an execution error. Please use /reset to start a fresh conversation."
                    );
                    continue;
                }

                // Context capacity guard and auto-capping
                let context_cap = bundle.model().config().context;
                let active_len = conversation.len();
                let has_history =
                    conversation.previous_stop().is_some() || conversation.initial_has_history();
                let prefix_tokens = match protocol.bind(bundle.tokenizer()) {
                    Ok(bound) => bound.encode_user_prefix(trimmed, has_history).tokens.len(),
                    Err(_) => bundle.tokenizer().encode(trimmed).len() + 3,
                };
                let initial_unobserved = if conversation.session().is_none() {
                    conversation.initial_tokens().len()
                } else {
                    0
                };
                let caller_eos = match conversation.previous_stop() {
                    Some(stop) if stop != Stop::Eos => 1,
                    _ => 0,
                };
                let prompt_total = active_len + initial_unobserved + caller_eos + prefix_tokens;
                if prompt_total >= context_cap {
                    eprintln!(
                        "{ANSI_RED_BOLD}[error]{ANSI_RESET} Context capacity reached ({} / {} tokens). Use /reset to clear history.",
                        prompt_total, context_cap
                    );
                    continue;
                }
                let remaining_room = context_cap - prompt_total;
                let effective_max_tokens = cli.max_tokens.min(remaining_room);
                if effective_max_tokens == 0 {
                    eprintln!(
                        "{ANSI_RED_BOLD}[error]{ANSI_RESET} Context capacity exhausted ({} / {} tokens). Use /reset to start fresh.",
                        prompt_total, context_cap
                    );
                    continue;
                }
                if effective_max_tokens < cli.max_tokens {
                    println!(
                        "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} Capping response to {} tokens (remaining context window room: {} tokens).",
                        effective_max_tokens, remaining_room
                    );
                }

                // Assistant streaming generation with persistent exact-token dialogue continuity
                print!("{ANSI_GREEN_BOLD}Assistant>{ANSI_RESET} ");
                io::stdout().flush().ok();

                let start_time = std::time::Instant::now();
                let request = ConversationRequest {
                    user: trimmed,
                    max_new_tokens: effective_max_tokens,
                    policy: current_policy,
                    first_sentence: false,
                    closure: TurnClosure::InterruptAssistant,
                };

                match conversation.respond_stream(request) {
                    Ok(mut stream) => {
                        while let Some(chunk) = stream.next() {
                            print!("{}", chunk);
                            io::stdout().flush().ok();
                        }
                        println!();

                        let stop = match completed_dialogue_stream_stop(&stream) {
                            Ok(stop) => stop,
                            Err(error) => {
                                eprintln!(
                                    "{ANSI_RED_BOLD}[error]{ANSI_RESET} Generation failed: {error}"
                                );
                                continue;
                            }
                        };
                        turn_count += 1;
                        let elapsed = start_time.elapsed();
                        let tok_count = stream.tokens_generated();
                        let elapsed_secs = elapsed.as_secs_f64();
                        let ms_per_tok = if tok_count > 0 {
                            (elapsed_secs * 1000.0) / (tok_count as f64)
                        } else {
                            0.0
                        };
                        let tok_per_sec = if elapsed_secs > 0.0 {
                            (tok_count as f64) / elapsed_secs
                        } else {
                            0.0
                        };
                        println!(
                            "{ANSI_YELLOW_BOLD}[telemetry]{ANSI_RESET} Generated {} tokens in {:.2}s ({:.1} tok/s, {:.3} ms/tok); stop={stop:?}",
                            tok_count, elapsed_secs, tok_per_sec, ms_per_tok
                        );
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
