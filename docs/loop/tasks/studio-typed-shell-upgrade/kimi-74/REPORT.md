# REPORT: finish preserved Studio acceptance delivery, kimi-74

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Preserved implementation/acceptance base: `c4d350b8a5af364945471c7d2566c0f1db62bfe6`. This unit delivers the earlier browser/native host repairs and their evidence; it does not deliver the later cowork workbench, full Cowork parity, a production publish, or a useful-model claim.

## What did not work

- `loop pr-check --local` on the combined recovery found two task ORDERs. This predecessor is therefore delivered separately, and the workbench will merge the resulting `main`. At `c4d350b8a`, this unit had one valid `kimi-71`/`kimi-73` chain but no active `kimi-73/REPORT.md`. This repair supplies the required record without changing the implementation.
- The first fresh-worktree Vite build failed because the generated `web/studio/pkg/uor_r4_integer.js` was absent. This is missing build preparation, not a source-code fix: the repository's documented scalar WASM, deterministic synthetic fixture and Tauri preparation commands are used instead. The first typed run did pass **436/436 tests across 49 files** and TypeScript checking before that build refusal.
- The existing packaged evidence belongs to code head `42bc46f81a5f7c3ccc7508866162bdbd7a42d89c`, executable SHA-256 `c8ddeb4039c1721139c990a06c3d188620596cdbb3cdd654308fa3074f53c8c6`. Correction from kimi-78: the later loopback static-source guard and canonical-root validation are production host changes, not test-only differences; they are not covered by that old package. That old executable is not represented as a fresh build of this delivery head. The package is unsigned development evidence, not a signed release.
- Static-WASM and `uor-chat` native-runner browser builds do not provide native Git/terminal actions. The separate token-protected desktop loopback host supplies the measured host workflow; page RPC cannot enable or approve terminal authority itself. Exact `42` is the geometric arithmetic tool, not learned conversation. Post-reopen artifact readiness is not claimed.
- Prior loopback JSON and PNG were copied byte-identical to this worktree's `local/kimi/studio-final-acceptance/pre-c4d-delivery/` before any same-named rerun. Their SHA-256 values are `a9d9c0899806630b4daebcfc6cbc1003c4710a8f1aedc19911af1589c081715f` and `21c84e3b3161bcd1dd8835d4b355bf3be0cb3ecb2b155cd3de13a72c9768606a`.
- The current scope rule reports the serving-notebook path as an advisory D76 warning, not a hidden pass. FILEMAP warnings for unrelated existing folders are preserved. No real browser profile, credential, external test endpoint, production deployment, accessibility certification, M7/M8 completion, or Phase B behavior is claimed.
- The first merged default native run returned HTTP 401 from the unchanged MCP fixture's `remote_json_and_sse_share_session_protocol_and_cors`. Three isolated runs and the subsequent complete default/GUI protocol suites passed; the trigger is not established, no source fix is claimed, and this remains a disclosed intermittent test result rather than hidden success.
- Full repository gates passed **11/11** at `4ec5bd59b`. The later merge adds other labs' source and preserves this unit's implementation; final policy/gate evidence and exact-head review, GitHub checks, protected queue/merge, and fresh-main verification remain delivery steps, not inferred passes.

## What changed

- The existing bounded-process repair allows normal child exit after pipe closure until the actual deadline, while preserving timeout, output-cap, descendant-pipe and termination boundaries.
- The existing desktop loopback host serves reviewed typed `dist`, exposes host-only CLI terminal enablement/exact command preauthorization, and keeps page approval state read-only. Expired approvals are refused and consumed without executing the command.
- The existing browser verifier retains exact two-session identity/transcript state and exercises workspace selection, current-file attachment/save, non-timeout Git inspection, one approved `pwd`, separately cancelled reuse, self-enable refusal, and restored state. The existing package evidence records equivalent native IPC interactions and grant resets at its own code/executable identity.
- This repair adds only `kimi-74/ORDER.md` and this REPORT. No model/API/engine/worker/tokenizer, application behavior, roadmap, release default, or production-publish change is introduced by the repair.

