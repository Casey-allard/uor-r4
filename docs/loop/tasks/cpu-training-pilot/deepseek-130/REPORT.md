# REPORT: unseen-key target prototypes, deepseek-130

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[point prototypes](../../../harness/prep/cpu-training-pilot-point-prototypes-2026-11-04.md).
Base: `6f653f2b4`.

## What did not work, first

- **The fixed `k = 2` target-prototype construction failed the registered
  two-seed bar.** On seed 1 it scored 0/60 unseen-key openings, the same as
  point-off and worse than its shuffled-target prototype control at 3/60. On
  seed 2 it scored 3/60, beating point-off by three and the shuffled control by
  three, but both gains were below the pre-registered paired threshold of four.
- **The two-point local construction did not restore the raw-neighborhood
  result.** Raw nearest scored 2/60 and 1/60, and the existing eight-row vote
  scored 3/60 on both seeds. Replacing 48 raw rows with 25 prototype rows lost
  signal on seed 1 and produced only the same 3/60 opening score as the vote on
  seed 2.
- **The complete teacher-forced non-loss condition also failed on seed 1.**
  Prototype / point-off / shuffled full top-1 was 169 / 169 / 172. Seed 2 was
  166 / 163 / 163, which passed only that secondary non-loss condition.
- **This is not a general point-geometry negative.** It closes this fixed
  target-balanced medoid-plus-farthest-residual construction, the fixed
  60-example training panel, the 60-opening evaluation panel and this Q24 point
  representation. It does not reject other local neighborhoods, other point
  populations or GPU qualification.
- **The second pre-run review did not establish exclusive launch freshness.**
  Its `GO` is confirmed for the exact head `b6652aaa4`, but the reviewer saw the
  builds start during review. The first launch had already stopped harmlessly
  in preflight because a JSON number was compared as a string; `b6652aaa4`
  replaced that check with one typed `jq` predicate. Exact source, input,
  binary, output and repeat gates passed here, so this is a recorded process
  risk rather than evidence that the run fields are wrong. It nevertheless
  falls short of proving that all build work began after the review completed.

## What changed

The point-transfer driver now derives a deterministic target-balanced prototype
table. For each supervised first-reply token it:

1. selects an observed medoid by exact within-target L1 sum;
2. selects the remaining distinct observed point with maximum minimum L1
   distance to the selected set;
3. breaks every distance tie by signed lexicographic point order and then
   target id; and
4. retains at most two rows per target, refusing malformed, one-target,
   unchanged or non-reducing tables.

The run compares those prototypes with point-off and with a duplicate-safe
shuffled-target control that preserves the treatment point multiset and target
counts. Raw nearest, raw eight-row vote, one medoid and shuffled medoid remain
descriptive controls. All arms use the same derived Hamming/turn memory and
the complete CPU/integer served-forward path.

## Measurements

Both seeds used the fixed 8-key train / 8-key evaluation split, 54 training
documents / 50 evaluation documents, 60 training samples and 60 held-out
samples, with zero held-out exact-memory hits. Training produced 48 raw point
rows and 25 prototype rows covering 18 distinct target values.

| Seed | Point-off | Raw nearest | Raw vote k=8 | Prototypes | Shuffled prototypes |
|---:|---:|---:|---:|---:|---:|
| 1 | 0/60 | 2/60 | 3/60 | **0/60** | **3/60** |
| 2 | 0/60 | 1/60 | 3/60 | **3/60** | **0/60** |

| Seed | Prototype vs point-off gain / threshold | Prototype vs shuffled gain / threshold | Full prototype / point-off / shuffled | Registered pass |
|---:|---:|---:|---|---|
| 1 | 0 / 2 | -3 / 4 | 169 / 169 / 172 | no |
| 2 | 3 / 4 | 3 / 4 | 166 / 163 / 163 | no |

Opening target-distance sums were exactly equal across all arms:
64,821,741 on seed 1 and 80,445,779 on seed 2. Every point-bearing arm changed
all 60 treatment openings relative to point-off.

All eight arms on both seeds and the seed-1 repeat retained zero mismatches over
**2,110 positions, 540,160 Q12 query coordinates, 2,110 token decisions and
16,880 kept cells per seed**. The seed-1 repeat matched every non-timing and
non-resource report field, normalized to SHA-256
`1cfb09489442b2c3888e8a13b845deb57c5dd5ab7c7076551982aee294b496a1`.

Evaluation walls were **1.884 / 1.905 s** and peak RSS was
**309,096 / 307,028 KiB**. The repeat took 1.891 s at 307,020 KiB. Two isolated
16-job release builds took 36 s together, produced the identical binary
SHA-256 `5f868792c4ec9e6e502170a242bde9ebea89a9f2335c9badacec8e405061812c`,
peaked at 1,210,192 KiB and occupied 519,558,068 target bytes. The retained
pipeline occupies 52,970,997 bytes, below the 67,108,864-byte limit.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary identity, build receipt,
three reports and every copied artifact. Raw run roots remain under
`local/deepseek/point-prototypes/2026-11-04-attempt-1/`.

### Accounting

The prospective 1,320-second admission covered 600 seconds of preparation and
review, 300 seconds of build, 120 seconds of controls/evaluation and 300
seconds of delivery. Measured build plus evaluation was **41.680792822 s**.
Preparation/review, validation and delivery elapsed time were not instrumented
and are left explicitly unmeasured rather than imputed. The ledger preserves
the admitted projection and records the measured compute separately.

## Checks run at the head

- Fresh non-author TEST FITNESS: FIT; the result was recorded at
  `4cbdf2a60`.
- First pre-run review: GO at `4cbdf2a60`; its first public launch stopped in
  preflight before evaluation.
- Corrected recipe and second pre-run review: GO at `b6652aaa4`, with the
  launch-freshness limitation stated above.
- Full `uor-r4-cpu-train` release suite: **243 tests passed, 0 failed, 4
  expected ignores** (198 library and 45 binary tests).
- Point-transfer binary suite: **21 passed**, covering prototype selection,
  farthest residual and tie rules, target balance, shuffled-target
  preservation, point fallback and exact CPU/serving parity.
- Both public seeds and the cross-build seed-1 repeat completed. All per-root
  `SHA256SUMS` passed, the repeat's normalized report diffed clean, and all
  artifact and report hashes matched the evidence record.
- D23, D25 and D32 ratchets remained unchanged.
- Artifact source matched implementation pin `34103fab4`; all five public input
  hashes matched.
- `prep-check` reported this sheet FIT and complete.

## Skills used

No system-provided Codex skill applied. The repository's experiment,
evidence-record and delivery rules were followed directly.

## Still open

- A target-local representation that preserves the useful raw-neighborhood
  evidence while reducing noisy labels without post-observation tuning.
- Complete replies, fresh phrasing, corpus-scale training and GPU
  qualification.
- Claude's separately owned long-window export and the CPU served-forward audit
  that will consume it.

## Plain summary

This unit tested a materially different point representation: one observed
medoid plus one farthest local residual for each supervised target. The table
was built deterministically, changed every held-out opening and preserved exact
CPU/integer serving parity. It nevertheless failed the registered comparison
on both seeds: seed 1 lost to its shuffled control, and seed 2's three-row gain
was below the required margin. This closes only the fixed two-prototype
configuration; it is not a useful-chat or GPU-qualification result.
