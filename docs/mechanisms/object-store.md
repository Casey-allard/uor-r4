# Exact byte object store

**Status: READY.** It is built and byte-tested: 60 object-store tests pass, and a second-language client reads from it correctly. No model uses it yet, and the durable-memory acceptance bars for milestone M3 (remembering across sessions; see the [milestones](../start/glossary.md)) have not been run.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

This is storage, not thinking. It keeps data exactly, so a chat memory can be saved, checked and copied between machines without any byte changing. The data is cut into small pieces. Each piece is fingerprinted, and the pieces are arranged in a tree. Repeated content is stored once. Any byte range can be read back and checked on its own. When a file changes a little, only the changed pieces need to travel. It does not shrink new text; unique text takes about twice as much room. It exists so that a future model has something exact to quote from.

## What it replaces in a transformer

Nothing. A transformer has no part like it. It sits beneath the model as durable memory outside it. The nearest transformer idea is a key-value cache that survives a restart, but this store holds raw source bytes with no keys, no facts and no addresses. [Exact memory](./exact-memory.md) is the part that would address facts. Its node ids are content fingerprints and are never geometric addresses.

![A document drawn as a row of 32-byte tiles folding into a three-child tree with a root fingerprint on top. A highlighted 23-byte range lights only the branches that cover it: 8 verified nodes, 384 bytes. Beside it, two versions of a file with shared subtrees greyed out and only the changed records travelling. Below, a bar chart of stored sizes for four test files.](../figures/mechanisms/object-store.svg)

*Left, a small read touches only a few verified nodes. Middle, an edit sends only the changed records. Right, repeats shrink and unique text grows.*

## How it works

**Cutting and folding (proof of the format, measurement of the sizes).** `fold(bytes)` splits the data into ordered 32-byte atoms. The last atom is zero-padded. A 32,768-byte object is therefore 1,024 atoms. The atoms become the leaves of a canonical ternary (three-child) tree of records.

A record holds a scale byte and a 12-byte fingerprint (BLAKE2b-96, a hash) of the scale and payload. After that comes either a 32-byte leaf or three ordered 12-byte child ids. A leaf record is 45 bytes and an internal record is 49. A child's scale must equal its parent's scale minus one, which forbids cycles. Equal chunks hash to the same record, so repeated content is stored once while every ordered occurrence is kept.

**Manifest and version.** A manifest pins the length, the ordered roots and a whole-object SHA-256. The object version is the SHA-256 of the canonical manifest. `rebuild` checks the padding and the whole-object SHA-256, so the byte obligation is

$$\mathrm{rebuild}(\mathrm{fold}(b)) = b$$

for every input $b$ within the limits (a tested obligation, with 20 integration controls at delivery; not a machine-checked proof).

**Range reads.** `range(start..end)` descends only the branches that intersect the range. It authenticates each visited record against the root. A 23-byte read of a 32,768-byte object touched 2 leaves and needed 8 authenticated nodes, 384 serialized bytes. A missing, unrelated leaf did not block that range. It did block a full rebuild.

**Token bindings.** Occurrence bindings tie every token occurrence of a pinned tokenizer to an object version and a byte span. They are versioned separately from the object, so exact token, occurrence and version identity never merges with a geometric address. The adapter refuses lossy or normalised sources instead of guessing offsets.

**Peers.** A receiver asks only for node ids it is missing, verifies each digest, and commits all at once or not at all. The wire format is specified byte by byte in [`PROTOCOL.md`](../../crates/uor-r4-object-store/PROTOCOL.md).

**What it does not do.** Deduplication is not general compression. Unique 32,768-byte content takes 70,972 record bytes, which is $70{,}972/32{,}768\approx 2.17$ times the raw size (measurement). Pins prove integrity, not who sent the data. No collision fixture was built for the 12-byte node id.

**Rule check.** There is no model, no learning and no geometry here. The source has no softmax, sigmoid or tanh (searched), no matrix multiplication and no floating point apart from a test timer. The hash primitives are integer. No instruction-level audit against the zero-multiplier serving rule (D11) has been run. It adds nothing to the discrete training path (D32).

