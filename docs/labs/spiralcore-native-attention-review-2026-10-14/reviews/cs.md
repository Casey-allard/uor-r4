# Computer science and data structures assessment

Reviewed source: UOR-R4 `88c413599`, supplied Showcase `core.js` / `transport.js`, and the mathematics, network, and ML specialist memos. This is the CS/algorithms role within the same Codex swarm, not an independent external-lab verdict. No model, attention/reference, tokenizer, training, pod, or paid execution occurred. A new byte-only reproduction is reported below.

## Constructive finding

The architectural idea is coherent and implementable: a short reconstruction descriptor can name an exact object held in a shared content DAG, a read can select particular objects or subranges using compact metadata, and the endpoint can reconstruct only the selected values. The descriptor can be the unit of attention without making its hash a semantic metric. Exact bytes, occurrence identity, learned geometry, and the chosen transformation are complementary fields of that unit.

In particular, the supplied Showcase's grammar already supports selective byte reconstruction. We independently built and ran a small Rust reader over its actual serialized nodes. This is stronger evidence for the user's proposed bridge than merely saying a full decoder exists. It still does not establish that a proposed relevance law is good: that is the remaining learned-model question, not an objection to the data representation.

The simplest compatible design has three separate stores:

1. An immutable exact object DAG, storing literal bytes and ordered concatenations by content identity.
2. An occurrence structure, recording document/version, positions, token spans, and repeated appearances of the same content.
3. Versioned learned annotations and query indexes, using the existing SpiralCore actions/root states and the actual model/BinderPlan identity.

This permits deduplicated storage without erasing recency, causality, or evidence provenance. Geometry may change during learning while the source bytes and their identity remain immutable. A new synchronization read and today's Flock can consume the same descriptor interface.

## What can run directly on instructions?

For the Showcase's ternary node of scale s, the represented padded span is exactly `32 * 3^s` bytes. Its three child IDs are ordered (`core.js:166–191`), so their offsets are derived without inspecting child bytes. A range query follows only intersecting branches and copies requested leaf portions. The frontier's scales derive from the base-three expansion of the atom count (`161–164`). This is an exact random-access representation; the root is not required to contain all source bytes itself.

The Rust reproduction retained actual source-generated node IDs and record framing. Across 813 intervals in four synthetic fixtures it reconstructed the requested bytes exactly. For a 23-byte range inside 32,768 unique bytes, it traversed eight authenticated records, including two leaf occurrences. Those records occupy 384 serialized bytes versus 70,972 bytes for all 1,532 records. These are selected-record counts after construction, not a measured whole-pipeline speedup. The JavaScript source built the entire store, and the Rust harness loaded it into memory. The successful result establishes selective dependency access; an on-demand disk/network backend remains unimplemented.

The repeated fixture is particularly relevant to the user's “sheet music” description: 7,776 bytes were represented by eight distinct records occupying 380 serialized bytes, while reconstruction still produced all 243 ordered leaf occurrences. Instructions name reuse and concatenation; their expanded meaning can be much larger than the stored instruction graph. The manifest and framing are extra. A second occurrence is a real occurrence even when no additional content bytes need storage.

Other exact operations can live on cached compositional summaries: byte/token counts, declared finite-state parser transitions, exact per-frame candidate lists, structural properties, or explicitly represented transforms. Their correctness condition is the math specialist's factorization criterion: objects represented by the same summary must give the same answer for every query covered by the claim. A lossy summary can be perfectly sufficient for a limited query family; it need not recover everything. Full reconstruction remains the fallback when an operation lacks a more efficient sufficient summary.

An instruction such as `Map(action, node)` can lazily compose exact geometric actions until selected payloads are needed, provided the decoder semantics specify which payload space the action acts on. It must not silently change the original byte object's identity or treat coordinate motion as a byte transformation without a matching decoder. The exact byte object, transformed carrier, and reconstruction-program identity are distinct typed values.

## Tokenization is an interface to preserve or intentionally change

The current discrete corpus loader consumes pretokenized UORT data: a 64-byte header followed by two bytes per token, plus a separately pinned response mask (`corpus.rs:143–186`). It verifies tokenizer/BOS and corpus lineage, but does not itself expose raw-byte random-access tokenization. There is therefore no existing automatically interchangeable “compressed source token” input path to invoke.

The universally compatible transport route is straightforward: compress original bytes into the exact DAG, transport the descriptor plus missing dependencies, reconstruct those same bytes, and invoke the same pinned tokenizer. This preserves its input exactly. It can save transfer/storage without changing model behavior; it does not by itself reduce tokenization or attention work.

