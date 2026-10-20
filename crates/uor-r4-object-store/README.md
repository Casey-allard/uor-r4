# Exact byte object store

A model-independent D19 prototype implementing the Showcase ordered 32-byte,
ternary reconstruction DAG. Repeated content shares records; unique content may
expand. Records retain the original BLAKE2b-96 format. UOR manifests and token
occurrence bindings have separate versioned formats.

`Store::fold` writes immutable records; `export_records` is sorted by NodeId and
`import_records` publishes transactionally. `range` authenticates only visited
records against a caller-pinned manifest. Only `rebuild` also verifies the whole
SHA-256. Neither identity is an authorization signature; short node IDs do not
provide a collision-free or hostile-network security claim.

`bindings::ingest` uses the existing byte tokenizer once and requires exact
source-byte reconstruction. Repeated tokens retain separate occurrence indexes
and byte spans. Tokenizer loading/cache ownership remains caller scope;
unsupported offsets or prefix-space insertion are refused. Serialized bindings
require their own trusted provenance; they cannot establish authorization or
prove canonical tokenization merely by containing matching identity fields.

The eager `RecordBackend` is a reference store. DeepSeek owns production lazy
DAG reading. Its provisional adapter seam and exact-source check are in the
[implementation record](../../archive/labs/d19-object-store-2026-10-15/README.md).
No model integration, attention change or training permission follows.

```sh
CARGO_TARGET_DIR=local/object-store-target cargo test -p uor-r4-object-store --offline
CARGO_TARGET_DIR=local/object-store-target cargo run -p uor-r4-object-store --example exact_bytes --offline
```

The example performs byte/data-structure work only. An optional caller-owned
output path exercises save/restart. Save validates before replacement; concurrent
writers and crash durability of directory metadata are outside this prototype.
Default numeric limits and all executed checks are recorded in the implementation
record. Raw source bytes, retained record bytes and new record bytes are separate
accounts; none includes caller tokenizer memory, manifest/binding bytes or a
claim of measured transport speed.

I/O failures expose typed `ErrorKind` and an optional raw OS code. Ingestion uses
the object-size ceiling; occurrence reads use the receiving store's returned-byte
ceiling. A token valid for storage can therefore exceed a reader's quotation
budget. This preserves independent storage and response-size policies.