## What has been tried

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 14 Oct | Review of six outside SpiralCore documents, with their own test suites | Supplied suites pass (6, 4 and 10 groups). One MiB cold exchange 62,579 bytes against 438 bytes warm. The proposed attention head is unproven | positive | [record](../../archive/labs/spiralcore-attention-assessment-2026-10-14/README.md) |
| 14 Oct | Selective range read and repeated against unique fixtures | 23-byte range of a 32,768-byte object: 8 nodes, 384 bytes. Repeated 7,776-byte object: 8 unique records (380 bytes) rebuild 243 ordered leaves | positive | [record](../../archive/labs/spiralcore-native-attention-review-2026-10-14/README.md) |
| 14 Oct | Independent adversarial review (basis for decision D34) | Separates the store from attention. Benefit is warm and edit reuse. Unique content expands 2.17 times, so it is its own crate on the durable-memory line | inconclusive | [record](../../archive/labs/spiralcore-attention-independent-review-2026-10-14/README.md) |
| 14 Oct | On-demand reader for the reconstruction tree, two backends | 203 ranges identical and equal to the source. 13 reader tests pass. It separates "the nodes I touched verified" from "the whole object verified". Nothing consumes it yet | positive | [record](../../archive/labs/recon-dag-reader-2026-10-14/README.md) |
| 15 Oct | The crate, with fixed limits (16 MiB object, 1,000,000 records) and malformed-import controls | Kept as a source prototype, not promoted into the model. 20 object-store and 13 tokenizer tests pass; 22 controls after the review fixes. Stored sizes: unique 32,768 to 70,972; repeated 7,776 to 380; UTF-8-and-NUL text 605 to 421; empty 0 to 0 | positive | [record](../../archive/labs/d19-object-store-2026-10-15/README.md) |
| 17 Oct | Peer replication; a receiver holding an older version | 1,048,576-byte object, receiver 16 atoms behind: 22 records, 1,014 bytes, 3 rounds. 16 scattered edits add 131 records. A wrong pinned identity is rejected with 0 records imported | positive | [record](../../archive/labs/object-store-peer-2026-10-17/README.md), [hardening](../../archive/labs/object-store-peer-hardening-2026-10-17/README.md) |
| 17 Oct | 12 warm receivers pulling at once against one by one | 0.864 s concurrent against 3.506 s sequential | positive | [record](../../archive/labs/object-store-peer-concurrency-2026-10-17/README.md) |
| 17 Oct | Opt-in larger fetch batches on a 52,535,088-byte random artifact over a wide-area link | 236.19 s default against 91.50 s with a large batch (2.58 times). 143,659,164 wire bytes either way. Locally the large batch was slower: 37.4 s against 18.8 s | positive | [record](../../archive/labs/object-store-batch-2026-10-17/README.md) |
| 17 Oct | Cold join: compressed download, local fold, then warm deltas | A cold store pull is a documented loss: 5,725,408 bytes against 39,920 compressed for a 2 MiB artifact. The supported recipe avoids it. 57 package tests pass | positive | [record](../../archive/labs/object-store-join-2026-10-17/README.md) |
| 17 Oct | Fleet tools: pipelined verification, multi-replica audit, failover, repair | Each is covered by focused tests (14 of 14 verify tests). The after-change cross-machine timing was not recorded | inconclusive | [record](../../archive/labs/object-store-verify-pipeline-2026-10-17/README.md) |
| 18 Oct | Pipelined fetch with an emulated 8 ms delay | 3.706 s with one request in flight against 1.324 s with eight (2.8 times). Not a real wide-area measurement | positive | [record](../labs/object-store-pipelined-fetch-2026-10-18/README.md) |
| 18 Oct | A pure-Python client audits and pulls from the Rust server over loopback | 262,144 bytes pulled byte-identical. One check, one replica: cold audit stale (exit 3), then after the pull verified (exit 0). Client side only; loopback only | positive | [record](../labs/uorobj01-conformance-2026-10-18/README.md) |
| 18 Oct | Fleet demo: audit, failover and repair across two loopback replicas with the Rust CLI | Initial audit: 4 checks, 0 verified, 4 stale, 3 unknown (exit 2); failover sync wrote 3 artifacts; repair fetched the missing one. 60 object-store tests passed, 0 failed (from the [batch-plumbing record](../labs/object-store-batch-plumbing-2026-10-18/README.md)) | positive | [record](../labs/object-store-fleet-maintenance-2026-10-18/README.md) |

