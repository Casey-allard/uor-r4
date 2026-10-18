# Read-only CUDA token-to-cell export — 17 October 2026

The same-cell reference scorer needs the actual vocabulary-to-cell mapping used
by the model. Existing snapshots contain codebook vectors, and existing DEV
captures contain per-row admitted slots; neither is a declared4096-token map.
[PM ruling6240513498](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6240513498)
approves host analysis of the sealed TRAIN counts and requires this map to be
exported on CUDA through the existing `token_cell`, with no host reimplementation.

**Scope:** source implementation plus Gate9 CUDA construction at
`96ad8a4750398a8ecbf89e711f351de1580d7991`. All five declared compiler/host/link
commands PASS under [PM dispatch6241202400](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6241202400).
The binary was archived without invocation at that construction stage; construction releases
no finite or learning work. Source delivery is protected merge
`d46e050f4dfe4177f599410f34e1647cc9b2fc4f`, whole-tree equal to reviewed ca707.
The separately released [Battery2 attempt](../attention-preflight-2026-10-17/README.md)
failed its raw-data staging check before any project binary ran; it supplies
no runtime-export or mechanism result. Its release is consumed.
The later separately released [Battery3 preflight](../attention-preflight-2026-10-17/README.md),
under actual PM6242930665, completed the export at the same executable/source
identities:4096 u16LE cells/8192bytes, one typed restore/export call, unchanged
snapshot and zero export forwards/proposals/applies. The restore includes the
existing temporary initialization/upload/validation overhead. All26 battery
command/contract steps returned0, including initial admission/sensitivity and
9-control qualification, with full archive before own-pod down. This adds scoped
runtime-export evidence; learned Relation and the separate paired dose remain
NOT_RUN. No token map alone establishes full same-cell attribution.
Gate8 and its binary retain their earlier identities and evidence scopes.

## Explicit hash names for the next source

[PM6243260600 and the bounded runner repair](../attention-hash-domains-2026-10-17/README.md)
reserve `model_snapshot_sha256` for typed Snapshot serialization (B) and add
`model_storage_sha256` for the stored model-Value serialization (A). Historical
Battery3's same-named export field carries A while its eval field carries B;
the existing checkpoint-to-eval validation already checked the consistent pair.
Those accepted historical bytes remain unchanged, without rerun or rescore.
Canonical DriverCheckpoint and raw file identities remain separate. The new
source passes Gate10's five compiler/host commands at dfa3, with its debug
binary archived without invocation; protected delivery precedes dose use. Gate9 and
Battery3 evidence below retains its exact earlier source scope.

## Exact interface and format

```text
discrete-stack attention-export-token-cells CONFIG DRIVER_CHECKPOINT NEW_REPORT_DIR
```

The output directory is exclusive. The command validates the configured attention
lineage and typed DriverCheckpoint, reserves `token_cells.u16` before constructing
CUDA state, restores that exact checkpoint, and checks epoch and full snapshot
equality both before and after export. A failed command keeps available partial
files and a FAILED report; it never overwrites a prior map or claims success from
an empty/partial file.

`token_cells.u16` is exactly8192 bytes:4096 unsigned16-bit words, little-endian,
without a header, in ascending token-ID order0 through4095. Every cell must be
in0..255; ABSENT is refused. The host only copies, checks storage bounds, encodes
bytes, synchronizes the file and verifies readback. It never evaluates root
membership, a token's geometric address, model admission or ranking.

`report.json` uses `uor-r4.attention-token-cells/1` and binds:

- Actual config/checkpoint file paths and SHA256s, canonical config and typed
  checkpoint SHA256, lineage, explicit stored model-Value and typed Snapshot
  SHA256 domains in the repaired source, and source/kernel/
  geometry/spec/read/initialization/seed metadata.
- Actual output-file SHA256, encoding, count and byte inventory.
- Existing checkpoint epoch/cursor, unchanged snapshot, one model restore, one
  export kernel and4096 `token_cell` evaluations; zero forwards, proposals and
  updates, and no training capability.

The report's epoch may describe an already-learned checkpoint. It records no
new learning: export is read-only. The ordinary typed restore still performs
its existing CUDA initialization/upload/validation internally; this cost and
execution are part of the one restore, not a host reference or an extra training
step. Old-source checkpoints are not bridged to this changed source identity.

## Minimal device change

The new ordinary entry `discrete_export_token_cells` receives const codebook,
geometry and spec pointers, plus writable output and status only. Each token
calls the existing `token_cell(table,g,s,token,2,0,0)`, the identity-candidate map
used by the current writer. Invalid cells raise device status8. No route, gate
or epoch pointer is passed. The kernel and its Rust method call no forward,
proposal or apply routine.

