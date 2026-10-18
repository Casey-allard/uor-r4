# Training driver: opt-in row-parallel counterfactual path, 18 October 2026

**What:** `DriverConfig.counterfactual_path` selects which exact counterfactual kernel scores each proposal in `PreparedDriver::propose_next` and `transition`. The values are `lanes_v1`, the ordinary kernel and the default, and `rows_v1`, the row-parallel kernel from #2630. #2630 showed on an RTX 5090 that the two give identical losses, chooser input and statuses, with `rows_v1` about 56× faster. The [parity record](../counterfactual-rows-parity-2026-10-18/README.md) has the evidence.

**Why now:** the owner decided on 18 October (~04:45Z) to stop Dose 1 on the ordinary kernel and rerun it on the fast path: "Stop it; rerun on the fast path" (#2029 6245711750). This wiring is that rerun's prerequisite.

**Identity:**
- **Config bytes:** the default is never serialized (`skip_serializing_if = "CounterfactualPath::is_lanes"`), so every existing `DriverConfig` keeps its bytes and its `config_sha256`. A `rows_v1` config serializes the field, so the chosen kernel is part of the bound configuration. The test `counterfactual_path_default_keeps_config_bytes_and_rows_is_explicit` pins both.
- **Driver source:** `driver_source_sha256` changes, because `training.rs`, `attention_learning.rs` and `driver_qualification.rs` are edited. New runs bind the new identity. The stale `prime_zeta` pin test already fails on main.
- **Learning:** no learning law, objective, schedule, read or arm changes.

**Checks:**
- `cargo test -p uor-r4-discrete --lib`: 90 passed.
- `cargo check --features cuda --all-targets` with a stub nvcc: clean.
- fmt: clean.

The GPU end-to-end check is the Dose 1 re-run's own preflight battery: init, the Q1 gate and paired arms. Both arms set `rows_v1`. No compute ran here; $0.
