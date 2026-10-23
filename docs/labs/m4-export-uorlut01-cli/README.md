# M4 export CLI: host preparation record

PR #3087 (`kimi-31`) supplies the first `uor-chat` path for a table-native discrete artifact and the host prerequisites for #2859. This is **partial**: `References #2859`; no trained checkpoint, held-out quality score, export-gap measurement, M4 acceptance or GPU run exists. The design note is [#2859 comment 6371804384](https://github.com/UOR-Foundation/uor-r4/issues/2859#issuecomment-6371804384), and the unresolved run inputs are registered in [PRE-REGISTRATION.md](PRE-REGISTRATION.md).

## CLI and finite control

At source `a42181a512f9c574075be261cba1f1efc530a478`:

- `uor-chat --discrete MODEL.lut --prompt-ids CSV` loads `uor-r4.lut-discrete/1` through `IntegerDiscreteModel` and emits fixed-budget greedy ids. The initial prompt must fit the context, which then slides one position per generated token. Wrong schemas, bad tokens, over-context prompts, nonzero temperature and unsupported text/session modes are refused.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --test discrete_chat_cli`: **3 passed, 0 failed**. Two fresh processes produced byte-identical JSON; each run's eight generated ids matched direct `evaluate_prefix` predictions. The fixture needs no trained model, external artifact, new dependency or `Cargo.lock` change.
- This proves deterministic fresh-process reload of one toy artifact. It does not prove conversational persistence, language quality, trained capability or machine-code coverage of the new CLI root.

## Static instruction audits

Build: `CARGO_TARGET_DIR=local/kimi-m4-target ~/.cargo/bin/cargo build --release --locked -p uor-r4-integer --features serving-wasm` (initial 6 m 56 s; incremental 1 m 15 s; `rustc 1.97.1`). Native commands used the same script with each artifact path, `--tap`, and `--git-commit a42181a512f9c574075be261cba1f1efc530a478`; the stack target also used `--stack`.

| Artifact and SHA-256 | Declared audit scope | Result |
|---|---|---|
| `libuor_r4_integer.rlib`, `4c4c5d21f11517c1b6dac79f01a5496f232685782c1be467f8a487e1a8d5f2e6` | 59 matched ranges; no transitive call-graph certification | 0 float, 0 mul, 0 div; 5/5 TAP checks pass |
| `uor-r4-stack`, `f2d63b6fafeb29b11799990a27160170859aa824ada979d76b11cec3b68d14a3` | 44 matched ranges; 92 call-graph functions; 44 scheduler instantiations | 0 violations; 5/5 TAP checks pass |
| `uor-chat`, `0da3c6f1bd1a923eecbe0bb05e0c40f2ec2377767276050c2f00a5754c413f89` | 55 matched ranges; 113 call-graph functions | 0 violations; 5/5 TAP checks pass |

WebAssembly build used `PATH="$HOME/.cargo/bin:$PATH"`, `CARGO_TARGET_DIR=local/kimi-m4-target`, `RUSTFLAGS="-C target-feature=-simd128"`, and `wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`. `scripts/audit_zero_matmul_wasm.py ... --serving --call-graph --tap` passed **7/7** serving roots with `mul=0`, `div=0`, `float=0`, `unclassified=0`. Artifact: 381,818 bytes, SHA-256 `2c6e22ec1451702f5fb4bd5a3f5f7177b5f2583206fae05d2e0aaf5417dbb05f`.

These are scoped absence measurements, not whole-binary or model-quality proofs. The new discrete CLI root is outside the named native and WebAssembly root sets. Indirect calls, allowlisted callees and absent/inlined symbols are not thereby certified; resolving discrete-root coverage is a pre-FIT blocker in the draft sheet.

## Test-suite result and environmental limits

`cargo test --locked -p uor-r4-integer --features serving-wasm --no-fail-fast` completed all targets: **500 passed, 36 failed, 3 ignored**. The library suite passed **303/303**, the `uor-chat` binary tests **3/3**, and the new discrete CLI tests **3/3**.

- One M7 failure was caused by `wasm-pack` finding Homebrew's shadowing `cargo-ai` instead of the required Rust cargo. With `PATH="$HOME/.cargo/bin:$PATH"`, the focused M7 rerun passed **2/2**. Thus 501 tests passed across the full run plus that focused rerun; the M7 failure is resolved.
- The remaining **35 failures in seven targets** all come from absent historical bundle directories under `/Users/casey.allard/uor-r4-investigations`: `adversarial_challenger_m1_token_boundary` (6), `adversarial_challenger_m2_projection_latency` (5), `challenger_m2_stress_test` (5), `challenger_m3_hardware_invariants` (3), `diagnostic_slot_scaling` (3), `real_bundle_alignment_m1` (5), and `zero_matmul_4k_projection_m2` (8). The entire parent directory is absent. A focused reproduction also fails on fresh `main`; these are inherited fixture-availability failures, not kimi-31 regressions. The full suite is therefore **not green**, and no fixture was deleted, ignored, substituted or edited to hide that result.

All work was host-only with $0 GPU spend. Before any trained-artifact run, resolve the accepted checkpoint, held-out chat stream and #2861 scorer, add discrete-root audit coverage, restore the standard fixture availability needed for a green suite, then complete FIT and owner release.
