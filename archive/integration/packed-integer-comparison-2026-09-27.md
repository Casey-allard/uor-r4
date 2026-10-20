# Packed integer coefficient comparison — September 27, 2026

The shared packed representation preserves all declared stable behavior in the
frozen comparison on the two retained width-256 Quaternion and Householder-pair
models. All 16 processes completed successfully and all eight baseline/candidate
pairs match. Independent review supports protected delivery without another
benchmark sweep. Existing language weaknesses remain.

The [portable evidence packet](../evidence/packed-integer-comparison-2026-09-27.json)
contains the frozen plan, binary and artifact identities, comparison results,
actual generated outputs, complete saved chat DTO identities, exact storage
analysis and independent review. This completes the workload requirement from
the [implementation/preparation record](packed-integer-preparation-2026-09-27.md).
The [prospective work card](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5855822870)
binds the launch conditions. No new artifact fitting or model selection occurred.

## Change and source boundary

Baseline source is `9d78ebadb0107ee4566c7787d1037a031bb117f7`; candidate source
is `d0844ba0c1e7a22803c18767796336bfb8ab376e`. The later candidate PR head
`013f2c1c6142c33a02d8fbac54b4734fcc533d49` changes only documentation.
Both versions load the identical retained bundles; no serialized coefficients,
tables, tokenizer, context policy or parameter values change. The candidate
keeps signed4 coefficients packed and shares cached parameter backing through
`Arc`. Existing four focused arithmetic checks and release builds remain the
executed source checks; this comparison reused those bound binaries.

The preparation's scoped opcode inspection remains its own evidence: it does
not become a new full-path audit merely because output equality passes. The
owner's native, multiplier-free serving target remains unchanged. This is an
interim D0-b storage improvement; dense parameter access remains.

## Actual behavior preserved

| Workload | Compared scope per version |
|---|---|
| Retained replay | 4,096 positions: 1,024 each for Read and NoRead on each of two models; distribution/state hashes, predictions, mass/slot invariants and historical comparisons |
| Source responses | 128 greedy requests across the two models, with complete decision and output records |
| Continuations | Ten categorical requests across the two models, preserving tokens, sampled state and stops |
| Separate conversational session | Two turns per model, four complete saved DTO snapshots, all visible text and token counts |

The replay also exercises its recorded `TextSession` boundary. Across baseline
and candidate, replay contains 8,192 position records; paired positions are
4,096. Source and continuation generation total 138 requests **per version**.
These denominators are not interchangeable with model calls, because prefix
processing and the session boundary have their own calls.

Only frozen timing exclusions were applied: replay load/model/wall fields and
hashes of timing-bearing files, and generation timing fields. Probability/state
hashes, decisions, selected IDs, text, stops and RNG remain compared where the
interface records them. Complete chat DTOs have no exclusions. Chat presentation
banners, save acknowledgements and timing telemetry are outside response text.

The frozen conversational CLI predates the separate terminal-error reporting
fix. It does not expose terminal error/stop detail or per-step distributions.
This comparison therefore establishes equality of the recorded DTOs and visible
emissions, not equality of unrecorded fields. There was no save-load-resume
experiment. These two width-256 artifacts do not establish width-576 equivalence.

The principal read all ten candidate continuations and four distinct chat
emissions. Independent review read all 128 unique source replies as well. Both
reviews retain the same limitations: generic repetition, unstable referents,
incoherent actions and unfinished replies. Example chat emissions remain
“Once there was a little girl named Lily” and “Once upon a time, there was a”.
No general prose or chat qualification follows from preserving these outputs.

## Storage and observed cost

Both actual artifacts contain 1,672,704 signed4 coefficients and 5,762 signed16
coefficients. The baseline retains one decoded map plus eleven deep-cloned hot
parameter arrays. The candidate shares their packed backing.

| Logical retained coefficient payload | Baseline bytes | Candidate bytes |
|---|---:|---:|
| Signed4 arrays, including cached copies | 6,690,816 | 836,352 |
| Signed16 arrays | 11,524 | 11,524 |
| **Total** | **6,702,340** | **847,876** |

The source-derived reduction is **5,854,464 bytes (87.35%)**. Packing a single
signed4 copy accounts for 75% of that copy; sharing removes a separate 3,345,408
bytes of duplicated codes. This logical accounting excludes cloned metadata,
allocator overhead, tables, session memory, 92,192 bytes of scaled bias/age
caches and 16,384 bytes of output-shift caches. The public loader still constructs
decoded vectors temporarily, so this is not a peak-load memory proof.

The on-disk `hard-parameters.bin` was already packed at 847,876 bytes and remains
unchanged. There is no artifact-size saving. A nonempty-history Read still visits
1,672,960 signed4 scalar coefficients, including the full vocabulary projection.
Logical span size is not measured physical memory traffic or D5 selected access.

| Workload | Baseline wall (s) | Candidate wall (s) | Baseline peak child RSS (bytes) | Candidate peak child RSS (bytes) |
|---|---:|---:|---:|---:|
| Quaternion replay | 3.997 | 3.988 | 16,138,240 | 13,615,104 |
| Householder-pair replay | 4.012 | 3.984 | 16,351,232 | 13,598,720 |
| Quaternion source | 6.496 | 6.516 | 16,007,168 | 13,549,568 |
| Householder-pair source | 6.496 | 6.477 | 15,990,784 | 13,631,488 |
| Quaternion continuation | 1.558 | 1.552 | 15,646,720 | 13,369,344 |
| Householder-pair continuation | 1.292 | 1.290 | 15,761,408 | 13,565,952 |
| Quaternion chat | 0.309 | 0.315 | 15,810,560 | 13,713,408 |
| Householder-pair chat | 0.315 | 0.313 | 15,826,944 | 13,762,560 |

Observed peak child RSS is lower in every pair by 2,064,384–2,752,512 bytes,
or 13.04–16.83%. Model-clock changes are small and mixed: replay ratios range
0.9702–0.9863; source ratios are 0.9958 and 1.0007; continuation ratios are
0.9605 and 0.9917 (candidate/baseline). This is one counterbalanced pass with
no variance estimate. Supervisor polling and resource probes contribute to
whole-process clocks, especially the short chat runs. These observations support
a scoped memory result, not general speed or energy savings.

Complete comparison supervisor time was **50.773970167 seconds**. The nested
process clocks sum to 48.909727543 seconds and are not added to supervisor wall.
Maximum child RSS was 16,351,232 bytes. At the last resource observation, new
apparent comparison material was 22,532,752 bytes, internal free space was
30,267,596,800 bytes, SSD free space was 172,499,419,136 bytes and reported free
memory was 75%. Final seal/output and delivery material are additionally retained.
Preparation, independent review and delivery wall continue on the shared ledger;
this process receipt does not reset their accumulated cost. No allowance was
raised for this comparison. No external compute or deletion occurred.

## Decision

Deliver the packed shared representation through its protected PR. Preserve the
old/new bundles, source bindings, sealed reports and negative language evidence.
Do not repeat the workload sweep after its declared risks are resolved. The
remaining native model problem is learned capability and its retention through
discretization; the completed historical width-576 observation and prior
learned-rounding evidence inform that distinct next choice.
