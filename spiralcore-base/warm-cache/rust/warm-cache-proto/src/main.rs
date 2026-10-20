use warm_cache_proto::*;

fn main() {
    let mut cell = KuramotoCell::new(7, PHASE_BITS);
    for _ in 0..5000 { cell.step(0.001); }
    let s = cell.snapshot();
    println!("forward R={:.4} epoch={} phase_q={}", s.r, s.epoch, s.phase_q);

    let cell2 = KuramotoCell::new(11, PHASE_BITS);
    let (_after, fwd) = record_forward(cell2, 2.0, 0.001);
    let rev = replay(&fwd);
    println!("replay_error={:.3e} frames={}", replay_error(&fwd, &rev), fwd.len());

    let h1 = TimeHeader::from_snapshot(0, 0, 0, &s);
    let h2 = TimeHeader::from_snapshot(0, 1, 0, &s);
    let a = roots(0, 1, b"payload", &h1);
    let b = roots(0, 1, b"payload", &h2);
    println!("content_equal={} temporal_equal={} paired={:02x?}",
        a.content == b.content, a.temporal == b.temporal,
        &paired_witness(&a.witness, &b.witness, 0)[..8]);

    let seed = Seed::new(vec![9u8; 1 << 16], 1);
    let identical = make_warm(&seed, Plan::Identity, h1.clone());
    println!("cold_bytes={} warm_identical_bytes={} warm_payload={} result={:?}",
        seed.cold_wire_len(), identical.wire_len(), identical.plan.payload_len(),
        receive_warm(&seed, &h1, &identical));

    let patch = make_warm(&seed, Plan::Patch(vec![(10, 4), (20, 5), (30, 6)]), h1.clone());
    println!("patch_wire_bytes={} patch_payload={} result={:?}",
        patch.wire_len(), patch.plan.payload_len(), receive_warm(&seed, &h1, &patch));

    let mut local = h1.clone();
    local.epoch += 1;
    println!("stale_clock={:?}", receive_warm(&seed, &local, &identical));
}
