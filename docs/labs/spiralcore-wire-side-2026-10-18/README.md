# Wire transport side exploration  2026-10-18

Owner request (2026-10-17/18): merge the standalone wire-compression/transport work into uor-r4 as a
self-contained side exploration and add it to the roadmap and plan, with Kimi PM kept in the loop for
the new run-prompt protocol.

## Delivered

- `spiralcore-base/wire/` (78 files, ~544 KB): Python adapter pair, socket layer, routed hub, opt-in
  mux, process interposer, smokes/benchmarks/interop matrix; Rust `wire-rs` crate (WireSocket/
  WireListener, routed hub, adapter edge/gateway, mux gateway, native mux client), detached from the
  main workspace; docs and bounded evidence logs.
- Canonical transport docs: WIRE-ADAPTER, WIRE-SOCKET, INTERPOSER, RUST-BINDING, PERFORMANCE,
  OPTIMIZATION-CLOSEOUT, WIRE-USABILITY.
- Roadmap: side-exploration subsection in docs/integration/project-track.md; newest-first entry in
  docs/integration/current-state.md.

Not changed: no model crate, no D11 serving, no training hold, no dose path, no root workspace member
list; the crate is standalone with its own [workspace] table.

## Verification (in this worktree, copied tree)

    cargo build --release --offline in spiralcore-base/wire/crates/wire-rs   PASS
    python3 tools/wire_socket_routing_smoke.py                             WIRE_SOCKET_ROUTING_PASS
    bash tools/wire_rust_hub_smoke.sh                                      WIRE_RUST_HUB_PASS

Source repository (review repo) recorded before the copy: WIRE_INTEROP_MATRIX_PASS 18/18. The
EVIDENCE_OK line is withdrawn: the cited tools/check_evidence.py is not in the copied folder. The
18/18 claim was re-measured on main 03d82fb7b on 2026-10-18 (see labs/wire-matrix-repro-2026-10-18).
Conflict-marker scan on touched docs: zero markers.

## Measured headline (committed evidence, loopback, loaded host)

    Rust peer 1.25 MiB compressed   2.10 / 2.86 / 3.30 ms min/mean/max
    plain TCP 1.25 MiB              1.52 / 1.92 / 2.16 ms
    Rust adapter 64 KiB round trip  0.21 / 0.36 / 0.43 ms (Python adapter 0.54 / 0.59 / 0.87)
    Rust hub 256 KiB                0.40 / 0.43 / 0.50 ms
    Rust mux 8 x 128 KiB            2.61 / 3.70 / 4.36 ms
    1 KiB RTT                       wire 41.8 us vs plain 55.1 us (level 1: 33.9 us)

Compression backend zlib-rs: ~2,900 MiB/s compress, ~5,000-8,300 MiB/s decompress; realistic-corpus
ratio equal to miniz_oxide (0.24489 vs 0.24480). Full detail and limits in
spiralcore-base/wire/docs/OPTIMIZATION-CLOSEOUT.md and PERFORMANCE.md.

## Residual limits

- Protocol is zlib-framed for interoperability; the measured residual cost is compression CPU.
- Loopback only; host-load sensitive (same-run harness is the reliable signal).
- No TLS/authentication inside the framing; external TLS/WireGuard/SSH is the guidance.
- Interposer: macOS/Linux only, numeric-IPv4, blocking connect.
- No SpiralCore/FBS routing integration and no model-path use.

## Coordination

- Claim: branch `deepseek/spiralcore-wire-side-exploration`, lab deepseek (docs/harness/claims.json,
  registered before mutation).
- PM: kimi/pm1 is in the loop via the PR and the #2029 reference; this side exploration does not enter
  the falsify-then-optimize path or dispatch queue without a PM/owner decision.
- Review: exact-head independent review is required before the merge queue per protocol.
- Disk: evidence logs are bounded files; the 2026-10-18 disk-full incident source (a recursive
  self-writing corpus log) was removed in the review repo and is not part of this delivery; the 30 GiB
  floor and bounded-log rule stand.

## Next

PR: https://github.com/UOR-Foundation/uor-r4/pull/2613

Independent exact-head review, protected merge, then branch/worktree cleanup and
`scripts/storage/uor-hygiene --apply`. Any promotion into the roadmap path is a PM decision.
