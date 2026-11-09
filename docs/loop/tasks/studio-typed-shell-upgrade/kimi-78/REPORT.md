# REPORT: close exact-head acceptance review findings, kimi-78

Lab: kimi. Item: [studio-typed-shell-upgrade](../../../../roadmap/items/studio-typed-shell-upgrade.md) (#3500). Order: [ORDER.md](ORDER.md). Base: `af86d7ca07d4ab97936f6254b35f210f10da560d`. Fresh reviewer agent-61 returned CHANGES; #3879 remains unqueued until final review.

## What did not work

- Mixed-case source requests bypassed the lowercase extension check on the default case-insensitive macOS filesystem. `/src/main.TS` returned **200** before this repair; it now returns **404**. The old regression covered lowercase only.
- The active records incorrectly described later `src-tauri/src/lib.rs` changes as test-only. The loopback static-source guard and explicit canonical-root validation are production host changes after the historical `42bc46f81` package and are not covered by that GUI acceptance.
- The 15 ms MCP test asserted a timeout but did not prove the intended request reached the fixture. Fixture join also swallowed unexpected panics, and expected client disconnects could panic a response writer.
- The `759da8585` dirty-tree evidence, `7e3afedb7` clean exact-head evidence, and `af86d7ca0` docs-only intermediate state were not consistently distinguished. `pr-check --local` is not the required full repository gate.

## What changed

- Static serving rejects `.ts`, `.tsx`, and `.map` ASCII-case-insensitively before canonicalization. Renamed/default roots cover lowercase, uppercase, and mixed-case variants; valid assets remain served. Regular-file `--studio-root` values now have an explicit failing regression.
- The MCP test requires a complete authorized `tools/call` for `read_file` before accepting the original 15 ms timeout. Response writes tolerate deliberate disconnects; fixture thread join exposes unexpected panics. Original 15 ms/25 ms limits and size-cap/refusal assertions are preserved. Production MCP behavior is unchanged.
- Records now identify production host changes separately from historical packaged evidence and retain the three evidence identities and actual clean evidence path. Pre-merge and merged test counts are labeled rather than conflated.
- No model/API/engine/worker/tokenizer or neutral-workbench implementation was added by this repair.

## Measurements

- Mixed-case static regression: **RED `/src/main.TS` = 200 → GREEN 404**, including `.Ts`, `.TSX`, `.Tsx`, `.MAP` and the default source root. Regular-file root and valid canonical-directory checks pass.
- Complete default suites: **11/3/21/7/13/8** (library/main/controls/MCP/review/sidecar), PASS. Complete GUI suites: **13/3/25/8/13/9**, PASS.
- MCP protocol: **7/7 default**, **8/8 GUI**. The exact complete authorized hanging call is asserted, and fixture join checks unexpected panics.
- Previous frontend checks remain **436/436** typed across 49 files and **113/113** plain-JS; no frontend implementation was changed by this repair. Rust formatting and diff checks pass.
- Earlier evidence is retained, not relabeled: `759da8585` dirty-tree, `7e3afedb7` clean host/dist, `af86d7ca0` docs-only intermediate. Current final-head acceptance is measured below after commit.

## Checks run at the head

- `cargo test --manifest-path src-tauri/Cargo.toml --test desktop_client_controls every_static_root_refuses_source_typescript_and_source_maps -- --exact --nocapture`: PASS after the demonstrated mixed-case failure.
- `cargo test --manifest-path src-tauri/Cargo.toml --test mcp_protocol`: PASS, including the exact hanging-call identity and fixture thread join.
- `cargo test --manifest-path src-tauri/Cargo.toml --bin uor-desktop-client studio_root`: **2/2**, including regular-file refusal.
- `CARGO_TARGET_DIR="$PWD/target/acceptance-tests" cargo test --manifest-path src-tauri/Cargo.toml`: complete default PASS; the same with `--features gui`: complete GUI PASS.
- `node web/studio/verify_studio_loopback.mjs` at clean `d39953a7d4ebab479673eba50e06481151b740bc`: PASS, 15 initial/12 reopened requests, zero external/raw-TypeScript/runtime errors, exact session/transcript restoration, native edit/save, guarded approval and refusal. Host SHA-256 `430ac620f81ac73e67f9bf4aa63f70bff4f9f9add8285f24f390d1d7f92e9dd4`; dist SHA-256 `30358639f919fc55fb8fa75b6daf6df71e33e10ac9da7cb5b02452ce51fade55`.
- `UOR_LAB=kimi uor-harness loop gates --full`: **11/11 PASS** at `d39953a7d4ebab479673eba50e06481151b740bc`; logs `local/kimi/gates/1794203482-58704/`. Independent review, GitHub checks and protected merge are pending, not inferred.

## Skills used

Fresh independent exact-head review, systematic root-cause investigation, red-first mixed-case/root/MCP fixture regressions, production-boundary and claim review, full default/GUI verification, and verification before completion.

## Still open

A fresh exact-head reviewer, PR/body and required GitHub checks, protected merge and fresh-main verification remain. No production publish, signed release, full Cowork parity, useful-chat result, or fresh packaged-GUI acceptance is claimed for this unit.

## Plain summary

The static-source boundary now handles real macOS case-insensitive paths, and the MCP fixture proves which request timed out without hiding unexpected failures. Production versus historical package scope and retained evidence identities are explicit. Delivery still requires the final full gate and independent review.