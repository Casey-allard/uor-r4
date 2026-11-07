# Prep sheet: short reply-masked SFT from (b64L) (one-model-end-to-end, deepseek-22, 7 Nov 2026)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [deepseek-22](../loop/tasks/one-model-end-to-end/deepseek-22/ORDER.md).

TEST FITNESS: PENDING (fresh non-author review required).

PRE-RUN: PENDING (fresh D67 review required).

**Why.** The current (b64L) model clears the human-text panel but serves
0 complete replies and 0 EOS stops on all 256 `m2-fresh256` rows. D3 scores
EOS at 756 per million against DEV's 85,812. Under `per-turn/v1`, the current
sealed artifact puts reference EOS at median rank 7 and in the top 8 for
198/256 rows, but never at rank 1. This run tests the smallest change aimed at
that result: continue the exact weights on assistant-reply/EOS-masked dialogue,
then require both ranking and free-generation movement before scaling.

This is a developmental training run. It does not change a milestone bar or
substitute for a GPU qualification run.

## Question

At 1,024 optimizer steps from the exact (b64L) seed-1 checkpoint, with loss only
on assistant responses and EOS, does the exported integer model move EOS to
rank 1 often enough to terminate at least 16/256 held-out replies, while
preserving its human-continuation score?

## Base artifacts

Seed 1 is the training arm:

- store: `hf://datasets/caseyallard/uor-r4-store:results/claude/one-model-end-to-end-d3-accum-2026-11-06/runs-2/s1`
- archive `d3-accum-s1.tar.gz`:
  `f7852f2540e58c17b2311d39fb47f8e53471e2cda8d6136f81b034cdf6bf5bba`
- checkpoint: `oh/run/b64L-s1/models/step-016384`
- run config `oh/run/b64L-s1/config.json`:
  `6b2b84640e37d1f6322a5b4ab6db0d783469d7adf4db1c7e67b532d4310056f2`

| checkpoint file | SHA-256 |
|---|---|
| `model.safetensors` | `6fd2d764171cc416d5ff272e19e3ebe0644183ab9c43ca0662706756ac75f5de` |
| `config.json` | `1045acb61d92d390f0f61617ffa4c1847437473c9694c09787b48bf46e761ed2` |
| `hamming_read.safetensors` | `672de97fcc329b10c3cde591f66047352851f9fe1d02df85a168190f5ba12ddd` |

Config: width 256, context 192, six recurrent layers, four Hamming heads,
objective `nearest`, `train_as_served: true`, one-atom SpiralCore/FBS arm.
The completed seed-1 run records `dev_top1_hits=192` (writer head) and
`dev_nearest.hits=3,185` (the served nearest head) on DEV.

Seed 2 is pinned for the conditional replicate, not run in this unit:

- store: `.../one-model-end-to-end-d3-accum-2026-11-06/runs-2/s2`
- archive `d3-accum-s2.tar.gz`:
  `1f24f03a4b638f7ad6fafd184f980bca324de758085a62e5b5d0dfb3a4f6ab32`
- run config: `f21567d0b43ba5fcc444bda840ad77bf8a5e9b46315ed8e515d223619f15e593`
- checkpoint `config.json`:
  `778145167a18ff3b1006d4f609b7f608cc528b0dd753279b3ff38c8c63a88b95`
- checkpoint `model.safetensors`:
  `13f3ae6230831ad9797224c0d20ecb6d15463fc28a3fcbea0ddfd9f0046c3306`
- checkpoint `hamming_read.safetensors`:
  `3de5f52daa5d48504c3bba757e6ede95dd6a31bcba42653c5d12a6c1c5c35c7d`

`train-in-space --init-from` must load exactly the selected seed's trunk and
Hamming read, write their hashes to `out/init.json`, and start a fresh
optimizer and schedule.

## Data split

