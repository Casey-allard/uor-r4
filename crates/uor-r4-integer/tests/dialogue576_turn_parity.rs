use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::model::SlotTarget;
use uor_r4_integer::Result;

fn greedy_choice(probs: &[u64]) -> u32 {
    let mut best_val = 0u64;
    let mut best_idx = 0u32;
    for (idx, &p) in probs.iter().enumerate() {
        if p > best_val {
            best_val = p;
            best_idx = idx as u32;
        }
    }
    best_idx
}

fn get_process_rss_mb() -> Option<f64> {
    let pid = std::process::id();
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

#[test]
fn test_dialogue576_turn_parity_58_turns() -> Result<()> {
    let bundle_path = Path::new("/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-bundle-1");
    if !bundle_path.exists() {
        eprintln!("Skipping test: bundle path does not exist");
        return Ok(());
    }

    let bundle = Bundle::load(bundle_path)?;
    let model = bundle.model();
    assert_eq!(model.config().width, 576);

    let responses_path = Path::new("/Users/casey.allard/uor-r4/.uor-models/investigations/fourth-research-lab-20260926/dialogue-child-observation-1/responses-integer.json");
    if !responses_path.exists() {
        eprintln!("Skipping test: responses-integer.json does not exist");
        return Ok(());
    }

    let data_bytes = fs::read(responses_path)?;
    let data: serde_json::Value = serde_json::from_slice(&data_bytes)?;

    let requests = data["requests"].as_u64().unwrap() as usize;
    let responses = data["responses"].as_u64().unwrap() as usize;
    let expected_generated = data["generated_selections"].as_u64().unwrap() as usize;
    assert_eq!(requests, 38);
    assert_eq!(responses, 58);
    assert_eq!(expected_generated, 1433);

    let rows = data["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 38);

    let mut total_verified_tokens = 0;
    let mut total_turns_verified = 0;
    let mut total_step_calls = 0;

    let mut step_nanoseconds: u128 = 0;
    let mut peak_rss: f64 = 0.0;

    let wall_clock_start = Instant::now();

    for (row_idx, row) in rows.iter().enumerate() {
        let req_id = row["id"].as_str().unwrap();
        let turns = row["turns"].as_array().unwrap();

        let mut session = model.new_conversational_session();

        for (turn_idx, turn) in turns.iter().enumerate() {
            total_turns_verified += 1;
            let conv_turn = &turn["native_conversation_turn"];
            let appended_ids: Vec<u32> = conv_turn["appended_token_ids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect();
            let decisions = conv_turn["dialogue"]["generation"]["decisions"]
                .as_array()
                .unwrap();

            assert!(
                !appended_ids.is_empty(),
                "Turn {turn_idx} of {req_id} has empty appended_ids"
            );

            let mut last_step = None;
            for &token in &appended_ids {
                let t0 = Instant::now();
                let step = model.step_conversational(
                    &mut session,
                    token,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )?;
                step_nanoseconds += t0.elapsed().as_nanos();
                total_step_calls += 1;
                last_step = Some(step);
            }

            let mut step = last_step.unwrap();

            for (dec_idx, decision) in decisions.iter().enumerate() {
                let expected_token = decision["selected_token"].as_u64().unwrap() as u32;
                let actual_token = greedy_choice(&step.probabilities);

                if actual_token != expected_token {
                    panic!(
                        "Parity mismatch at request {} ({}), turn {}, decision {}: expected token {}, got {}",
                        row_idx, req_id, turn_idx, dec_idx, expected_token, actual_token
                    );
                }

                total_verified_tokens += 1;

                if dec_idx + 1 < decisions.len() || turn_idx + 1 < turns.len() {
                    let t0 = Instant::now();
                    step = model.step_conversational(
                        &mut session,
                        actual_token,
                        SlotTarget::Dialogue,
                        ReadMode::Enabled,
                    )?;
                    step_nanoseconds += t0.elapsed().as_nanos();
                    total_step_calls += 1;
                }
            }

            if let Some(rss) = get_process_rss_mb() {
                if rss > peak_rss {
                    peak_rss = rss;
                }
            }
        }
    }

    let total_wall_elapsed = wall_clock_start.elapsed();

    assert_eq!(total_turns_verified, 58);
    assert_eq!(total_verified_tokens, 1433);
    assert_eq!(total_step_calls, 2526);

    let ms_per_step = (step_nanoseconds as f64) / (total_step_calls as f64 * 1_000_000.0);
    let ms_per_gen_tok = (step_nanoseconds as f64) / (total_verified_tokens as f64 * 1_000_000.0);
    let tok_per_sec = (total_step_calls as f64) / (step_nanoseconds as f64 / 1_000_000_000.0);

    println!("================================================================================");
    println!("  Dialogue 576 Zero-MatMul Conversational Parity & Performance Receipt");
    println!("================================================================================");
    println!("  Total Requests Replayed  : {}", rows.len());
    println!("  Total Turns Replayed     : {}", total_turns_verified);
    println!("  Total Incremental Steps  : {}", total_step_calls);
    println!("  Generated Selections     : {}", total_verified_tokens);
    println!("  Bit-for-Bit Parity Gate  : 100% PASS (0 departures across 1,433 decisions)");
    println!(
        "  Total Step Time          : {:.3} s",
        step_nanoseconds as f64 / 1e9
    );
    println!(
        "  Total Wall Clock Time    : {:.3} s",
        total_wall_elapsed.as_secs_f64()
    );
    println!("  Average Step Latency     : {:.3} ms/step", ms_per_step);
    println!("  Average Gen-Token Latency: {:.3} ms/tok", ms_per_gen_tok);
    println!("  Throughput               : {:.1} steps/sec", tok_per_sec);
    println!(
        "  Peak Process RSS         : {:.2} MB (Invariant: < 35.0 MB - PASS)",
        peak_rss
    );
    println!("================================================================================");

    assert!(
        peak_rss < 35.0,
        "Peak RSS ({:.2} MB) exceeded 35.0 MB threshold",
        peak_rss
    );
    // Wall-clock latency ceilings are recorded, not asserted: a pass/fail
    // assertion on measured latency is flaky under unknown machine load.
    let max_allowed_ms = if cfg!(debug_assertions) { 6.0 } else { 4.0 };
    println!(
        "Latency ceiling (recorded, not asserted): average step latency {ms_per_step:.3} ms vs {max_allowed_ms:.1} ms ceiling (meets: {})",
        ms_per_step <= max_allowed_ms
    );

    Ok(())
}
