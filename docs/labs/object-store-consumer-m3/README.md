# M3 Object-Store Consumer and Durable-Memory Pre-Registration — 22 October 2026

- **Roadmap item:** `object-store-consumer-m3` (Issue `#2873`)
- **Milestone:** M3 (durable conversation memory)
- **Mechanism:** `uor-r4-object-store` (`durable_consumer`) + `uor-r4-integer` (`session_codec::export_scope` / `import_scope`, `UR4SESS1`)
- **Design note:** [`#2873#issuecomment-6339008473`](https://github.com/UOR-Foundation/uor-r4/issues/2873#issuecomment-6339008473)
- **CHARTER §5.2 bar decision card:** [`#2028#issuecomment-6339008955`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955) — **owner-registered 2026-10-23T02:36:36Z** ([owner decisions comment `6356034555`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6356034555)): Casey selected **"Yes, register all six (Recommended)"**, and card `6339008955` is registered exactly as proposed. This releases nothing (see the status).

## 1. Open Decisions Resolved

1. **First model consumer (`durable_consumer` + `UR4SESS1`):**
   - `crates/uor-r4-object-store/src/durable_consumer.rs` (`DurableObjectMemoryStore`, `IdentityScope`, `FactProvenance`, `FactVersionRecord`, `FactReadOutcome`, `Ur4Sess1HeaderView`) and `crates/uor-r4-integer/src/session_codec.rs` (`ScopedSessionIdent`, `export_scope`, `import_scope`) implement both durable-session persistence (`export_scope` / `import_scope` with `UORMEM01` + `UR4SESS1`) and occurrence-bound exact fact quotation (`assert_fact`, `revise_fact`, `retract_fact`, `ingest_non_memory_prose`, `read_fact`, `reset_scope`) over `uor_r4_object_store::Store`.
2. **Where exact identity meets geometry:**
   - Exact token IDs (`u32`), occurrence indices and byte spans (`Occurrence`), and content-addressed versions (`ObjectVersion = [u8; 32]`) remain strictly inside `uor-r4-object-store` (`Manifest`, `Bindings`, `DurableObjectMemoryStore`) and never merge with geometric carrier coordinates (`UnitS3Q30`, `IcosianId`, `E8` roots). The store meets the model only at the discrete address/emission boundary (exact slot lookup returning verified bytes and token IDs or an explicit refusal), adding nothing to the discrete training path (`uor-r4-discrete`, D32).
3. **Who owns range reads:**
   - `uor_r4_object_store::RecordBackend` is the single shared validated-record trait, and `Store::range_from` (paired with `Bindings::read_occurrence`) owns authenticated partial range and token-occurrence reads on the model consumer path, while `uor-r4-recon-dag` remains the standalone lazy tree reader over the same record boundary.
4. **Pure integer execution (CHARTER §3.1–§3.3, D11, D23, D30, D32):**
   - Zero floating-point types or operations, zero softmax or squashing functions, and zero dense matrix multiplications appear anywhere on this path. All framing (`UORMEM01`, `UR4SESS1`, `UOROBJ01`, `UORBND01`) and digest checks are exact integer operations.

## 2. Finite CPU Control Suite (`crates/uor-r4-object-store/tests/durable_consumer.rs`)

| Control test | What it verifies | Observed result |
|---|---|---|
| `write_then_read_back_byte_identical_and_occurrence_bound` | `assert_fact` and `revise_fact` ingest source turns via `PinnedTokenizer` + `Bindings`, verify active and previous values via `Store::range_from` and token occurrences, detect unrevised conflicts (`conflict: true`), resolve conflicts on explicit `FactAction::Revise`, and resolve conversational pronouns (`"she"` $\to$ `"Alice"`) | PASS |
| `restart_then_restore_byte_identical_and_ur4sess1_round_trip` | `export_scope` folds a `UORMEM01` envelope carrying a `UR4SESS1` snapshot, active/revised facts, and non-memory prose turns into `Store`; after `Store::save` $\to$ `Store::load` $\to$ `import_scope`, the `UR4SESS1` bytes, fact versions, previous values, and re-exported `Manifest` (`ObjectVersion`) match bit-for-bit | PASS |
| `retracted_or_unstated_fact_refused_rather_than_guessed` | Unstated queries return `FactReadOutcome::RefusedUnstated` with `0` backend fetches; retracted facts (`retract_fact`) return `FactReadOutcome::RefusedRetracted { retracted_at_version }` with `0` backend fetches rather than guessing | PASS |
| `non_memory_prose_never_silently_overwrites_stored_facts` | `ingest_non_memory_prose` folds distraction turns mentioning the same entity words into `Store` without overwriting, mutating, or conflicting with stored facts (`#1718`) | PASS |
| `explicit_reset_and_strict_identity_scope_isolation` | Distinct `IdentityScope` triples (`user_id`, `project_id`, `session_id`) never leak facts across scopes; empty scope identifiers are refused; `reset_scope` wipes facts, antecedent, and `UR4SESS1` snapshot cleanly | PASS |
| `hostile_or_corrupt_record_and_envelope_refused_fail_closed` | Tampered `UR4SESS1` magic/trailing bytes, tampered `UORMEM01` magic, truncated envelopes, forged manifest SHA-256 digests, and cross-scope manifest imports fail closed (`Err(Error::SchemaMismatch \| Error::IntegrityFailure \| Error::BindingMismatch)`) with zero state mutation | PASS |
| `session_codec::tests::export_and_import_scope_round_trip_and_refuse_corrupt_envelope` | `uor_r4_integer::session_codec::export_scope` and `import_scope` round-trip `ScopedSessionIdent` + `SerializedStackSession` (`UR4SESS1`) inside a canonical `UORMEM01` envelope and reject corrupt magic/trailing bytes | PASS |
| `m3_durable_memory_panel::panel_executes_through_durable_consumer_and_beats_no_memory_control` | Executes all 384 items of `m3-durable-memory-panel-v1.jsonl` through `DurableObjectMemoryStore` + `PinnedTokenizer` + `UR4SESS1` (`64/64` provenance, `64/64` correction current + `64/64` prior lineage, `128/128` abstention refusal, `64/64` `#1718` non-memory prose, `64/64` reset & scope isolation, `64/64` `UR4SESS1` restart/restore) against a paired `no-memory` negative control (`0/64` recall, `L-010`) | PASS |

Full `uor-r4-object-store` suite: **77 passed, 0 failed, 1 ignored** (plus `5 passed, 0 failed` in `uor-r4-integer::session_codec`).

## 3. Pre-Registration for the M3 Durable-Memory Bars (`m3-memory`)

> **Status:** FIT (`TEST FITNESS: FIT`, 2026-10-25) — the 384-item panel is **frozen and consumer-verified** (below) and the CHARTER §5.2 thresholds are **owner-registered** (2026-10-23T02:36:36Z, card `6339008955`; see §3). On-model execution awaits M1/M2 model selection (`status: held`).

### Frozen Panel and Hashing Protocol

- **Panel identifier:** `m3-durable-memory-panel-v1` — **FROZEN & CONSUMER-VERIFIED (deepseek-62, 2026-10-24; repaired antigravity-57, 2026-10-25).**
  - **File:** [`m3-durable-memory-panel-v1.jsonl`](m3-durable-memory-panel-v1.jsonl), 179,096 bytes, 384 canonical JSONL items (one object per line, 64 per sub-panel).
  - **Whole-object SHA-256:** `582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1`.
  - **Folded `Manifest::version()`:** `88c27b94526c32b0f06fed7fabeb7a2abcf0b31a2f2534517b8656240eddce70`.
  - **Frozen and verified by:** `crates/uor-r4-object-store/tests/m3_durable_memory_panel.rs`, which deterministically generates the panel, asserts the committed bytes are a fresh generation, folds them into `uor_r4_object_store::Store`, re-checks both pins, and executes all 384 items through `DurableObjectMemoryStore` + `UR4SESS1` against a paired `no-memory` negative control (`L-010`) on every `cargo test -p uor-r4-object-store` run (`antigravity-57` repaired two instrument defects in the initial `deepseek-62` generator: `correction_entry` now revises the same `(name, "hometown")` slot so `prior_value` lineage is non-vacuous `64/64`, and `abstention_entry` `retracted` items now retract the asserted `(name, "hometown")` slot rather than an unstated slot).
- **Composition (384 total scored turns across 6 sub-panels of 64 items each):**
  1. `provenance` (64 items): Multi-turn fact assertions followed by direct and pronoun-mediated queries requiring exact byte-span and token-occurrence quotation.
  2. `correction` (64 items): Two-step and three-step contradiction/revision chains (`FactAction::Assert` followed by `FactAction::Revise`) querying both the current value and the superseded value.
  3. `abstention` (64 unstated + 64 retracted = 128 queries across 64 dialogues): Queries for unstated entities/relations and explicitly retracted facts (`FactAction::Retract`), scored with both exact refusal (`RefusedUnstated` / `RefusedRetracted`) and the R14 `AbstentionDiscrimination` instrument (`crates/uor-r4-training/src/bin/chat-grade.rs`).
  4. `non_memory_prose` (64 items, `#1718`): Stored facts interleaved with 4–8 turns of non-memory prose containing overlapping entity nouns and distractor values, followed by fact recall.
  5. `reset_isolation` (32 explicit `reset_scope` items + 32 multi-tenant `IdentityScope` cross-query items = 64 items): Post-reset queries and cross-scope queries verifying zero retention after reset and zero cross-scope leakage.
  6. `restart_restore` (64 items): Mid-dialogue `export_scope` (`UORMEM01` + `UR4SESS1`) $\to$ `Store::save` $\to$ fresh process `Store::load` $\to$ `import_scope`, comparing post-restore greedy token generation against uninterrupted continuation over 32 generated tokens per item.

### Owner-Registered Acceptance Thresholds (CHARTER §5.2)

**Registration record.** Casey selected **"Yes, register all six (Recommended)"** at **2026-10-23T02:36:36Z** (owner decisions comment [`6356034555`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6356034555)); card [`6339008955`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955) is **registered exactly as proposed**. The registered bar text is: *"64-item frozen panels: provenance ≥ 0.90; correction ≥ 0.90 latest and ≥ 0.85 lineage; abstention ≥ 0.95 refusal with R14 (AnsweredFactHitRate ≥ 0.50, SelectiveAbstentionGap > 0); #1718 exact-zero silent overwrites; reset and scope isolation exact; UR4SESS1 restart byte-identical 64/64"*. The table below is that registration restated per sub-panel; where wording differs, the registered text above governs. Registration releases nothing: the model selection, the non-author FIT and Casey's typed release are still required.

| Sub-bar | Metric | Proposed threshold |
|---|---|---|
| `provenance` | Exact value + verified occurrence span match rate | $\ge 0.90$ ($58/64$) |
| `correction` | Current-version accuracy / prior-version lineage accuracy | $\ge 0.90$ current ($58/64$) and $\ge 0.85$ prior ($55/64$) |
| `abstention` | Refusal accuracy on unstated + retracted queries, plus R14 `AbstentionDiscrimination` | $\ge 0.95$ refusal ($122/128$), $\text{AnsweredFactHitRate} \ge 0.50$, $\text{SelectiveAbstentionGap} > 0$ |
| `non_memory_prose` (`#1718`) | Stored fact retention after distractor prose turns | $1.00$ ($64/64$, zero silent overwrites) |
| `reset_isolation` | Post-reset refusal rate and cross-scope isolation rate | $1.00$ ($64/64$, zero leaks) |
| `restart_restore` | Whole-object SHA-256 verification and token-for-token identical continuation (`IntegerStackSession` + `UR4SESS1`) | $1.00$ ($64/64$ exact token-for-token match) |

### Target Model and Execution Path

- **Target artifact:** The selected M1/M2 exported integer stack artifact (`UORLUT01`, served via `IntegerStackModel` + `IntegerStackSession` + `DurableObjectMemoryStore` with `PinnedTokenizer`).
- **Execution environment:** Deterministic integer serving check (`uor-r4-integer --features serving-wasm` + `uor-r4-object-store`), zero float, zero softmax, zero matmul.

### Stop Conditions

- Abort immediately if any `restart_restore`, `non_memory_prose` (`#1718`), or `reset_isolation` item exhibits a single byte mismatch, silent fact overwrite, or cross-scope leak (these are structural invariants that must hold at $1.00$ on CPU controls before and during on-model evaluation).
- Do not start the on-model `m3-memory` run until M1 is met and M2 selects a candidate integer stack artifact (`status: held`, `#2873#issuecomment-6394411611`).

### Non-Author `TEST FITNESS` Review (`CHARTER.md` §4.5)

TEST FITNESS: FIT

- Reviewed by fresh-context non-author reviewer (`antigravity` subagent `5822df5c-462f-417b-b6dd-56f3dd32660d`, [`#2873#issuecomment-6394411357`](https://github.com/UOR-Foundation/uor-r4/issues/2873#issuecomment-6394411357), 2026-10-25).
