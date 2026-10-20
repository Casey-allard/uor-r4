# D19 exact object-store implementation — 15 October 2026

Implemented and byte-tested prototype; no model capability result. Source/data-structure scope under
D24 and the PM's no-FIT adjudication in comment6181310800. No model/GPU compute, pod or
training; seed-1 untouched and seed-2/M2 HELD. Contract: #2487, source98427106b, with all
five approval nits incorporated here. DeepSeek owns the production DAG reader; the store
implements the compatible validated-record boundary separately.

## Numeric limits fixed before controls

`Limits::default()`: logical object **16 MiB**; retained unique records **1,000,000**;
import/export record bytes **64 MiB**; returned bytes **16 MiB**; traversal/construction
node visits **1,000,000**; token occurrences **1,000,000**. These are configurable resource
ceilings, not scientific acceptance bars. Manifests have a fixed structural maximum of
128 roots; record payloads are fixed 32/36 bytes, scale span arithmetic is checked in u64.
Input length and output/traversal budgets are checked before reserve/expansion; records
are validated before transactional publication; persistence validates/encodes before
creating a temporary file and renames only after writing and syncing it.

Record export order is lexicographically ascending NodeId. Import accepts any order,
rejects malformed/conflicting records, and publishes all-or-nothing. The normative byte
obligation is `rebuild(fold(bytes)).bytes() == bytes`, with canonical padding and a fresh
whole-object SHA-256 check. Sorted export order is distinct from the Showcase adapter's
DFS-post order; tests reconstruct that order separately for its retained golden hashes.

Original `.records`/`.raw`/`.manifest` fixtures are regenerable from the hash-pinned supplied
Showcase ZIP (A2 SHA2566399240e51ad5f5347390690ce45f460eda390bbe1d9c5e86dee0e8fd5c26754)
via the retained `make-fixtures.cjs` adapter. New controls recreate the same bytes in Rust
and compare retained source-output hashes; no Python/Node model or fixture execution is
required. Original evidence and peer work remain unchanged.

## Delivered source and executed evidence

Decision: **KEEP as a source/data-structure prototype; NOT YET PROMOTED into the model.**
The implementation lives in `crates/uor-r4-object-store`. Its only runtime crate
consumers are SHA-256 and the shared byte tokenizer; it imports no model crate.
Two allocation-free tokenizer accessors expose decoded token size and prefix-space
policy without changing tokenization. The adapter refuses unsupported/lossy source
bytes rather than inventing offsets. `max_occurrences` applies to token bindings;
core DAG expansion is bounded by returned bytes and occurrence visits.

Executed locally as authorized byte/data checks:

| Check | Observed result |
|---|---|
| `cargo test -p uor-r4-object-store -p uor-r4-tokenizer --offline` | 20 object-store integration tests and 13 existing tokenizer tests passed; no ignored tests |
| `cargo clippy -p uor-r4-object-store --all-targets --offline -- -D warnings` | Passed |
| `cargo fmt -p uor-r4-object-store -p uor-r4-tokenizer --check` | Passed |
| `cargo run -p uor-r4-object-store --example exact_bytes --offline -- local/example-records` | Exact byte quotation and restart passed; raw 61 bytes, retained/new 90 record bytes, 2 visits |
| Actual #2489 reader source adapter | Compiled and ran whole/range byte comparison, missing-record mapping and distinct integrity claims; passed |

The 20 controls comprise 9 storage/bounds/error controls, 6 binding controls, and
5 original-source compatibility groups. Small fixtures check every byte interval
at atom and carry boundaries; malformed imports, wrong scales, padding, digest
mismatch, substituted valid records, missing/unvisited branches, persistence failures,
conservative allocation limits and repeated-occurrence reconstruction are covered.
There is no constructed short-hash collision fixture or hostile-network security
qualification. Strict scale decrease prevents cyclic traversal; hash validation
and expected-scale refusal are tested rather than claiming an impossible self-hash
fixture was generated.

The original fixtures reproduce these byte accounts exactly (new equals retained
on an empty store, and a repeated fold adds zero record bytes):

| Fixture | Raw bytes | Retained unique record bytes |
|---|---:|---:|
| unique | 32,768 | 70,972 |
| repeated | 7,776 | 380 |
| UTF-8 / NUL | 605 | 421 |
| empty | 0 | 0 |

These are byte counts, not transport time, RAM use, compression throughput or
language scores. Manifests, bindings and tokenizer caches are additional storage.
Whole reconstruction preserves all repeated occurrences. Partial range reads
report visited-occurrence wire bytes and distinct record count separately; those
are not network fetch/cache measurements.

## Reader seam and ownership

[Proposal6182765160](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6182765160)
was [provisionally ratified by PM6182842815](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6182842815).
DeepSeek is offline per PM6182762104. Implementation may proceed; the interface is
**provisional until DeepSeek reviews it on return**, not declared mutually stable.
No mutation of #2489, its worktree or its uncommitted corrections occurred.

`RecordBackend::fetch(&self, NodeId) -> Result<Option<Record>>` returns a private-field
validated record. The adapter maps absence to reader `Error::Missing`, checks the
returned ID, converts to raw `Node { scale, data }`, and supplies `record_count()`
for `Backend::stored()`. The reader keeps its independent hash/scale/padding checks.
The canonical manifest exposes length, ordered roots/scales and whole SHA-256;
its UOR framing is not confused with the reader's experimental manifest format.

[Adapter source](evidence/dag_adapter.rs) was compiled against **published** reader
`b32651b82c2ef3075ed49c472e0d7d432b008946`, whose `src/lib.rs` SHA-256 is
`2bbfc9fc05df3adb7a1f4b4aff2fe378ee71ab7d6fff947c29bfc40dc650f696`.
The adapter SHA-256 is
`fe434f6351f0c5a63af3d9dceb24a1e4661afd3f250fd7a9c4323b0004e2548b`.
This checks the actual trait, not a handwritten stand-in. It does not qualify all
reader failure paths or supersede its outstanding review. The isolated check's
Cargo lock and log are retained here; the reader source remains owned by #2489.

To reproduce, extract that exact Git source to an isolated local crate named
`uor-r4-recon-dag`, edition2021, dependency `sha2 = "0.10"`, with its own empty
`[workspace]`. Create an isolated driver package using `evidence/dag_adapter.rs`
as `src/main.rs`, with path dependencies on this object-store crate and that
reader crate. Use the retained `evidence/adapter-Cargo.lock` as its Cargo.lock;
run `CARGO_TARGET_DIR=local/object-store-target cargo run --offline --locked
--manifest-path <driver>/Cargo.toml`. No model or GPU is involved.

## Boundaries and next consumer

Manifest digest and token occurrence metadata require caller authentication and
access control. Partial reads cannot freshly verify absent bytes or the whole
SHA-256; even a full-length `range` remains a partial-verification type until
`rebuild` verifies the digest. Bindings preserve exact object/tokenizer/occurrence
identity, not semantic/geometric addresses. Imported bindings require separately
trusted provenance; a selected token-byte agreement does not prove canonical full
source tokenization. Tokenizer construction and its persistent merge cache remain
caller-owned, outside per-operation store allocation limits.

No `DurableMemoryStore` caller, model head, shared geometric writer, training
schedule or serving authority changes. STATUS/ROADMAP and milestone headlines
remain unchanged. D19 infrastructure line count0/3 unchanged; no language result.
Next: protected implementation review and delivery, DeepSeek's seam confirmation
on return, then separately scoped durable-session integration. Seed-2 and M2
remain HELD; only the already personally released seed-1 run proceeds untouched.
