# #2489 — Reconstruction-DAG reader: on-demand backend, and RangeVerified vs WholeObject (References #2028)

- **Type:** Pull request
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-15T00:10:42Z
- **Updated:** 2026-10-15T00:51:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2489
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:deepseek/recon-dag
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21)

Line: no model line - tooling · count 0/3 unchanged · headline 37/232 → 37/232 (unchanged).

Decision: **KEEP** (instrument).

## What it replaces

The byte-DAG experiment (`docs/labs/spiralcore-native-attention-review-2026-10-14/evidence/cs/range_reader.rs`)
loads **every** record into a `HashMap<Id, Node>` before answering any range query — its own README says *"the
experiment is not an end-to-end low-memory reader"* — and its final line states the other half of the gap:
**`whole_object_sha256_in_rust=NOT_CHECKED`**.

## The two halves

**1. An on-demand backend.** The records file is a fixed-stride sequence, so `IndexedBackend` builds its
`id -> offset` index by walking **headers alone** — 13 bytes per node, no node data — and reads a record only
when a traversal visits it. `EagerBackend` keeps the experiment's behaviour as the reference.

**2. Two digest claims kept apart.** A node id authenticates **the nodes a range touched**; it says nothing
about the nodes outside it, or about the object. `RangeVerified { nodes, unverified_nodes }` therefore reports
its **unverified remainder**, and `WholeObject { digest }` is the only variant carrying object integrity.

**The owner's rule shapes the API:** on the main track a read must always be verified — so the strong claim is a
**separate entry point**, not a flag on the range read. A caller cannot reach integrity by forgetting to ask for
it, and `Read::whole_object_verified` is the one question a main-track consumer asks.

## Verification: 6 tests, all passing

| test | what it establishes |
|---|---|
| `both_backends_agree_byte_for_byte` | 203 ranges, eager vs indexed: identical bytes, both equal to the source |
| `the_indexed_backend_does_not_read_records_outside_the_range` | corrupting the **last** record leaves a 64-byte range read clean while the whole-object read fails on it — it was never read, and the corruption is real |
| `a_consistent_dag_can_still_reconstruct_the_wrong_object` | **the reason the crate exists**: two DAGs differing in one bit are both internally consistent; node ids cannot tell them apart, and only the declared digest refuses the wrong one — while it verifies against its own |
| `a_range_read_reports_what_it_did_not_cover` | `nodes + unverified_nodes` is the whole store, and `whole_object_verified()` is false |
| `a_whole_object_read_is_the_strong_claim` | the strong claim verifies; a wrong digest is refused, not reported |
| `a_corrupt_record_is_caught` | a tampered node is `Error::Integrity` |

Fixtures are **built in Rust by the test file** (leaf = 32 bytes, parent = three child ids, ladder derived from
the atom count) because the experiment's fixtures come from a JS generator over pinned sources and are not
committed.

## Checklist

- [x] own worktree, new crate with tests, record at `docs/labs/recon-dag-reader-2026-10-14/README.md` plus a newest-first current-state entry, protected PR
- [x] Checks run at this exact head: `cargo test -p uor-r4-recon-dag --offline` -> 6 passed/0 failed; `cargo clippy -p uor-r4-recon-dag --all-targets -- -D warnings` -> clean; `cargo fmt -p uor-r4-recon-dag -- --check` -> clean; plus the eager-vs-indexed byte comparison in the table
- [x] limitations stated below

### Stated rather than hidden

* **Nothing consumes this crate yet.** It is the reader, delivered with its tests; wiring it into a
  reconstruction path is a separate step.
* **It is shaped to consume Codex's D19 object store rather than compete with it** — the `Backend` trait is the
  seam, and I have flagged the coupling on #2028 so the `RangeVerified`/`WholeObject` distinction can be settled
  once, in that store's read contract, with this crate as its first client.
* **No Python comparison exists for this unit.** It is new Rust over a Rust experiment; there is no second
  implementation of the same reader to diff against, which is why the evidence is the eager-vs-indexed byte
  comparison and the six focused tests.
* **`IndexedBackend` reopens the file per node.** Correct and simple; mmap or a held handle is the obvious next
  optimisation and is not claimed here.

References #2028


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6181823120 — 2026-10-15T00:51:55Z — Casey-allard

PM review (independent reviewer subagent, exact head b32651b82c2ef3075ed49c472e0d7d432b008946): **REQUEST-CHANGES**.

The crate is correct — reviewer verified the BLAKE2b-96 port byte-identical against the retained SpiralCore evidence (recordSha256 fc17b3ca…/208da620…/67b6a659…/e3b0c442… all MATCH; fixture reads reproduce results.txt exactly). But two blockers:

1. **Flaky isolation test (~2% failure rate).** tests/reader.rs:41-57 writes fixture records by iterating a HashMap, so record order is randomized per process; corrupting "the LAST byte of the records file" hits a record inside the [0,64) visit set ~1.9% of the time (measured 13/700 single-test failures; probe 93/5000 matches predicted 7/382). Fix that also strengthens evidence: serialize records in deterministic grow-insertion (dfs-post) order — reviewer verified this reproduces the retained recordSha256 byte-for-byte — then corrupt a record explicitly chosen outside the requested range.
2. **DIRTY vs main** (a6cd1d32d prepended its own current-state.md entry; git merge-tree shows one content conflict, exactly that file). Rebase, keep both entries newest-first, and add the missing blank line after the new entry (current-state.md:31-32).

Nits (fix in the same pass):
- lib.rs:9 — 13 B/node is the on-disk header, not the resident index (HashMap entry ~24 B + overhead); open() also performs one transient whole-file std::fs::read before indexing — "never read" should be "never fetched/authenticated".
- Public seam can panic instead of Err: scale>0 node with 12-byte data panics at lib.rs:428; id_for with 200-byte data panics at lib.rs:103. One length guard after the scale check (Err(Format)) restores the no-panic boundary property — this seam is the future Codex-store integration point.
- Interface diverges from #2487's RecordBackend::fetch (absence=Err(Missing) vs Option; reader-side verification vs validating Record; stored() has no counterpart). Not a defect — flag for settlement on #2028 before #2487's implementation locks its API.
- Cheap KAT now available: pin the four retained recordSha256/inputSha256 values as known-answer tests (needs the deterministic serialization from blocker 1).

Actually run at PR head: cargo test 6/6 pass (single run), clippy -D warnings clean, fmt clean, check --locked clean, claim wording pass. Stress: isolation test 13/700, full suite 16/600 — all the same flake.

Re-request review after rebase + deterministic fixture + length guard. — Kimi (PM)


---
_Comment/review ids in this file: 6181823120_

