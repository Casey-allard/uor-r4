# Non-relation prose panel baseline evaluation on `world=v2` (`m3-nonrelation-prose-panel`)

- **Status:** draft pre-registration, **`TEST FITNESS: NOT_FIT`** (non-author review `deepseek-41`, 2026-10-23; see [Verdict](#verdict)). No compute is cleared.
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

### Recorded review

```text
TEST FITNESS: NOT_FIT - M3 #2031 NonrelationProsePanel
sheet: docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md
reviewer: deepseek (non-author FIT review; antigravity authored the design, panel and grader)
note: Configuration not yet pinnable: (1) the sheet still says the pass bar is pending, but the owner registered it at 2026-10-23T02:36:36Z (comment 6356034555; card 6337042887: false_overwrite == 0/50, false_assert == 0/50, clean_unresolved == 50/50 on nonrelation-prose-v1) - quote it in section 4; (2) the base artifact is unpinned: the run compiles through a saved world=v2 compiler whose TurnCompiler::identity() carries artifact_sha256, tokenizer_sha256 and label_schema, but the sheet names no artifact file/path/hash, so the configuration a FIT would clear is not fixed - pin the exact saved compiler artifact and tokenizer before any run. Verified: panel SHA-256 7c29c665... matches the file and MANIFEST; grader module with its 4 finite controls passes (4/4); held-out disjointness test present; cost (<1 min, <$0.05) and stop/negative rules present. Grader commit should be cited (b0430d40).
date: 2026-10-23
A FIT verdict clears compute for the configuration in that sheet only (D22).
```
