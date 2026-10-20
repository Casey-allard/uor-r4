# #725 — observation: add transactional per-shard checkpoints for exactly-once raw resume

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-15T05:35:12Z
- **Updated:** 2026-08-15T07:30:34Z
- **Closed:** 2026-08-15T07:29:53Z
- **Labels:** enhancement, graph-compiler
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/725
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

PR #724 hardens observation provenance, output preflight, and full-command
`ObservationSession` locking, but deliberately leaves one raw-observation
recovery limitation unchanged.

The autoregressive `observe` path checkpoints only the global teacher state in
the 25-byte `state.bin` layout:

    records u64 | stories u64 | rng u64 | done u8

Shard records, `.prob` rows, and optional `.trace` rows are flushed before that
checkpoint is written. If the process exits after writing complete, aligned
rows from the next story but before advancing `state.bin`, current reconciliation
cannot distinguish those uncommitted rows from committed rows. Resume restores
the earlier teacher state and appends the same story again.

The text observation driver already carries an authoritative per-shard
checkpoint and trims uncommitted article tails. Raw observation needs the same
transactional boundary without changing teacher arithmetic or observation rows.

# Problem

Raw recovery detects malformed, missing, short, torn, and mutually unaligned
streams, but alignment alone does not prove commitment.

An aligned mid-story tail can therefore survive reconciliation and be followed
by a regenerated copy of the same deterministic rows. This is not silent wire
corruption, but it prevents an exactly-once raw-resume guarantee and can distort
expensive observation or trace campaigns.

# Goal

Make raw `observe` and `observe_sharded_traced` resume converge byte-for-byte to
an uninterrupted run after every supported aligned-tail crash window.

# Transaction model

For each whole-story boundary:

1. Append the story's base, probability, and optional trace rows.
2. Flush and synchronize every affected shard stream.
3. Atomically publish an authoritative checkpoint containing the resulting
   per-shard committed lengths and global teacher state.
4. Publish/repair the existing 25-byte `state.bin` compatibility mirror.
5. If `done`, finalize shard kappa entries and the merged stream idempotently.

On resume:

1. Load and strictly validate the complete checkpoint/payload snapshot under the
   existing `ObservationSession`.
2. Validate every shard and companion before changing any one of them.
3. Truncate all aligned bytes beyond the committed vector.
4. Repair the secondary state mirror if needed.
5. Restore the committed teacher state and regenerate the next story.

The authoritative checkpoint therefore selects either the previous or next
whole-story transaction. Rows beyond it are tentative regardless of alignment.

# Scope

- Add a canonical, versioned, self-identifying authoritative raw checkpoint.
  Prefer a distinct file such as `raw-committed.bin` so the existing text
  `committed.bin` layout remains byte-compatible and the two driver formats
  cannot be confused.
- Record:
  - format magic and schema version;
  - shard fan-out/layout;
  - `n`, `stories`, RNG state, and `done`;
  - the stable observation identity/layout binding;
  - committed base-record bytes or record count for every shard.
- Derive and validate committed `.prob` bytes from the fixed probability-row
  width and committed `.trace` bytes from the manifest-pinned trace-row width.
  Explicit companion lengths are also acceptable, but contradictory redundant
  lengths must be rejected.
- Publish a zero-length initial checkpoint before the first raw shard append, so
  a crash during the first story also has an authoritative rollback boundary.
- Use a unique `create_new` temporary, `sync_all`, atomic same-directory
  replacement, and appropriate directory synchronization. A stale temporary is
  never authoritative; symlink or nonregular checkpoint entries are refused.
- Extend the joint raw preflight used by graph-cli so checkpoint and all shard
  prefixes are validated under the same `ObservationSession` before any
  identity setter, tokenizer export, mirror repair, truncation, append, or
  finalization.
- Use a two-phase recovery: inspect every base/`.prob`/`.trace` entry first,
  then truncate. A bad later shard must not be discovered after an earlier
  shard has already changed.
- Keep recovery idempotent if interruption occurs partway through truncating
  tentative tails.
- Preserve current completed-shard validation, kappa pinning, canonical shard
  order, target-extension refusal, and full-session writer exclusion.
- Update module documentation, `crates/uor-r4-graph-compiler/README.md`, and
  `docs/MODEL_LIFECYCLE.md` with the new guarantee, failure model, and legacy
  policy.

# Non-goals

- No teacher arithmetic, tokenizer, geometry, attention, trace-profile, routing,
  cover, or deployed-runtime change.
- No observation-record, probability-row, or trace-row wire change.
- No change to shard routing, canonical merge ordering, or content-addressed
  sample IDs.
- No automatic adoption of an unfinished legacy raw directory by treating its
  current aligned lengths as committed; that boundary is inherently
  unprovable.
- No unsafe `--force` or best-effort legacy-resume mode.
- No claim of protection from filesystems that violate the documented atomic
  rename/synchronization assumptions.

# Dependencies and overlap

- Builds on merged PR #724, especially its `ObservationSession`, strict entry
  validation, and joint preflight seams.
- Extends closed #59 and #603; it does not reopen their record or trace-profile
  work.
- Preserve #170's deterministic shard/worker invariants.
- #613 is analogous checkpoint work for cover sweeps, not an ownership overlap.
- Independent of the remaining #704 attention/GPT-2 arithmetic migration.
  This issue and its PR must not close #704.

# Acceptance criteria

- [ ] The raw checkpoint has an exact canonical encoding with magic/version,
      exact-length decoding, checked counters, bounded shard count, and
      identity/layout binding.
- [ ] Every per-shard committed base length is record-aligned, companion lengths
      are derivable and checked, and their total record count equals checkpoint
      `n` without overflow.
