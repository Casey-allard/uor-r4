# Pre-registration: input-preserving query-port training

Run ID: `cpu-training-pilot-raw-query-port-2026-11-05`

## Status

COMPLETE - CONTROL_FAULT. Both seeds completed and exact verification
reproduced a behaviorally vacuous shuffled control. No model change is
promoted.

## Question

Can the existing writer-query port-root objective improve held-out geometry on
an exportable one-table artifact when the learned token input/writer table
stays raw?

This is D84 CPU development research. It is not GPU qualification, a language
result or a complete-reply result.

## Why this is new

Deepseek-183 trained the final-gain query ports from a table-anchored baseline
and rejected the objective. Deepseek-185 then showed that the table anchor's
recurrent-input role caused essentially all of the decoding collapse; direct
writer-row anchoring was comparatively neutral.

This unit keeps every table row byte-identical to the source artifact. Only the
16 final-gain codes that scale the query's first and last eight-coordinate
blocks may move. The result is therefore a valid single-table `UORLUT01` with
the learned input code preserved exactly.

## Base artifact

Public dual-c131 raw FBS artifacts:

- seed 1:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut`,
  SHA-256
  `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`;
- seed 2:
  `/home/callard/projects/codex-playground/uor-r4/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut`,
  SHA-256
  `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.

Both are memory-free and Hamming-free FBS artifacts.

## Data split

Training corpus:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/m2-evidence/train-control-v3`

- tokens:
  `a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5`;
- response mask:
  `89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f`;
- manifest:
  `980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672`;
- split `train`, filtered power **235** conversations / **2,708** response
  rows.

Panel:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/m2-evidence/dev-panel-v3`

- tokens:
  `add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442`;
- response mask:
  `da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01`;
- manifest:
  `ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949`;
- split `dev`, filtered power **58** conversations / **616** response rows.

Both manifests bind tokenizer
`a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`,
vocabulary 4,096 and BOS 0. The public panels were used in earlier CPU work;
this is component calibration, not fresh-capability evidence.

## Fixed construction

`cpu-query-port-driver --baseline-mode raw` starts the trainer and controls from
the source artifact. It does not reconstruct or write an anchored table.

For every response target `t`:

- derive cell `c = t mod 256`;
- derive roots `c / 16` and `c % 16` from the 16 parsed SpiralCore roots;
- calculate exact Q12 L1 distance from query coordinates `0..8` and `W-8..W`
  to the two roots.

Only final-gain coordinates `0..8` and `W-8..W` are trainable. At each of eight
steps every coordinate is evaluated at `+1` and `-1`; a move is accepted only
when it strictly lowers training target-root distance without reducing
training target admission, top-1 or top-8. Ties retain the lower coordinate and
positive move.

The final artifact and all controls must preserve every non-gain byte, every
non-port gain coordinate, and the complete raw token table. Every accepted move
must run `CpuPanelEvaluator` with zero query, token and kept-cell mismatches.

## Arms

| Arm | Construction |
|---|---|
| raw frozen | unmodified source artifact |
| trained | eight true-cell query-port steps from raw frozen |
| shuffled cells | same trainer from raw frozen with target cell rotated by one; up to 32 attempts to match accepted moves |
| random gain | exactly the trained accepted move count from raw frozen, selected deterministically over port coordinates |

The raw-frozen arm is the training baseline and the direct export target. No
table-anchored arm exists in this registration.

## Source and binary

- base: `d634c77e5`;
- implementation pin: `771c99ecf42ee344b43f439dfff8666d953f04da`;
- driver:
  `target/deepseek/raw-query-port/release/cpu-query-port-driver`,
  SHA-256
  `7210266b5c71e838f5b7e8a7f29ce3275abd57bc11ca20dbf2ea17612d295e81`;
- host: AMD Ryzen 9 9950X, 16 physical cores and 32 hardware threads;
- execution: 16 Rayon workers.

## Output

