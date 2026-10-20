# Checkpoint-0 source-identity bridge — diagnostic only

**Source delivery, subsequently merged as #2542.** The [separate sealed diagnostic](../propose-subphase-diagnostic-run-2026-10-16/README.md) records actual bridge use under PM's later execution release, with original bytes unchanged and zero apply. The source-delivery contract below is retained. DeepSeek owns independent exact-head
review under [the four prospective criteria](https://github.com/UOR-Foundation/uor-r4/pull/2541#issuecomment-6212970029).
The [sheet](../../harness/prep/m1-propose-subphase-diagnostic-2026-10-16.md)
records [Kimi's FIT (design)](https://github.com/UOR-Foundation/uor-r4/pull/2541#issuecomment-6210781684)
in this delivery's first commit, per [PM routing](https://github.com/UOR-Foundation/uor-r4/pull/2541#issuecomment-6214061543).
This does not authorize diagnostic execution before bridge review, protected
merge/identity verification and the final pinned execution packet. Training,
M2 and Stage B remain HELD; Q4 NOT_EXECUTABLE. M1 qualification instrumentation
subline count0/3 and suspended headline37/232 unchanged; no registry count reset.

**Justification:** kernel unchanged; source-set identity moved because profiling
sources are part of the module source.

## Exact boundary

`checkpoint_bridge.rs` is a host metadata adapter in the standalone timing
instrument, not a production loader change. It accepts only the original
restart-5 seed1/epoch0 K16 Route file, whose exact bytes have SHA256
`9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4`.
The only admitted source mapping is:

- Before: `f6c4cc9f7a842ecb7a58fb569dcb7af61f84eb533cb4a059ee614be9c0f18fa7`.
- After: `4c0bf3ab97a82c3fed5187eb448b5d007a7dbe7271908b4f491e66d27697ea45`.
- Unchanged PTX: `1a21d6c3e2773a427c14ebff9499e03386147ab41b3ce413b7b5c9123925fd28`.
- Geometry: `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327`.
- Initialization config file: `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4`.

These constants are checked against the explicit bridge declaration in the
[diagnostic pin](../propose-subphase-diagnostic-2026-10-16/diagnostic-pin.json),
compiled identity and the actual file digest, not supplied as a caller-defined
wildcard. The file keeps its original hash; only the diagnostic's executable
source pin changes to the already gate-checked #2541 identity. Any other file,
source pair, seed, epoch, Q4 selection, config, kernel, geometry or training
permission refuses. Refusals name the artifact path and pre/post source hashes.
The original archive and snapshot remain untouched.

After exact authentication, the adapter changes **only** `source_sha256` in a
temporary JSON value, checks equality of all other fields, and hands the bytes
to the unchanged typed `Snapshot` parser and `Device::from_snapshot`. It never
writes a rebound checkpoint file. The existing A/R policy selection remains
after adaptation and before device admission. No training importer, production
source file, kernel, model arithmetic, `.apply` call or refusal interlock changes.
No promise is made about arbitrary code a caller might write to misuse public
APIs; this delivered bridge is reachable only from the diagnostic instrument,
whose existing zero-apply/epoch/snapshot invariants remain mandatory.

## Recorded mapping, not implicit admission

The plan includes the full bridge pin. Before device admission, each cell
records `uor-r4.checkpoint0-source-bridge-receipt/1` with `bridge_used:true`,
pre/post source hashes, original file hash/path, in-memory serialized hash,
unchanged non-source payload hash, PTX/geometry/config pins, seed/epoch,
`evaluation_only:true`, `training_resume:false`, `rebound_checkpoint_written:false`
and the separately selected cell policy. Status explicitly says mapping
prepared, not device admission or successful measurement. If admission fails,
the existing fault path seals the mapping receipt and failure together.

## Host metadata validation

The ordinary host suite has 26 passing tests and one sealed-artifact test
ignored by default. It checks each pin boundary, same-kernel wrong-source
refusal, wrong bytes, Q4/training refusal and the explicit no-bridge no-op.
Strict all-target host clippy passes with existing dependency warnings visible.

The ignored test was explicitly run against the actual archived checkpoint
using `UOR_SEALED_CHECKPOINT0=... cargo test --manifest-path
docs/labs/k16-timing-2026-10-15/instrument/Cargo.toml authentic_sealed_checkpoint
-- --ignored --nocapture`: one pass. This only reads, hashes, parses and compares
stored JSON; no CPU model/reference execution. It proves all non-source fields
equal, both identities recorded, mutation/whitespace/second-rebind rejection,
and no training permission. Metadata-derived canonical in-memory SHA256:
`d70ec39b97613de568050ffac4eba2293c9900753fdbb8146229578dd40d0ddd`;
unchanged non-source payload SHA256:
`2c2490a00545fc58c35fb5c3756382b7eb2fc14586e43ae85103019ca6aeaab9`.
This is not a GPU admission, control or diagnostic receipt.

## Compile recipe and delivery

The same delivery documents the [fresh-pod recipe](../compute.md#reproducible-fresh-pod-cuda-compile-gates-16-october)
and adds `scripts/pod/uor-cuda-check-remote`: explicit `/root/.cargo/bin` PATH,
Rust1.97.1, CUDA12.8/compute_120, Git clone with exact-head assertion, and
standalone check/strict-clippy logs with explicit exits. No model execution.
Codex supplies the bridge's new-head compile-only transcript, archive MD5 and
cost as PR comments. DeepSeek independently testing the recipe remains a
separate next-use portability check; it is not claimed from author testing.

The prior #2541 dual gates were already complete before its approval and merge
at `4bf154c3e`; no duplicate old-head lease is needed. Its production source and
kernel bytes remain unchanged here, preserving the bridge's fixed target hash.
Next: DeepSeek exact-head bridge review, protected merge and execution-packet
verification. No diagnostic, fixture or training run in this delivery.