The exact AOT entry inventory gains this one entry. Existing initializer,
`token_cell`, writer, numerical helpers and learning-law bodies are unchanged;
the old CUDA source is recovered byte-for-byte by removing the new entry.
The separate decomposition/reuse source slices remain unchanged. The ordinary
source/AOT/driver identities intentionally change, so a fresh exact-head CUDA
construction transcript and a newly bound binary are required before delivery
and execution. No waiver, lint cap or generic runtime is added.

## Checks and delivery boundary

- Host no-CUDA all-target package check: PASS.
- Two focused host metadata controls: PASS — exact raw encoding/order/length/
  range, and refusal without CUDA instead of input reads or host geometry.
- Formatting and diff checks: PASS.
- Independent exact-source and actual packet review: AGY **APPROVE** at96ad,
  [6241052759](https://github.com/UOR-Foundation/uor-r4/pull/2601#issuecomment-6241052759).
- Gate9 CUDA all-target check: PASS (35.76s); strict CUDA package all-target
  `--no-deps` Clippy `-D warnings`: PASS (27.30s).
- Gate9 host no-CUDA all-target check: PASS (2.03s); metadata library suite:
  **82 PASS /0 FAIL /1 ignored**, test execution7.47s (build8.28s separate).
- Debug CUDA `discrete-stack` build/link: PASS (27.61s), binary SHA256
  `5abe54c3ce64dd66006b5ff50697d0c416071ba50c473014a0053e6bd63099ed`.
  Executable invocation and runtime compiled identity: **NOT_RUN**.
- Device map correctness, export and immutable runtime receipt: **NOT_RUN**.

## Actual Gate9 transcript and closure

The frozen24-member packet manifest is
`9ea1b5b162ae96d513a9c421add737fb296a62f44afe209cd19ac752e9ef9e1b`;
installed independent readiness is
`6937fb24f681bbb067b17b751ff47e4a8504500acf4a71c17d9851233e705f8a`.
The exclusive dispatch binds the actual PM receipt and AGY review, accepts the
strict package scope and build/archive without invocation, and uses a fresh
128-bit attempt label. The original operator completed with exit0, setup0 and
remote command0; no continuation, restart, on-lease patch or retry occurred.
Initial and final exact-source/tracked/index/unexpected-path guards PASS.

Own`vast-56451966` used RTX5090, driver595.99.02, CUDA12.8.93,
compute_120 and Rust1.97.1. The full58-remote/43-operator/24-packet SHA inventory
passed. `icloud:UOR-R4/results/codex/attention-compile-gate9-archive.tar`
is40,995,328bytes, MD5`22c08eb316a9f669cb723b601112cc0c`; full cloud readback
verified before normal release21:19:38Z/down21:19:44Z. Fresh fleet after down
showed0pods; this is a dated observation, never ownership of a peer's pod.
Late final operator/fleet/ledger closure files are preserved separately after
down and are not represented as pre-down remote evidence. Raw provider
diagnostics remain private; published receipts disclose no CREATE/up/lease data.

Provider-recorded uptime282s at$0.964/h estimates$0.075513333; the separate
up-to-down285s wall proxy estimates$0.076316667. These are estimates, not an
invoice; storage/transfer remain UNMEASURED. Carried Codex mixed episode
estimate is$4.594265555 plus PM spending, with its earlier scopes retained.
Construction permits protected source delivery after actual outcome and
merge-forward review; it establishes neither a correct runtime map nor a
learned-attention result. Gates1–9 releases are consumed, not reusable.

## Next dependency and scientific scope

The source-input dependency is `identity → init once → export that checkpoint →
eval/probes/qualification`, unless an existing exact-source checkpoint is
explicitly supplied. Export cannot precede creation of its bound checkpoint.
The PM packet must bind these phases with hash seals without inventing another
manual approval between them. There is no requirement that future GPU output
exist before the lease that produces it.

PM6240513498 assigns the additional DEV admitted-slot capture to the dose/
dual-checkpoint phase. A token map alone does not recreate those row-specific
admissions. Battery count/B3/export observations must remain distinct from full
same-cell dose reports; protected sheet/packet reconciliation remains PM-owned.
The frozen Battery1 draft remains preserved and non-executable until its actual
producer/validator contract is rebound. No full same-cell result follows here.

Line: M1 learned copy-successor link, Unit2c input producer. Headline unchanged;
no scientific result or mechanism negative, and no progress-count reset. This
is the specific missing input for the already-declared decisive learning test,
not a new mechanism or timing series. Training/M2/StageB remain HELD;
Q4 NOT_EXECUTABLE. Source/FIT releases neither compute nor training.
