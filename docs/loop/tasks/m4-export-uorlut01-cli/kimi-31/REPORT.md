# REPORT: m4-export-uorlut01-cli, kimi-31

Lab: kimi. Order: [ORDER.md](ORDER.md). PR #3087 is partial and says `References #2859`; the roadmap item remains `ready`.

## What did not work, first

- **The full crate suite is not green.** With `--no-fail-fast` it completed every target: 500 passed, 36 failed, 3 ignored. The M7 failure was the shadowing Homebrew `cargo-ai`; with `PATH="$HOME/.cargo/bin:$PATH"`, M7 passed 2/2. The remaining 35 failures are in seven targets and all require historical bundles under the absent `/Users/casey.allard/uor-r4-investigations`: `adversarial_challenger_m1_token_boundary` (6), `adversarial_challenger_m2_projection_latency` (5), `challenger_m2_stress_test` (5), `challenger_m3_hardware_invariants` (3), `diagnostic_slot_scaling` (3), `real_bundle_alignment_m1` (5), and `zero_matmul_4k_projection_m2` (8). A focused reproduction fails identically on fresh `main`. No test was ignored, removed, substituted or edited. The ORDER's full-suite-green condition is unmet, so the PR stays draft pending restored fixtures or Casey's decision on a scoped acceptance.
- **The notebook row was not retained.** `loop pr-check --local` refused it because this item has no `notebook:` metadata; the exemption cannot fire. The measured result is preserved in `docs/labs/m4-export-uorlut01-cli/README.md`. Adding the metadata is a separate PM item/scope decision, not a silent widening in this PR.
- **The new discrete CLI root is outside the existing audit root sets.** Their passes are scoped to the named legacy/stack roots, not whole-binary or new-root compliance. Discrete-root audit coverage is a pre-FIT blocker in the draft sheet.
- The first WebAssembly build failed because `wasm-pack` found Homebrew `cargo-ai` (`unknown command 'metadata'`). Prepending `~/.cargo/bin` fixed it and independently repaired the M7 parity test.
- Text chat/REPL, discrete `UR4SESS1` persistence, trained checkpoint export, held-out BPB, export gap, throughput and energy were not implemented or measured. No GPU ran and no trained capability is claimed.

## What changed

- Added `uor-chat --discrete MODEL.lut --prompt-ids CSV`: existing-schema loading, prompt validation, fixed-budget greedy `evaluate_prefix` predictions, bounded sliding context, deterministic JSON, and fail-closed unsupported modes.
- Added a self-contained toy `UORLUT01` integration fixture: two fresh processes match each other and direct library predictions; wrong-schema and invalid-input controls fail. No new dependency or `Cargo.lock` change.
- Updated the integer README and serving/mechanism pages; posted the design at #2859 comment 6371804384; added a PENDING pre-registration and the host record with exact audit scopes, artifact hashes, costs and failures.
- Size: tests 1 file +230/-0; code 1 file +125/-8; docs 7 files +216/-4 (including this REPORT). Total 571 additions/12 deletions against the ORDER's about-650-line estimate: **0.88x**.

## Measurements

Host-only, Mac laptop, `rustc 1.97.1`, $0 GPU spend. Initial release build 6 m 56 s; incremental release build 1 m 15 s.

- `CARGO_TARGET_DIR=local/kimi-m4-target ~/.cargo/bin/cargo test --locked -p uor-r4-integer --features serving-wasm --test discrete_chat_cli`: **3 passed, 0 failed**. Two fresh processes produced byte-identical JSON; 8/8 generated ids per process matched direct `evaluate_prefix`.
- Native audits at `a42181a512f9c574075be261cba1f1efc530a478`: rlib 59 matched ranges, no call graph; stack 44 ranges + 92 call-graph functions + 44 scheduler instantiations; chat 55 ranges + 113 call-graph functions. All report 0 float, 0 mul, 0 div and 5/5 TAP checks. Exact artifact hashes are in the lab record.
- WebAssembly audit at the same source: **7/7 roots pass**, all `mul=0`, `div=0`, `float=0`, `unclassified=0`; artifact 381,818 bytes, SHA-256 `2c6e22ec1451702f5fb4bd5a3f5f7177b5f2583206fae05d2e0aaf5417dbb05f`.
- Full suite: **500 passed, 36 failed, 3 ignored**; M7 focused rerun after PATH repair **2/2**, leaving 35 inherited fixture-availability failures. Library tests **303/303** and `uor-chat` binary tests **3/3** pass.

## Checks run at the head

- `uor-harness loop gates --quick`: 3/3 checks GREEN at `a94368d35` before implementation.
- `uor-harness loop gates --full`: **11/11 GREEN** at `095f7e272536085e2fd3c2e41e7e92e23bd9d149`, after the scope repair and merges of current main.
- `cargo fmt -p uor-r4-integer -- --check` and `git diff --check`: PASS.
- `cargo clippy -p uor-r4-integer --features serving-wasm --all-targets -- -D warnings`: 46 library and 55 all-target findings, all in unchanged files; none in `uor-chat.rs` or `discrete_chat_cli.rs`. Fresh `main` independently fails the library check with the same 46 findings. This is inherited lint debt, not a clean-Clippy claim.
- Early `loop pr-check --local` found the notebook scope and sheet-heading failures; both were repaired and rechecked, leaving only the then-missing REPORT.
- `loop report --check`: PASS at `095f7e272`. Final `loop pr-check --local`: verdict PASS at `095f7e272`, with every applicable rule green.

## Skills used

`uor-research-architect`; Superpowers `using-superpowers`, `executing-plans`, `using-git-worktrees`, `test-driven-development`, `systematic-debugging`, `requesting-code-review`, and `verification-before-completion`. No private Loop Kit text is reproduced here.

## Still open

- Restore the seven targets' historical fixtures or obtain Casey's decision on the unmet full-suite condition; do not mark ready from the scoped audits alone.
- Add `notebook:` metadata through the separate PM item/scope path if the row is to be committed; no row was forced into this PR.
- Resolve discrete-root audit coverage, the accepted trained checkpoint, frozen held-out chat stream and #2861 lawful scorer before FIT. Text chat and discrete session persistence remain future work.
- M4 stays HELD. No FIT, owner release, export, quality score, export gap or throughput result is claimed.

## Plain summary

The chat program can now read the project's discrete integer file and produce deterministic token ids from raw prompt ids. Two separate program runs gave the same answers as the library, and malformed inputs were refused. Existing machine-instruction checks passed within their documented limits. The work does not show useful language, measure the 0.90 bits-per-byte target, or certify the new command's machine code. Thirty-five older tests still cannot run because their historical model folders are missing, so this remains a draft until that condition is resolved or explicitly accepted.
