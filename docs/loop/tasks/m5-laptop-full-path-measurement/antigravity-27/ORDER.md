# ORDER: m5-laptop-full-path-measurement, antigravity-27

Lab: antigravity. Item: [m5-laptop-full-path-measurement](../../../../roadmap/items/m5-laptop-full-path-measurement.md). Base: `3aee6353c`.

## Goal

The item's done-when reads:

> M5 bar: "Measured energy per token, memory and tokens per second on an M1-class laptop". Latency (ms/tok), peak RSS, energy (mJ/tok) and weights_per_token over total parameters are measured and reproducible across ingestion, routing, memory lookup and emission, with no hidden host float fallback or uncounted external daemon.

Because M4's accepted model artifact (`#2859`, `m4-export-uorlut01-cli`) does not exist yet and the owner's M1 run on that artifact is the authoritative M5 record, this task (`antigravity-27`, `References #2862`) delivers the four-phase `uor-r4-stack cost` instrumentation, the Protocol v3 specification (`docs/compute/energy-measurement-protocol-v3.md`), the finite receipt-validation tests that catch both v2 defects (`work_rate_w` inconsistency and $z$-vs-$t$ confidence intervals), the provisional Apple M1 rehearsal receipt on `mk-synthetic-stack` (`seed = 5`), and the staged owner commands.

## Cancel probe (run first)

- Command: `sh -c 'trap "exit 130" INT TERM; sleep 30' & pid=$!; sleep 0.2; kill -INT $pid; wait $pid; echo exit=$?` -> `exit=130` in ~0.2 s, zero orphan processes.
- `docs/labs/m5-laptop-full-path-measurement/` and `docs/compute/energy-measurement-protocol-v3.md` do not exist yet, and `#2862` had 0 comments prior to our claim. Proceeding.

## Files in scope

- `crates/uor-r4-integer/**` (estimated ~380 lines across `src/stack/session.rs`, `src/bin/uor-r4-stack.rs`, and `tests/m5_full_path_cost_controls.rs`)
- `docs/compute/energy-measurement-protocol-v3.md` (estimated ~140 lines: Protocol v3 sheet)
- `docs/labs/m5-laptop-full-path-measurement/**` (estimated ~220 lines across `README.md` and `provisional-synthetic-stack-receipt.json`)

## Done when

- Design note and PM numeric-threshold decision request are posted on `#2862` and `#2033`.
- `IntegerStackModel` and `uor-r4-stack cost` report exact `weights_per_token / total_parameters` (`total_4bit_weights` and `total_learned_parameters`), per-phase timing and parameter/byte counts across `ingestion`, `routing`, `memory_lookup`, and `emission`, continuous kernel high-water-mark RSS (`getrusage` `ru_maxrss`), Student-$t$ ($t_{0.975, n-1}$) confidence intervals, `work_rate_w == gross_j_per_tok * tokens_per_s` validation, and zero-float / zero-multiply disassembly & process-isolation checks.
  - *Order note on bullet 2 (`getrusage ru_maxrss` $\to$ `vmmap` / `VmHWM`):* Because `crates/uor-r4-integer/src/bin/uor-r4-stack.rs` enforces `#![forbid(unsafe_code)]`, `libc::getrusage` FFI cannot be called; continuous kernel high-water-mark memory is instead read via `vmmap -summary <pid>` (`Physical footprint (peak)`) on macOS and `/proc/self/status` (`VmHWM`) on Linux, with strict refusal when unavailable unless `--allow-provisional` is passed.
- Finite tests in `crates/uor-r4-integer/tests/m5_full_path_cost_controls.rs` (`CARGO_TARGET_DIR=local/antigravity/target ~/.cargo/bin/cargo test -p uor-r4-integer --features serving-wasm --test m5_full_path_cost_controls`) verify the four-phase counters, the `weights_per_token / total_parameters` identity, and rejection of the two v2 receipt defects (`40.5 W` vs `54.53 W` and $z$-based `0.037 mJ` vs $t$-based `0.080 mJ` on `archive/ops/measurement_report_rnd.json`).
- Protocol v3 (`docs/compute/energy-measurement-protocol-v3.md`) and the provisional Apple M1 rehearsal receipt (`docs/labs/m5-laptop-full-path-measurement/provisional-synthetic-stack-receipt.json` + `README.md` with staged `m4-serving.md` row) are committed (`References #2862`).

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**`, `docs/labs/m5-laptop-full-path-measurement/**`, or `docs/compute/**` needs modification.
- Setting a numeric M5 acceptance threshold beyond "measured" requires an owner/PM decision.
