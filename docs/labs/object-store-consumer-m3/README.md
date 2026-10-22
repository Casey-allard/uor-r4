# M3 Object-Store Consumer and Durable-Memory Pre-Registration — 22 October 2026

- **Roadmap item:** `object-store-consumer-m3` (Issue `#2873`)
- **Milestone:** M3 (durable conversation memory)
- **Mechanism:** `uor-r4-object-store` (`durable_consumer`) + `uor-r4-integer` (`session_codec::export_scope` / `import_scope`, `UR4SESS1`)
- **Design note:** [`#2873#issuecomment-6339008473`](https://github.com/UOR-Foundation/uor-r4/issues/2873#issuecomment-6339008473)
- **CHARTER §5.2 bar decision card:** [`#2028#issuecomment-6339008955`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955)

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

Full `uor-r4-object-store` suite: **73 passed, 0 failed** (up from 67 baseline tests).

## 3. Draft Pre-Registration for the M3 Durable-Memory Bars (`m3-memory`)

> **Status:** DRAFT — awaiting M1/M2 model selection, owner ratification of theCHARTER §5.2 bar decision card ([`#2028#issuecomment-6339008955`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6339008955)), a non-author `TEST FITNESS: FIT` review, and a typed owner release before any on-model serving or GPU qualification run.

### Frozen Panel and Hashing Protocol

- **Panel identifier:** `m3-durable-memory-panel-v1` (to be frozen as a canonical JSONL file folded into `uor-r4-object-store` and pinned by its `Manifest::version()` / whole-object SHA-256 prior to the non-author FIT review).
- **Composition (384 total scored turns across 6 sub-panels of 64 items each):**
  1. `provenance` (64 items): Multi-turn fact assertions followed by direct and pronoun-mediated queries requiring exact byte-span and token-occurrence quotation.
  2. `correction` (64 items): Two-step and three-step contradiction/revision chains (`FactAction::Assert` followed by `FactAction::Revise`) querying both the current value and the superseded value.
  3. `abstention` (64 unstated + 64 retracted = 128 queries across 64 dialogues): Queries for unstated entities/relations and explicitly retracted facts (`FactAction::Retract`), scored with both exact refusal (`RefusedUnstated` / `RefusedRetracted`) and the R14 `AbstentionDiscrimination` instrument (`crates/uor-r4-training/src/bin/chat-grade.rs`).
  4. `non_memory_prose` (64 items, `#1718`): Stored facts interleaved with 4–8 turns of non-memory prose containing overlapping entity nouns and distractor values, followed by fact recall.
  5. `reset_isolation` (32 explicit `reset_scope` items + 32 multi-tenant `IdentityScope` cross-query items = 64 items): Post-reset queries and cross-scope queries verifying zero retention after reset and zero cross-scope leakage.
  6. `restart_restore` (64 items): Mid-dialogue `export_scope` (`UORMEM01` + `UR4SESS1`) $\to$ `Store::save` $\to$ fresh process `Store::load` $\to$ `import_scope`, comparing post-restore greedy token generation against uninterrupted continuation over 32 generated tokens per item.

### Proposed Acceptance Thresholds (Submitted via CHARTER §5.2 Decision Card)

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
- Do not start the on-model `m3-memory` run until M1 is met, M2 is released, the frozen panel SHA-256 is committed, a non-author `TEST FITNESS: FIT` review is recorded, and Casey types an owner release.
