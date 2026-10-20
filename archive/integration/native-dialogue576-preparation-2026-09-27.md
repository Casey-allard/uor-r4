# Native width-576 integer development profile

References #973. This delivery connects the retained R1d native model shape to
ordinary integer serving through an explicit development profile. It does not
promote an artifact or establish numerical retention, useful conversation,
geometric advantage or full-path efficiency. The fixed complete-prefix study
keeps its own unchanged scientific conditions and checkpoint recovery plan. The [source-bound packet](../evidence/native-dialogue576-preparation-2026-09-27.json)
records the executed checks and resource scope.

## Why this change

The recovered R1d learner has 576 state/value coordinates, read width 64,
vocabulary 4096 and context 256. The retained integer loader accepted only
width 128/256; ordinary value storage and accumulation were fixed at 256, and
normalization used power-of-two width shifts. Removing the loader guard alone
would truncate values or index outside the accumulator. This implementation
provides the actual wide storage and kernels while keeping the existing default
profile and the separate Google `SessionState` layout.

`ServingProfile::Dialogue576` requires explicit artifact metadata, Quaternion
transport, Dot read and Full access. Width576 is not inferred from shape alone.
The default loader remains the retained profile. Dense signed4 rows have widths
576 or 1152; the tied vocabulary projection reads width-576 rows. Each ordinary
session retains all 576 value coordinates for up to 256 causal tokens, with keys
and queries still 64-dimensional. Read width64 is not a 64-token memory policy.
The fixed-256 conversational API rejects this profile before its scratch access.

For Q11 input codes in [-32767, 32767], with S the sum of squared codes, the new
normalizer uses

```
V = floor((S << 58) / 9) + floor(2^86 / 100000) + 1
D = floor_sqrt(V)
y = clip(sign(x) * round_ties_away((abs(x) << 42) / D), -32767, 32767)
```

This is the declared integer approximation to width-576 RMS with the retained
epsilon. Products, quotient and square root use the existing bounded integer
algorithms. The /9 quotient floors before epsilon and square root. Out-of-range
private-helper inputs are rejected. Existing 128/256 normalization remains the
old specialized path. Source arithmetic and emitted instruction inspection have
different evidence scopes; neither proves complete application multiplier
freedom or language equivalence to the floating model.

## Explicit conversion and portable provenance

The offline Rust example takes these arguments:

```
dialogue-integer-bridge FIT_ROOT CORPUS_MANIFEST TOKENIZER TABLE_ROOT NEW_REPORT_ROOT
uor-r4-integer pack-dialogue576 NEW_REPORT_ROOT/packed TABLE_ROOT TOKENIZER NEW_BUNDLE
```

The first entry consumes the strict `LegacyDialogueArtifact` import of historical
R1d, calibrates the existing signed4 multiplicative/signed16 additive dyadic
grids, writes a new packed child, and reloads it through the F32 emulator and
integer profile. It executes zero new forward, generation, backward or optimizer
update calls. Calibration still performs offline numerical work. The child
carries `CalibratedForExport`, equal historical start/completed steps and
`ramp_steps=1` compatibility metadata; this is not a fitted QAT ramp.

Before export the wrapper rechecks every original parameter shape and F32 hash.
Portable metadata binds the validated parent, tokenizer, literal-role protocol,
conversion source/executable/source-file hashes and zero new update counts.
Both outputs are claimed before model loading, and the table root is verified.
Successfully sealed children remain immutable if a later reload fails. The new
packaging wrapper also seals ordinary failed attempts and rejects reuse of an
existing root. If writing that failure record itself fails, the cleanup error
can replace the original returned diagnostic; both paths remain errors and no
successful bundle is reported.

Packaging requires a sealed packed child and tables. Loading the resulting
bundle requires both `serving_profile=dialogue576` and
`provenance_kind=development`, matches the full numerical contract and model,
and validates the actual dialogue protocol against the included tokenizer.
It needs no provider or original checkout at inference. The existing retained
bundle metadata is unchanged. Direct generic profile export remains a research
API; packaging this historical-parent conversion additionally requires its
portable lineage.

This example converts historical R1d, not a new full-prefix or role-only study
checkpoint. Those children require their own verified import and lineage.
The ordinary `generate` CLI still uses raw text framing. Exact training-protocol
turns use the existing `generate_dialogue`/`DialogueConversation` library APIs;
this change does not adopt Google's `uor-chat` path for 576.