To avoid reconstructing everything at every read, perform tokenization once at ingestion and retain a pinned token sequence with occurrence-to-byte-span mapping alongside the exact byte DAG. At query time the native reader consumes existing token/carrier descriptors and reconstructs only selected raw content when the application needs it. This shares the existing model alphabet and writer. Cost includes initial tokenization and mappings, and these annotations must be keyed to the precise tokenizer and normalization configuration.

A stronger incremental tokenizer can consume selected subranges only if it retains the real tokenizer's necessary incoming state and boundary conditions. A bounded overlap is valid only if a proved bound applies to that tokenizer; arbitrary fixed 32-byte cells are not token boundaries. Our byte reproduction makes the distinction concrete: the four bytes for `🌀` straddle a 32-byte cell boundary and reconstruct correctly together, while a range of the interior two bytes is byte-exact but invalid UTF-8. UTF-8 boundary handling is only the first layer; lexical normalization and subword merges may have additional context requirements. We did not execute or characterize the actual tokenizer here.

Learning directly on opcodes such as literal/reference/concatenate/transform/repeat is a second legitimate route. It is a new model representation, since the model sequence then depends on the codec and instruction grammar. Two equivalent reconstruction programs may decode to the same text but have different structures. Choose a canonical program form, or explicitly specify which equivalent encodings the learned read must treat alike. No such invariance follows merely from exact decoding. This is a fair experiment to design, not an accidental consequence of inserting a compressor before the current tokenizer.

## Review of the exact sixteen-frame Flock result

The mathematics specialist's `16K` argument is correct for the current source's selection law. For a fixed block, level, causal cut, BinderPlan, and effective model state, distance and coordinate Hamming depend only on one of sixteen frame roots. Within each frame the remaining order is recency descending then node ID ascending (`discrete.cu:165–181`). Discarding candidates beyond the first K in one frame cannot remove a global top-K result: K candidates of the same frame precede each discarded candidate under every query frame. Per-frame truncation is mergeable over unique occurrence IDs.

The result preserves the current selector exactly; it does not select a better attention law. It is not automatically a sufficient candidate set for a new relational head. Such a head may make an older excluded occurrence relevant through a relation absent from the Flock comparator. If a new head uses the old shortlist, that is an explicit admission restriction, not equivalence to reading all eligible nodes.

There is a useful additional source-specific separation: each parent's frame is its first child's frame (`discrete.cu:144`), recursively its leftmost leaf's frame. Building the frame-selection index does not require computing every parent's transported/snapped payload. After selection, materialize only the selected payload nodes using their real children and actions. This makes the descriptor idea actionable even before inventing a new attention mechanism.

However, a selected high-level payload can depend on nearly the whole eligible context. Cached summaries may help repeated queries, but a small output or shortlist is not a proof that constructing it is always cheap. At present C192 the full BinderPlan has `(3*192 - sum_digits_base3(192))/2 = (576-4)/2 = 286` nodes; a maximum `16K = 128` shortlist at K8 is not necessarily smaller than every individual level. Benefits depend on the actual candidate counts, reuse, memory costs, and model updates. The existing writer cost also remains.

The critical cache identity is the **effective** model state, not just its committed epoch. Current proposals overlay a token/block action without committing an epoch (`discrete.cu:24–33`). For head b, frame block is `B-1` if b=0, otherwise 0 (`106–107`). A proposed change to block b alone can preserve that head's frame selection index while invalidating its payloads. A change to the frame block may alter query geometry, frame classes of many repeated occurrences and ancestor nodes, transport actions, and payload summaries. Changes to neither block do not affect that head. Gate-only proposals preserve selection and candidate payloads and change the final combination. These dependencies provide a route to exact selective invalidation; GPU parity must verify it before use.

Bind cached indexes to the document/version, window start/end and BinderPlan, causal cut, geometry/action schema, parameter epoch plus virtual overlay, block, level, K and deterministic occurrence tie convention. A cache built after a later causal cut cannot in general answer an earlier cut by filtering a truncated shortlist: required earlier candidates may have been discarded. Use a persistent prefix index or the exact cut identity. Sliding a window can regroup the ternary tree, invalidating a summary even when most source text is unchanged.

## Persistence, missing state and authenticity

Content IDs can deduplicate the object store, but a sequence of child **references** retains order and multiplicity. Attention occurrence identity must additionally include a root/version and ordered path or equivalent canonical position. Different model epochs cannot share a learned summary merely because they share immutable bytes. A decoder/operator schema change also changes the interpretation of a descriptor.

