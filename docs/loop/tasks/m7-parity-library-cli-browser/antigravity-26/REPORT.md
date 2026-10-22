# REPORT: m7-parity-library-cli-browser, antigravity-26

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

1. **Initial bar-registration comment (`6352106589`) listed pre-implementation `vocab = 64` prompts:** As DeepSeek noted in [`#3008` comment `6352639499`](https://github.com/UOR-Foundation/uor-r4/pull/3008#issuecomment-6352639499), our initial design note on `#2867` (`6352106589`) was posted before inspecting `mk_synthetic_stack.rs` (`vocabulary = 40`, token IDs `0..40`) and therefore listed `P1_pair`..`P7_octave` with token IDs up to `63`, whereas the delivered harness (`m7_parity_library_cli_browser.rs`, `verify_m7_parity.mjs`, `preregistration-DRAFT-2026-10-22.md`, and `README.md`) executes the `vocabulary = 40`-valid 8-prompt panel (`P0_bos`..`P7_octave`). We posted a revised bar-registration request at [`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431) that explicitly supersedes `6352106589` with the exact delivered 8-prompt panel and SHA-256 digests.
2. **No accepted M4 `UORLUT01` model artifact exists yet:** As documented in [`antigravity-25/REPORT.md`](../antigravity-25/REPORT.md), passing on `seed = 5` `mk-synthetic-stack` (`UORLUT01`) proves the three-host parity harness works and is not a model capability or milestone result.

## What changed

- **Docs (`13` lines changed: `+12 / -1` in `antigravity-26`; cumulative PR diff: `+147 / -1` code, `+1,054 / -0` tests, `+127 / -2` docs across 6 scoped files):**
  - [`docs/labs/m7-parity-library-cli-browser/README.md`](../../../../labs/m7-parity-library-cli-browser/README.md) (`+11 / -1`): Linked the revised `#2867` bar-registration request ([`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431), superseding `6352106589`), added `§4 Re-Run Commands & CI Coverage Note` with the exact reproduction commands, and noted that standard CI jobs do not execute the feature-gated `--features serving-wasm` parity test or the Node.js browser harness.
  - [`docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md`](../../../../labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md) (`+1 / -0`): Linked the revised `#2867` bar-registration request ([`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431), superseding `6352106589`).

## Measurements

- **Three-host 8-prompt parity (`m7-parity-prompts-v1` on `seed = 5` `mk-synthetic-stack`):**
  - Uninterrupted 48-token parity across Rust library, `uor-chat` CLI, direct WASM, and Studio Web Worker: `8 / 8` prompts (`384 / 384` tokens, `0` discrepancies).
  - Fresh-process `32 + 16` save/restore parity (including cross-host CLI $\to$ library restore): `8 / 8` prompts (`384 / 384` tokens, `16 / 16` `UR4SESS1` snapshots byte-for-byte identical across all hosts, `9,782 B` to `16,130 B`).
  - Planted-fault controls (`planted_fault_controls_detect_token_and_snapshot_tampering`): `3 / 3` tamper cases rejected.

## Checks run at the head

- `uor-harness loop report --check`: `exit 0`.
- `uor-harness loop pr-check --local`: `exit 0`.
- `uor-harness loop gates --full`: `exit 0` (`7 / 7` gates `GREEN`).

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `context-guard`.

## Still open

- PM registration of the proposed `m7-parity-prompts-v1` parity bar in [`preregistration-DRAFT-2026-10-22.md`](../../../../labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md) (requested in [`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431), superseding `6352106589`), backfilling the staged six-column research notebook row from [`docs/labs/m7-parity-library-cli-browser/README.md`](../../../../labs/m7-parity-library-cli-browser/README.md) (`§5`) into `docs/research/notebook/m4-serving.md` once `notebook:` is added to `m7-parity-library-cli-browser.md`, and re-running the three-host parity suite on the accepted M4 `UORLUT01` model artifact once `#2857` (`m4-export-uorlut01-cli`) produces one. References #2867.

## Plain summary

This repair order (`antigravity-26`) resolves DeepSeek's review finding on PR `#3008` by posting a revised bar-registration request on `#2867` (`6352825431`, superseding `6352106589`) that lists the exact `vocabulary = 40` 8-prompt panel (`P0_bos`..`P7_octave`) executed by the Rust, CLI, and browser parity harness, updating the lab README and pre-registration sheet links to point to the revised comment, and adding the exact reproduction commands and CI coverage note to the lab README.
