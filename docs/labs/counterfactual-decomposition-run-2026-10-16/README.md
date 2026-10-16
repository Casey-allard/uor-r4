# Counterfactual decomposition result — 16 October 2026

**Complete bounded diagnostic: repeated baseline work is material and writer
work dominates all four cells.** The original checkpoint produced all 32
proposals (8 excluded warmup, 24 timed), with matched U/D observations and zero
apply. The measured baseline share is **46.921799–47.220974%** and combined
writer share is **99.979269–99.979452%** of diagnostic lane cycles. Every retained
lane satisfies the prospective H2 ≥20% and H3 ≥80% routing bars. Maximum
incremental-clock perturbation/spread relative to minimum U is below 0.77%,
within the declared 20% bound. These locate redundant work; they do not measure
the speedup from removing it. H1 launch-speedup magnitude remains NOT_APPROVED.

Training/M2/Stage B remain HELD; Q4 NOT_EXECUTABLE. M1 qualification count 0/3
and suspended headline 37/232 are unchanged. This is not qualification, a p95
estimate, a training rate, a language result or a mechanism verdict.

## Measurements and their scope

A = `nearest_address_margin_v1`; R = `occupied_cell_rank_hinge_v1`. Each row
below retains all 16 completed nonfaulted lanes in each of 3 timed D samples
(48 lane observations per cell). Percentages use lane end−start cycles,
including uncovered overhead; final terminal stores are excluded from lane
clocks but remain inside the CUDA event. No lanes or samples were discarded.

| Policy / operation | Ordinary event range, seconds | Baseline share, % | Combined writer share, % | max(E,S) / min(U), % |
| --- | --- | --- | --- | --- |
| A / Route(0) | 7.449555–7.474665 | 46.921799–46.930721 | 99.979269–99.979436 | 0.753047 |
| A / VectorBlock(0) | 7.404304–7.429064 | 47.210596–47.220535 | 99.979312–99.979405 | 0.739000 |
| R / Route(0) | 7.452932–7.453434 | 46.922942–46.931601 | 99.979275–99.979452 | 0.767453 |
| R / VectorBlock(0) | 7.407624–7.408632 | 47.211046–47.220974 | 99.979296–99.979404 | 0.755978 |

H2's numerator is baseline read + baseline writer + eligibility. H3's is
baseline + virtual writer. They overlap and are not additive savings. All
four cells return `baseline_material:true`, `writer_dominant:true`,
`unresolved:false`; these are descriptive routing decisions only.

The combined selected-cell selection/member-scan phases occupy
**94.104057–94.171533%** of lane cycles, occupancy scans **5.801593–5.840467%**,
and baseline + virtual geometric reads **0.018972–0.019067%**. Writer final
work is about 0.0028% for A and 0.0338–0.0352% for R. Nested writer spans
partition their parents; do not add them again to writer or wall time.
Membership classification versus distance instructions inside the member
loop remain unresolved by these aggregate spans.

Every timed lane visits all 64 rows, with all 64 baseline-eligible, no early
exit and matching eligibility masks. Each proposal therefore repeats the
same 64 baseline rows across 16 candidate lanes: 1,024 baseline evaluations
for 64 distinct rows. Each timed cell contains 3,072 baseline and 3,072 virtual
calls. Each lane records 1,024 selected cells and 4,456,512 token-cell calls
for each baseline/virtual side, including the separate target-cell calls.
Baseline member distances are 262,144 per lane; virtual Route matches that,
while virtual VectorBlock retains the observed candidate-dependent counts
245,760 / 246,015 / 246,270 / 246,525 / 262,144. The unchanged virtual geometry
must remain visible to any later reuse design.

E is maximum absolute paired D−U event difference; S is maximum within-mode
range. A Route E/S = 56,098,632 / 26,453,613 ns; A VectorBlock = 54,717,773 /
24,760,254 ns; R Route = 57,197,754 / 1,267,578 ns; R VectorBlock = 56,000,000 /
2,031,250 ns. Every-lane threshold stability and the perturbation bound pass.
**no profiling-off control for the already-merged enclosing host/event instrument — E/S bounds only the incremental internal clocks.**

H4 remains deprioritized: propose wall minus counterfactual + chooser events
is 48,743–89,178 ns across the 24 timed records; chooser events are
9,535–29,279 ns. Individual non-wait host spans are at most 29,886 ns. The
seconds-long status-read span overlaps GPU execution and is not transfer
time. These observations do not justify allocation-first work or removing
status/epoch checks. H1's grid1/block16 serialization remains source-established;
its causal speedup requires a separately reviewed launch-only intervention.

## Complete dose and preservation

The [unaltered raw record](counterfactual-decomposition-record.json) is
255,683 bytes, SHA256
`a190547844b937508ed13731437063d226a7871f9800b928e52ec1b786197430`.
Its status is `complete`, faults empty. It retains 8 warmup and 24 timed
proposals, 16 full mode/batch accounting records, 2 resource admissions,
2 policy starts/summaries and the source-identity bridge. Each of 16 U/D pairs
agrees on all 16 losses, status and five proposal words. All 256 D terminal
lanes are retained, including 192 timed lanes. Raw 40-word traces are the
source for the share ranges above. Epochs stay 0; before/after snapshot hashes
agree for both policies; apply_calls 0; the original checkpoint/config bytes
pass their post-run checks. No rebound checkpoint is written.

