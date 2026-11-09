# REPORT: acceptance identity and source-serving repairs, kimi-75

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Reviewed base: `73f7529675aa45885ee32d698ec3cccf6d452e20`. Implementation: `759da85854b76d8eacdd64719824461ca6a9e7c2`, plus the separately ordered test-fixture repair in `../kimi-77/ORDER.md`. PR #3879 remains draft pending protected delivery.

## What did not work

- Fresh reviewer agent-59 returned CHANGES: historical GUI evidence was not consistently distinguished from current source checks; reopen asserted substrings rather than exact session/transcript identity; loopback evidence lacked source/host/dist identity; renamed/default roots could serve source TypeScript and invalid `--studio-root` values silently fell back.
- The renamed-root TypeScript regression returned **200** before repair and **404** afterward. Valid roots still work; missing, flag-like, nonexistent and non-directory explicit roots fail rather than silently falling back.
- The first complete default run at `759da8585` failed `remote_auth_timeout_and_size_caps_fail_explicitly`; later descendant fixtures also lacked their expected PID files. A full typed run under competing builds timed out once in `studioNativeTheme.test.ts` without an assertion mismatch. These were measurement/fixture failures, not model or application negatives.
- The historical package executable `c8ddeb4039c1721139c990a06c3d188620596cdbb3cdd654308fa3074f53c8c6` belongs only to code `42bc46f81a5f7c3ccc7508866162bdbd7a42d89c`. The loopback static-source guard and canonical-root validation are later production host changes and are not covered by that historical packaged acceptance; no fresh packaged GUI acceptance is claimed for this unit.

## What changed

- Every static root refuses `.ts`, `.tsx` and `.map`; explicit `--studio-root` requires an existing canonical directory.
- Browser reopen compares the exact active session ID and complete message array. Loopback evidence binds HEAD/tree/status, host-binary SHA-256 and deterministic dist hashes. Prior evidence is preserved.
- `kimi-77` makes the MCP fixture wait for a complete HTTP request on a blocking accepted socket before evaluating credentials. The deliberate delay applies to `tools/call`, so the original 15 ms limit tests the hanging call rather than connection setup. Stdio retains its 25 ms hanging-call assertion. Production MCP behavior is unchanged.
- Two descendant fixtures signal readiness through a pipe and use a one-second startup-plus-timeout bound. Timeout and no-surviving-descendant assertions remain; production process behavior is unchanged.
- No model, engine, worker, tokenizer, R4 API, Claude-owned, production-publication, or neutral-workbench implementation is included in this predecessor.

## Measurements

- At the earlier implementation head, default Rust suites were **9/3/21/7/13/8** and GUI suites **11/3/25/8/13/9** (library/main/controls/MCP/review/sidecar). The later merged counts are **11/3/21/7/13/8** and **13/3/25/8/13/9**, recorded in `../kimi-77/REPORT.md`. All passed after the fixture repairs.
- Frontend: **436/436** tests across **49 files**, **113/113** plain-JS tests, typecheck and build PASS. `test_typed_runtime.py`: **4/4**; `test_tauri_runtime.py`: **7/7**. The load-related visual timeout passed in the subsequent complete run without changing its timeout or assertions.
- Fresh real loopback at `2026-11-09T04:29:03.354Z`: 15 initial/12 reopened requests, zero external/raw-TypeScript/runtime errors, exact native edit/save bytes, non-timeout Git inspection, one approved `pwd`, separately cancelled reuse, page self-enable refusal, `exactTranscript: true`, and `sessionIdMatches: true`.
- `759da8585` is retained as dirty-tree evidence at `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/759da85854b76d8eacdd64719824461ca6a9e7c2/{identity.json,loopback-acceptance.json,loopback-acceptance.png}`.
- `7e3afedb7` is clean exact-head host/dist evidence at `/Users/casey.allard/uor-r4/local/kimi/studio-final-acceptance/7e3afedb753deb9ea2128b4ae3d39624c6b0bad2/{identity.json,loopback-acceptance.json,loopback-acceptance.png}`.
- `af86d7ca0` was a docs-only intermediate head before the mixed-case repair, not the final delivery head. Current final-head acceptance and review are recorded in `../kimi-78/REPORT.md`.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and `git diff --check`: PASS after formatting.

## Deferred minors

- Host CLI approval remains order-dependent and its 60-second validity starts at host launch; existing host-owned behavior is unchanged.
- The loopback verifier does not add a separate read-only terminal-enable query; actual page self-enable refusal is measured and the Rust contract covers the query.
- Later package-build outputs are build contracts only, not GUI acceptance. The sole historical GUI identity remains `42bc46f81`/`c8ddeb4039…`.

## Still open

Full repository gates, fresh exact-head review, PR update and required GitHub checks, protected merge, fresh-main verification and safe generated-output cleanup. The neutral workbench, real PTY and full agent/runtime/API parity remain separate work. No production publication, signed release, accessibility certification, full Cowork parity or useful-model behavior is claimed.

## Plain summary

The predecessor now has measured source-serving repairs, exact browser identity/restoration, and complete default/GUI/typed/plain-JS checks. The fixture fixes retain the original security and timeout assertions and do not turn this unsigned development acceptance into a finished application or model claim.
