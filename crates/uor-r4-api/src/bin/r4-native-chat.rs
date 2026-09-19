//! Terminal adapter for an explicitly supplied native artifact.
use std::io::{self, Write};
use uor_r4_api::native_capability_api::{CompletionRequest, NativeModel, SessionConfig};

fn format_telemetry(token_count: usize, stopped_by: &str, elapsed_us: Option<u64>) -> String {
    if let Some(elapsed) = elapsed_us {
        if token_count > 0 {
            let tok_s = (token_count as f64 * 1_000_000.0) / elapsed.max(1) as f64;
            let us_tok = elapsed as f64 / token_count as f64;
            format!(
                "[{} model tokens; {}; {:.1} tok/s, {:.1} µs/token]",
                token_count, stopped_by, tok_s, us_tok
            )
        } else {
            format!("[0 model tokens; {stopped_by}]")
        }
    } else {
        format!("[{token_count} model tokens; {stopped_by}]")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut model_path = std::env::var("UOR_R4_MODEL").ok();
    let mut temperature: f64 = 0.8;
    let mut top_k: usize = 10;
    let mut seed: Option<u64> = None;
    let mut prompt = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model" => model_path = Some(args.next().ok_or("--model requires a path")?),
            "--tokenizer" => {
                let tok_path = args.next().ok_or("--tokenizer requires a path")?;
                std::env::set_var("UOR_TOKENIZER", tok_path);
            }
            "--temperature" => {
                temperature = args
                    .next()
                    .ok_or("--temperature requires a float")?
                    .parse()?;
            }
            "--top-k" => {
                top_k = args.next().ok_or("--top-k requires an integer")?.parse()?;
            }
            "--seed" => {
                seed = Some(args.next().ok_or("--seed requires an integer")?.parse()?);
            }
            "--help" | "-h" => {
                println!(
                    "r4-native-chat --model PATH [--tokenizer PATH] [--temperature FLOAT] [--top-k INT] [--seed INT] [PROMPT]\n\
                     UOR_R4_MODEL and UOR_TOKENIZER can supply paths. No default model or fallback replies.\n\
                     Interactive commands: /reset, /stats, /exit"
                );
                return Ok(());
            }
            "--" => {
                prompt.extend(args);
                break;
            }
            _ if arg.starts_with("--") => return Err(format!("unknown option: {arg}").into()),
            _ => prompt.push(arg),
        }
    }
    let path = model_path
        .ok_or("supply --model PATH or UOR_R4_MODEL; serving requires a trained artifact")?;
    let model = NativeModel::load_from_bytes(&std::fs::read(&path)?)?;
    let sess_seed = seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(20260918)
    });
    let mut session = model.create_session(SessionConfig {
        user_id: "cli-user".into(),
        project_id: "terminal".into(),
        session_id: format!("cli-{}-{}", std::process::id(), sess_seed),
        temperature,
        top_k,
        ..Default::default()
    })?;
    eprintln!(
        "UOR-R4 pre-alpha | artifact {} | context {}",
        model.artifact_cid(),
        model.metadata().max_context_tokens
    );
    if !prompt.is_empty() {
        let response =
            session.complete_stream(CompletionRequest::new(prompt.join(" ")), |piece| {
                print!("{piece}");
                let _ = io::stdout().flush();
                true
            })?;
        eprintln!(
            "\n{}",
            format_telemetry(
                response.token_count,
                &response.stopped_by,
                response.elapsed_us
            )
        );
        return Ok(());
    }
    let mut line = String::new();
    loop {
        print!("uor> ");
        io::stdout().flush()?;
        line.clear();
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        match line.trim() {
            "/exit" | "/quit" => break,
            "/reset" => {
                session.reset()?;
                eprintln!("Context and memory cleared.");
                continue;
            }
            "/stats" => {
                println!("{}", serde_json::to_string_pretty(model.metadata())?);
                continue;
            }
            "" => continue,
            _ => {}
        }
        // Preserve the newline separating successive user turns.
        let result = session.complete_stream(CompletionRequest::new(line.clone()), |piece| {
            print!("{piece}");
            let _ = io::stdout().flush();
            true
        });
        match result {
            Ok(response) => eprintln!(
                "\n{}",
                format_telemetry(
                    response.token_count,
                    &response.stopped_by,
                    response.elapsed_us
                )
            ),
            Err(error) => eprintln!("\n[error: {error}]"),
        }
    }
    Ok(())
}
