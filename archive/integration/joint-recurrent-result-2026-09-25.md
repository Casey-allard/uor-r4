# D8 rung 1 result: learned recurrent memory with geometric transport

September 25, 2026. References #973 and #820; delivery: [PR #1389](https://github.com/UOR-Foundation/uor-r4/pull/1389).

## Decision

**Accept the frozen rung 1 engineering continuation gate for these two offline
artifacts.** Both improve their retained training loss, beat the selected
count/cache development baseline, generate varied text after reload, and show a
substantial likelihood penalty when the combined read/copy path is disabled.
The actual generations also contain semantic drift, role confusion, malformed
words and repetition. This is not useful general conversation, coherent-story
qualification, general reasoning, coding, or alpha capability.

The quaternion arm does not beat its matched ordinary recurrent control on
natural likelihood. One paired seed and the mixed source-edit outcomes do not
establish a geometric advantage. The next constructive step is **D8 rung 2:
quantization-aware continuation of this same learner**, retaining both selected
checkpoints and the ordinary control. Further A1–A4 selector tuning stays parked.

The [frozen campaign](joint-recurrent-campaign-2026-09-24.md) owns the criteria.
The [compact evidence](../evidence/joint-recurrent-result-2026-09-25.json) contains
actual generated text, all source-edit outcomes, artifact identities, joined
likelihoods and descriptive position slices. The [selection receipt](../evidence/joint-recurrent-selection-2026-09-25.json)
was recorded before population evaluation. The [closeout](../evidence/joint-recurrent-closeout-2026-09-25.json)
separates measured wall time, overlapping process time, retained storage and
unresolved gross compiler allocation.

## What was learned and what remains floating point

The Rust learner has 1,678,466 parameters, vocabulary 4,096, state width 256,
read width 64 and context 256. A single language-loss graph trains token encoding,
transport, recurrent state, causal contextual Q/K/V writes and reads, and a
normalized vocabulary/pointer mixture. There is no transformer backbone or
runtime teacher. Quaternion transport is in the actual prediction graph; it is
not merely metadata on an ordinary token prior.

These are still **offline continuous models** with floating-point affine maps,
soft reads and nonlinearities. No integer export or terminal parameter-sparsity
claim is made. Prime/zeta admission, exact H4/icosian execution, a mutable
Hamiltonian, long-lived exact session memory, and physical energy savings are
not demonstrated by this run. The Householder-pair comparator matches parameter
shapes, initial arrays, sample schedule and local rotation scale; the two global
transport families are not identical.

## Completed exposure and preserved learning

| Quantity, per arm | Measured value |
|---|---:|
| Global optimizer steps | 7,324 |
| Total sampled target visits | 29,999,104 |
| Retained 64-token warmup | 8,617,984 visits / 2,104 updates |
| Corrected 256-token phase | 21,381,120 visits / 5,220 updates |
| Final training batch / context | 16 / 256 |
| Selected checkpoint | Final, step 7,324, in both arms |

The owner correctly challenged the initial 64-token training / 256-token
evaluation mismatch. That discrepancy was a **training horizon**, not the
64-coordinate read width. Safe checkpointing and 56 quaternion alignment updates
preserved a matched warmup. The explicit horizon transition retained weights,
AdamW moments, clocks and total dose; its changed batch/window dimensions define
a declared new sampling phase. No claim that all 30 million visits used 256-token
windows is made.

Two subsequent disk-guard stops also preserved complete optimizer checkpoints.
Both arms ultimately reached exactly the same final step and dose, continuing
their own unchanged counter schedule. All selected/rejected profiles and stopped
attempts remain available. An independent execution audit reconciled all 15
modern resume records and their parent hashes. All 23 retained fitting reports
account for 60,440,576 gross target visits, including 442,368 off-lineage profile
visits; those extra profiles are resource costs, not extra selected-model dose.

| Selection / fit measure | Quaternion | Ordinary Householder pair |
|---|---:|---:|
| Step 4,714 recorded tune NLL | 2.288454473 | 2.263618886 |
| Step 7,324 recorded tune NLL | 2.213686883 | 2.182432771 |
| Last attempt retained loss, before → after | 2.536506653 → 2.224424839 | 2.502806425 → 2.200559616 |
| Save/reload retained-loss absolute difference | 0 | 0 |

