# ORDER: m7-parity-library-cli-browser, antigravity-26

Lab: antigravity. Item: [m7-parity-library-cli-browser](../../../../roadmap/items/m7-parity-library-cli-browser.md). Base: `0b9f0a2ca`.

## Goal

Repair PR `#3008` in response to DeepSeek's `CHANGES` review ([`#3008` comment `6352639499`](https://github.com/UOR-Foundation/uor-r4/pull/3008#issuecomment-6352639499)) so that the bar-registration request on `#2867` and the lab docs match the delivered `vocabulary = 40` 8-prompt panel (`m7-parity-prompts-v1`) and print the exact re-run commands and CI note.

## Cancel probe (run first)

- DeepSeek posted `CHANGES` on PR `#3008` at `0b9f0a2cae0d71d036085b42afca78923400f21b` (`https://github.com/UOR-Foundation/uor-r4/pull/3008#issuecomment-6352639499`) because the initial bar-registration comment on `#2867` (`6352106589`) listed a pre-implementation `vocab = 64` prompt table (`P1_pair`..`P7_octave`) whereas the delivered harness (`m7_parity_library_cli_browser.rs`, `verify_m7_parity.mjs`, `preregistration-DRAFT-2026-10-22.md`, and `README.md`) executes the `vocabulary = 40` 8-prompt panel (`P0_bos`..`P7_octave`).
- Revised bar-registration request posted at [`#2867` comment `6352825431`](https://github.com/UOR-Foundation/uor-r4/issues/2867#issuecomment-6352825431). Proceeding with the doc updates.

## Files in scope

- `docs/labs/m7-parity-library-cli-browser/README.md` (estimated ~20 lines: link revised `#2867` comment `6352825431`, print exact re-run commands, and note CI coverage)
- `docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md` (estimated ~10 lines: link revised `#2867` comment `6352825431` superseding `6352106589`)

## Done when

- `docs/labs/m7-parity-library-cli-browser/README.md` and `docs/labs/m7-parity-library-cli-browser/preregistration-DRAFT-2026-10-22.md` cite the revised `#2867` bar-registration comment (`6352825431`, superseding `6352106589`).
- `docs/labs/m7-parity-library-cli-browser/README.md` includes the exact re-run commands and notes that standard CI does not run `--features serving-wasm` parity tests or the Node browser harness.
- `uor-harness loop report --check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass cleanly.

## Stop and ask if

- Any file outside `crates/uor-r4-integer/**`, `web/**`, `docs/products/**`, or `docs/labs/m7-parity-library-cli-browser/**` needs modification.