- [ ] A fresh raw run publishes a zero-length authoritative checkpoint before
      its first shard append.
- [ ] Checkpoint publication follows payload flush/sync and atomically precedes
      the 25-byte `state.bin` mirror.
- [ ] Resume validates every shard and sidecar before mutation; missing,
      nonregular, shorter-than-committed, conflicting, overflowed, malformed, or
      wrong-layout entries fail with a focused error and byte-identical
      payload/provenance state.
- [ ] Aligned base/`.prob`/`.trace` tails longer than the committed prefix are
      truncated to that prefix and regenerated exactly once.
- [ ] Recovery remains correct when different shards or companions contain
      different amounts of tentative aligned data.
- [ ] A missing or corrupt `state.bin` after authoritative checkpoint
      publication is repaired from the checkpoint without rewriting committed
      rows.
- [ ] A crash after the final checkpoint but during shard-manifest
      finalization/merge resumes idempotently.
- [ ] Minimal and traced failure-injection runs converge to the exact shard,
      probability, trace, manifest, state, merged bytes, and kappas of an
      uninterrupted reference run.
- [ ] Fully finalized, kappa-validated legacy raw bundles remain readable and
      replayable without rewriting historical bytes.
- [ ] Unfinished legacy raw outputs without the new authoritative per-shard
      boundary fail closed with migration guidance.
- [ ] Existing text `committed.bin`, `state.bin`, observation records, and
      public reader APIs remain compatible.
- [ ] Checkpoint state is O(number of shards), deterministic, timestamp-free,
      and independent of thread/completion order.

# Verification

Focused tests:

- Canonical checkpoint encode/decode round-trip and negative cases:
  truncated/extra bytes, unknown version, invalid `done`, fan-out mismatch,
  misaligned lengths, counter overflow, total mismatch, identity/layout
  mismatch, symlink/nonregular entries.
- Deterministic failure injection at:
  - first-story append before any commit;
  - base row written before its probability/trace companion;
  - complete aligned rows written to one or several shards before checkpoint;
  - all streams synchronized before checkpoint replacement;
  - checkpoint replaced before `state.bin`;
  - final checkpoint published before or during shard finalization.
- A late invalid shard test proving no earlier tail is truncated during the
  read-only validation phase.
- Minimal and traced reference-vs-resume directory equality tests.
- Legacy matrix: fresh, incomplete legacy refused, completed legacy accepted,
  new checkpoint resume, and mixed raw/text checkpoint formats refused.
- Small-fixture checkpoint overhead measurement; no multi-hour observation run
  is required.

Commands:

- `cargo test -p uor-r4-graph-compiler --offline`
- `cargo test -p uor-r4-graph-cli --lib --offline`
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `cargo fmt --all --check`
- `cargo check -p uor-r4-graph-format --no-default-features --offline`
- `cargo check -p uor-r4-graph-format --no-default-features --features alloc --offline`
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
- `python3 scripts/check_claim_wording.py`

# Compatibility and migration

`state.bin` remains exactly 25 bytes because `state.bin` plus `merged.bin` is an
existing recorded-corpus input contract. The new authoritative checkpoint is a
host-side observation sidecar; it does not alter v4 records, `.prob` rows,
trace rows, manifest identity, merged corpus bytes, teacher outputs, or deployed
artifacts, and it requires no kappa re-pin.

Automatic migration of unfinished legacy raw output is impossible: aligned
bytes do not reveal whether they were covered by the last whole-story
checkpoint. Such directories must restart in a fresh output location.
Fully finalized legacy directories are different: the complete manifest,
record totals, and content kappas prove their final boundaries, so they remain
readable without synthesizing or rewriting a checkpoint.

# Claim status

- Checkpoint layout and transaction ordering: **Definition**.
- Byte-identical convergence after the enumerated crash windows:
  **Guarantee**, initially Proposed and Structural once the failure-injection
  suite passes.
- Atomic replacement and synchronization behavior of the host filesystem:
  **Assumption**.
- Checkpoint latency/throughput overhead: **Empirical Criterion**, reported on
  the small deterministic fixture.

# Ownership and next action

Priority: P1/high before the next multi-hour raw or traced observation
campaign. `Casey-allard` is actively implementing this issue on
`issue-725-transactional-raw-checkpoint`; merge and closure follow only after
the PR binding checks pass.

References #724, #603, #59, and #170. Does not close #704.

## Comments (1)

### comment 5301161796 — 2026-08-15T07:30:34Z — Casey-allard

Delivered by PR #726 at merge commit `cf7b8988e56b4baac5427d213bd7c1a5026769d4`.

Binding merge-group evidence: https://github.com/UOR-Foundation/uor-r4/actions/runs/31871385711 — full nextest workspace suite, BDD, doc tests, clippy/fmt, no_std/alloc ladders, deterministic rebuild, non-vacuous canonical kappa reproduction, Gate C trend, wasm, fuzz, all-features dormant tests, R1-R6 conformance, Kani, and audit passed.

The merged implementation covers the complete #725 contract: canonical `R4RAWCOM` schema-1 checkpointing, zero-boundary publication, durable payload -> checkpoint -> mirror ordering, validate-all-before-mutation recovery, heterogeneous aligned-tail truncation, exact minimal/traced crash convergence, mirror repair, idempotent finalization/merge recovery, mixed-format refusal, and finalized legacy compatibility.

The small four-shard diagnostic recorded a 120-byte authoritative checkpoint plus 25-byte mirror, median 16,612,542 ns and mean 16,473,677 ns across 31 publications, with no portable timing threshold.

Issue #725 is complete and closed; no arithmetic, observation-row, trace-row, deployed-runtime, or kappa fixture change was included.


---
_Comment/review ids in this file: 5301161796_

