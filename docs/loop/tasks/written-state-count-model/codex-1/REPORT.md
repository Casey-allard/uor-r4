# REPORT: written-state-count-model, codex-1

Lab: codex. Order: [ORDER.md](ORDER.md).

## What did not work, first

On each of the two released M1 artifacts, all 16 native replies diverged at generated token 0. Selecting the true value occurrence while retaining the production +256 pointer boost repaired zero openings. Forcing the true token ID immediately before the geometric writer repaired all 16 openings, then lost the second token of every requested `red kite`. After a forced full answer, the artifact reliably appended punctuation before EOS. These are three distinct decisions. A source-selector repair alone cannot fix the first one on this panel.

## What changed

Two research instruments expose the first divergent forward decision in the released D11 writer and compare address functions in equally sized next-token count tables. Neither changes the shipped artifact. The first instrument adds trace and intervention modes to the existing AOT scorer. The second uses GPU integer counts, model freeze and mass calculation, with the same native integer bits metric across arms. The task scopes intersected another lab's open work at a directory glob. The function and source files used here are distinct. No numerical size estimate was in the ORDER.

## Measurements

**Role swap.** Sixteen prompts comprise two value families × two assignments × two fact orders × two questions. Both question variants within a pair share the facts. Every released file was checked by SHA-256 against the same M1 release. Both have the Relation read, eight kept leaves and one sweep. Identity-frame Pick changes only the pointer choice; the trained Relation read still computes its original features. All positions below are zero based. [Panel and tokenizer key](panel-key.json) · [Registered interventions](ROLE-SHEET.md) · [FIT and release](ROLE-FIT.md).

In seed 1, the prompt “Ann chose red. Ben chose blue. What did Ann choose? Ignore Ben.” ends at token position 30. Red is source position 7, ID 677. Orbit predecessor 3 selected successor 4, ID 2241 (` Ann`). The native mixer contained red with score 682 and selected ID 1155 (` G`) with score 59,826. Red's cell 71 missed the eight admitted cells. The model emitted 1155 and fed it back. Changing the question to ask for Ben changes the expected ID to 924 (` blue`, occurrence 12). The selector still chose position 4 and the model generated the same complete ID sequence. Keeping the Ann question and forcing the red source instead changed its score to 938, with winner and emitted ID unchanged. Keeping the native selector instead and overriding only the ID handed to the geometric writer produced 677.

A static read of seed 1 reconstructed 40,512 of the winner’s score from 633 counts at the joint key (last token 28, read code 9241), versus 576 from nine counts for red. The last-token-only term supplied another 13,952 versus 88. Read-only, last-token/recurrence and read/recurrence terms account for the remainder. The complete breakdown is in [the role diagnosis](ROLE-RESULT.md).

Across both seeds the native selector chose the wrong occurrence on every opening. Even after the asked-source boost, the winning mixer margin over the correct token ranged from 16,196 to 58,888 (seed 1) and 14,532 to 56,077 (seed 2). The direct asked-writer control produced the correct first ID on 16/16 and the entire value span on 12/16 per seed. For all four requests for kite, ID 1999 was second behind 878 (`ak`) with cell 9 already admitted. Each seed assigns those two IDs identical 32-component codebook rows. Their row hash is f6c6124e9d2dc248ce741dc76ff2954bf872cad2ea49fbe3940069bea7e5e8b3. Equal cell, equal distance, and the deterministic lower-ID tie rule prevent the larger ID from winning. A post-hoc static census per seed finds 4,008 distinct (row, cell) pairs for 4,096 IDs, 87 collision classes and 88 IDs dominated by a lower-ID alias. This is a lower bound on unreachable token types, independent of their corpus frequencies.

Forcing the full asked span and releasing generation gave period then EOS on 12/16 and exclamation mark then EOS on 4/16 per seed. That fails the registered immediate-EOS scoring rule. This experiment provides no general prose or reasoning score. All ten arms scored 0/16 exact replies per seed under that rule.

**Role reproduction.** On Wick with an sm_89 AOT build, run `d11-scorer role-swap inputs/seed1.d11 inputs/panel.json seed1.json`, repeat in a fresh process, and substitute `seed2.d11` for the second artifact. Seed 1 repeats byte for byte. The [private raw trace, timings and checksums](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/3dc8a08a53cef4606861ac97f19046021dd6b2d3/results/codex/written-state-diagnostics-20261118/role-01.tar.gz) is pinned and download-verified, SHA-256 c6a3e78b7ca5dae23eaf943ecbb979ee4317b2bbba4cf8e9dc0fbf5ebb36356b. Per-seed wall times were 1.16 and 1.13 seconds on shared Wick. They describe these diagnostic jobs only.