Original discovery roots:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/raw-query-port/2026-11-05/seed-{1,2}`.

Fresh control-verification roots:
`/home/callard/projects/codex-playground/uor-r4/local/deepseek/raw-query-port/2026-11-05/verification/seed-{1,2}`.

The original roots were absent at registration. The fresh verification root
was absent when the behavioral-control repair was registered.

## Steps

Run only after FIT and D67 GO:

```sh
set -euo pipefail
SOURCE=/home/callard/projects/codex-playground/uor-r4/.worktrees/deepseek-raw-query-port
SHARED=/home/callard/projects/codex-playground/uor-r4
OUT=$SHARED/local/deepseek/raw-query-port/2026-11-05/verification
BIN=$SOURCE/target/deepseek/raw-query-port/release/cpu-query-port-driver
TRAIN=$SHARED/local/deepseek/m2-evidence/train-control-v3
PANEL=$SHARED/local/deepseek/m2-evidence/dev-panel-v3
SEED1=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-1-raw_both.uorlut
SEED2=$SHARED/local/deepseek/public-fbs-dual/dual-c131-seed-2-raw_both.uorlut

test ! -e "$OUT"
git -C "$SOURCE" merge-base --is-ancestor 771c99ecf42ee344b43f439dfff8666d953f04da HEAD
test "$(sha256sum "$BIN" | cut -d' ' -f1)" = 7210266b5c71e838f5b7e8a7f29ce3275abd57bc11ca20dbf2ea17612d295e81
sha256sum -c <<EOF
a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd  $SEED1
da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1  $SEED2
a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5  $TRAIN/tokens.u16
89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f  $TRAIN/response_mask.u8
980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672  $TRAIN/manifest.json
add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442  $PANEL/tokens.u16
da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01  $PANEL/response_mask.u8
ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949  $PANEL/manifest.json
EOF

mkdir -p "$OUT"
run() {
  seed=$1 artifact=$2 artifact_sha=$3
  RAYON_NUM_THREADS=16 timeout 1200s "$BIN" \
    "$artifact" "$TRAIN/tokens.u16" "$TRAIN/response_mask.u8" "$TRAIN/manifest.json" \
    "$PANEL/tokens.u16" "$PANEL/response_mask.u8" "$PANEL/manifest.json" \
    "$OUT/seed-$seed" \
    --expected-artifact-sha256 "$artifact_sha" \
    --expected-train-tokens-sha256 a7511c32fdf1532ef903e0f04e34a48664e4ba68cefde1414ab2e057198cdaf5 \
    --expected-train-mask-sha256 89144e366a9b9856a10376e983b4dc98c673fb6aa1bd6315f970f4bdfe57234f \
    --expected-train-manifest-sha256 980b91b33946457350151c1044b33eaeabb2034bb8137dcbc49b324f29c26672 \
    --expected-panel-tokens-sha256 add1f3954fc67e12822bdac395d7167bf6d6a0f074856efe2beec2c4ac81e442 \
    --expected-panel-mask-sha256 da1ae9ee3bdc4fe1bf4a84a12ee9c1ac02ee51a419e3806cb77df430a44a3c01 \
    --expected-panel-manifest-sha256 ce489b38e91b2514f302560e65ac8bf7c337ca8a15b2aedba428e312bb003949 \
    --baseline-mode raw \
    --seed 188 >"$OUT/seed-$seed.stdout.json" 2>"$OUT/seed-$seed.stderr.log"
}

