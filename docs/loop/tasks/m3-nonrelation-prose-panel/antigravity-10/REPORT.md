# REPORT: m3-nonrelation-prose-panel, antigravity-10

Lab: antigravity. Order: [ORDER.md](ORDER.md).

References #2856.

## What did not work, first

- In the initial draft of `data/panels/nonrelation-prose-v1.txt`, row `nrp-028` had a missing space (`"thedistant"` instead of `"the distant"`), which was fixed before freezing the SHA-256 digest (`7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`).
- In the first compile of `crates/uor-r4-training/src/nonrelation_prose_panel.rs`, `StackStore::new(16)` failed with `error[E0061]: this function takes 2 arguments but 1 argument was supplied` (`StackStore::new(lineage, capacity)` takes `(u64, usize)`) and matching `Written::Corrected` / `Written::Conflict` as enum variants failed with `error[E0223]: ambiguous associated type` because `Written` in `scoped_memory.rs` is a struct with fields `action: Update` and `conflict: bool`.
- As a deliberate negative-gate test of the `#1718` false-overwrite detector, removing `written.action == Update::Correct` from `overwrote_stored_fact` in `apply_prose_action` caused `nonrelation_prose_panel::tests::grader_detects_false_assert_and_false_overwrite_including_issue_1718_relation_4` to fail as expected (`panicked at crates/uor-r4-training/src/nonrelation_prose_panel.rs:704:9: assertion left == right failed: left: 2, right: 3`).
- In the first draft of this `REPORT.md`, `uor-harness loop report --check` failed because the template headings (`## Checks run at the head`, `## Skills used`, `## Still open`, `## Plain summary`) were named differently.

## What changed

- Posted the design note on `#2856` (`https://github.com/UOR-Foundation/uor-r4/issues/2856#issuecomment-6337042540`) and the proposed pass-bar decision card to the PM/owner on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`, CHARTER §5.2).
- Added the frozen 50-turn plain-text non-relation prose panel at `data/panels/nonrelation-prose-v1.txt` (SHA-256 `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`, registered in `data/panels/MANIFEST.sha256`), with 10 turns in each of the 5 categories (`small_talk`, `opinion`, `nonrelation_question`, `anaphoric`, `statement_near_miss`) split into 25 `in_domain` probes and 25 `heldout_v2_disjoint` turns.
- Added `crates/uor-r4-training/src/nonrelation_prose_panel.rs` (wired into `crates/uor-r4-training/src/lib.rs` under `#[cfg(test)]`) implementing `ProsePanel::load_frozen`, `seed_world_v2_store`, `apply_prose_action`, `grade_prose_panel`, and 4 unit tests covering SHA-256 verification, clean `Unresolved` grading, false-assert and `#1718` false-overwrite detection on relation 4 (`hometown`), and 4-word run disjointness of the held-out slice from `milestone_world.rs` and `milestone_world_v2.rs`.
- Added the one-page pre-registration sheet `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` (`TEST FITNESS: PENDING`) and lab record `docs/labs/m3-nonrelation-prose-panel/README.md`.

## Measurements

- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib nonrelation_prose_panel` on macOS aarch64: 4 passed, 0 failed in 0.06s.
- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training --lib` on macOS aarch64: 42 passed (38 existing + 4 new in `nonrelation_prose_panel::tests`), 0 failed in 0.32s.
- `CARGO_TARGET_DIR=local/antigravity/cargo-target ~/.cargo/bin/cargo test -p uor-r4-training` on macOS aarch64: 67 passed total across 3 test binaries (`lib`: 42, `dialogue-recall-corpus` bin: 25, `doc-tests`: 0), 0 failed.

## Checks run at the head

- `uor-harness loop gates --full`: `fmt-claim-wording` GREEN, `fmt-harness` GREEN, `claim-wording` GREEN, `loop-gates-watch` GREEN, `loop-drive` GREEN, `git-hooks` GREEN (bound in `receipts.jsonl`).
- `shasum -a 256 -c data/panels/MANIFEST.sha256`: 31/31 files OK (including `nonrelation-prose-v1.txt`).
- `uor-harness ratchet d23`: OK (`4451` remaining across `165` files, baseline `4451`, `+0`).
- `uor-harness ratchet d25`: OK (`93` remaining across `26` files, baseline `93`, `+0`).
- `~/.cargo/bin/cargo run -q --locked -p xtask -- links`: `930 broken` (`+0` vs merge base).

## Skills used

- None.

## Still open

- Running the `world=v2` compiler over `data/panels/nonrelation-prose-v1.txt` to record baseline `false_assert` and `false_overwrite` rates is a GPU evaluation run and stays gated on a non-author FIT review of `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md`, the owner's pass-bar decision on `#2028` (`https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`), and a typed D40 owner release (CHARTER §4.5, §5.1, §5.2), tracked on `#2856`.

## Plain summary

When a user chats with the model, a turn compiler decides whether each message states a new fact (`Assert`), updates an earlier fact (`Correct`), asks for a stored fact (`Query`), or is ordinary conversation that touches no stored fact (`Unresolved`). In an earlier defect (`#1718`), an ordinary conversational sentence starting with words like `"Actually, ..."` was mistaken for a correction (`Correct`) on relation 4 (`hometown`) and overwrote a previously stored fact. This task builds a frozen 50-sentence test panel (`data/panels/nonrelation-prose-v1.txt`) of ordinary conversation across five categories (small talk, opinions, general questions, back-references like `"Actually, ..."`, and statements about third-party objects), along with a Rust grader (`crates/uor-r4-training/src/nonrelation_prose_panel.rs`) that pre-loads a test memory store with all ten `world=v2` facts and counts how often a compiler falsely asserts a fact (`false_assert`) or overwrites an existing stored fact (`false_overwrite`), plus a draft pre-registration sheet (`docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md`) for running the GPU baseline once reviewed and released.
