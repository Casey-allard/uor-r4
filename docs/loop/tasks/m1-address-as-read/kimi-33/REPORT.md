# REPORT: m1-address-as-read, kimi-33

Lab: kimi. Order: [ORDER.md](ORDER.md).

References #2833

## What did not work, first

The first generator drafts failed tests for missing composition, duplicate addresses, an unsupported `Store::default()` call, and a composite replacement address (`264`). The counts were corrected to four stated arms per entity, two historical arms, ten unwritten and ten evicted rows; `Store::new(Limits::default())` and a reversed prime pair fixed construction. Every expected read is now verified during generation. No model checkpoint, FIT verdict, bar, hold or order experiment changed. The held checkpoint and full BPB-stream digest remain open, and the synthetic instrument panel is not language-capability evidence.

## What changed

Added `address-panel-freeze`, a Rust-only generator with no new dependencies. It emits canonical JSONL for 40 current stated-fact hits, 20 previous-version hits, 10 unwritten `Absent` rows and 10 capacity-1 `Evicted` rows, checking each result through `AddressDirectory` and `Store`. Updated the sheet's panel pin, prerequisite table and PM freeze record, and added the lab record. The earlier `NOT_FIT` and its remaining prerequisites are preserved. Harness size at the committed head: **code 1 file +330/−0; docs 4 files +84/−3; tests 0 files +0/−0; generated 0 files +0/−0; other 0 files +0/−0**. No numeric estimate was given, so a ratio is not applicable.

## Measurements

- Panel: 80 rows / 21,364 bytes, SHA-256 `0712371d20e6eb1294c8faebeea7ab9827e2b91c0decfb5d85a52b8cca286789`.
- Two local generations are byte-identical; 80 unique prompts and addresses; 40/20/10/10 family counts.
- Uploaded as `caseyallard/uor-r4-store/data/m1-address-as-read/panel-v1.jsonl`, HF commit `59f964aa60b83b4684abaa34daddd0c6db768d4d`; remote readback matches exactly.
- `cargo test -p uor-r4-object-store`: **78 passed, 0 failed, 1 intentionally ignored** (`m3_durable_memory_panel::write_frozen_panel`).
- `uor-harness loop gates --full`: **11/11 GREEN**.
- GPU evaluation, model scoring, training, order-effect measurements and owner release: **NOT_RUN**.

## Checks run at the head

- `git diff --check`: pass.
- Generator red-green tests: initial missing implementation failed; final composition/reproduction and uniqueness tests 2/2 pass.
- Full object-store suite: 78 passed, 1 ignored.
- `uor-harness loop gates --full`: 11/11 GREEN.
- `uor-harness loop report --check`: PASS at the committed head.
- `uor-harness loop pr-check --local`: PASS at the committed head, including scope, ORDER-first commit and owner-path checks.

## Skills used

`uor-research-architect`, `superpowers:using-superpowers`, `superpowers:brainstorming`, `superpowers:test-driven-development`, `superpowers:systematic-debugging`, and `superpowers:verification-before-completion`.

## Still open

The candidate checkpoint and full BPB-stream digest remain open. The sheet stays `NOT_FIT` and #2833 stays `ready`; a later non-author FIT, PM dispatch and Casey's release are still required before any GPU oracle run. The panel is a deterministic object-store instrument, not proof of conversational memory or language ability.

## Plain summary

The address-read code already worked, but the planned 80-row test panel did not exist. This task creates that panel from synthetic versioned facts, proves every expected read through the object store, uploads it, and pins its exact hash in the pre-registration. It does not run a model, change a bar, or make the checkpoint available. The remaining work is to pin the checkpoint and the BPB stream, then obtain a fresh fitness review and the owner's release before any GPU test.
