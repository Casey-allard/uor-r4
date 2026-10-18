# Object-store pipelined FETCH  2026-10-18

Follow-up to #2607's opt-in FETCH batch. The batch reduced the number of round trips per pull, but
`pull_one` was still strict lock-step: one FETCH, one NODES reply, one chunk at a time. On a
high-latency link every chunk was a full RTT, so the cold-pull time stayed round-trip bound even with
a large batch. `verify_series` already pipelined manifest requests in windows of 64; pull did not.

## Change

- `peer::pull_one` now sends up to a window of FETCH frames before reading their NODES replies in
  order. The default window is `DEFAULT_FETCH_WINDOW = 4` (conservative: every frame is a full
  batch); window 1 reproduces the old lock-step path exactly.
- New public variants keep the old APIs source-compatible:
  `pull_selected_with_window`, `pull_selected_with_batch_window`,
  `pull_series_with_batch_window`; the existing `pull_*` functions delegate with the default.
- `objpeer --fetch-window N` (1..=64) applies to `pull`/`sync`/`repair` and is rejected on
  subcommands that ignore it. `pull_one` takes a small `PullOptions` struct (send_done, batch,
  window) so the private API stays clippy-clean.
- Protocol unchanged: FETCH/NODES framing, ordering and limits are identical; this is client-side
  scheduling only.

## Verification (host-only, no model compute)

Delay-injecting TCP relay (8 ms sleep per forwarded chunk, standing in for RTT) in
`tests/objpeer_cli.rs`; same-run matrix, byte-identical output each time:

    PIPELINE batch=128  window=1  secs=3.706   (lock-step baseline)
    PIPELINE batch=128  window=4  secs=1.902   (~1.95x faster)
    PIPELINE batch=128  window=8  secs=1.324   (~2.8x faster)
    PIPELINE batch=1024 window=4  secs=0.738

Single run on a loaded host, proxy sleeps are not a calibrated network model; the assertion is only
that windows 4 and 8 beat lock-step by >=25%. Full package: `cargo test --offline -p uor-r4-object-store`
61 passed / 0 failed; `cargo clippy --offline -p uor-r4-object-store --all-targets -- -D warnings`
clean; fmt clean. Rejection test covers `verify --fetch-window` and out-of-range values.

## Limits

- Default window stays 4 because each in-flight frame is a full batch: a large window can increase
  socket buffering and latency; the CLI exposes 1..=64 for tuning, not a new default. The effective
  window is also capped by outstanding request bytes (256 KiB), so very large batches degrade to
  window 1 rather than risking a socket-buffer stall.
- The proxy delays per forwarded chunk; it approximates RTT but is not a real WAN or a calibrated
  impairment model. Real-link claims still need a WAN measurement.
- Batch selection and pipelining are orthogonal: the batch still bounds nodes per FETCH, the window
  bounds how many FETCHes are in flight.
- No protocol/server change; no model-path code.

No model compute. Record only.
