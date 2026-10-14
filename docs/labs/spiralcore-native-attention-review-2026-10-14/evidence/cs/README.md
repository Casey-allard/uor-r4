# Byte DAG selective reconstruction experiment

This experiment addresses one concrete question: can actual SpiralCore Showcase reconstruction instructions retrieve only a selected byte interval without first reconstructing the whole object? It runs **no model, attention, tokenizer, learned reference, training, or GPU work**.

`make-fixtures.cjs` is an adapter to the original source's `fold` and `recordsFor` functions. The latter's serialized node records are retained unchanged: one scale byte, twelve ID bytes, and 32 leaf bytes or 36 ordered child-ID bytes. The tiny whitespace `.manifest` files are explicitly an experiment adapter, not the Showcase's actual manifest/wire format. `.raw` files are synthetic byte oracles. Before loading either source module, the adapter pins both original SHA-256 values: `core.js` is `7ca3dcfb552ca6ff3fc8696734265bcd9b993aa8b8e6242984060ace8b35bc22`, and `transport.js` is `1d254a75dfe497f91b302ada0d581eab3407c7ea8f3994920a58d9a2652dc0fe`.

`range_reader.rs` is a standalone dependency-free Rust byte reader. It implements the node's single-block BLAKE2b-96 calculation independently and checks the original source-generated IDs. For a requested half-open byte range, it derives subtree spans from scale, visits only overlapping ordered child paths, authenticates visited records, and emits the requested bytes. Identical content IDs may occur at different positions; traversal preserves those positions. An absent unrelated subtree is irrelevant to a selected range. An absent required record, a scale mismatch, corrupted required bytes, or an invalid requested range is a typed error.

The construction/ingestion stage is included honestly: the JavaScript adapter read and folded every source byte, exported every unique record, and the Rust process loaded all records into a `HashMap`. Before the selective-read tests it authenticated every source-generated record once. Therefore the experiment is **not** an end-to-end low-memory reader or an actual network fetch benchmark. The selected-range counters isolate the records touched by that range after the store exists. They show which records a future on-demand store would need; they exclude manifest bytes, envelopes, index overhead, initial construction, and runtime representation overhead.

Reproduce from this evidence directory after extracting the original supplied ZIP:

```sh
node make-fixtures.cjs /absolute/path/to/SpiralCore_Showcase /absolute/path/to/reproduction
rustc --edition=2021 -O range_reader.rs -o /absolute/path/to/reproduction/range-reader
/absolute/path/to/reproduction/range-reader /absolute/path/to/reproduction
```

Pass the extracted original Showcase directory as the first adapter argument and a chosen fixture output directory as the second. Both arguments are filesystem paths; the reader receives that output directory as its sole argument. The fixture outputs are deterministic except for measured construction times. `parent-results.txt` is the primary agent's rerun and matches `results.txt` byte-for-byte; `parent-construction.json` and the three `*-resource.txt` files retain that rerun's construction/build/read observations. The binary and generated fixtures need not be distributed to reproduce them.

Observed runtimes: Node `v26.4.0`, Rust `1.97.1 (8bab26f4f 2026-07-14)`. The first standalone compile omitted `--edition=2021` and failed because `TryInto` is absent from the default edition's prelude; there was no resulting executable. The explicit edition command compiled and the experiment exited 0. The source was subsequently formatted with `rustfmt --edition 2021`; no semantics changed. Construction timings in `construction.json` are one local observation including the original source's asynchronous pause and serialization, not performance estimates or a benchmark distribution.

Observed results, retained verbatim in `results.txt`:

* 813 selected byte intervals matched `.raw` bytes exactly: 270 each for unique, repeated, and UTF-8 data, plus three empty-object cases. These include whole-object intervals as controls and zero-length intervals.
* All 1,549 distinct source-generated records authenticated under independent Rust BLAKE2b-96, and range/refusal/UTF-8 controls brought the assertion count to 2,374. The count is a test inventory, not a model result.
* A 23-byte interval `[16379,16402)` in a 32,768-byte object visited eight records, including two leaf occurrences: 384 serialized record bytes, out of 1,532 stored records / 70,972 serialized bytes.
* Removing a nonvisited leaf left that interval readable, while full reconstruction refused the missing leaf. Removing a visited leaf refused the range. Altering visited data refused its hash; altering its declared scale refused its scale.
* The 7,776-byte repeated input occupied eight distinct records / 380 record bytes. Full reconstruction retained 243 leaf occurrences in their correct order. Deduplicated storage did not collapse multiplicity.
* The 605-byte UTF-8 fixture places `🌀` across the first 32-byte boundary. `[31,35)` reconstructs the character; `[32,34)` reproduces the exact two requested bytes and correctly fails UTF-8 interpretation. An exact byte range is not automatically a valid character/token range.

Integrity scope: range reads check their visited hash-linked records against the supplied root references. The experiment does not independently verify the whole object's SHA-256 in Rust, and a selective read cannot freshly hash bytes it did not read. Fixture SHA-256 values and source-generated records are pinned in `construction.json`. Production range authenticity needs a trusted manifest/root and the declared content-hash assumptions. Missing unvisited data does not imply whole-object availability.

No tokenizer/attention behavior, semantic preservation, CUDA parity, actual network traffic, steady-state speed, or training benefit was tested. This is a successful data-structure bridge to a future selected-value reader, not that future reader itself.