**Reading these honestly.** Everything here is a test of storage and transport, not of a model. The sizes and timings are real measurements, but on small fixtures, synthetic random data and loopback links. The two recorded losses are real costs of unique content, not defects: cold pulls of unique data, and the 2.17 times expansion. Early defects (a batch cap that stalled a cold 1 MiB pull, a repeated-id request bug, a `--batch` flag ignored by sync and repair) were implementation faults and are fixed.

## What has never been tested

- Any model reading from the store, whether exact quotation feeding the emitter or durable-session export and import. `export_scope` and `import_scope` are unchanged and unwired.
- The third milestone's durable-memory bars on a chosen model: provenance, correction, "I do not know" for unstated or retracted facts, explicit reset, restart, and token-for-token identical restore.
- A real wide-area measurement of pipelined fetch and of the verification pipeline.
- A hostile network. Pins give integrity, not authentication.
- Occurrence bindings against the real tokenizer at corpus scale.
- Storage cost on real chat data against an ordinary compressor.

## What would make it ACTIVE

1. **Name the first model consumer**: exact quotation into the emitter, or durable-session persistence through the store.
2. **Pre-register the matching durable-memory bar** and run it on a selected model.
3. **Decide where exact identity meets geometry.** Exact token, occurrence and version identity must stay separate from geometric addresses.
4. **Decide who owns range reads**: the store, or the separate reconstruction-tree reader, or one shared record backend.

## Also known as

`uor-r4-object-store`; the D19 durable-memory store; exact ternary byte-object store; `UOROBJ01` and `objpeer` replication; Showcase-compatible record tree; reconstruction-DAG store.

## Code

- [`store.rs`](../../crates/uor-r4-object-store/src/store.rs): fold, rebuild, range, missing records, import and export, save and load, resource limits.
- [`record.rs`](../../crates/uor-r4-object-store/src/record.rs) and [`manifest.rs`](../../crates/uor-r4-object-store/src/manifest.rs): the validated record encoding and the versioned manifest.
- [`bindings.rs`](../../crates/uor-r4-object-store/src/bindings.rs): pinned-tokenizer ingestion and the separately versioned occurrence bindings.
- [`peer.rs`](../../crates/uor-r4-object-store/src/peer.rs) and [`bin/objpeer.rs`](../../crates/uor-r4-object-store/src/bin/objpeer.rs): peer replication and its command-line tool (serve, pull, sync, verify, repair, join).
- [`PROTOCOL.md`](../../crates/uor-r4-object-store/PROTOCOL.md): the byte-accurate wire spec for ports to other runtimes. Scripts: [`cold_join.sh`](../../crates/uor-r4-object-store/tools/cold_join.sh), [`fleet_maintenance.sh`](../../crates/uor-r4-object-store/tools/fleet_maintenance.sh).
- [`uor-r4-recon-dag`](../../crates/uor-r4-recon-dag/src/lib.rs): the separate on-demand reader, with range-verified against whole-object claims.
- Design: [`d19-object-store-design-2026-10-14`](../../archive/labs/d19-object-store-design-2026-10-14/README.md).

## Related mechanisms

[exact-memory](./exact-memory.md): stores facts under exact addresses inside the model path; this store holds the source bytes beneath it. [relation-read](./relation-read.md): decision D34 chose the reranker for attention and this store for exact compressed reads, as separate work. [spiralcore-fbs](./spiralcore-fbs.md): shares the ternary tree idea and the Showcase format, with no learning. [flock-read](./flock-read.md): admits a small exact index; the store's range reads were reviewed alongside it. [tokenizer](./tokenizer.md): the pinned byte-pair tokenizer binds token occurrences to byte spans. [lut-serving](./lut-serving.md): the restore is specified on the served path; the design note says session export bytes could be stored here, which is not yet done.
