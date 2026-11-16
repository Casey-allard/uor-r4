//! `donor-corpus`: the D87 production donor pass's client (claude-400).
//!
//!   donor-corpus bank     ...   (see src/bank.rs)
//!   donor-corpus generate --server URL --prompts FILE --out DIR --budget KEPT_TOKENS
//!                         --instrument-commit SHA [--parallel P] [--max-seconds T]
//!                         [--shard-records R] [--model-name N] [--max-tokens M]
//!                         [--effort E] [--seed S] [--extra-body FILE] [--limit N]
//!                         [--api vllm|openai] [--api-key-env NAME | --api-key-file PATH]
//!                         [--server-wait-seconds W]
//!   donor-corpus seal     --dir DIR --prompts FILE --model FILE --machine FILE --engine FILE
//!   donor-corpus verify   --dir DIR --prompts FILE
//!   donor-corpus balance  --server URL (--api-key-env NAME | --api-key-file PATH)
//!
//! `generate` sends each bank prompt to a vLLM OpenAI-compatible server and keeps the
//! teacher's reasoning text and final answer text per turn. It resumes from whatever a
//! previous segment left in DIR. No logits, probabilities, scores or ranks are requested
//! or stored.

use m2_donor_corpus::{cmd_balance, cmd_generate, cmd_seal, cmd_verify, Args, Res};

fn args_map(args: &[String]) -> Res<Args> {
    let mut m = Args::new();
    let mut i = 0;
    while i < args.len() {
        let k = args[i]
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --flag, got {}", args[i]))?;
        let v = args
            .get(i + 1)
            .ok_or_else(|| format!("--{k} needs a value"))?;
        m.insert(k.to_string(), v.clone());
        i += 2;
    }
    Ok(m)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match args.split_first() {
        Some((cmd, rest)) => args_map(rest).and_then(|m| match cmd.as_str() {
            "bank" => m2_donor_corpus::bank::cmd_bank(&m),
            "generate" => cmd_generate(&m).map(|v| println!("{v}")),
            "seal" => cmd_seal(&m).map(|v| println!("{v}")),
            "verify" => cmd_verify(&m).map(|v| println!("{v}")),
            "balance" => cmd_balance(&m).map(|v| println!("{v}")),
            other => Err(format!("unknown command {other}")),
        }),
        None => {
            Err("usage: donor-corpus bank|generate|seal|verify|balance --flag value ...".into())
        }
    };
    if let Err(e) = res {
        eprintln!("donor-corpus: {e}");
        std::process::exit(1);
    }
}
