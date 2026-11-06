//! Interactive Conversational Chatbot CLI (`uor-chat`)
//!
//! Real-time streaming terminal REPL powered by the native UOR-R4 Geometric Language Model.
//! Executes with strictly zero transformers and zero hardware matrix multiplication in the served runtime.
//! Persistent dialogue continuity via exact-token history and literal-role protocol.

use sha2::{Digest, Sha256};
use std::io::{self, BufRead, Read as IoRead, Write};
use std::path::{Path, PathBuf};
use std::process;
use uor_r4_integer::bundle::{create_test_bundle_with_byte_vocab, Bundle};
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::generation::conversation::{
    ConversationError, ConversationRequest, DialogueConversation, DialogueConversationStream,
    TurnClosure,
};
use uor_r4_integer::generation::Stop;
use uor_r4_integer::loopback::{serve_loopback, LoopbackConfig};
use uor_r4_integer::model::{IntegerModel, IntegerSession, IntegerStep};
use uor_r4_integer::openai::OpenAiServing;
use uor_r4_integer::sampling::SamplePolicy;
use uor_r4_integer::stack::{
    is_dialogue_query, recall_from_dialogue, ContentReadout, IntegerStackModel, PromptReplyReadout,
    SieveRecall, StackChat, StackChatError, StackError, StackReply,
};

/// How the stack path picks each token: `IntegerStackSession::serve_step`.
const STACK_DECODING: &str =
    "greedy serve_step with optional complete prompt/reply retrieval and artifact-bound content-readout sidecars";
use uor_r4_integer::{IntegerError, Result};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

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
    /// A `UORLUT01` discrete SpiralCore/FBS artifact (`--discrete`).
    discrete_path: Option<PathBuf>,
    tokenizer_path: Option<PathBuf>,
    /// Optional artifact-bound content readout for the `--stack` path.
    content_readout_path: Option<PathBuf>,
    /// Optional artifact-bound complete prompt/reply readout for loopback serving.
    prompt_reply_path: Option<PathBuf>,
    /// Literal-role dialogue version for the stack artifact (default 2).
    protocol: u8,
    threads: usize,
    /// One-shot turn: answer this message, print the record and exit.
    say: Option<String>,
    /// One-shot raw token-ID prompt (`--prompt-ids 0,7,19,31`) for `--stack` parity verification.
    prompt_ids: Option<Vec<u32>>,
    /// Path to write a `UR4SESS1` binary session snapshot (`UR4SESS2` with a content readout) after a `--stack` step batch.
    save_session: Option<PathBuf>,
    /// Path to read a `UR4SESS1` binary session snapshot (`UR4SESS2` with a content readout) before a `--stack` step batch.
    restore_session: Option<PathBuf>,
    /// Loopback OpenAI service address (`127.0.0.1:<port>`), only with `--stack`.
    serve: Option<String>,
    /// Optional same-origin Studio root for the loopback service.
    studio_root: Option<PathBuf>,
    /// Optional additional exact browser Origin accepted by the loopback service.
    origin: Option<String>,
    /// Read the loopback API token once from standard input, for headless serving.
    token_stdin: bool,
    system_prompt: Option<String>,
    /// One-turn explicit system-memory value for the stack chat path.
    memory: Option<String>,
    /// Derive the next memory value from prior user turns.
    memory_sieve: bool,
    temperature: f64,
    top_k: usize,
    seed: u64,
    read_mode: ReadMode,
    max_tokens: usize,
    verify_kernel: bool,
}

#[derive(Default)]
struct PendingMemory(Option<String>);

impl PendingMemory {
    fn new(value: Option<String>) -> Self {
        Self(value)
    }

    fn value(&self) -> Option<&str> {
        self.0.as_deref()
    }

    fn set(&mut self, value: &str) -> std::result::Result<String, StackChatError> {
        let content = StackChat::memory_content(value)?;
        self.0 = Some(value.to_owned());
        Ok(content)
    }

    fn clear(&mut self) {
        self.0 = None;
    }

    fn finish_turn(&mut self) {
        self.clear();
    }
}

struct ConversationMemory {
    pending: PendingMemory,
    sieve: bool,
    user_turns: Vec<String>,
}

