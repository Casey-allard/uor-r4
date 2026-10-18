# Attention artifact hash domains — 17 October 2026

[PM disposition6243260600](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243260600)
formally accepts the completed [Battery3 preflight](../attention-preflight-2026-10-17/README.md)
without rerun or rescore, and commissions this bounded emission/validation repair
before the paired learning dose. The same stored snapshot has two legitimate
serialization hashes. The earlier export called the stored model-Value hash
`model_snapshot_sha256`, while evaluation used that name for the typed Snapshot
hash. Comparing those two fields directly rejects correct matched artifacts.

## Explicit domains and bindings

| Domain | Exact bytes | Future evidence field |
| --- | --- | --- |
| A | Existing stored model `serde_json::Value` serialization | `model_storage_sha256` |
| B | Supported typed `Snapshot` serialization | `model_snapshot_sha256` |
| C | Canonical typed `DriverCheckpoint` serialization | `driver_checkpoint_sha256` |
| D | Actual raw file bytes | The applicable `FilePin.sha256` |

`DriverCheckpoint.model_sha256` keeps its historical Domain-A meaning and format.
The repair adds explicit A/B metadata to export/evaluation/progress/transfer
records and checks both against the same actual nested checkpoint. Shared
helpers only deserialize/re-encode stored metadata and hash the resulting bytes;
they create no Device, CUDA context, address, geometry, model score or admission.
The existing qualification INIT LOCK observations are persisted with their typed
hashes without introducing any extra control, initialization or forward call.

The checkpoint-bound evaluation path preserves Domain A from the original
stored Value, including explicitly present default fields. It checks Domain B
and epoch against the evaluator's existing single typed Snapshot read before
emitting that A. Restoring typed state may normalize defaults; its normalized
Value hash must not silently replace the original checkpoint's storage hash.
The `eval-corpus` CLI and attention/transfer callers pass the retained exact
checkpoint through this metadata binding. Ordinary Device-only evaluation
continues to report the canonical Value generated from its current snapshot.
No extra snapshot read, initialization or Forward is introduced.

The actual Battery3 checkpoint is the concrete witness: Domain A is
`ea67bedd3b4a3c6c34d3431008fefd451a5b10895306fa6c2ccda4497ab4a7f8`,
Domain B is
`bb160787b43d3bb342e8707d6ff2d3b473f56d6524954335a77920b1e02c00da`.
Its existing `init-b3/report.json` cross-domain validation passed. These are not
corrupted or mismatched storage. Accepted historical files and their hashes stay
unchanged; they are not rewritten to the corrected names.

## Source and execution scope

The change is confined to metadata emission, integrity validation and focused
serialization/refusal fixtures. It changes no CUDA kernel, initialization,
selection, objective, numerical computation, proposal, sole apply transition,
learning law, registered scientific panel, control count, dose or frozen Route
completion/timing source. New-format progress requires its explicit bindings;
it does not silently migrate old progress or checkpoint lineage.

The source identity and future executable identity change even though numerical
behavior does not. Old construction passes, binaries and checkpoints retain
their exact source scope. Exact-head independent source/packet review, applicable
compiler construction and protected delivery are needed before the dose packet
is rebound. This source commission is no model or provider execution release.

## Source-review correction retained

Independent review of the first candidate a2af13cc identified the explicit-
default storage edge before any construction or runtime release. Its report
SHA is `b99f1b53a6c209834367a6aa579ed902fc5c800276e35ae617b4232cf3972ce8`.
The narrow checkpoint-bound metadata repair includes the existing eval-corpus
CLI leaf; it does not tighten legacy checkpoint admission or add an artifact
bridge. The initial candidate and prospective compiler manifest are preserved
as predecessor evidence, not promoted to a final approval or paid attempt.

## Verification and next dependency

The owned `codex/cf1-attention-hash-domains` candidate passes host no-CUDA
all-target checking, formatting and85 metadata library tests (0failed,1ignored).
The first full metadata run had83PASS/1FAIL/1ignored because a pre-existing
storage fixture needs the ignored `local` parent in a fresh worktree. The failed
log is retained; creating that parent, as the accepted compiler recipe already
does, yielded the passing rerun without a source change. New named-field
metadata tests passed in both runs. The initial six-file candidate had84 passing
tests; the exact stored-checkpoint correction adds a third host metadata test
and passes85/0/1 at its final source. Its author verification report SHA is
`7f49ce69c336ade601d0b13e2086209ff9c0514520f322f3eeab7df373fbff0c`. The two real typed-Snapshot serialization
fixtures are CUDA-feature scoped: source authored, execution NOT_RUN. They
construct no Device and test only encoding/default omission/refusals. CUDA
construction and debug linking now PASS at exact source
`dfa3f483090a714b9e56891e74f6ef197087ca53` under distinct
[PM dispatch6243606649](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6243606649),
after AGY exact source/packet APPROVE6243576819/6243576996. Gate10's frozen
24-member manifest is
`59cef84aba46f8ee7f549f005cc2ba6de8f97d31c56fe6c67e167aea7c99b6e9`;
its independently installed readiness is
`7047c59d9c1b608b457fea0b9d1e3bd8e798db541f1c656c966cd8a1ca4a2df2`.
No old gate release is reused.

All five command exits and the original command marker are0: CUDA all-target
check35.62s, strict CUDA package `--no-deps` all-target Clippy `-D warnings`
27.40s, host no-CUDA all-target check2.09s, host metadata build8.35s with
85PASS/0FAIL/1ignored (test execution7.41s), and debug CUDA link27.59s. These
are Cargo-reported durations, not end-to-end lease or model throughput.
Initial/final exact-source guards pass. The linked ELF SHA is
`d87c9f14dacca8a67024352cee1c1c2d6b6644b9a064ca3bdcf8d51c71830f2b`;
it is archived without invocation, so runtime identity remains
UNAVAILABLE_NOT_EXECUTED. New artifact emission, CUDA-feature serializer-test
execution, initialization, export, evaluation, qualification and learning are
NOT_RUN at this source. Original operator85354/setup/command all return0, with
no continuation, restart or paid retry. All58 remote,43 operator and24 frozen
packet SHA entries verify. The full archive is41159168bytes/MD5
`81cc3a7285c6517d1035649c393acf51`, stored and cloud-readback verified before
own pod56471322 release00:22:05Z/down00:22:12Z on18October. Fresh fleet status
then reports0pods; this is a dated observation. Late lifecycle files are sealed
separately after down, never claimed to exist in the earlier archive.

Native provider uptime251s at$.964/h gives$.067212222 as an elapsed-cost proxy;
the separate254s up/down wall proxy is$.068015556. These alternatives are not
summed or called invoices. The carried mixed Codex estimate becomes$4.804471110
plus PM work; storage/transfer remain UNMEASURED. This is compiler construction,
not runtime qualification, learned-attention evidence or a dose release.

After source review, construction and protected delivery, bind the actual new
source/executable and final artifact contract into the one-seed-first paired
Relation/frozen-zero packet. Kimi retains FIT, protected queue and execution
release. AGY owns its separate consumer and conforms to the explicit domains;
its final evidence synthesis is not a separate learning-start prerequisite.
There is no extra timing run, seed-choice loop or fourth acknowledgement phase.
The registered reply37/232 remains SUSPENDED, learned Relation NOT_RUN and the
recorded line count0/3 is neither reset nor reassigned by this repair. Training,
M2 and StageB HELD; Q4 NOT_EXECUTABLE.