A successful partial read is not a whole-object availability certificate. Our reproduction removed a leaf outside the queried interval: the interval still reconstructed exactly, while full reconstruction refused the missing leaf. This is desirable demand loading, provided the application labels the scope correctly. Removing a required leaf, changing its scale, or corrupting its bytes caused typed refusal. Missing state should trigger bounded fetch/retry or an explicit unavailable result, never an invented zero payload or negative model-quality observation.

For authenticity, a partial reader checks each visited node against the hash-linked path under a trusted root/manifest and the declared content-hash assumptions. It cannot freshly verify a whole-file SHA-256 without reading every byte. The original Showcase's full `rebuild` does that check at `core.js:247`; our range reader deliberately does not make that claim. A production API should distinguish `RangeVerifiedAgainstRoot` from `WholeObjectDigestVerified` and from `WholeObjectLocallyAvailable`.

The current byte tree uses fixed 32-byte cells, so insertion can change chunk alignment and many content IDs. Stable subtree reuse after such updates is a separate choice: retain the current format and account for its edits, or design a versioned rope/content-defined boundary scheme with new compatibility tests. No edit policy should rewrite a published immutable root in place. Garbage collection must retain roots used by checkpoints, active requests and provenance; model annotations can expire independently.

## Minimal implementation contract

The first compatible interface can be narrow:

```text
ObjectRef = codec/schema + exact root(s) + byte length + digest identity
OccurrenceRef = ObjectRef + document/version + ordered path/span + causal position
GeometryRef = occurrence + model/checkpoint + overlay + block/frame + root/action labels
RangeRequest = ObjectRef + [start,end) + integrity scope + byte/node budget
RangeReply = exact bytes OR MissingDependencies / SchemaMismatch / IntegrityFailure
ReadRequest = query carrier + exact causal BinderPlan + effective model state + read law
ReadReply = ordered occurrences + payload references + actual transport actions
```

Keep raw byte retrieval separate from GPU model arithmetic. Packed root labels can remain on the GPU through actual action/distance tables. Candidate selection can use a source-proved sufficient index; selected payloads use existing exact transport/combination and the same writer. This initial path changes representation and access strategy while preserving the model law. It has a crisp exact parity target.

A finite relational synchronization head is a different selectable read law. For a first bounded design, all eligible descriptors in the existing C192 BinderPlan are a comprehensible domain. Sparse edge generation must itself be bounded: direct parent/child links and declared per-frame links are computable without scanning every pair; claiming a sparse final graph after a dense pair search would hide cost. Preserve input payloads and make the changed selection/interaction explicit. A strict-descent law guarantees a property of the specified finite updates, not semantic relevance or good optimization.

## Complete cost and next qualification steps

Account for ingestion (source scan, hashing, compression, tokenization, maps), persisted bytes (object records, occurrence structure, model annotations, indexes), per-query work (candidate inspection, dependency fetches, selected payload reconstruction, transport, writer), and learning work (counterfactual overlays, invalidations/rebuilds, repeated proposals). Add cold misses, persistence and version maintenance. The local byte proof establishes a dependency reduction for a specific range, not a GPU speed/energy saving.

The smallest later GPU qualification for the unchanged-law implementation should compare the same pinned states/inputs and exact traces across original and compact reads: selected occurrence IDs, actions, payload roots, output roots, writer admission/ranking, and proposal acceptance. Include repeated content, ties, partial roots, causal cuts, sliding-window regrouping, frame-block versus payload-block overlays, gate-only proposals, and stale/missing state. Exact agreement is meaningful here because unchanged behavior is the claim. GPU model tests remain NOT_RUN and need the existing FIT/owner gates.

Only after that should actual-shape cost measurements decide whether packing/indexing is worthwhile. A new relational head has its own construction controls and trained-in, grounded quality assessment; it is neither validated nor invalidated by the exact Flock acceleration result. Selective reconstruction and the shared descriptor contract can support both lines without making either a prerequisite for the other.

## Discussion with the systems integrator

I supplied the first-leaf frame dependency and block-specific invalidation argument. The integrator challenged whether `16K` could silently constrain the new head; we agreed it is sufficient only for current Flock and must not be generalized. We also agreed that identical decoded text may have different reconstruction graphs, so canonicalization or declared encoding invariance is necessary for claims that the read depends only on content. The bounded C192 all-eligible candidate graph is a concrete first relational domain, with 286 retained nodes before causal eligibility filtering. Training remains held.

Evidence is in `local/swarm/cs-evidence/README.md`, source adapter, Rust reader, fixtures, `construction.json`, and `results.txt`. The first compile's missing edition flag is recorded there; the corrected build and byte experiment passed. No model-quality conclusion follows.