## Executed preparation and limits

Numerical/converter source is pinned at `558aedf7c572550c22fff5cd0df1c2241ff59977`;
final wrapper source is `e75acd94281a4bfff99ece8e5228ea76a1647265`. The six other
changed source files are byte-identical across those commits. Rust 1.97.1,
`aarch64-apple-darwin`, offline release builds and the existing shared external
SSD cache were used. The converter uses default CPU features; it performs no
model forward in this preparation. Frozen binaries and all source hashes are
bound in the packet.

Ten distinct focused checks pass: wide rational normalization and bounds; full
value-coordinate storage and ordinary Read/NoRead stepping; strict profile
metadata; portable tokenizer/protocol/clock binding; failed packaging and
unchanged repeated roots; packed 576/1152 matrix/embedding/vocabulary arithmetic;
retained 128/256 normalization and geometry-bound loading; and exact synthetic
parameter export/reload against the existing quantizer. Two unchanged bundle
checks repeat after the wrapper repair. Direct rustfmt and whitespace checks
pass. No broad suite or additional learned-model calls were run.

| Frozen artifact | Unique inspected ranges | Forbidden instructions detected | Historical mandatory omissions |
|---|---:|---:|---:|
| Ordinary integer executable |20|0|3 conversational-only helpers|
| Integer library archive |35|0|0|

These counts combine the existing numerical audit and the new explicit wide
ranges without double-counting. New ranges include the 576 normalizer
(299 instructions),576 value accumulator (137), ordinary value read (403), and
serving step(3,481), plus the normalization dispatcher. The executable normalizer
contains the division and integer-root loops; its only external direct calls
are Rust allocation/error routines. Division also has a separately emitted
131-instruction range in the archive. Integer square root is not separately
emitted. Archive call relocations are unresolved in the saved disassembly, so
callee interpretation comes from the executable. All inspected ranges have
nonzero instruction counts. Missing ranges remain missing; this is not a
transitive call-graph or whole-process certification.

Build/check supervision totals 778.381 seconds across three preserved attempts.
The first two ended on shared-volume growth guards, after all scientific unit
checks had passed; no Rust compiler failure was observed. The last command
finishes only the interrupted converter compilation. Maximum child RSS is
3,333,554,176 bytes; maximum sampled process-group RSS is 3,393,503,232 bytes.
The three frozen binary/library payloads total 8,107,912 bytes. These are
preparation costs, not serving performance.

The original six-hour source phase, 18-hour lab ceiling, verified 722,400,000 ms
shared allowance, 4 GiB compile RSS cap, 64 MiB preparation-material cap, 512 MiB
external growth allowance and physical reserve remain. Necessary local
extensions were recorded before use: the aggregate build allowance rose from
900 to 1,200 seconds, and shared-container growth headroom was distinguished
from the 64 MiB material cap. The completed build ultimately used less than the
original 900 seconds. APFS/VM allocation was not deleted or hidden by rebasing the
original physical snapshot. The reserve remains 25,971,130,368 bytes plus 128 MiB.
Owned build and model work now run sequentially. Every interruption, review,
preparation and overlapping fit interval remains in the cumulative ledger once.

The dialogue control's resource interruption and exact 512-to-1024 recovery are
[recorded on the owning issue](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5855294255).
They are not a numerical or language verdict on this profile. The packet retains
its own source/build/resource snapshot; changing live model status belongs in
[current state](current-state.md).

## Cost and next decision

This is a dense-access interim serving path under D0-b. It does not satisfy D5
selected parameter access. The576 parent contains 5,429,826 F32 elements across 21
arrays. Source-derived packed coefficient payload is 2,725,956 bytes versus
847,876 bytes at 256. A 256-position value cache stores 589,824 payload bytes
versus 262,144; neither quantity is total RSS, traffic, latency or energy.

After the fixed dialogue pair frees the one owned model slot, record a separate
prospective conversion/observation projection. Preserve the historical parent,
create a new development artifact, and observe actual numerical and complete
reply behavior through the bound dialogue adapter. Distinguish parameter
quantization, interface quantization and integer arithmetic when locating a
regression. Keep the existing open panel and decoder fixed; no automatic
quantizer, dose, seed or decoding sweep follows from a weak result. Fresh
held-out evaluation remains unopened. Failed candidates remain evidence.