impl ConversationMemory {
    fn new(pending: Option<String>, sieve: bool) -> Self {
        Self {
            pending: PendingMemory::new(pending),
            sieve,
            user_turns: Vec::new(),
        }
    }

    fn value_for(&self, user: &str) -> (Option<String>, bool) {
        if let Some(value) = self.pending.value() {
            return (Some(value.to_owned()), true);
        }
        if !self.sieve {
            return (None, false);
        }
        let history: Vec<&str> = self.user_turns.iter().map(String::as_str).collect();
        let value = match recall_from_dialogue(&history, user) {
            SieveRecall::Value(value) => Some(value),
            SieveRecall::Absent => Some("none".to_owned()),
            SieveRecall::NotRequested => None,
        };
        (value, false)
    }

    fn finish_turn(&mut self, user: &str, explicit_memory: bool) {
        if explicit_memory {
            self.pending.finish_turn();
        }
        if !is_dialogue_query(user) {
            self.user_turns.push(user.to_owned());
        }
    }

    fn reset(&mut self) {
        self.pending.clear();
        self.user_turns.clear();
    }
}

fn handle_memory_command(
    memory: &mut ConversationMemory,
    command: &str,
) -> std::result::Result<String, StackChatError> {
    let rest = command
        .strip_prefix("/memory")
        .ok_or_else(|| StackChatError::InvalidMemory("not an /memory command".into()))?
        .trim();
    if rest.is_empty() {
        let mode = if memory.sieve { "auto" } else { "manual" };
        let queued = match memory.pending.value() {
            Some(value) => format!("Queued {}", StackChat::memory_content(value)?),
            None => "No memory value is queued.".to_owned(),
        };
        Ok(format!("Mode: {mode}. {queued}"))
    } else if rest.eq_ignore_ascii_case("clear") {
        memory.pending.clear();
        Ok("Cleared the queued memory value.".to_owned())
    } else if rest.eq_ignore_ascii_case("auto") {
        memory.sieve = true;
        Ok("Automatic dialogue memory enabled.".to_owned())
    } else if rest.eq_ignore_ascii_case("manual") {
        memory.sieve = false;
        Ok("Automatic dialogue memory disabled.".to_owned())
    } else {
        Ok(format!("Queued {}", memory.pending.set(rest)?))
    }
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            bundle_path: None,
            stack_path: None,
            discrete_path: None,
            tokenizer_path: None,
            content_readout_path: None,
            prompt_reply_path: None,
            protocol: 2,
            threads: 1,
            say: None,
            prompt_ids: None,
            save_session: None,
            restore_session: None,
            serve: None,
            studio_root: None,
            origin: None,
            token_stdin: false,
            system_prompt: None,
            memory: None,
            memory_sieve: false,
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
    eprintln!("       uor-chat --discrete <ARTIFACT.lut> --prompt-ids <CSV> [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!(
        "  -b, --bundle <PATH>         Path to model bundle directory containing bundle.json"
    );
    eprintln!(
        "      --stack <ARTIFACT>      UORLUT01 geometric-stack artifact (model.lut); served"
    );
    eprintln!("                              directly by the D11 integer stack engine");
    eprintln!("      --discrete <ARTIFACT>   UORLUT01 discrete SpiralCore/FBS artifact; one-shot");
    eprintln!("                              raw-token serving through IntegerDiscreteModel");
    eprintln!("      --tokenizer <PATH>      tokenizer.json for --stack (required with it)");
    eprintln!("      --content-readout <P>   artifact-bound content-readout sidecar for --stack");
    eprintln!("      --prompt-reply <P>      complete prompt/reply readout for --serve");
    eprintln!(
        "      --protocol <1|2>        literal-role dialogue version for --stack (default: 2)"
    );
    eprintln!("      --threads <INT>         worker threads for --stack steps (default: 1)");
    eprintln!("      --say <TEXT>            answer one turn, print a JSON record and exit");
    eprintln!(
        "      --prompt-ids <CSV>      step comma-separated token IDs on --stack or --discrete,"
    );
    eprintln!("                              print JSON and exit");
    eprintln!(
        "      --save-session <PATH>   write UR4SESS1/UR4SESS2 snapshot bytes after --prompt-ids"
    );
    eprintln!(
        "      --restore-session <P>   restore UR4SESS1/UR4SESS2 snapshot bytes before --prompt-ids"
    );
    eprintln!("      --serve <ADDR>          serve OpenAI-compatible JSON/SSE on 127.0.0.1:<port>");
    eprintln!("                              loads the same --stack artifact; no CLI subprocess");
    eprintln!("      --studio-root <PATH>    serve this Studio on the same origin with ephemeral bootstrap");
    eprintln!("      --origin <ORIGIN>       additionally accept this exact browser Origin (off by default)");
    eprintln!("      --token-stdin           read the API token once from stdin (required without --studio-root)");
    eprintln!("  -s, --system <PROMPT>       Initial persistent system persona");
    eprintln!(
        "      --memory <VALUE>        Queue System: Memory: <VALUE>. for the next stack-chat turn"
    );
    eprintln!(
        "      --memory-sieve          Derive System: Memory from prior user turns in the REPL"
    );
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
            "--serve" => {
                idx += 1;
                if idx < raw.len() {
                    args.serve = Some(raw[idx].clone());
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--serve'");
                    process::exit(1);
                }
            }
            "--studio-root" => {
                idx += 1;
                if idx < raw.len() {
                    args.studio_root = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--studio-root'"
                    );
                    process::exit(1);
                }
            }
            "--origin" => {
                idx += 1;
                if idx < raw.len() {
                    args.origin = Some(raw[idx].clone());
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--origin'");
                    process::exit(1);
                }
            }
            "--token-stdin" => {
                args.token_stdin = true;
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
            "--memory" => {
                idx += 1;
                if idx < raw.len() {
                    args.memory = Some(raw[idx].clone());
                } else {
                    eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--memory'");
                    process::exit(1);
                }
            }
            "--memory-sieve" => {
                args.memory_sieve = true;
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
            "--discrete" => {
                idx += 1;
                if idx < raw.len() {
                    args.discrete_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--discrete'"
                    );
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
            "--content-readout" => {
                idx += 1;
                if idx < raw.len() {
                    args.content_readout_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--content-readout'"
                    );
                    process::exit(1);
                }
            }
            "--prompt-reply" => {
                idx += 1;
                if idx < raw.len() {
                    args.prompt_reply_path = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--prompt-reply'"
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
            "--prompt-ids" => {
                idx += 1;
                if idx < raw.len() {
                    let mut parsed = Vec::new();
                    for piece in raw[idx].split(',') {
                        let trimmed = piece.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        match trimmed.parse::<u32>() {
                            Ok(id) => parsed.push(id),
                            Err(_) => {
                                eprintln!(
                                    "{ANSI_RED_BOLD}error:{ANSI_RESET} invalid token id '{trimmed}' in '--prompt-ids'"
                                );
                                process::exit(1);
                            }
                        }
                    }
                    if parsed.is_empty() {
                        eprintln!(
                            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--prompt-ids' requires at least one integer token id"
                        );
                        process::exit(1);
                    }
                    args.prompt_ids = Some(parsed);
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--prompt-ids'"
                    );
                    process::exit(1);
                }
            }
            "--save-session" => {
                idx += 1;
                if idx < raw.len() {
                    args.save_session = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--save-session'"
                    );
                    process::exit(1);
                }
            }
            "--restore-session" => {
                idx += 1;
                if idx < raw.len() {
                    args.restore_session = Some(PathBuf::from(&raw[idx]));
                } else {
                    eprintln!(
                        "{ANSI_RED_BOLD}error:{ANSI_RESET} missing argument for '--restore-session'"
                    );
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

    let artifact_count = usize::from(args.bundle_path.is_some())
        + usize::from(args.stack_path.is_some())
        + usize::from(args.discrete_path.is_some());
    if artifact_count > 1 {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--bundle', '--stack', and '--discrete' are different serving paths; pass one"
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
    if args.content_readout_path.is_some() && args.stack_path.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--content-readout' requires '--stack'");
        process::exit(1);
    }
    if args.prompt_reply_path.is_some() && args.serve.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--prompt-reply' requires '--serve'");
        process::exit(1);
    }
    if args.serve.is_some() && args.stack_path.is_none() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--serve' requires '--stack' and '--tokenizer'"
        );
        process::exit(1);
    }
    if args.serve.is_some()
        && (args.say.is_some()
            || args.prompt_ids.is_some()
            || args.save_session.is_some()
            || args.restore_session.is_some()
            || args.memory.is_some()
            || args.memory_sieve
            || args.system_prompt.is_some())
    {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--serve' is incompatible with one-shot, session, memory, and persona options");
        process::exit(1);
    }
    if args.serve.is_some() && args.max_tokens != 128 {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--max-tokens' is a request setting, not a server setting; omit it with '--serve'");
        process::exit(1);
    }
    if args.studio_root.is_some() && args.serve.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--studio-root' requires '--serve'");
        process::exit(1);
    }
    if args.origin.is_some() && args.serve.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--origin' requires '--serve'");
        process::exit(1);
    }
    if args.serve.is_some() && args.studio_root.is_none() && !args.token_stdin {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} headless '--serve' requires '--token-stdin' (or use '--studio-root' for in-memory bootstrap)");
        process::exit(1);
    }
    if args.token_stdin && args.serve.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--token-stdin' requires '--serve'");
        process::exit(1);
    }
    if args.prompt_ids.is_some() && args.stack_path.is_none() && args.discrete_path.is_none() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--prompt-ids' requires '--stack' or '--discrete'"
        );
        process::exit(1);
    }
    if (args.save_session.is_some() || args.restore_session.is_some()) && args.stack_path.is_none()
    {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--save-session' and '--restore-session' require '--stack'"
        );
        process::exit(1);
    }
    if args.memory.is_some() && args.stack_path.is_none() {
        eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} '--memory' requires text chat with '--stack'");
        process::exit(1);
    }
    if args.memory.is_some() && args.prompt_ids.is_some() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--memory' requires a text turn; it does not apply to '--prompt-ids'"
        );
        process::exit(1);
    }
    if args.memory_sieve && args.stack_path.is_none() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--memory-sieve' requires text chat with '--stack'"
        );
        process::exit(1);
    }
    if args.memory_sieve && (args.say.is_some() || args.prompt_ids.is_some()) {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--memory-sieve' requires the multi-turn REPL"
        );
        process::exit(1);
    }
    if args.say.is_some() && args.prompt_ids.is_some() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--say' and '--prompt-ids' are mutually exclusive; pass one"
        );
        process::exit(1);
    }
    if args.discrete_path.is_some() && args.say.is_some() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} discrete text chat is not available; use one-shot '--prompt-ids'"
        );
        process::exit(1);
    }
    if args.discrete_path.is_some() && args.prompt_ids.is_none() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--discrete' currently requires one-shot '--prompt-ids'; text chat and REPL are not available"
        );
        process::exit(1);
    }
    if args.discrete_path.is_some() && args.threads != 1 {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} the discrete evaluator is single-threaded; '--threads' must be 1"
        );
        process::exit(1);
    }
    if args.discrete_path.is_some() && args.system_prompt.is_some() {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} '--system' requires text chat; use only raw '--prompt-ids' with '--discrete'"
        );
        process::exit(1);
    }
    if artifact_count == 0 && !args.verify_kernel {
        eprintln!(
            "{ANSI_RED_BOLD}error:{ANSI_RESET} missing required argument '--bundle <path>', '--stack <artifact.lut>', or '--discrete <artifact.lut>'"
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
    fn stack_protocol_defaults_to_the_trained_corpus_encoding() {
        assert_eq!(CliArgs::default().protocol, 2);
    }

    #[test]
    fn dialogue_memory_commands_and_turn_lifecycle_are_exact() {
        let mut memory = ConversationMemory::new(None, false);
        assert_eq!(
            handle_memory_command(&mut memory, "/memory").unwrap(),
            "Mode: manual. No memory value is queued."
        );
        assert_eq!(
            handle_memory_command(&mut memory, "/memory Pickle").unwrap(),
            "Queued Memory: Pickle."
        );
        assert_eq!(memory.pending.value(), Some("Pickle"));
        assert_eq!(
            handle_memory_command(&mut memory, "/memory").unwrap(),
            "Mode: manual. Queued Memory: Pickle."
        );
        let (value, explicit) = memory.value_for("first user turn");
        assert_eq!(value.as_deref(), Some("Pickle"));
        assert!(explicit);
        memory.finish_turn("first user turn", explicit);
        assert_eq!(memory.pending.value(), None);

        assert_eq!(
            handle_memory_command(&mut memory, "/memory tomatoes").unwrap(),
            "Queued Memory: tomatoes."
        );
        assert!(handle_memory_command(&mut memory, "/memory two\nlines").is_err());
        assert_eq!(memory.pending.value(), Some("tomatoes"));
        assert_eq!(
            handle_memory_command(&mut memory, "/memory clear").unwrap(),
            "Cleared the queued memory value."
        );
        assert_eq!(memory.pending.value(), None);
        assert_eq!(
            handle_memory_command(&mut memory, "/memory auto").unwrap(),
            "Automatic dialogue memory enabled."
        );
        assert!(memory.sieve);
        assert_eq!(
            handle_memory_command(&mut memory, "/memory manual").unwrap(),
            "Automatic dialogue memory disabled."
        );
        assert!(!memory.sieve);
    }

    #[test]
    fn automatic_dialogue_memory_tracks_user_turns_and_resets() {
        let mut memory = ConversationMemory::new(None, true);
        let (none, explicit) = memory.value_for("Hello!");
        assert_eq!(none, None);
        assert!(!explicit);
        memory.finish_turn("The new puppy at our house is named Pickle.", explicit);
        let (value, explicit) = memory.value_for("What did we name the new puppy?");
        assert_eq!(value.as_deref(), Some("Pickle"));
        assert!(!explicit);
        memory.finish_turn("What did we name the new puppy?", explicit);

        handle_memory_command(&mut memory, "/memory Goldie").unwrap();
        let (override_value, explicit) = memory.value_for("What did we name the new puppy?");
        assert_eq!(override_value.as_deref(), Some("Goldie"));
        assert!(explicit);
        memory.finish_turn("What did we name the new puppy?", explicit);
        let (restored_value, explicit) = memory.value_for("What did we name the new puppy?");
        assert_eq!(restored_value.as_deref(), Some("Pickle"));
        assert!(!explicit);

        let (absent, explicit) = memory.value_for("What did I eat for breakfast yesterday?");
        assert_eq!(absent.as_deref(), Some("none"));
        assert!(!explicit);

        memory.reset();
        assert_eq!(memory.user_turns.len(), 0);
        assert_eq!(memory.pending.value(), None);
    }

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

fn run_discrete_parity(cli: &CliArgs) -> Result<()> {
    let artifact = cli
        .discrete_path
        .as_deref()
        .ok_or_else(|| IntegerError::Invalid("--discrete requires an artifact path".to_owned()))?;
    if cli.temperature != 0.0 {
        return Err(IntegerError::Invalid(
            "the discrete path serves greedy IntegerDiscreteModel predictions only; --temperature must be 0"
                .to_owned(),
        ));
    }
    let prompt = cli.prompt_ids.as_deref().ok_or_else(|| {
        IntegerError::Invalid("--discrete requires one-shot --prompt-ids".to_owned())
    })?;
    let model = uor_r4_integer::discrete_d11::IntegerDiscreteModel::load(artifact)?;
    let context_limit = model.shape().context as usize;
    if prompt.len() > context_limit {
        return Err(IntegerError::Invalid(format!(
            "prompt length {} exceeds discrete context {context_limit}",
            prompt.len()
        )));
    }
    if let Some(&id) = prompt.iter().find(|&&id| id >= model.shape().vocabulary) {
        return Err(IntegerError::Invalid(format!(
            "token id {id} is outside the discrete vocabulary 0..{}",
            model.shape().vocabulary.saturating_sub(1)
        )));
    }

    let mut context = prompt.to_vec();
    let mut generated_ids = Vec::with_capacity(cli.max_tokens);
    for _ in 0..cli.max_tokens {
        let prediction = model.evaluate_prefix(&context)?.prediction;
        generated_ids.push(prediction);
        if context.len() == context_limit {
            context.remove(0);
        }
        context.push(prediction);
    }
    let record = serde_json::json!({
        "schema": "uor-r4.uor-chat-discrete-parity/1",
        "artifact": artifact,
        "artifact_sha256": model.artifact_sha256(),
        "model_schema": uor_r4_integer::discrete_d11::DISCRETE_SCHEMA,
        "context": context_limit,
        "decoding": "greedy IntegerDiscreteModel::evaluate_prefix",
        "prompt_ids": prompt,
        "generated_ids": generated_ids,
    });
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
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
    Codec(String),
    Io(std::io::Error),
    Json(serde_json::Error),
    Usage(String),
}

impl std::fmt::Display for StackCliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat(error) => write!(formatter, "{error}"),
            Self::Model(error) => write!(formatter, "{error}"),
            Self::Codec(error) => write!(formatter, "{error}"),
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

fn read_launch_token_stdin() -> std::result::Result<String, StackCliError> {
    let mut bytes = Vec::new();
    let mut terminated = false;
    loop {
        let mut chunk = [0u8; 32];
        let read = io::stdin().read(&mut chunk)?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > 257 && !chunk[..read].contains(&b'\n') {
            return Err(StackCliError::Usage(
                "--token-stdin requires 1-256 bytes followed by a newline".into(),
            ));
        }
        bytes.extend_from_slice(&chunk[..read]);
        if chunk[..read].contains(&b'\n') {
            terminated = true;
            break;
        }
        if bytes.len() > 257 {
            return Err(StackCliError::Usage(
                "--token-stdin requires 1-256 bytes followed by a newline".into(),
            ));
        }
    }
    if !terminated {
        return Err(StackCliError::Usage(
            "--token-stdin requires 1-256 bytes followed by a newline".into(),
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| StackCliError::Usage("--token-stdin must be UTF-8".into()))?;
    let token = text.trim_end_matches(&['\r', '\n'][..]);
    if token.is_empty()
        || token.len() > 256
        || token.bytes().any(|byte| byte <= 0x20 || byte == 0x7f)
    {
        return Err(StackCliError::Usage(
            "--token-stdin must be one nonempty token without whitespace or control characters"
                .into(),
        ));
    }
    Ok(token.to_owned())
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
    println!("  Decoding          : {STACK_DECODING}, no float sampling");
    if let Some((opening, reply)) = chat.model().turn_memory_table_lengths() {
        println!(
            "  Turn Memory       : opening {opening} keys | reply {} keys (reply start marked each turn)",
            reply.map_or_else(|| "no".to_string(), |n| n.to_string())
        );
    }
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
    memory: Option<&str>,
    max_tokens: usize,
) -> std::result::Result<StackReply, StackCliError> {
    match chat.reply_with_memory(user, memory, max_tokens) {
        Ok(reply) => Ok(reply),
        Err(StackChatError::Context { needed, context }) => {
            println!(
                "{ANSI_YELLOW_BOLD}[notice]{ANSI_RESET} The turn needs {needed} of {context} positions; starting a new conversation."
            );
            chat.reset();
            chat.seed(initial)?;
            Ok(chat.reply_with_memory(user, memory, max_tokens)?)
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
    println!("  /memory <VALUE>       Queue System: Memory: <VALUE>. for the next user turn");
    println!("  /memory auto          Derive memory from prior user turns");
    println!("  /memory manual        Disable automatic memory");
    println!("  /memory               Show the memory mode and queued value");
    println!("  /memory clear         Clear the queued memory value");
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

/// Serve a geometric-stack artifact: one `--say` turn, a REPL, or the loopback API.
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
    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut model = IntegerStackModel::load(artifact)?;
    #[cfg(not(target_arch = "wasm32"))]
    model.set_threads(cli.threads)?;
    #[cfg(target_arch = "wasm32")]
    let _ = cli.threads;
    let content_readout_identity = if let Some(path) = cli.content_readout_path.as_deref() {
        let path_string = path
            .to_str()
            .ok_or_else(|| StackCliError::Usage("--content-readout path must be UTF-8".into()))?
            .to_owned();
        let (bytes, readout) =
            ContentReadout::read_sidecar(path, model.artifact_sha256(), model.shape().vocab)?;
        model.set_content_readout(readout)?;
        Some((path_string, hex::encode(Sha256::digest(&bytes))))
    } else {
        None
    };
    if let Some(address) = cli.serve.as_deref() {
        let (host, port) = address
            .rsplit_once(':')
            .ok_or_else(|| StackCliError::Usage("--serve expects 127.0.0.1:<port>".into()))?;
        if host != "127.0.0.1" {
            return Err(StackCliError::Usage("--serve binds only 127.0.0.1".into()));
        }
        let port = port
            .parse::<u16>()
            .map_err(|_| StackCliError::Usage("--serve port must be a u16".into()))?;
        let tokenizer_bytes = std::fs::read(tokenizer_path)?;
        let prompt_reply = if let Some(path) = cli.prompt_reply_path.as_deref() {
            let path_string = path
                .to_str()
                .ok_or_else(|| StackCliError::Usage("--prompt-reply path must be UTF-8".into()))?;
            let tokenizer_sha256 = hex::encode(Sha256::digest(&tokenizer_bytes));
            let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
                .ok_or_else(|| StackCliError::Usage("prompt-reply tokenizer is invalid".into()))?;
            let protocol = DialogueProtocol::literal_roles_version(&tokenizer, cli.protocol)
                .map_err(|error| StackCliError::Usage(error.to_string()))?;
            let (bytes, readout) = PromptReplyReadout::read_sidecar(
                path,
                model.artifact_sha256(),
                &tokenizer_sha256,
                cli.protocol,
                &[protocol.bos_id, protocol.eos_id, protocol.unk_id],
                model.shape().vocab,
            )?;
            Some((
                readout,
                path_string.to_owned(),
                hex::encode(Sha256::digest(&bytes)),
            ))
        } else {
            None
        };
        let mut serving = OpenAiServing::new(model, tokenizer_bytes, cli.protocol)?;
        if let Some((path, sha256)) = content_readout_identity {
            serving = serving.with_content_readout_identity(path, sha256)?;
        }
        if let Some((readout, path, sha256)) = prompt_reply {
            serving = serving.with_prompt_reply_readout(readout, path, sha256)?;
        }
        let mut config = LoopbackConfig::new(serving)
            .map_err(|error| StackCliError::Usage(error.to_string()))?;
        config.host = host.to_owned();
        config.port = port;
        if cli.token_stdin {
            config.token = read_launch_token_stdin()?;
        }
        config.studio_root = cli.studio_root.clone();
        config.origin = cli.origin.clone();
        return serve_loopback(config).map_err(|error| StackCliError::Usage(error.to_string()));
    }
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

    if let Some(prompt_ids) = cli.prompt_ids.as_deref() {
        let vocab = model.shape().vocab as u32;
        for &id in prompt_ids {
            if id >= vocab {
                return Err(StackCliError::Usage(format!(
                    "token id {id} is outside the stack vocabulary 0..{}",
                    vocab.saturating_sub(1)
                )));
            }
        }
        let mut session = model.session();
        if let Some(restore_path) = cli.restore_session.as_deref() {
            let raw = std::fs::read(restore_path)?;
            let parsed =
                uor_r4_integer::session_codec::decode(&raw).map_err(StackCliError::Codec)?;
            session.restore_state(&parsed)?;
        }
        let mut tok = prompt_ids[0];
        for &next_prompt_tok in &prompt_ids[1..] {
            session.step(tok)?;
            tok = next_prompt_tok;
        }
        let mut generated_ids = Vec::with_capacity(cli.max_tokens);
        let generation_started = std::time::Instant::now();
        for _ in 0..cli.max_tokens {
            tok = session.serve_step(tok)?.0;
            generated_ids.push(tok);
        }
        let generation_seconds = generation_started.elapsed().as_secs_f64();
        let generated_tokens_per_second = if generation_seconds > 0.0 {
            generated_ids.len() as f64 / generation_seconds
        } else {
            0.0
        };
        let saved_bytes = uor_r4_integer::session_codec::encode(&session.save_state());
        let session_sha256 = hex::encode(Sha256::digest(&saved_bytes));
        if let Some(save_path) = cli.save_session.as_deref() {
            std::fs::write(save_path, &saved_bytes)?;
        }
        let record = serde_json::json!({
            "schema": "uor-r4.uor-chat-stack-parity/1",
            "artifact": artifact,
            "artifact_sha256": model.artifact_sha256(),
            "tokenizer": tokenizer_path,
            "content_readout": content_readout_identity.as_ref().map(|(path, _)| path),
            "content_readout_sha256": content_readout_identity.as_ref().map(|(_, sha256)| sha256),
            "tokenizer_cid": chat.protocol().tokenizer_cid,
            "protocol": chat.protocol(),
            "context": chat.context(),
            "decoding": STACK_DECODING,
            "prompt_ids": prompt_ids,
            "generated_ids": generated_ids,
            "generation_seconds": generation_seconds,
            "generated_tokens_per_second": generated_tokens_per_second,
            "last_token": tok,
            "position": session.position(),
            "weights_per_token": model.weights_per_token(),
            "session_bytes_len": saved_bytes.len(),
            "session_sha256": session_sha256,
        });
        println!("{}", serde_json::to_string_pretty(&record)?);
        return Ok(());
    }

    if let Some(text) = cli.say.as_deref() {
        let started = std::time::Instant::now();
        let reply = stack_reply(
            &mut chat,
            &initial,
            text,
            cli.memory.as_deref(),
            cli.max_tokens,
        )?;
        let seconds = started.elapsed().as_secs_f64();
        let memory = cli
            .memory
            .as_deref()
            .map(StackChat::memory_content)
            .transpose()?;
        let record = serde_json::json!({
            "schema": "uor-r4.uor-chat-stack-reply/1",
            "artifact": artifact,
            "artifact_sha256": model.artifact_sha256(),
            "tokenizer": tokenizer_path,
            "content_readout": content_readout_identity.as_ref().map(|(path, _)| path),
            "content_readout_sha256": content_readout_identity.as_ref().map(|(_, sha256)| sha256),
            "tokenizer_cid": chat.protocol().tokenizer_cid,
            "protocol": chat.protocol(),
            "context": chat.context(),
            "decoding": STACK_DECODING,
            "user": text,
            "memory": memory,
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
    let mut memory = ConversationMemory::new(cli.memory.clone(), cli.memory_sieve);
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
                        "/memory" => {
                            println!(
                                "{ANSI_YELLOW_BOLD}[uor-chat]{ANSI_RESET} {}",
                                handle_memory_command(&mut memory, trimmed)?
                            );
                        }
                        "/reset" | "/clear" => {
                            chat.reset();
                            chat.seed(&initial)?;
                            memory.reset();
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
                let (turn_memory, explicit_memory) = memory.value_for(trimmed);
                if let Some(value) = turn_memory.as_deref() {
                    let source = if explicit_memory { "explicit" } else { "sieve" };
                    println!(
                        "{ANSI_YELLOW_BOLD}[memory {source}]{ANSI_RESET} {}",
                        StackChat::memory_content(value)?
                    );
                }
                print!("{ANSI_GREEN_BOLD}Assistant>{ANSI_RESET} ");
                io::stdout().flush().ok();
                let started = std::time::Instant::now();
                match stack_reply(
                    &mut chat,
                    &initial,
                    trimmed,
                    turn_memory.as_deref(),
                    cli.max_tokens,
                ) {
                    Ok(reply) => {
                        let seconds = started.elapsed().as_secs_f64();
                        print_stack_reply(&reply, chat.tokens().len(), seconds);
                        memory.finish_turn(trimmed, explicit_memory);
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

    if cli.discrete_path.is_some() {
        if cli.verify_kernel {
            eprintln!(
                "{ANSI_RED_BOLD}error:{ANSI_RESET} '--verify-kernel' verifies the retained bundle kernel; it does not apply to the discrete path"
            );
            process::exit(1);
        }
        if let Err(error) = run_discrete_parity(&cli) {
            eprintln!("{ANSI_RED_BOLD}error:{ANSI_RESET} {error}");
            process::exit(1);
        }
        process::exit(0);
    }

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