Fixed shape: seed1, rows64, V4096/B4/C192/K16, Route read, the original
synthetic windows/targets. Per policy, batch0 is the excluded warmup and timed
pairs follow U→D, D→U, U→D; Route(0) precedes VectorBlock(0). Fresh identical
carried windows are used for each mode. No Stage1 controls were repeated,
no corpus or seed2 was introduced, and no fixture/kernel/launch was changed.

Both resource admissions match the exact loaded cubin reports: U registers
88/88 and local/stack 21,536/21,536; D registers 126/126 and local/stack
22,096/22,096; shared0, AOT spills0, max launchable block640/512. The unchanged
zero-delta, +1024-byte local allowance and block16 gates pass. These attributes
are not occupancy or runtime-spill measurements.

## Authority and identities

The design [FIT6219037898](https://github.com/UOR-Foundation/uor-r4/pull/2545#issuecomment-6219037898),
protected image-binding repair #2556 (`afd6f721a`) and independently approved
[Stage1 pass #2557](../counterfactual-controls-pass-2026-10-16/README.md)
(`223eb0a5f`) precede the [independent readiness review](readiness-verdict.md)
and [distinct release6225716504](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6225716504).
The [release text](execution-release.md) published complete final manifest
and pin bytes before rental. Only the manifest's null release reference changed
from prospective SHA256 `a72939ca2e1413ac74b1c9d92d7307e6ee9f62b983cad6393ecefe1b45c32303`.
The decomposition pin remained byte-identical through review and execution.

| Identity | SHA256 |
| --- | --- |
| [Execution manifest](execution-manifest.json) | `7a910f8603073197df30cd78e57e163a28a569c1e596eeae690bcd0696f8fe5c` |
| [Decomposition pin](decomposition-pin.json) | `2db6dbba039bd90c544c3b9b25ac15abb935301608b2e40b92c134c75b3a3a34` |
| Archived executable | `02f35afde79e913170ed7ff73f6d7dd58e5d98485f82f9f7fa6b184227c5cc91` |
| Original checkpoint | `9a76743e401759b7d3aabc708ab0a26a9470ea474dc140f328c7b1aab59e49e4` |
| Original configuration | `eff7a77e1c3c8454324c9ddf31a6c08bdbe1b213d3324fe7b7889c0fe7665ae4` |

Complete [compiled identities](execution-identity.json) bind the same
`compiled/uor-r4-discrete-63b6a13f528115e7` sm_120 cubin/report set as Stage1.
Construction used Rust1.97.1/CUDA12.8.93 at c3908df4f, whole-tree equal to
protected afd6f721a. Crate/instrument bytes remain unchanged through the
Stage1 receipt. The bridge changes only source_sha256 in memory and records
unchanged non-source payload SHA256
`2c2490a00545fc58c35fb5c3756382b7eb2fc14586e43ae85103019ca6aeaab9`.

Calibration addendum6222721133 stays disclosed: uncapped zero-spill D126/22,096
and194/21,536 were observed under the superseded register bar, both meet the
amended static envelope, and no runtime timing comparison selected between
them. Capped56/56-spill build is superseded and excluded. No rebuild or tuning
occurred after Stage1.

## Operations and next decision

One owned Vast RTX5090, `vast-56305593`, session
`codex-cf1-decomposition-run1`, quoted $0.536/h. Local 32,504,672 KiB and remote
157,258,236 KiB passed the 30GiB floor; fleet/rate admission passed. SSH accepted
the existing key after 138s. Runtime driver580.95.05 is recorded; do not attribute
cross-host timing differences solely to source. The single command ran
22:17:45Z–22:21:46Z and exited0. This 241s includes setup/checks and the diagnostic,
not a training-rate measurement. [Transcript](command-transcript.txt).

All 26 [remote manifest](SHA256SUMS.remote) entries, 18 operator entries and
5 input entries verify; final input hashes also pass after execution. Full archive
`icloud:UOR-R4/results/codex/decomposition-run1-archive.tar`, **26,301,440 bytes**,
MD5 **`d02a60124645611ea82720f1ff99513b`**. Cloud MD5 readback preceded release
22:22:44Z and deletion22:22:52Z; fleet0 and operator exit0 verified. Unknown/live
state would have retained the lease. Prior protected delivery cleanup/hygiene
completed before this release; peer material was retained.

Ledger uptime492s × $0.536/h = **$0.073253333 estimate**, including startup and
preservation. Cumulative 29-lease Codex estimate **$3.174466388**, plus separately
recorded PM cost; construction/startup subtotal $0.741212499 unchanged. These
are estimates, not invoices; storage/transfer USD UNMEASURED.

**Next bounded work:** after independent outcome review and protected receipt,
pre-register GPU baseline read/writer/eligibility once per exact row and reuse
it across the 16 candidate lanes of the same proposal. Keep the existing native
writer and virtual computation unchanged, preserve model binding/epoch/policy/
carried batch/targets and every eligibility/error/tie rule, and compare all
losses/status/proposal words before measuring any saved time. This is the
smallest direct removal of the observed candidate-invariant repetition.
Membership indexing is also supported as a later writer-focused intervention;
it must preserve crowded-cell tokens, exact order and virtual endpoint changes.
Do not combine its claimed savings with baseline reuse or a launch redesign.

This sequence implements the owner's same-native-math/minimal-surface direction:
remove redundant evaluations of the existing geometric operators in CUDA,
without changing the model law or introducing another learning representation.
No optimization source or new execution release is included in this receipt.
