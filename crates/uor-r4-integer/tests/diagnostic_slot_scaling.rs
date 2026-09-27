//! Empirical breakdown of step_conversational latency components across memory sizes
use std::path::Path;
use std::time::Instant;
use uor_r4_integer::bundle::Bundle;
use uor_r4_integer::config::ReadMode;
use uor_r4_integer::model::SlotTarget;

const REAL_QUATERNION_BUNDLE: &str =
    "/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-quaternion-1";

#[test]
fn test_m2_latency_scaling_vs_dialogue_slots() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    let mut session = model.new_conversational_session();

    println!("\n=== STEP_CONVERSATIONAL LATENCY vs DIALOGUE SLOTS ===");
    println!("Slots | Enabled Latency(ms) | NoRead Latency(ms) | Iterations");
    println!("------+---------------------+--------------------+-----------");

    let slot_checkpoints = [0, 20, 50, 100, 150, 200, 224];
    let mut current_slots = 0;

    for &target_slots in &slot_checkpoints {
        while current_slots < target_slots {
            let _ = model
                .step_conversational(
                    &mut session,
                    (current_slots % 4000) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap();
            current_slots += 1;
        }

        // Measure 20 iterations at this slot count
        let iters = 20;
        let t0 = Instant::now();
        for i in 0..iters {
            let _ = model
                .step_conversational(
                    &mut session,
                    ((current_slots + i) % 4000) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::Enabled,
                )
                .unwrap();
        }
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        // Measure NoRead at the same slot count
        let t0_noread = Instant::now();
        for i in 0..iters {
            let _ = model
                .step_conversational(
                    &mut session,
                    ((current_slots + i) % 4000) as u32,
                    SlotTarget::Dialogue,
                    ReadMode::NoRead,
                )
                .unwrap();
        }
        let elapsed_noread_ms = t0_noread.elapsed().as_secs_f64() * 1000.0 / iters as f64;

        println!(
            "{:5} | {:17.3} | {:15.3} | {:10}",
            target_slots, elapsed_ms, elapsed_noread_ms, iters
        );
        current_slots += iters * 2;
    }
}

#[test]
fn test_m2_detailed_step_conversational_breakdown() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");

    use uor_r4_integer::session::ChatSession;
    let mut chat_session = ChatSession::new(&bundle, None, 42).expect("chat session");
    for i in 0..224 {
        let mut pending = Vec::new();
        let _ = chat_session
            .step_stream((i % 4000) as u32, &mut pending)
            .unwrap();
    }

    let iters = 100;
    let mut latencies = Vec::with_capacity(iters);
    let mut pending = Vec::new();
    for i in 0..iters {
        let t0 = Instant::now();
        let _ = chat_session
            .step_stream(((300 + i) % 4000) as u32, &mut pending)
            .unwrap();
        latencies.push(t0.elapsed().as_micros() as f64 / 1000.0);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = latencies.iter().sum::<f64>() / iters as f64;
    let p50 = latencies[iters / 2];
    let p90 = latencies[(iters as f64 * 0.90) as usize];
    let p95 = latencies[(iters as f64 * 0.95) as usize];
    let p99 = latencies[(iters as f64 * 0.99) as usize];
    let min = latencies[0];
    let max = latencies[iters - 1];

    println!("\n=== CHAT_SESSION.STEP_STREAM AT 224 SLOTS (100 ITERS) ===");
    println!("min: {min:.3} ms, p50: {p50:.3} ms, mean: {mean:.3} ms, p90: {p90:.3} ms, p95: {p95:.3} ms, p99: {p99:.3} ms, max: {max:.3} ms");
}

#[test]
fn test_m2_micro_breakdown() {
    let bundle_path = Path::new(REAL_QUATERNION_BUNDLE);
    let bundle = Bundle::load(bundle_path).expect("load bundle");
    let model = bundle.model();

    let mut session = model.new_conversational_session();
    for i in 0..224 {
        let _ = model
            .step_conversational(
                &mut session,
                (i % 4000) as u32,
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
                ((500 + i) % 4000) as u32,
                SlotTarget::Dialogue,
                ReadMode::Enabled,
            )
            .unwrap();
    }
    let total_us = t0.elapsed().as_micros() as f64 / iters as f64;
    println!(
        "\n=== TOTAL STEP_CONVERSATIONAL LATENCY: {:.3} ms ({:.1} us) ===",
        total_us / 1000.0,
        total_us
    );
}
