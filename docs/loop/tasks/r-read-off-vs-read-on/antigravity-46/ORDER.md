# ORDER: r-read-off-vs-read-on, antigravity-46

Lab: antigravity. Item: [r-read-off-vs-read-on](../../../../roadmap/items/r-read-off-vs-read-on.md). Base: `49164086b`.

## Goal

The item's done-when reads:

> Paired read-on versus read-off results on the same retained checkpoints are recorded as notebook rows, with the classification instrument versus mechanism stated. The bar for a "read adds something" verdict is not yet registered; registering it, with the score it is read on, is part of How to start. For scale only: on the 14,287 development rows a context-free successor table scores 6,300 top-1, and the trained read scored 149.

This order executes **Step 4** of [r-read-off-vs-read-on.md](../../../../roadmap/items/r-read-off-vs-read-on.md) ("Build the switch and the battery script"):

1. Wire `ReadInitialization::RouteBypassV1` (`R-Off`, `s[13] = 1` in `route_vector` and `discrete_validate`, refusing `Parameter::Route` proposals) and `ReadInitialization::RoutePermutedActionsV1` (`R-Perm`, applying the fixed-point-free signed-axis group derangement $g'(a) = (5a + 1) \bmod 64$ to `routes` at device initialization) into `crates/uor-r4-discrete/src/{lib.rs,discrete.cu,device.rs,training.rs,attention_preflight.rs,d11_export.rs}`, with host unit tests and `#[ignore = "requires a CUDA GPU"]` device tests.
2. Stage the 10 zero-update arm configs (`a8-r-on.json`, `a8-r-off.json`, `a8-r-perm.json`, `a8-f-on.json`, `a8-f-off.json`, `p8-r-on.json`, `p8-r-off.json`, `p8-r-perm.json`, `p8-f-on.json`, `p8-f-off.json`) and the Rust-driven battery script `docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`, and update `docs/labs/r-read-off-vs-read-on/README.md` and `docs/harness/prep/r-read-off-vs-read-on-DRAFT-2026-10-23.md`.

## Cancel probe (run first)

- Probe: `uor-harness loop gates --quick` (cancelled with SIGINT)
- Result: exited `130` cleanly; `docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh` and the `RouteBypassV1` / `RoutePermutedActionsV1` variants in `crates/uor-r4-discrete/src/lib.rs` are absent at base `49164086b`. Continuing.

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-read-off-vs-read-on*`
- `docs/labs/r-read-off-vs-read-on/**`

## Done when

- `ReadInitialization::RouteBypassV1` and `ReadInitialization::RoutePermutedActionsV1` are wired in `crates/uor-r4-discrete/src/{lib.rs,discrete.cu,device.rs,training.rs,attention_preflight.rs,d11_export.rs}` with host unit tests (`cargo test -p uor-r4-discrete`) and `#[ignore = "requires a CUDA GPU"]` device tests (`route_read_off_device_tests`).
- `docs/labs/r-read-off-vs-read-on/evidence/` holds the 10 zero-update arm configs and `read-off-battery.sh` (validated by `sh -n` and a Rust config/table SHA-256 test).
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass.

## Stop and ask if

- Any file outside the declared scope must change.
- Any GPU execution, checkpoint/bar decision, or owner release is needed.