**Equal-byte count.** Wick trained 15 direct-address arms on 1,047,021 D87 TRAIN tokens in 483 complete documents. The 523,023-token DEV-tune prefix held 62 even records for selecting suffix order and 61 odd records for comparison. The latter cover 265,125 targets and 539,966 decoded bytes. Budget per image was 131,072 bytes, including the 32-byte header and 16,384-byte unigram. The two geometric controls used 8,224 bytes for keys and had 3,326 four-entry rows. SplitMix and the 12 suffix arms had 3,583. Each used a four-successor count head with the same smoothing. The matched re-seeded control permuted the root actions while preserving their multiset. [Count design](COUNT-SHEET.md) · [FIT](COUNT-FIT.md).

| Address | All odd BPB | 13-predecessor novel BPB | Response-novel BPB | Transport minus comparator, ALL BPB and paired 95% interval |
|---|---:|---:|---:|---|
| Two-frame transport | 4.606925 | 4.734169 | 4.791204 | — |
| Re-seeded group | 4.609493 | 4.729193 | 4.786368 | −0.002568 [−0.009771, +0.003856] |
| SplitMix | 4.559509 | 4.757186 | 4.814794 | +0.047415 [+0.007325, +0.094760] |
| Ordered suffix, k=1 selected on even records | 3.844426 | 3.953841 | 3.991524 | +0.762498 [+0.700148, +0.833649] |

The secondary subsets contained 243,576 targets / 496,749 bytes and 227,288 targets / 456,629 bytes respectively. Novelty is relative to this TRAIN prefix. Reported BPB and paired differences are truncated independently. The sign of the ALL difference favors the one-token suffix and SplitMix. The registered ≥200 odd-record gate also failed. Verdict is descriptive. This specific key and four-entry head do not earn scaling. Distinct failure explanations include coarsening of two frames, hash collisions, key occupancy and the limit of four successors. The controls do not identify those contributions individually. The run took 63.93 s including verification and measurement; peak RSS of 871,064 KiB exceeded the 512 MiB planning estimate. The fixed model-byte comparison is separate from training and evaluation memory. The run was `equal-count inputs/c420 inputs/seed1.d11 results/count-01/sweep/results.json`. No DEV-test records were used.

The reporting replay reproduced every initial field, mass and frozen-image digest. The transport and re-seeded arms each occupied 892 rows, retaining 161,686 and 160,482 TRAIN events among their four successors per row. SplitMix occupied 3,583 rows and retained 193,079 events. The selected suffix occupied 2,228 rows and retained 442,129. This suggests a test of how many distinct successors each address must represent. Budget alone did not equalize effective use of the table. [Original aggregate results](count/sweep.json) · [All per-record sufficient statistics](count/replay.json) · [Private count evidence bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/3f422acad4008a0d16cdc6a93444fc69ffddf059/results/codex/written-state-diagnostics-20261118/count-01.tar.gz). Bundle SHA-256 31bf6538fd47f30a63b2011914d97c13a780dee098b442c69d944416eb40473a.

## Checks run at the head

The Wick role diagnostic recorded 2,275 independently checked source selections, 1,387 Rust production forward comparisons on native-mode steps, 345 CUDA changed-label comparisons and a byte-identical fresh-process seed-1 replay. A fixture with one altered whitespace byte was rejected before scoring. The standalone scorer's host tests passed 26 with one ignored. See the [per-arm and per-step record](ROLE-RESULT.md). The count instrument passed device/host finite image and Q32 parity and the positive-mass and normalization controls before empirical scoring. Both teams used the same Wick 4090 under serialized Codex research jobs. Bounded finite checks, native oracles, and held-out language experiments have different denominators.

## Skills used

Native-space-cycle and UOR research architect supplied the native representation and experiment-chain checks.

## Still open

1. Carry the selected occurrence, exact S0 token ID and a bounded span cursor through one explicit copy decision in the research path. Test source-oracle and free-selection modes, swap roles and disable copying, then check shared-opening values through release back to free decoding. Require an effect controlled by the asked role. This is a proposed next experiment.
2. Test the registered written-state P1 GPU/serving count replay before interpreting the full P2 comparison. The bounded count result leaves the canonical S3 key, E8 miss path, error-written rows, lossless end-to-end artifact and broader quality/cost bars open. M2 conversation, coding and reasoning have separate evaluations.

## Plain summary

Swapping the names in the question changed the geometric read code. The model still selected the same place in the prompt and began with the same wrong token. Supplying the right place was insufficient because the learned scores still favored that token. Sending the right ID directly to the writer repaired the first word. For one two-word answer, the writer then confused two tokens with identical coordinates. Preserving the exact token ID through a deliberate copy would let the next test measure whether the model can learn when and where to copy. In the second experiment, using the previous token to look up counts was more predictive than the two-frame geometric address at the same storage budget.
