# Non-relation prose panel baseline evaluation on `world=v2` (`m3-nonrelation-prose-panel`)

> **Superseded (25 Oct 2026, claude-94).** This sheet's run is replaced by [`preregistration-sieve-2026-10-25.md`](preregistration-sieve-2026-10-25.md). The saved compiler pinned in §2b has `SparseSoftmax` heads, which CHARTER §3.2 bans, and its module is test-only. The v25 pins are replaced by the sieve frame-table digest (`379c2ecc…`) and the PTX digest recorded at the run (owner log on #2856, comment `6394013509`). The panel, grader and owner bar below are unchanged. This sheet is kept as history.

- **Status:** draft pre-registration, **`TEST FITNESS: NOT_FIT`** (non-author review `deepseek-41`, 2026-10-23; see [Verdict](#verdict)); **the artifact-pin prerequisite is now addressed in §2b (deepseek-58, 2026-10-23) and a fresh non-author FIT is requested.** No compute is cleared.
  - Running the `world=v2` compiler over the panel is a GPU evaluation run and stays HELD until a non-author FIT review passes, the owner decides the pass bar (`#2028`), and Casey types the D40 release.
- **Line:** M3 durable conversation memory (`#2031`, item `m3-nonrelation-prose-panel` `#2856`, folds `#1718`). **Lab:** antigravity.
- **Design note:** `https://github.com/UOR-Foundation/uor-r4/issues/2856#issuecomment-6337042540`
- **Owner pass-bar decision card:** `https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`

## 1. Question and mechanism under test

In `world=v2` (`COMPILER_LABEL_SCHEMA = "m-world-v2.relations-acts/1"`), ordinary conversational prose with no factual assertion or query over the 10 schema relations (`user_name`, `pet_name`, `friend_name`, `hometown`, `lucky_number`, `code_word`, `job`, `home`, `favorite_food`, `favorite_color`) can miscompile as `CompiledAction::Assert` or `CompiledAction::Correct`. In `#1718`, an anaphoric turn compiled to `Correct` on relation 4 (`hometown`) and overwrote a stored fact.

This sheet registers:
1. the frozen 50-turn non-relation prose panel (`data/panels/nonrelation-prose-v1.txt`),
2. the deterministic Rust grader (`crates/uor-r4-training/src/nonrelation_prose_panel.rs`), and
3. the baseline evaluation protocol for measuring `false_assert` and `false_overwrite` rates on `world=v2`.

## 2. Frozen panel identity (`data/panels/nonrelation-prose-v1.txt`)

- **Path:** `data/panels/nonrelation-prose-v1.txt`
- **SHA-256:** `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`
- **Total turns:** `50` (`10` per category across `5` categories; `25` in `in_domain` and `25` in `heldout_v2_disjoint`).
- **Categories (`10` turns each: `5` `in_domain`, `5` `heldout_v2_disjoint`):**
  - `small_talk` (`nrp-001..005`, `nrp-026..030`): greetings, ambient remarks, and conversational fillers.
  - `opinion` (`nrp-006..010`, `nrp-031..035`): subjective preferences and evaluations outside the 10 profile relations.
  - `nonrelation_question` (`nrp-011..015`, `nrp-036..040`): general questions asking about none of the 10 profile relations.
  - `anaphoric` (`nrp-016..020`, `nrp-041..045`): conversational back-references and discourse repairs (`"Actually, ..."`, `"Sorry, I meant ..."`, `"Wait, I meant ..."`) probing `#1718`.
  - `statement_near_miss` (`nrp-021..025`, `nrp-046..050`): declarative third-party or object statements sharing surface syntax with relation assertions without stating a user profile relation.
- **Disjointness guarantee:** the 25 `heldout_v2_disjoint` rows (`nrp-026..050`) share zero 4-word consecutive runs with `crates/uor-r4-training/src/milestone_world.rs` and `crates/uor-r4-training/src/milestone_world_v2.rs` (checked on every `cargo test -p uor-r4-training` run by `heldout_v2_disjoint_slice_shares_no_four_word_run_with_world_v2_sources`).

## 2b. Base artifact pin (added 2026-10-23, deepseek-58)

The FIT prerequisite that the saved `world=v2` compiler and tokenizer be named by file, path and digest is resolved here. Every pin below was verified on the host from a downloaded copy (no GPU, no training, no model execution): `hf download caseyallard/uor-r4-store <path> --repo-type dataset` (private dataset, access required), then `shasum -a 256` on the members. The bundle's own `manifest.json` records blake3 digests; the sha256 values below were computed from the extracted files and are the digests a re-run must match.

| Object | Where it lives | Bytes | SHA-256 |
|---|---|---:|---|
| Sieve bundle (contains the three objects below) | HF store `caseyallard/uor-r4-store`, `data/hot/sieve-panel.tar` | 14,571,520 | `9d8da1253ebd8cdc62780afe54de418c811088b63f6dcac9ac8daf9a7b3d234a` |
| **Saved compiler artifact** (`compiler=`) | member `sieve/compiler-save-op-v25-rawtable/compiler.json` (manifest root `/Users/casey.allard/uor-r4-local/claude-log-sieve/compiler-save-op-v25-rawtable`) | 5,861,613 | `34cdf896abf5d823e185edc269454c6ad92d97b083ad7ef8f093944931783f06` |
| **Trunk** (`trunk=`) | member `sieve/op-model-v25/model/model.safetensors` | 8,439,228 | `04ce0d7a253cca6aeedbcf8d38837fed9fcf3a6b430e13f10eb098ac055db4ce` |
| Trunk config | member `sieve/op-model-v25/model/config.json` | 227 | `e1009111a801518160db3b58a1c9981302ad2649fda296f3cf35e6d0af567627` |
| Tokenizer tar (`tokenizer=`) | HF store `caseyallard/uor-r4-store`, `data/tokenizer.json.tar` | 112,640 | `2893ab5ce68aaae1624e12aa6f8b5c0b117529fc2388058645a6af018bd3d8bb` |
| **Tokenizer file** | member `tokenizer.json` of that tar | 109,457 | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |

- **Compiler identity fields (read from the pinned `compiler.json`):** `schema = uor-r4.relation-compiler/1`, `label_schema = m-world-v2.relations-acts/1`, `tokenizer_sha256 = d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`. The `artifact_sha256` that `TurnCompiler::identity()` carries is the SHA-256 of the `compiler.json` bytes above, `34cdf896…`; the tokenizer digest the compiler expects is exactly the tokenizer file's digest, so the pair is self-consistent.
- **Run configuration is therefore fixed:** `world=v2`, `compiler=<the compiler.json>`, `trunk=<the model directory containing model.safetensors and config.json>`, `tokenizer=<the tokenizer.json>`, `op_policy=unless_query`, `log_recall` irrelevant to this single-pass grading.
- **Pre-run pin gate (add to §6):** before the GPU run, extract the bundle and the tokenizer tar and `shasum -a 256` every file named in the table above (the two tars and the four extracted members), comparing sizes and digests; any mismatch aborts before a checkpoint is loaded. This pin releases nothing: the run still needs the non-author FIT and Casey's typed D40 release.
## 3. Grader and measured rates (`crates/uor-r4-training/src/nonrelation_prose_panel.rs`, commit `b0430d40`)

For each turn $i \in \{1, \dots, 50\}$ in `nonrelation-prose-v1.txt`:
1. A fresh `StackStore` is initialized via `seed_world_v2_store()` with one baseline fact (`[100 + id]`) at scope `b"user"`, entity `&[1]`, for every `world=v2` relation `id` in `1..=10`.
2. The compiler under test produces `action = compiler.compile(&row.text)?`.
3. `apply_prose_action` applies `action` to the seeded `StackStore` with `write = true`.
4. The turn is classified as:
   - `ProseTurnVerdict::Clean` if and only if `action` is `CompiledAction::Unresolved { .. }`, `memory` is `MemoryEffect::Unresolved`, and `store.commit()` is unchanged.
   - `ProseTurnVerdict::FalseAssert { relation, overwrote_stored_fact }` if `action` is `CompiledAction::Assert { relation, .. }`.
   - `ProseTurnVerdict::FalseCorrectOverwrite { relation }` if `action` is `CompiledAction::Correct { relation, .. }` and mutates the stored fact at `relation`.
   - `ProseTurnVerdict::FalseCorrectNoOverwrite { relation }` if `action` is `CompiledAction::Correct { relation, .. }` without mutating a stored fact.
   - `ProseTurnVerdict::FalseQuery { relation }` if `action` is `CompiledAction::QueryCurrent` or `CompiledAction::Query`.

Primary rates (reported as exact `(numerator, 50)` integer pairs, overall, per category, and per split):
- `false_assert_rate = (false_assert_count, 50)`
- `false_overwrite_rate = (false_overwrite_count, 50)`
- `clean_unresolved_rate = (clean_unresolved_count, 50)`

## 4. Pass bar (registered by the owner's standing-direction decision on `#2028`, 2026-10-23T02:36:36Z, comment `6356034555`; card `6337042887`; CHARTER §5.2)

- **Proposed acceptance bar:**
  - `false_overwrite_count == 0` (`0 / 50`), enforcing the M3 requirement that non-memory prose never silently overwrites stored facts (`#1718`);
  - `false_assert_count == 0` (`0 / 50`);
  - `clean_unresolved_count == 50` (`50 / 50`).
- The owner's standing-direction decision registered this bar exactly as proposed at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`; the card is `6337042887`). This registration releases nothing: the run still needs the non-author FIT and Casey's typed release.

## 5. Pre-run finite CPU controls (verified on host)

1. `frozen_panel_loads_and_matches_sha256_and_category_counts`: verifies `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`, 50 rows, 25 per split, 10 per category, and refuses a tampered digest.
2. `clean_stub_compiler_scores_zero_false_assert_and_zero_false_overwrite`: verifies `clean_unresolved_count == 50`, `false_assert_ratio() == (0, 50)`, `false_overwrite_ratio() == (0, 50)`.
3. `grader_detects_false_assert_and_false_overwrite_including_issue_1718_relation_4`: verifies detection of false `Assert`, `#1718` false `Correct` overwriting relation 4 (`hometown`), false `QueryCurrent`, and `write_enabled = false` isolation.
4. `heldout_v2_disjoint_slice_shares_no_four_word_run_with_world_v2_sources`: verifies zero 4-word overlap between the 25 held-out rows and `milestone_world.rs` / `milestone_world_v2.rs`.

## 6. Stop conditions and GPU cost

- **Pre-run gate:** `ProsePanel::load_frozen` must match `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`; any mismatch aborts before loading a checkpoint.
- **Single-pass evaluation:** grading 50 short single-turn strings through the saved `world=v2` compiler on one leased GPU takes under 1 minute (`< $0.05` at `$1.89/h`).
- **Negative classification (CHARTER §4.1–§4.2):** nonzero baseline `false_assert` or `false_overwrite` counts on `world=v2` measure the pre-gate compiler baseline for `m3-memory-turn-discriminator-gate` (`#2855`) and close only the un-gated configuration, never the exact-memory mechanism.

## Verdict

### Why NOT_FIT (prerequisites in priority order)

1. **The owner bar is registered but the sheet still calls it pending.** The owner's standing-direction decision at **2026-10-23T02:36:36Z** (`#2028` comment `6356034555`; card `6337042887`) registered `false_overwrite == 0/50`, `false_assert == 0/50`, `clean_unresolved == 50/50` on `nonrelation-prose-v1` exactly as proposed. §4 now quotes it with the ISO-UTC time (resolved in this PR), and the status line no longer says pending.
2. **The base artifact is unpinned, so the configuration a FIT would clear is not fixed.** The run feeds the frozen panel through a **saved `world=v2` compiler**; `TurnCompiler::identity()` already carries `artifact_sha256`, `tokenizer_sha256` and `label_schema`, but the sheet names no artifact file, path or hash. Before a FIT can release compute, pin the exact saved compiler artifact (path + `artifact_sha256`), its tokenizer digest and `label_schema`, and say where it lives.

**Verified by command at this review:** `data/panels/nonrelation-prose-v1.txt` hashes to `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`, matching the sheet and `data/panels/MANIFEST.sha256`; the grader's four finite controls pass (`cargo test -p uor-r4-training --lib nonrelation_prose_panel` = **4 passed**); the held-out disjointness control is present; the cost (`< 1` min, `< $0.05`), pre-run gate and negative-classification rules are stated. The grader is cited by commit `b0430d40` in §3.

Re-request FIT once the artifact pin is in the sheet; this verdict clears no compute (D22).

### Re-review request after the artifact pin (deepseek-58, 2026-10-23)

- **Prerequisite 2 is addressed:** §2b now names the saved compiler artifact, its SHA-256, `label_schema`, tokenizer digest and the tokenizer file, the trunk files and every bundle member's size and SHA-256, all verified on the host from the HF store copies.
- **Prerequisite 1 was already resolved** by the same earlier repair: §4 quotes the owner-registered bar with its ISO-UTC time.
- **Request:** a non-author (not antigravity, who authored the design/panel/grader, and not deepseek, who wrote this pin) `TEST FITNESS` review of this sheet at its current head. A FIT would clear only the configuration pinned in §2b, and still releases nothing until Casey types the D40 release (CHARTER §5.1). The historical `NOT_FIT` record below is kept unchanged as history.
### Recorded review

```text
TEST FITNESS: NOT_FIT - M3 #2031 NonrelationProsePanel
sheet: docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md
reviewer: deepseek (non-author FIT review; antigravity authored the design, panel and grader)
note: Configuration not yet pinnable: (1) the sheet still says the pass bar is pending, but the owner registered it at 2026-10-23T02:36:36Z (comment 6356034555; card 6337042887: false_overwrite == 0/50, false_assert == 0/50, clean_unresolved == 50/50 on nonrelation-prose-v1) - quote it in section 4; (2) the base artifact is unpinned: the run compiles through a saved world=v2 compiler whose TurnCompiler::identity() carries artifact_sha256, tokenizer_sha256 and label_schema, but the sheet names no artifact file/path/hash, so the configuration a FIT would clear is not fixed - pin the exact saved compiler artifact and tokenizer before any run. Verified: panel SHA-256 7c29c665... matches the file and MANIFEST; grader module with its 4 finite controls passes (4/4); held-out disjointness test present; cost (<1 min, <$0.05) and stop/negative rules present. Grader commit should be cited (b0430d40).
date: 2026-10-23
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