The pinned tokenizer is
`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.

The reproduction chain is:

1. OASST2 reply-masked TRAIN, source revision
   `179dd21fc55192153d94adb0e0ce8f69e222bf75`, from
   `results/claude/real-text-corpus-2026-11-05/oasst2/train/`.
   `tokens.u16` is `8bdcabfb41284a54575ea2f730f532a896b29dc69250098957b46c6bcae40b90`,
   `response_mask.u8` is
   `a2b0b2a3ce8cc7491b8ced86eda487353ba4e70da0f9aa5a63161ca6802de7a1`,
   `manifest.json` is
   `d5e16e17a2e237bb1a705539cb38c9fa6e7b0fa68fc0ec72eaa667c8ec894e40`.
2. `m-world-recall-corpus world=v2 conversations=20000 seed=1 recall=oracle
   context=384 protocol=2 exclude=M2_EXCLUSIONS`.
3. `M2_EXCLUSIONS` is rebuilt by `m2-sft-exclusions` from the sealed M2
   `plan.json` and `labels.json`; it contains 96 sorted unique strings and hashes
   to `e0a96dd64c6da637f077c0833fb96408028102082467e1a0dbbeed1edf285c98`.
   The build rejected 0 exact user-turn draws and reproduced the non-excluded
   token/mask bytes exactly.
4. `mix-chat-corpus` concatenates OASST2 then M-World. The final store is:

| file | bytes | SHA-256 |
|---|---:|---|
| `tokens.u16` | 12,888,818 | `3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3` |
| `response_mask.u8` | 6,444,377 | `67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b` |
| `manifest.json` | 3,103 | `1c7aee4dfaffdde07f9c64a5b887b504ee21355038b6ae00897a8a97407d9bbe` |

The final mixed store is uploaded read-only at
`hf://datasets/caseyallard/uor-r4-store:results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/inputs/`;
commit `5c8765455433a669df13bd2a1677c051383d5397`, read back with all three
hashes matching. The pod must re-hash the three files before training.

| reading | value |
|---|---:|
| rows | 25,071 |
| tokens | 6,444,377 |
| response/scored tokens | 4,059,876 (63.0%) |
| EOS targets | 79,679 |
| EOS per million scored targets | 19,625 |
| replies | 79,679 |
| median scored reply length | 11 |

The M-World source alone has tokens
`3b7b556c8e45c8d90c28bc8dec74b4082620a15d87cc055a401f21630c3b7c35`, mask
`9665387299077f0560a657a2f5f8ace8605acc7292a27e7cf759d3b73b5b5543`,
manifest
`427279468476f307ded50ff5f9fe544f5c6840be76db53dd8539019ee14db9b8`,
71,895 EOS at 87,139 per million. OASST2 contributes 7,784 EOS and remains
79.7% of scored tokens.

`corpus-panel-overlap` decoded all 25,071 documents and 79,679 assistant
responses against all 96 M2 strings:

| overlap check | count |
|---|---:|
| exact plan/query/accepted string in a document | 0 |
| exact accepted answer in an assistant response | 0 |
| document sharing a panel word 6-gram | 0 |
| assistant response sharing a panel word 6-gram | 0 |

Audit SHA: `34a9646195cd2dd929d5ffcc779bbcbfe5157f55554f9894981ee225ba20f053`.
This supports the M2 exclusion claim for this store. A positive still unlocks a
matched-data control before the gain is attributed to M-World or the reply mask.

The human continuation guard uses OASST2 held-out, not DEV:

- `tokens.u16` `02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e`
- `response_mask.u8` `9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833`
- `manifest.json` `7474ba9331f242148dd9833d5cb085307c8e0d86314d13f285a0ab1c42b8b081`
- deterministic 512-row sample fingerprint
  `b724656f29bdade21aecb372d0568c48c81d78c80f43fe4de8ec667a7824ccc4`
- base integer nearest top-1: **132/512**
- guard floor: **93/512** (70%, rounded up)

