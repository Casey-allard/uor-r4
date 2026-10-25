# Lab Record: `m3-nonrelation-prose-panel` (`#2856`, `antigravity-10`)

> **25 Oct 2026 (claude-94, #3188): the baseline is measured on a lawful compiler.** The saved world=v2 compiler this record pinned has `SparseSoftmax` heads, which CHARTER §3.2 bans, and its module is test-only. So the run below was never lawful (owner log on #2856, comment `6394013509`). claude-94 adds a lawful prime-sieve frame compiler and grades the frozen panel on the GPU through it:
> - sheet: [`preregistration-sieve-2026-10-25.md`](preregistration-sieve-2026-10-25.md);
> - compiler: `crates/uor-r4-training/src/sieve_frame_compiler.rs`;
> - kernel: `crates/uor-r4-training/src/sieve_frame.cu`;
> - run binary: `crates/uor-r4-training/src/bin/prose-panel-sieve.rs`;
> - action types, moved unchanged so they build outside tests: `crates/uor-r4-training/src/turn_action.rs`.
>
> The result is in [§ Sieve baseline](#sieve-baseline-claude-94-25-oct-2026). The sections after it are the earlier record, kept unchanged.

## Sieve baseline (claude-94, 25 Oct 2026)

One RTX 5090 (Vast, driver 590.48.01, nvcc 12.8), pod `vast-57621556`, about 6 minutes, run ref `3e99a0d63912e4dfd7b8a7043c1274d13137e250`. Gates: decision review APPROVE (#2856 comment `6394013509`), FIT (`6394193653`), pre-run GO (`6394214199`).

**Negatives first.**
- **R2 is 0/220.** The compiler compiles no development phrasing at all. Every miss is `Unresolved`, so no unseen wording produced a wrong write. Exact frames reach only the wordings they were built from.
- **The agreement gate was never exercised on the panel.** No panel turn aligned with any frame: the total of surviving frames across the 50 turns is 0. The panel is clean because nothing aligned, not because frames disagreed.

| Measure | Value |
|---|---|
| tokens/s, lean build (upload to download) | **896,898** (732 turns, 4,280 words, 4,772 µs); the `cuda` build gives 811,682 |
| Panel false_assert | **0 / 50** |
| Panel false_overwrite | **0 / 50** |
| Panel clean_unresolved | **50 / 50** |
| R1: training phrasings with development values, plus each query phrasing once, exact | **462 / 462** (floor ≥ 95%: met) |
| R2: development phrasings, exact | **0 / 220** (no bar) |
| Registered bar (owner bar with the floor) | **met** |
| Finite CUDA controls | 12 / 12 on both builds |
| Second build (`cuda` against `sieve-cuda`) | `decisions.json` byte-identical, sha256 `d3f05ecf7d26058dac0c43f82069f7409a2eed240352eff90cb95aad4228f645` |
| D29 interlock | passed on both features, 5 tests each: the bootstrap ran it under `cuda` (`d29.log` in the HF store's `bin/3e99a0d63912e4dfd7b8a7043c1274d13137e250-sm120.tar`, sha256 `0cf7dba482606ed1d2fe774ecfe11c6e5e19aa0d1bbbaa4865689793dab6db5f`), and the job ran it on the lean build (`d29-lean.log`) |

**Pins.**
- panel `7c29c665…a3de7`;
- frame table `379c2ecc4f1d04b49168d76e43ac93f3d07730e1016e0a281608740efb46604f` (84 frames, 79 lexicon words);
- PTX `b80f96c492e743e774c4a49def072f2356be5cab646707fb2906381e5c3a1806`;
- binaries: lean `ab27e436…c65d`, cuda `71cf2a10…cc56`;
- job script `c5ab371d…0a1b`.

**Outputs.** HF store `caseyallard/uor-r4-store`, `results/claude/prose-panel-sieve-2026-10-25/`: `record.json`, `decisions.json` and `controls.json` for each build, plus the binaries, logs and job script. The folder's `SHA256SUMS` covers the run's own 12 files, and `SHA256SUMS.all` covers all 16.

**Reading (scope: the sieve compiler on the world=v2 schema, D22).** The baseline false-assert and false-overwrite rates are both 0/50, with true statements still compiling (R1 462/462). That is the item's done-when. It is not a pass of `m3-memory-turn-discriminator-gate`: that gate's work is a compiler that compiles unseen phrasings (R2 above 0) while the panel stays at 0/0/50.

- **Item:** [`docs/roadmap/items/m3-nonrelation-prose-panel.md`](../../roadmap/items/m3-nonrelation-prose-panel.md) (`#2856`, folds `#1718`)
- **Design note:** `https://github.com/UOR-Foundation/uor-r4/issues/2856#issuecomment-6337042540`
- **Owner pass-bar decision card:** `https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`
- **Pre-registration sheet:** [`preregistration-DRAFT-2026-10-22.md`](preregistration-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`)

## Deliverables landed in `antigravity-10`

1. **Frozen plain-text non-relation prose panel (`data/panels/nonrelation-prose-v1.txt`)**:
   - SHA-256: `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7` (registered in [`data/panels/MANIFEST.sha256`](../../../data/panels/MANIFEST.sha256)).
   - 50 turns (`10` per category across `small_talk`, `opinion`, `nonrelation_question`, `anaphoric`, and `statement_near_miss`; `25` in `in_domain` and `25` in `heldout_v2_disjoint`).
   - The 25 `heldout_v2_disjoint` turns share zero 4-word consecutive runs with `crates/uor-r4-training/src/milestone_world.rs` and `crates/uor-r4-training/src/milestone_world_v2.rs`.
2. **Deterministic Rust grader (`crates/uor-r4-training/src/nonrelation_prose_panel.rs`)**:
   - Loads and SHA-256-verifies `data/panels/nonrelation-prose-v1.txt` (`ProsePanel::load_frozen`).
   - Seeds all 10 `world=v2` relations (`1..=10`: `user_name`, `pet_name`, `friend_name`, `hometown`, `lucky_number`, `code_word`, `job`, `home`, `favorite_food`, `favorite_color`) in a `StackStore` (`seed_world_v2_store`) and grades `TurnCompiler::compile` and `apply_prose_action` (`grade_prose_panel`).
   - Reports exact integer counts and `(numerator, denominator)` ratios for `clean_unresolved_count`, `false_assert_count`, `false_overwrite_count`, `false_correct_action_count`, and `false_query_count` overall, per category, and per split.
3. **Finite CPU test suite (`crates/uor-r4-training/src/nonrelation_prose_panel.rs::tests`)**:
   - `cargo test -p uor-r4-training --lib`: 42 passed (38 existing + 4 new in `nonrelation_prose_panel::tests`), 0 failed.
   - `cargo test -p uor-r4-training`: 67 passed total across 3 test binaries (`lib`: 42, `dialogue-recall-corpus` bin: 25, `doc-tests`: 0).
   - `uor-harness ratchet d23`: GREEN (`4451` remaining across `165` files, baseline `4451`, `+0`).
   - `uor-harness ratchet d25`: GREEN (`93` remaining across `26` files, baseline `93`, `+0`).

## Open items before closing `#2856`

- **Artifact pin (deepseek-58, 2026-10-23): DONE.** The sheet's §2b now names the saved `world=v2` compiler artifact, its SHA-256 (`34cdf896…`), `label_schema`, tokenizer digest (`d36d3e87…`) and tokenizer file, the trunk files and the HF-store bundle members with sizes and SHA-256 (verified on the host from `caseyallard/uor-r4-store`; host-only, no GPU). This resolves the remaining `NOT_FIT` prerequisite from the deepseek-41 review.
- **Still open:** a fresh non-author `TEST FITNESS` review of the pinned sheet (not antigravity, not deepseek), PM/owner confirmation that the pinned configuration is the intended one, and Casey's typed D40 release (CHARTER §4.5, §5.1, §5.2) to run the saved `world=v2` compiler over `data/panels/nonrelation-prose-v1.txt` on a leased GPU and record the baseline `false_assert` and `false_overwrite` rates in `docs/research/notebook/m3-durable-memory.md`.

## Non-author FIT review (deepseek-41, 2026-10-23)

Non-author `TEST FITNESS` review of [`preregistration-DRAFT-2026-10-22.md`](preregistration-DRAFT-2026-10-22.md): **NOT_FIT**, recorded in the sheet's `## Verdict`. No compute is cleared.

- **Resolved in the review:** the owner's bar is registered (`2026-10-23T02:36:36Z`, `#2028` comment `6356034555`, card `6337042887`) and §4 now quotes it with the time; the status line no longer says PENDING; the grader is cited by commit `b0430d40`.
- **Remaining prerequisite:** the saved `world=v2` compiler artifact is unpinned — the sheet must name the artifact file/path and its `artifact_sha256`, tokenizer digest and `label_schema` before a FIT can release the baseline run.
- **Verified:** panel hash `7c29c665…` matches the file and `MANIFEST.sha256`; four finite controls pass; held-out disjointness control present; cost, pre-run gate and negative rules stated.

## Artifact pin (deepseek-58, 2026-10-23)

The `NOT_FIT` prerequisite is resolved: `preregistration-DRAFT-2026-10-22.md` §2b now pins the saved `world=v2` compiler and tokenizer by file, size and SHA-256, re-downloadable from the private HF store with `hf download caseyallard/uor-r4-store <path> --repo-type dataset`. Verified on the host (no GPU): bundle `data/hot/sieve-panel.tar` `9d8da125…`; `sieve/compiler-save-op-v25-rawtable/compiler.json` 5,861,613 B `34cdf896…` with `label_schema = m-world-v2.relations-acts/1` and `tokenizer_sha256 = d36d3e87…`; `sieve/op-model-v25/model/model.safetensors` 8,439,228 B `04ce0d7a…` (config `e1009111…`); `data/tokenizer.json.tar` `2893ab5c…` whose `tokenizer.json` is 109,457 B `d36d3e87…`, matching the compiler's tokenizer digest. A pre-run hash gate is added to the sheet; the pin releases nothing.