Checkpoint choice uses the recorded F32-origin tune score, with an earlier-step
tie rule. Both final checkpoints were selected at 05:54:34 UTC; all four
population evaluations started afterwards. Later F64 row reductions do not
reselect checkpoints. Retained-batch losses across the horizon transition refer
to different samples and are not subtracted as a same-batch learning measure.

## Natural likelihood and combined read contribution

The unchanged evaluator scores 249,856 targets in 976 blocks of 256. The first
64 blocks supply selection data; the comparison tail has 233,472 targets in 912
blocks. These are **previously exposed development data**, including historical
reference checkpoint selection, not a fresh final test.

| Same comparison-tail population | NLL, nats/token ↓ |
|---|---:|
| Historical #1017 offline transformer reference | 1.574024 |
| Ordinary recurrent learner, read enabled | 2.085241 |
| Quaternion recurrent learner, read enabled | 2.110368 |
| Count model plus causal cache | 2.391786 |
| Count model alone | 2.405627 |
| Ordinary recurrent learner, whole-prefix NoRead | 2.561712 |
| Quaternion recurrent learner, whole-prefix NoRead | 2.592991 |

Quaternion improves on count/cache by **0.281418 nats/token**; ordinary improves
by **0.306545**. Quaternion trails ordinary by **0.025127**. The historical
transformer has different capacity and training exposure, so it is a reference
quality anchor rather than the matched geometry control.

Disabling reads increases NLL by **0.482623** for quaternion and **0.476471** for
ordinary, exceeding the frozen 0.02 threshold. This intervention removes **both
recurrent value feedback and pointer-copy probability** from the beginning of
the prefix. It establishes their combined contribution on these artifacts; it
does not isolate value feedback, geometric transport, or distant retrieval.

The Rust comparison exactly joins all six target streams, rejects identity or
population mismatches, and reproduces all 21 reported means with maximum absolute
difference 1.78e-15 nats. Native binary rows omit input IDs: input population is
bound by evaluator/checkpoint identities and shifted target records, not a
native per-row input-ID column. No significance test or independent-target
assumption is inferred from the paired row counts.

Positions 64–255 are now inside the trained horizon. Quaternion NLL there is
2.037855 enabled / 2.582316 NoRead; ordinary is 2.011926 / 2.549717. These are
descriptive slices. A later prediction position does not by itself prove that
the prediction retrieved information more than 64 positions away.

## Actual generation and source edits

All five historical prompts were generated from each selected loaded model with
seeds 2014–2018 and the frozen sampler; the same prompts were also generated with
NoRead. The principal and independent scientific reviewer read all actual text.
Read-enabled outputs are nonempty, varied and not constant or short-cycle token
collapse. Quaternion stops by EOS on two of five prompts and by the 128-token cap
on three; ordinary stops by EOS on one and by the cap on four.

Quality remains limited. The quaternion goose continuation changes to unrelated
Tom characters; its bicycle story drifts into toys and a mirror. The ordinary
goose output keeps the goose but confuses pronouns and introduces an unexplained
ball; its train story invents “Mave.” These observed failures remain visible in
the evidence, alongside successes. Passing the limited noncollapse gate does
not qualify sustained coherent prose.

| Frozen source-edit measure | Quaternion | Ordinary | NoRead, either arm |
|---|---:|---:|---:|
| Correct first noun | 28/32 | 31/32 | 0/32 |
| Exact noun-plus-period completion | 28/32 | 23/32 | 0/32 |
| Both variants completely correct | 14/16 pairs | 10/16 pairs | 0/16 |
| Output changes after source edit | 14/16 pairs | 16/16 pairs | 0/16 |

Quaternion chooses the distractor “door” for both doll/bear and box/bag pairs.
Ordinary usually retrieves the intended noun but sometimes keeps speaking, such
as “bell to the kitchen.” Its cap case produces “captain.” Thus quaternion's
strict completion lead is not a lead in first-noun retrieval. The paired edits
show meaningful source sensitivity on this finite panel without establishing
general task transfer.

