# REPORT: close remaining PTY identity races and reconcile delivery evidence, kimi-6

Lab: kimi. Item: [studio-ide-harness](../../../../roadmap/items/studio-ide-harness.md) (#3569). Order: [ORDER.md](ORDER.md). Base: `6fa2baf377e74080e57cb64330f091b701059e01`, tree `36d5b6701c9d3ee9bfdbecde9a21de011b4c0aca`; initial merged main `1e5e84a1d497222099a97e22da024b07759750a8`, latest merged main `f75725d3071abb9488e058ac158a04d8d093014d`. Fresh reviewer agent-67 returned CHANGES, recorded on PR #3890 at the base head. The PR remains draft and unqueued. This is the historical kimi-6 record; the active repair is kimi-7, with identity reconciliation in kimi-8, record corrections in kimi-9, and approval-flow wording correction in kimi-10.

## What did not work

- Agent-67 found that the mutable session id was assigned after the opened snapshot reached component state, leaving an unmount/workspace window that closed nothing. A rejected stale close discarded the returned backend id, and a normal close rejection could restore the old workspace's session. The stale-open tests only used successful close mocks.
- Red-first tests reproduced all three defects. `closes an opened shell when unmount happens before the session effect` observed **zero close calls**. `retains a rejected stale close for retry without mounting it` had no Retry close control. `does not restore an old session when its close fails after a workspace change` restored `session-a`.
- The first read could advance the cursor before xterm mounted; the first-output regression observed **zero writes**. A second shell never emitted a fresh truncation notice. The strengthened observer regression required two distinct notices and observed one.
- The first implementation pass broke two existing contracts: adopting a read snapshot without checking its session id could replace the tracked identity, and removing same-workspace session restoration made the original close-failure regression fail. Those are corrected: mismatched read responses are discarded; only normal same-workspace close failures restore visible identity, and stale/cross-workspace failures expose cleanup retry without mounting.
- The `58d14a29…` directory and `interactive-terminal-clean.log` were worktree-only. The roadmap called the PTY missing, named kimi-4 current, and omitted changed Tauri manifests from scope. kimi-4 said 444 tests, kimi-5 named ec49, and the 445-test notebook row linked to kimi-4. Historical package statements and host identities needed explicit scope.
- The optional direct default integer-crate invocation failed to compile legacy feature-gated APIs. The broader all-target attempt then passed its 499 library tests but failed six `adversarial_challenger_m1_token_boundary` cases because their hard-coded retired `/Users/casey.allard/uor-r4-investigations/integer-serving-20260925/bundle-*` fixtures are absent. These are instrument/environment limits, not a mechanism negative or an all-target pass. The supported `--lib --features serving-wasm` result is used below; unrelated tests were not edited or hidden.

## What changed

- The mutable runtime records the complete shell snapshot immediately when `openTerminal` resolves, before state/effect scheduling. Workspace changes update a synchronous revision ref and close through that identity. Stale opens always close without restoring/mounting; failed closes preserve the snapshot for a bounded user retry rather than silently dropping it.
- Normal same-workspace close failure retains visible session identity with the terminal disposed. Stale or cross-workspace failure exposes `Retry close` with no active session or polling. Opening is blocked while any shell or cleanup remains. Close completion cannot restore the old active session after a workspace revision.
- Reads start only after xterm mounts and are rejected if the response belongs to another session. Truncation notice state resets for each new mounted session. Close, input and resize paths check the tracked snapshot identity and disposal state.
- Item scope includes `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`; the ORDER records the narrow historical-scope exception for those existing manifests only. The item names this repair as current, no longer lists PTY as missing, and retains the agent/runtime/API work as open. Historical typed-shell/current-state package wording is explicitly historical; kimi-4/kimi-5 and notebook pointers are reconciled.
- Merged latest `origin/main` at `9dab054fc2727703336294ac5cf7c24787648079`, including its integer-runtime changes, and rebuilt/re-ran the integer, host, browser, PTY and native paths instead of reusing the previous package as final evidence.
- The previously worktree-only negatives and the agent-67 CHANGES result are copied to the main checkout and verified byte-for-byte. Final evidence is uploaded and read back before worktree cleanup.

## Measurements

- Five new red-first regressions fail before implementation for their intended reasons: pre-effect unmount leak, rejected stale close without retry, cross-workspace session restoration, missing second truncation notice, and first output lost before mount.
- After the repair and compatibility corrections, `src/components/TerminalSession.test.tsx` passes **18/18** and TypeScript passes. Complete typed suite **450/450 across 52 files**, plain-JS **113/113**, Playwright **45/45**, launcher **6/6**, worker/page/static/runtime checks and **7/7** WASM all pass at the merged source.
- Integer library **499 passed, 1 ignored** with `cargo test -p uor-r4-integer --lib --features serving-wasm`. Complete serial default **11/3/21/4/7/13/8** and GUI **13/3/25/4/8/13/9** pass. Repository gates **11/11** and local policy PASS with disclosed advisories, including the explicitly scoped historical manifest warning.
- Lifecycle implementation is `e449ae6270ccfc12939405fe66fe8e83f06cd7a8`; `c67eaa2a5fdd93fc0fa3128f2fc512e685555680` adds scope/evidence corrections. Acceptance source is `351172842123d2e443904613612658e165c552ff`, tree `3519c7737feb406d642c89e2a576c4cbf92c36f9`, merging `9dab054fc…`. The later `f75725d3071abb9488e058ac158a04d8d093014d` merge changes only float-training sources and leaves `src-tauri`, `web/studio`, `uor-r4-integer`, tokenizer and vision inputs byte-identical to this acceptance. Final report-only head/tree belongs in the PR and durable identity.
- Interactive loopback at 35117 returned `ANSWER:fixture-answer`, exact fixture cwd, `24 133`, exit and closed-input refusal, with 15 initial requests and zero external/raw-TypeScript/runtime errors. Host SHA `5cf47ec6ba16fc289263a545b7594c21d4a2466e35fba65b3d68edbd7e7de51a`, dist `ba21724f166d6720fdbca7edac3772b31247e6178cc67eea5d22f3aecba2a349`.
- Combined loopback restored exact transcript/session identity and native edit/save, with 15 initial/12 reopened requests and zero external/raw-TypeScript/runtime errors. Host SHA `959ebc234d9b5ed0b2191dd7cd97b418f480c8036e87c949622bbb6903187603`, dist `4da4dea2e67461b4295291f16472e8ae0d9975004309461a31273e26bfca7ee0`.
- Rebuilt WASM artifact SHA-256 `4a111daf7a1376b37445766c5d30bc297f509dfc355793b67c6234b5d29d9aba`, 806,759 bytes, **7/7** serving audit. Native package is 21.40 MiB with executable SHA-256 `d3db5a4983c4773e1ea264603cf91f8b35d105b113c5cda03af632fa6c3320e2`. Native first output included `NATIVE_FINAL`; `read` returned `ANSWER:fixture-answer`, exact fixture cwd and `24 80`, then exit 0. A distinct second-shell approval was cancelled with no new PTY. App and sidecar were absent after normal quit.
- Previously missing `58d14a29…` negatives, `interactive-terminal-clean.log` and the agent-67 CHANGES review are preserved in the main checkout and project store. File-based storage cannot retain empty fixture directories; a complete archive was uploaded, downloaded, extracted and compared byte-for-byte. Individual logs also match.

## Checks run at the head

- `cargo test -p uor-r4-integer --lib --features serving-wasm`: **499 passed, 1 ignored; 0 failed**. The unsupported-default and optional all-target attempts are retained separately as instrument/environment negatives, not represented as passing or as terminal failures.
- `CARGO_TARGET_DIR="$PWD/target/cowork-tests" cargo test --manifest-path src-tauri/Cargo.toml` and the same command with `--features gui`: complete default **11/3/21/4/7/13/8**, GUI **13/3/25/4/8/13/9**, PASS serially.
- `npm test --prefix web/studio`: **450/450 across 52 files PASS**, including terminal **18/18**. TypeScript and Vite build, **113/113** plain-JS, **45/45** Playwright, **6/6** launcher, worker/page/static/runtime, desktop and **7/7** WASM checks PASS.
- `node web/studio/verify_interactive_terminal.mjs` and `node web/studio/verify_studio_loopback.mjs`: PASS at 35117 with exact identities above.
- `UOR_COWORK_NO_OPEN=1 CARGO_TARGET_DIR="$PWD/local/kimi/cowork-parity-target" bash scripts/studio/open-cowork.sh`: PASS, 21.40 MiB unsigned package, followed by the actual native workflow above.
- `uor-harness loop gates --full`: **11/11 GREEN**; `uor-harness loop pr-check --local`: PASS with disclosed advisories. Tauri formatting and `git diff --check` PASS.
- Evidence is `local/kimi/harness-parity/351172842123d2e443904613612658e165c552ff/`, including `native-acceptance.md` and three native screenshots; combined identity is in the main checkout's `local/kimi/studio-final-acceptance/351172842123d2e443904613612658e165c552ff/identity.json`. Final record-only head gates are separately recorded. Earlier `5d5eae32…`, `6fa2baf377…`, `c67eaa2a5…`, `da6cbaf4…`, `3e201b987…` and `58d14a29…` evidence is intermediate.

## Skills used

Adversarial review reception, systematic lifecycle root-cause investigation, red-first test-driven repair, race/readiness analysis, solution-integrity and scope checks, artifact preservation, documentation synchronization, native accessibility acceptance, and verification before completion.

## Still open

Final record-only head gates/policy, fresh exact-head review, required GitHub checks, protected queue/merge, and fresh-main verification. Full persistent agent runtime, parallel tasks, project memory, scheduling/Dispatch, browser/computer workflows, live R4 API integration, production publication, signed release, and useful-model capability remain unestablished.

## Plain summary

The terminal records opened shells before an async cleanup can lose them, keeps failed closes retryable without restoring stale workspaces, and reads output only after the terminal exists. The repaired lifecycle and merged integer dependency have complete browser, loopback and packaged native evidence, with prior negatives preserved. Protected delivery remains pending; this is not full Cowork parity.
