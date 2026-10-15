# D19 exact object store: prototype contract

Design only; NOT YET IMPLEMENTED. Codex/Sol, 14 October 2026, base `4cef409fc`.
Authority: [D34](../../integration/DECISIONS.md#d34--revised-attentionmemory-sequence-accepted-reranker-first-relational-read-plus-a-separate-d19-object-store-crate), [PM assignment](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6178356343), and Claude's [independent review §2 and §9–§11](../spiralcore-attention-independent-review-2026-10-14/README.md) at `618e422ce`.

## Crate and consumers

`uor-r4-object-store` is a model-independent Rust library for exact byte storage on D19.
Its first external client is **DeepSeek's on-demand reconstruction-DAG reader**, as coordinated
in [6181045127](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6181045127).
The store exports a narrow `RecordBackend::fetch(NodeId) -> Result<Option<Record>>` interface;
`Record` has private fields and a validating decoder, so per-record hash/schema verification
cannot be asserted by an unchecked backend flag. `Store` is the eager reference backend;
DeepSeek owns the lazy production reader, cache/fetch strategy and its integration. The store's
reference `range/rebuild/missing` methods provide byte-comparison controls, not a competing
production reader. Backend lookup of one verified record never proves whole-object integrity.

The local ingestion/retrieval adapter binds an immutable
object version, pinned tokenizer identity, and occurrence-to-byte-span bindings. Subsequent
integration can store the existing durable-session export bytes and retrieve exact quotations;
`DurableMemoryStore::export_scope/import_scope` remains unchanged in this prototype. No model
crate dependency, tokenizer replacement, attention change or serving adoption follows.

Reuse the retained [Rust range reader](../spiralcore-native-attention-review-2026-10-14/evidence/cs/range_reader.rs)
and its original-Showcase fixtures as compatibility evidence. Check DeepSeek's live ownership
before changing any shared reader code; the new crate owns its implementation and leaves
that evidence intact. Use the existing SHA-256 dependency and shared tokenizer crate rather
than inventing hashing or tokenization variants. The short-node BLAKE2b-96 calculation follows
the verified Showcase reader, with original-record known-answer tests.

## API surface

| Surface | Contract |
|---|---|
| `Store::fold(bytes)` | Ordered 32-byte atoms, zero-padded last atom, canonical ternary frontier; insert immutable records and return a versioned `Manifest`. |
| `Store::rebuild(manifest)` | Reconstruct all ordered occurrences, check canonical padding and whole SHA-256; return `WholeObjectDigestVerified` bytes. |
| `Store::range(manifest, start..end)` | Byte offsets, exclusive end; visit only intersecting branches; authenticate every visited record and expected scale; return `RangeVerifiedAgainstRoot` bytes. |
| `Store::missing(manifest)` | Unique missing IDs reachable from currently available records; descend present records and stop at absent parents. Requery after importing a frontier; never pretend unknown descendants were enumerated. |
| `export_records` / `import_records` | Deterministic binary record exchange; verify IDs/schema and conflicting duplicates before changing the store. |
| manifest encode/decode + digest | Versioned canonical bytes pin length, ordered roots/scales and whole-object SHA-256; callers retain/authenticate this manifest digest separately. |
| `save` / `load` | Explicit file persistence of validated records, bounded input, restart round trip. Manifest and occurrence bindings are separately versioned serialized objects. No cross-process transaction or access-control claim. |
| ingestion / occurrence read | Tokenize once using the shared pinned byte tokenizer; bind each ordered token occurrence to exact object version and byte interval, validate lossless byte reconstruction; reject normalization/unsupported offsets instead of guessing. |

`NodeId` is 12 bytes; `ObjectVersion` is the SHA-256 of the canonical manifest, distinct
from a token ID, occurrence number, tenant ID or semantic/geometric address. An occurrence
binding carries object version, tokenizer identity, occurrence index, token ID and byte span.
Repeated equal chunks/tokens retain distinct ordered occurrences. Byte ranges may split UTF-8;
a text accessor refuses invalid UTF-8 rather than using lossy replacement. Storage parent
hashes, chunk boundaries and dictionary state are not model inputs.

## Compression and exactness

Preserve Showcase record encoding: scale byte, 12-byte BLAKE2b-96 of `scale || payload`,
then 32 bytes for a leaf or three ordered 12-byte child IDs for an internal record (45/49
bytes total). Reject truncated/trailing malformed data, wrong hashes, invalid scales,
noncanonical frontier/length/padding, out-of-bounds ranges and same-ID/different-record
collisions. A child scale must be exactly its parent's scale minus one; this also forbids
cycles. Empty objects and empty ranges have explicit valid encodings.

Deduplication saves repeated content and warm/edit transfer. It is not universal compression:
the retained unique 32,768-byte fixture uses 70,972 record bytes. Report raw bytes, retained
unique record bytes, new record bytes, and visited record bytes separately. Rebuilding always
retains occurrence multiplicity even when storage deduplicates equal nodes.

A partial result authenticates only its visited path against the **caller-pinned manifest**;
it does not freshly check the object's whole SHA-256. Every partial response explicitly
reports `whole_object_digest_verified: false` and the unverified remainder (the complement
of the returned interval); even a full-length call to `range` does not claim a SHA-256 check.
Main-track object verification requires a successful `rebuild` result bound to that same
manifest version; partial success alone cannot promote the object. A full rebuild checks that SHA-256 as
well. These are separate result types, not a boolean callers can misread. The manifest digest
is an identity binding, not a signature or authorization. BLAKE2b-96 is a short compatibility
address with finite collision resistance; conflicting local records are refused, and partial
verification makes no collision-free or hostile-network security claim. Stronger IDs require
a later explicit version, not silent wire-format drift.

Use checked size arithmetic and explicit limits for object length, input record bytes,
retained records, traversal visits and returned bytes. Enforce budgets before allocations or
expansion; deduplicated storage must not permit unbounded reconstruction. Fail with typed
`Missing`, `SchemaMismatch`, `IntegrityFailure`, `InvalidRange`, `LimitExceeded`, tokenizer
binding or I/O errors. No recoverable `panic`, `unwrap`, unsafe code, model operations or GPU
fallback. Failed import does not partially publish records. Failed persistence does not
replace the prior file; concurrent writers are outside this prototype contract.

## Delivery and checks

Merge this note through protected delivery, then implement the library and a byte-only usage
example with focused tests: original Showcase record compatibility; empty/atom/ternary
boundaries; all ranges on small fixtures; repeated content with ordered multiplicity;
missing-path versus unrelated-missing behavior; tampered node/manifest/digest; wrong scale,
cycle, collision and malformed input refusal; explicit allocation/traversal limits; restart
and incremental exchange; UTF-8/normalization refusal; object-version/tokenizer mismatch.
No language result, fresh-transfer result or compression-speed claim is inferred.

The implementation record will state executed tests and measured byte accounting. No model
compute, GPU, rental or training is part of this task. The existing personally released
seed-1 run is untouched; seed-2 and M2 remain HELD. No new FIT or release is requested for
these storage/data-structure checks (Claude §9 item 4). Model integration remains a separate
reviewed task. Milestone headlines and negative counts are unchanged.