These prompts contain only 56–62 input tokens; relevant nouns occur roughly
48–54 positions before the answer. They do not witness retrieval beyond the old
64-token horizon, exact 256-step recall, arbitrary version tracking or general
conversation memory. Their prompts and checks were frozen before fitting; no
probe-specific fitting or answer repair was performed.

## CPU/GPU execution and storage correction

RDC launched concurrent processes and independent reviews on the same local
8-core, 16-GiB Apple M1 Mac. It supplied no additional machine. Measured complete
updates selected **two CPU sequence-gradient workers per arm, two arms at once**,
with nested backend thread settings at one. Full 256-token sequences remain
intact; weighted mean gradients precede one global clipping/AdamW update.
Reduction order may differ from an unsplit batch; no bitwise trajectory claim.

Short profiles measured about 1,419/1,575 targets/s for two workers per arm,
1,333/1,479 for four, and 823 for the tested quaternion Metal run. Eight total
gradient workers were slower. Sustained final fitting measured **1,362/1,365
targets/s**; the final uninterrupted phase lasted about 4 h 17 min. These are
training throughput figures, not serving speed or physical-energy measurements.
The final phase's peak sampled combined RSS was 6,455,672,832 bytes (6.01 GiB),
below the 8-GiB aggregate stop.

The earlier 26.1-GB disk stop was unnecessarily restrictive while about 25.2 GB
was free and remaining outputs were projected at 200–250 MB. The principal
corrected it prospectively to a 20-GiB physical reserve plus 128-MiB stop margin
and 64-MiB checkpoint headroom: **21,676,163,072 bytes**. Training then completed
without another resource stop. Claude VM activity and macOS swap changes were
concurrent observations; the entire disk change is not attributed to either.

Inspected inactive compiler intermediates reclaimed 3,652,976,640 measured
physical bytes in the main recovery, with smaller cleanups separately receipted.
All learned models, executed binaries, reports, source and worktrees remain.
Retained artifact size is measured separately from gross build allocation;
overwritten/deleted compiler outputs are not fully reconstructible. **Exact
gross new-storage compliance is UNRESOLVED**, rather than inferred from current
free space. This accounting gap does not invalidate the witnessed model results
and is not a reason to rerun training. Inventory new build writes before the
next campaign; do not silently reset the cumulative ledger.

## Verification and next dependency

The executed release source is `ad4e639fedecf9d490035f9986be736117bd3e29`;
binary SHA-256 is
`f68d654294607df7891e300d572d08bffa772606674c80cdaad2b95c7d122238`.
Both final checkpoint, model, optimizer, data and evaluator identities were
independently reconciled. The complete final fit roots, four evaluations and
joined comparison are sealed; the comparison's file set was verified by Rust.

Eight focused release checks cover causal normalization, shared incremental
execution, complete history gradients, transport arithmetic, all Q/K/V/state
language gradients, data-cursor continuity, AdamW restart, and every named
gradient under two/four CPU batch partitions. The pinned offline Candle backend
retains its documented four-line Accelerate slice-length correction. This does
not change the portable serving crates' safety contract. Queue acknowledgements
execute no tests and are not counted as these checks.

Proceed through the existing ladder:

1. Specify the quantized forward graph, scales/gains, rounding, saturation and
   declared backward surrogate for this same student. Preserve exact token and
   occurrence identity. Cover transport, state, reads and output rather than
   hardening only an isolated selector.
2. Continue the selected quaternion and ordinary artifacts with matched exposure
   and a retained continuous comparison. Freeze numerical language-retention,
   source-edit and cost gates before fitting. Measure relaxed-versus-hard loss
   throughout, and inspect actual loaded hard-path generation early.
3. Apply the actual bounded admission masks and typed geometric transport in
   rung 3; measure admission separately from ranking. Separate copy from value
   feedback and use controlled distant-source examples before making stronger
   memory claims. These are integrated comparisons, not a new architecture sweep.
4. Complete integer/table export and terminal parameter-access/energy checks in
   rung 4. Final serving still excludes floating-point matrix multiplication,
   floating nonlinearities and a transformer backbone. A low-bit affine map
   remains a mathematical linear map and its actual parameter access must be
   reported under D0-b/D5.

Rung 2, integer serving, fresh final evaluation and multi-seed geometric
attribution are **NOT_RUN** in this result. #973 and programme #820 remain open.
