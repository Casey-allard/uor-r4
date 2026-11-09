# REPORT: isolate MCP and process timeout fixtures, kimi-77

Lab: kimi. Item: studio-typed-shell-upgrade (#3500), predecessor PR #3879. Order: [ORDER.md](ORDER.md). No production behavior changed.

## What did not work

- The complete default suite at `759da8585` passed library 9, CLI 3 and controls 21, then failed the remote MCP timeout/size test with a short-timeout transport error and a broken-pipe fixture write. Subsequent full runs exposed unexpected 400/401 classifications and descendant PID-file startup races.
- A focused MCP run initially passed after delaying only `tools/call`; that was not accepted as proof while the full suite exposed incomplete accepted-socket reads. Accepted fixture sockets now explicitly block under their one-second bound, and partial frames are refused before credential evaluation.
- Two 25 ms descendant fixtures could expire before creating the PID file. The fixtures now synchronize readiness through a pipe and use a one-second startup-plus-timeout bound. The timeout and no-surviving-descendant assertions remain intact.
- The earlier `759da8585` evidence names uncommitted fixture sources and is retained as dirty-tree evidence. Clean exact-head host/dist evidence is `7e3afedb7`; `af86d7ca0` was a docs-only intermediate head before the mixed-case repair, not a final delivery identity. Final gates and review remain pending.

## What changed

Only `src-tauri/tests/mcp_protocol.rs`, test code in `src-tauri/src/lib.rs`, and this record/ORDER were changed by this repair. Accepted fixture sockets explicitly block under their one-second bound; incomplete requests are refused before credential evaluation. Descendant readiness is synchronized, and both distinct closed-pipe tests are retained under distinct names. Production MCP/process code, R4 API/model/engine/worker/tokenizer and application visuals were not changed.

## Measurements

- Before the current-main merge, default Rust suites were **9/3/21/7/13/8** and GUI suites **11/3/25/8/13/9**. After retaining both distinct closed-pipe tests and correcting the merged `exited` binding, the serial merged-head suites are **11/3/21/7/13/8** default and **13/3/25/8/13/9** GUI (library/main/controls/MCP/review/sidecar), PASS. Concurrent overloaded runs retained timing failures and are not hidden by these later serial passes.
- The remote timeout test completes ordinary initialize/notification before its original 15 ms hanging tool call; the 1024-byte response cap still fails explicitly. Stdio retains its 25 ms hanging-call assertion and cleanup checks.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check`: PASS after formatting.
- Clean exact-head loopback at `7e3afedb753deb9ea2128b4ae3d39624c6b0bad2` passed with exact transcript/session identity and native workflow evidence. Evidence: `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/7e3afedb753deb9ea2128b4ae3d39624c6b0bad2/{identity.json,loopback-acceptance.json,loopback-acceptance.png}`. `af86d7ca0` differed only by report formatting at that point; it was not the final source head. Earlier typed/plain-JS/runtime results are in `../kimi-75/REPORT.md`.

## Checks run at the head

- `CARGO_TARGET_DIR="$PWD/target/acceptance-tests" cargo test --manifest-path src-tauri/Cargo.toml`: PASS at the corrected merged source.
- The same command with `--features gui`: PASS at the corrected merged source.
- `node web/studio/verify_studio_loopback.mjs`: exact session/transcript and native workflow acceptance PASS at `7e3afedb753deb9ea2128b4ae3d39624c6b0bad2`.

## Skills used

Systematic root-cause investigation, red-first regression checks, fixture synchronization, serial default/GUI verification, exact identity-bound browser acceptance, merge-conflict preservation, and verification before completion. Independent exact-head delivery review is required separately.

## Still open

Final repository gates, exact-head review, PR checks and protected delivery remain separate pending steps. The neutral workbench, interactive PTY, shared agent runtime and Claude-owned R4 API integration are separate work, not implied by this fixture repair.

## Plain summary

The test fixtures now measure the intended timeout and lifecycle behaviors rather than racing partial input or connection setup. Existing production behavior and short hanging-call assertions are preserved, and full default/GUI checks pass serially at the corrected merged source.