The in-run DEV split is pinned at tokens
`0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`
and mask
`a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
DEV `dev_nearest.hits` is recorded, but it is not the human-retention gate.

`m2-reply512` is `NOT_RUN`: its content remains unavailable. No substitute is
scored under that name.

## Recipe

```text
train-in-space \
  --data-root $DATA \
  --init-from $B64L_S1/oh/run/b64L-s1/models/step-016384 \
  --train-split $MIX \
  --train-sha256 3a2c397e4b1a13c488096f931cf377efaeed51d2de17c231ac72d4b6291899e3 \
  --train-mask-sha256 67af9f09152510edba99795f3ff247a6f17eb80c3907836cd7c0af6d7cddb28b \
  --out $SEAL/run \
  --objective nearest --retain-rates 1 --decay-mult 1 \
  --lr 0.0002 --lr-end 0 --train-as-served 1 \
  --context 192 --width 256 --hamming-read 4 \
  --batch 64 --micro-batch 16 --steps 1024 \
  --eval-every 256 --samples 16 --reply-tokens 64 --seed 1
```

The seed-2 replicate uses the seed-2 checkpoint/hashes above and `--seed 2`.
No turn memory, dropout, reply cut, fixed targets, writer anchor or fast-rival
replacement is enabled. `--micro-batch 16` is the memory-safe form of one
batch-64 update.

## Readings

Before training, export the sealed base artifact and reproduce:

- `one-user-turn/v1`: EOS min/p50/max rank 18/138/873, top-8 0/256;
- `per-turn/v1`: EOS min/p50/max rank 5/7/13, top-8 198/256, top-1 0/256;
- free generation: 0 complete, 0 EOS/BOS/context stops and 256 cap stops;
- base OASST2 held-out guard: 132/512.

At checkpoints 256, 512, 768 and 1,024, export and score:

1. `cpu-checkpoint-grade --checkpoint STEP --run-config $SEAL/run/config.json
   --snapshot-out ... --artifact-out ... --tokens OASST2_HELDOUT/tokens.u16
   --mask OASST2_HELDOUT/response_mask.u8 --manifest OASST2_HELDOUT/manifest.json
   --max-evaluation 512 --max-replies 1 --reply-tokens 1 --out ...`.
   `cpu-checkpoint-grade` refuses `--max-replies 0`; one one-token reply
   exercises the export path while the continuation gate reads the 512-row
   nearest grade.
   Record nearest top-1, its pass/fail against 93, and the export hashes.
2. `served-reply-panel --artifact ... --rendering per-turn/v1
   --teacher-forced 1 ...`. Record EOS top-1/top-8/top-32, median rank,
   all-reference ranks and tokens/s.
3. `served-reply-panel` with `per-turn/v1` and `one-user-turn/v1`, generation
   only. Record complete rows, pairs, EOS/BOS/cap/context stops and tokens/s.
4. From `progress.jsonl`, record finite loss, `dev_nearest.hits`,
   `dev_top1_hits` (writer diagnostic), and wall time. Do not use the writer
   count as the continuation gate.

`cpu-fbs-export`/`cpu-checkpoint-grade` must bind each step's model config,
run config, Hamming read and output `UORLUT01` hash.

## Bar and outcome map

This is a staged development run, not a milestone bar. The M2 acceptance bar
remains 104/256 complete rows and 52/128 pairs.

The first-match outcome map is:

| order | condition at the best checkpoint through 1,024 | outcome |
|---:|---|---|
| 1 | human OASST2 top-1 < 93/512 | **Continuation failure.** Stop; no seed 2 and no mechanism attribution. |
| 2 | teacher-forced EOS top-1 = 0 and median rank >= 7 through step 512 | **No ranking movement.** Stop early; this closes this data/recipe configuration only. |
| 3 | EOS top-1 >= 16/256, median rank <= 3, free EOS stops >= 16/256, complete accepted rows >= 4/256, and guard >= 93/512 | **Positive:** termination and some useful content moved together. Unlock seed 2 and a longer run from the best checkpoint. |
| 4 | EOS top-1 >= 16, free EOS stops >= 16, guard >= 93, complete rows < 4 | **Termination signal, content gap.** No seed 2 yet; first compare first divergence on per-turn replies, then extend answer-content SFT. |
| 5 | EOS top-1 >= 16, guard >= 93, free EOS stops < 16 | **Ranking-generation gap.** No seed 2; diagnose the teacher-forced/free-generation mismatch before adding EOS weight. |
| 6 | 0 < EOS top-1 < 16 or median rank improves below 7 but the other thresholds miss | **Partial ranking movement.** No seed 2; run one 2,048-step dose extension at a new root, then apply thresholds 1-5. |
| 7 | none of the above | **No signal at 1,024.** Close this configuration and inspect the scored windows; do not run seed 2. |

A positive result establishes that this recipe can move the served model. It
does not attribute the gain to M-World or the reply mask; that needs a matched
OASST2-only continuation on the same base and steps.

## Instrument checks

- `init.json` names the exact base model/config/Hamming hashes and fresh
  optimizer/schedule.
- The mixed store and OASST2 held-out files match every full hash and sample
  fingerprint in this sheet before the first forward.
- `corpus-panel-overlap` reproduces the four zero overlap counts and output SHA
  above.
- Each checkpoint reloads and exports; `cpu-checkpoint-grade` exports and grades
  the same bytes.
- The panel inputs, labels, plan, manifest and tokenizer equal full hashes
  `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`,
  `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`,
  `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`,
  `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`,
  and `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`.
- Seed-1 base free-generation and rank controls reproduce the recorded counts.
- Every output root is new and every seal is read back.

The executable review unit is deepseek-23:

- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh` is the pod-side
  input, training, checkpoint, stop, seal and readback chain;