## Measurements

- At merged implementation head `8116553ece2acf5089613ac999f462bd7b651529`: **436/436** typed tests across 49 files; **113/113** plain-JS tests; TypeScript and Vite builds pass; typed-runtime **4/4** and Tauri runtime **7/7** pass.
- Complete default desktop Rust tests passed **9/2/20/7/13/8** (library/main/controls/MCP/review/sidecar), and GUI tests passed **11/2/24/8/13/9**, after the disclosed initial MCP failure. The focused MCP case passed three consecutive isolated reruns.
- The real loopback workflow passed with 15 initial and 12 reopened requests, zero external/raw-TypeScript/runtime errors, exact native edit/save bytes, exact `42`, non-timeout Git inspection, one approved `pwd`, separately cancelled reuse, page self-enable refusal, and restored transcript/attachment/editor state. Evidence is `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/loopback-acceptance.json` and `.png`.
- Worker parity passed with `saveRestoreBitIdentical: true`, `midGenCloseResetGuarded: true`, and zero remote requests. Desktop client reports 32 streamed tokens and 38 invoked commands. Static/runtime page-load checks have zero external/raw-TypeScript/runtime errors.
- Scalar WASM SHA-256 `e03b6cd564e37d7ed4832aec2a51d730b2b7a83e373ca170aca0282aa0be3821` passes **7/7** serving audit checks with zero floating-point or unclassified instructions; integer multiply/divide follow D80/D81. The deterministic synthetic artifact retains SHA-256 `d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`.
- Historical packaged measurements are preserved in [kimi-71/REPORT.md](../kimi-71/REPORT.md) and [kimi-71/PACKAGED.md](../kimi-71/PACKAGED.md), with the exact code/executable identity and limits stated above.

## Checks run at the head

- `npm test --prefix web/studio`: **436/436** at `8116553ec`; this report follows in the next commit without changing implementation.
- `npm run typecheck --prefix web/studio` and `npm run build --prefix web/studio`: PASS after documented preparation; the initial missing-generated-pkg refusal is recorded above.
- `node --test web/studio/assets/js/*.test.mjs`: **113/113**. `python3 scripts/tests/test_typed_runtime.py`: **4/4**. `python3 scripts/tests/test_tauri_runtime.py`: **7/7**.
- `CARGO_TARGET_DIR=<absolute-target> cargo test --manifest-path src-tauri/Cargo.toml`: PASS on the complete rerun; the first run's one MCP failure is recorded above.
- The same command with `--features gui`: PASS.
- `node web/studio/verify_studio_page.mjs --strict-router`, `verify_studio_page_load.mjs --strict-router`, `verify_desktop_client.mjs`, and `verify_studio_worker.mjs`: PASS.
- `python3 scripts/audit_zero_matmul_wasm.py web/studio/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`: **7/7**.
- `CARGO_TARGET_DIR=<absolute-target> node web/studio/verify_studio_loopback.mjs`: `LOOPBACK STUDIO ACCEPTANCE PASSED`.
- `UOR_LAB=kimi uor-harness loop gates --full`: **11/11** at `4ec5bd59b`; final merged-head policy/report/gates run is listed below as a remaining delivery step.

## Skills used

Isolated worktree recovery, protected scope/delivery review, systematic build/setup diagnosis, red-first verification, prior browser/native acceptance inspection, and verification before completion. The fresh exact-head `uor-delivery-reviewer` is a separate required check and is not inferred from these skills.

## Still open

Final policy/report/gates run at the report commit, fresh exact-head review, GitHub required checks, protected queue/merge, fresh-main verification and merged-worktree cleanup. Typed production publication and final packaged launch remain separate. The later cowork workbench and all full-parity/runtime/API work remain separate as well.

## Plain summary

This delivers the previously measured desktop/browser host repairs with their limits intact. It does not silently combine two tasks, change the model, publish the app, or claim that a fixture demonstrates useful conversation. The newer neutral workbench follows in its own reviewed PR.