run 1 "$SEED1" a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd
run 2 "$SEED2" da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1
```

## Minimum power and checks

- two public artifact seeds;
- 2,708 training and 616 panel response rows per seed;
- eight candidate sweeps of 34 port-gain moves;
- at least one accepted true-cell move per seed;
- non-empty, moved-count-matched shuffled-cell and random-gain controls;
- the shuffled-cell behavior signature must differ from the trained behavior;
- raw baseline bytes exactly equal the source artifact;
- only port final-gain coordinates change; all table and other artifact bytes
  remain identical;
- every trained and control artifact parses as `UORLUT01`;
- exact CPU/serving query, token and kept-cell parity in every arm;
- admission, top-1 and top-8 safety recorded as separate predicates;
- artifact and step replay identity within the run.

## Bar

On both seeds the trained arm must:

- beat raw-frozen, random-gain and shuffled-cell controls on held-out target
  port-root distance;
- retain held-out target admission, top-1 and top-8 at or above raw frozen;
- preserve exact CPU/serving parity;
- reproduce the artifact and all steps in-process.

## Outcomes

- **KEEP:** both seeds clear every bar item.
- **REJECT:** either seed completes but misses a distance or safety item.
- **INSTRUMENT_FAULT:** input, parity, byte-isolation or repeat identity drift.
- **NO_MOVE:** the trained arm accepts zero moves.
- **CONTROL_FAULT:** shuffled cells cannot match the accepted move count.
- **CONTROL_FAULT:** the shuffled-cell artifact is behaviorally identical to
  the trained artifact.

## Cost and stop

- local CPU only, $0;
- two runs, 16 workers, at most 20 minutes each;
- stop above 1 GiB peak RSS;
- stop on source, input, artifact, parity, determinism or output-root drift;
- do not retune or rerun after a report is read.

## Claim boundary

A result covers only the eight-step raw-conditioned query-port gain
configuration on two public CPU-development artifacts. It does not establish
fresh-panel transfer, language quality, complete replies or GPU qualification.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-raw-query-port-2026-11-05.md
reviewer: deepseek fresh-context subagent raw_query_port_fit_d67_2
note: No blockers at ad4340df0; raw byte preservation, anchored
  compatibility, split safety reporting, exact parity, controls, pins and
  D84 scope verified; public arms not run.
date: 2026-11-05

A FIT verdict clears compute for the configuration in this sheet only (D22).

## Pre-run GO

GO - reviewer `deepseek fresh-context subagent raw_query_port_fit_d67_2`,
2026-11-05, at clean review head
`ad4340df0244727759cf19bc94b4445a86c97e3f`. The reviewer verified raw
byte-for-byte preservation, anchored-mode compatibility, separate
admission/top-1/top-8 fields, exact parity, matched controls, deterministic
replay, complete input pins and absence of both public output roots.

## Outcome

The first sealed discovery runs at binary `f3147470…` completed with eight
accepted moves on seed 1 and five on seed 2. Both seeds produced the same
trained and shuffled artifact. The raw query objective lowered held-out
distance relative to raw frozen and random gain:

- seed 1 raw/trained/random: `29,882,444 / 29,613,365 / 29,818,562`;
- seed 2 raw/trained/random: `23,870,298 / 23,863,304 / 23,802,494`.

The shuffled arm cannot support an independent comparison. The repaired driver
hashes ordered token and kept-cell decisions plus all panel and port metrics,
ignoring artifact identity, and classifies behaviorally identical
shuffled/trained arms as **CONTROL_FAULT**. It uses a fresh verification root.
No promotion follows; the original sealed reports remain preserved.

Final verification at binary `7210266b…` returned **CONTROL_FAULT** on both
seeds. Seed 1 took **71.766 s / 157,000 KiB**, and seed 2 took
**52.274 s / 154,856 KiB**. Its report/manifest SHA-256 pins are:

- seed 1:
  `5e1f102aa2d0491696116643adc17e37ac7db33e1d7541f490fee0e303ef40ee` /
  `f523f2a8dce30d670da6c362288d0b01c796a44d1b54ac86eaf72744327c6cc3`;
- seed 2:
  `f89d577977a2be9d49e4f15c8c82410adc90c836bfe0917c0678c336dfa3c967` /
  `d6762369d50b2de37bce83a8bc3c0ddfc3b8a6beedab00e8448d97cb5450f7e8`.

The trained arm beat the valid random-gain control on both seeds, but the
cell-rotation control needs redesign before this objective can produce a valid
mechanism comparison.