- `docs/labs/one-model-end-to-end/reply-sft-2026-11-07/run-s1.sh` is the
  one-pod controller and lease watcher.

## Steps

1. Record the fresh FIT and D67 review of the merged code, this sheet, job chain,
   exact input packet and seal paths.
2. `uor-pod status`; acquire one RTX 5090 32 GiB pod under a Codex-lab lease
   inside the four-pod/$8h caps. Never touch Claude's pods.
3. Build the exact merged pin, fetch and hash every input, run the training-unit
   CPU tests, and reproduce the base rank/free/continuation controls.
4. Run the 1,024-step training job in the foreground. At each 256-step
   checkpoint, export and score before proceeding. At step 512, apply row 2 of
   the outcome map and stop the process if it matches.
5. At completion or stop, seal checkpoints, exports, panel outputs, grades and
   logs under `results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1/`,
   read back, then take the pod down.
6. Apply the outcome map. Run seed 2 only after row 3 and a fresh decision
   review.

## Minimum test power and cost

Seed 1 with 256 fresh held-out rows is an exploratory signal test; seed 2 is
required before a reproducibility or capability claim. The thresholds are
multi-row and the 512-row continuation guard uses paired within-panel samples.

Resource projection:

- one RTX 5090 32 GiB; base micro-batch-16 peak about 27 GiB;
- 1,024 steps at 0.63-0.70 s/step: about 11-12 minutes;
- setup/build/input verification: 10 minutes; base controls: 2 minutes;
- five exports/grades and 15 panel passes: 8-10 minutes; seal/readback: 5 min;
- planned wall: 36-39 minutes. One full retry projection: 78 minutes;
- spend at $1.15/h: $0.69-0.75 planned, $1.50 with one retry;
- lease/watcher limit: 1.5 h, with healthy-run extension recorded if needed;
- local CPU: at most 8 Rayon threads for instruments; no local model training;
- local RAM: 16 GiB envelope; panel/grader run one at a time;
- new retained data: 20 MiB; pod checkpoints/exports: under 1 GiB; pod build:
  under 20 GiB; local build/scratch: under 10 GiB;
- current free space measured at review: 203,999,088 KiB, above the project's
  30 GiB + 128 MiB floor;
- `model-time.json` and an active storage receipt were absent at review; launch
  must re-check and record that state.

## Stop rule

- Instrument, pin, overlap, merge, or input failure: train nothing; fix and use
  a new root.
- Out of memory: retry micro-batch 8 once; a second failure is `NOT_RUN`.
- Non-finite loss, missing checkpoint, failed export/reload, or changed input
  hash: stop that seed.
- Apply the first-match outcome map before continuing past 512 and before seed 2.
- Never cancel a healthy run because a self-set timer or budget expires; renew
  and record. Never touch Claude's pods, leases, files or GPU devices.
