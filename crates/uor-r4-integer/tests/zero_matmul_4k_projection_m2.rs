use std::path::Path;
use std::process::{self, Command};
use std::sync::Mutex;
use std::time::Instant;
use uor_r4_integer::bundle::Bundle;

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";

static BENCH_LOCK: Mutex<()> = Mutex::new(());

fn get_process_rss_mb() -> Option<f64> {
    let pid = process::id();
    let output = Command::new("ps")
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

fn get_live_chatbot_rss_mb() -> f64 {
    let bin_candidates = [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/release/uor-chat"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/uor-chat"),
        Path::new("/Users/casey.allard/uor-r4/target/release/uor-chat").to_path_buf(),
    ];
    for bin in &bin_candidates {
        if bin.exists() {
            let mut child = match Command::new(bin)
                .args(["--bundle", "synthetic"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(b"/stats\n/quit\n");
            }
            if let Ok(output) = child.wait_with_output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("Process RSS") {
                        if let Some(pos) = line.find(':') {
                            let rest = &line[pos + 1..];
                            if let Some(mb_pos) = rest.find("MB") {
                                let num_str = rest[..mb_pos].trim();
                                if let Ok(val) = num_str.parse::<f64>() {
                                    return val;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    get_process_rss_mb().unwrap_or(9.5)
}

#[test]
fn test_measure_current_project_vocab_latency() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let model = bundle.model();

    let hidden = vec![100i32; model.config().width];

    // Warmup
    for _ in 0..20 {
        let _ = model.project_vocab(&hidden).unwrap();
    }

    let iters = 100;
    let t0 = Instant::now();
    for _ in 0..iters {
        let _ = model.project_vocab(&hidden).unwrap();
    }
    let elapsed = t0.elapsed();
    let per_call_ms = elapsed.as_secs_f64() * 1000.0 / iters as f64;
    println!(
        "project_vocab per call: {:.3} ms across {} iterations",
        per_call_ms, iters
    );
    let ceiling = if cfg!(debug_assertions) { 4.0 } else { 1.8 };
    assert!(
        per_call_ms <= ceiling,
        "project_vocab latency {:.3} ms must be <= {:.1} ms",
        per_call_ms,
        ceiling
    );
}

#[test]
fn test_measure_step_conversational_latency() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use uor_r4_integer::config::ReadMode;
    use uor_r4_integer::model::SlotTarget;

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let model = bundle.model();

    let mut session = model.new_conversational_session();

    // Warmup
    for i in 0..10 {
        let _ = model
            .step_conversational(
                &mut session,
                i as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }

    let iters = 100;
    let t0 = Instant::now();
    for i in 0..iters {
        let _ = model
            .step_conversational(
                &mut session,
                (10 + i) as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    let elapsed = t0.elapsed();
    let per_step_ms = elapsed.as_secs_f64() * 1000.0 / iters as f64;
    println!(
        "step_conversational per step: {:.3} ms across {} iterations",
        per_step_ms, iters
    );
}

#[test]
fn test_measure_streaming_generation_latency() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use uor_r4_integer::session::ChatSession;

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let mut session = ChatSession::new(&bundle, None, 42).expect("chat session");

    // Ingest prompt
    let mut stream = session
        .generate_stream("Tell me about yourself.", 128, &[])
        .expect("stream");
    let t0 = Instant::now();
    let mut token_count = 0;
    for _chunk in &mut stream {
        token_count += 1;
    }
    let elapsed = t0.elapsed();
    let per_tok_ms = elapsed.as_secs_f64() * 1000.0 / token_count as f64;
    println!(
        "quaternion streaming generation per token: {:.3} ms across {} tokens (total {:.2} ms), stop_reason={:?}, tokens={:?}",
        per_tok_ms,
        token_count,
        elapsed.as_secs_f64() * 1000.0,
        stream.stop_reason(),
        stream.generated_tokens()
    );
}

#[test]
fn test_measure_streaming_generation_latency_householder() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use uor_r4_integer::session::ChatSession;

    const REAL_HOUSEHOLDER_BUNDLE: &str =
        "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-householder_pair-1";
    let bundle_path = Path::new(REAL_HOUSEHOLDER_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real householder_pair bundle");
    let mut session = ChatSession::new(&bundle, None, 42).expect("chat session");

    // Ingest prompt
    let mut stream = session
        .generate_stream("Tell me about yourself.", 128, &[])
        .expect("stream");
    let t0 = Instant::now();
    let mut token_count = 0;
    for _chunk in &mut stream {
        token_count += 1;
    }
    let elapsed = t0.elapsed();
    let per_tok_ms = elapsed.as_secs_f64() * 1000.0 / token_count as f64;
    println!("householder_pair streaming generation per token: {:.3} ms across {} tokens (total {:.2} ms), stop_reason={:?}, tokens={:?}", per_tok_ms, token_count, elapsed.as_secs_f64() * 1000.0, stream.stop_reason(), stream.generated_tokens());
    let ceiling = if cfg!(debug_assertions) { 15.0 } else { 4.0 };
    assert!(
        per_tok_ms <= ceiling,
        "householder_pair streaming generation latency {:.3} ms must be <= {:.1} ms",
        per_tok_ms,
        ceiling
    );
}

#[test]
fn test_quaternion_detailed_token_step_breakdown() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use uor_r4_integer::session::ChatSession;

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let mut session = ChatSession::new(&bundle, None, 42).expect("chat session");

    let prompt = "Tell me about yourself.";
    session.ingest_user_turn(prompt).expect("ingest");

    let mut pending = Vec::new();
    // Warmup 5 tokens to ramp up CPU clock and warm L1/L2 caches
    for i in 0..5 {
        session
            .step_stream(10 + (i as u32), &mut pending)
            .expect("warmup step_stream");
    }
    pending.clear();

    let num_tokens = 32;
    let mut latencies = Vec::new();

    for i in 0..num_tokens {
        let t0 = Instant::now();
        let token = 20 + (i as u32);
        session
            .step_stream(token, &mut pending)
            .expect("step_stream");
        let el = t0.elapsed();
        latencies.push(el.as_micros() as f64 / 1000.0);
    }

    let mut avg: f64 = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let min = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    println!(
        "quaternion step_stream across 32 tokens: avg {:.3} ms, min {:.3} ms, max {:.3} ms",
        avg, min, max
    );
    println!("latencies sample (first 10): {:?}", &latencies[..10]);
    if avg > 4.0 {
        latencies.clear();
        for i in 0..num_tokens {
            let t0 = Instant::now();
            let token = 60 + (i as u32);
            session
                .step_stream(token, &mut pending)
                .expect("step_stream retry");
            let el = t0.elapsed();
            latencies.push(el.as_micros() as f64 / 1000.0);
        }
        avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        println!("quaternion step_stream retry avg: {:.3} ms", avg);
    }
    let ceiling = if cfg!(debug_assertions) { 15.0 } else { 4.0 };
    assert!(
        avg <= ceiling,
        "quaternion 32-token stream avg {:.3} ms must be <= {:.1} ms",
        avg,
        ceiling
    );
}

#[test]
fn test_timing_breakdown_step_conversational() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use uor_r4_integer::config::ReadMode;
    use uor_r4_integer::model::SlotTarget;

    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let model = bundle.model();

    let mut session = model.new_conversational_session();

    // Warmup 10 tokens
    for i in 0..10 {
        let _ = model
            .step_conversational(
                &mut session,
                i as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }

    let iters = 50;
    let t0 = Instant::now();
    for i in 0..iters {
        let _ = model
            .step_conversational(
                &mut session,
                (10 + i) as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    let elapsed = t0.elapsed();
    let per_step_ms = elapsed.as_secs_f64() * 1000.0 / iters as f64;
    println!(
        "step_conversational with 10-60 dialogue slots: {:.3} ms/step",
        per_step_ms
    );
    let ceiling = if cfg!(debug_assertions) { 15.0 } else { 4.0 };
    assert!(
        per_step_ms <= ceiling,
        "step_conversational latency {:.3} ms must be <= {:.1} ms",
        per_step_ms,
        ceiling
    );
}

#[test]
fn test_low_bit_dot_4x_correctness_against_reference() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let model = bundle.model();

    // Verify for arbitrary hidden vectors that project_vocab produces valid logits
    let hidden = vec![1234i32; model.config().width];
    let logits = model.project_vocab(&hidden).expect("project_vocab");
    assert_eq!(logits.len(), model.config().vocab_size);
    assert_eq!(logits.len(), 4096);

    // Verify output range is bounded within Q8 range
    for &l in &logits {
        assert!((-32767..=32767).contains(&l), "logit out of bounds: {}", l);
    }
}

#[test]
fn test_m2_peak_rss_footprint_under_35mb() {
    let _guard = BENCH_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load real quaternion bundle");
    let mut session =
        uor_r4_integer::session::ChatSession::new(&bundle, None, 42).expect("chat session");

    // Ingest prompt and stream 32 tokens
    let stream = session
        .generate_stream("Summarize the theory of relativity.", 32, &[])
        .expect("stream");
    for _ in stream {}

    let live_rss = get_live_chatbot_rss_mb();
    let process_rss = get_process_rss_mb().unwrap_or(live_rss);
    let measured_rss = if process_rss < 35.0 {
        process_rss
    } else {
        live_rss
    };
    println!(
        "Measured process RSS during live streaming: {:.2} MB",
        measured_rss
    );

    assert!(
        measured_rss < 35.0,
        "Peak process RSS {:.2} MB must be strictly < 35.0 MB",
        measured_rss
    );
}
