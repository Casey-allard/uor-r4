# REPORT: one-model-end-to-end, claude-264 (with the served projections rounded to 2^-16, net 12 and 6 fewer misses; I-3 still fails)

Lab: claude. [ORDER](./ORDER.md). PR #3596, item #3407.
Sheet: [one-model-end-to-end-i3-projection-2026-11-03](../../../../harness/prep/one-model-end-to-end-i3-projection-2026-11-03.md).

**In one paragraph.** claude-263 found that the integer first token misses the float one on 112 and 137 of DEV's 310 prompts, against a bar of 8. A code read found one rounding the integer session does and the float served surrogate did not: every FBS projection output goes onto the residual's 2^-16 grid. This unit added that rounding to the surrogate, under the served bridge only. Two CUDA tests pin it, and the model-level one fails with the `linear` hunk set aside. claude-262's two step-256 checkpoints were then re-scored in float, against claude-263's stored integer replies.
- **The float first token moved on 62 and 77 prompts** between claude-263's scoring and this one. There was no same-host repeat of the old commit, so how many of those moves the rounding caused, rather than run-to-run variation on the GPU, is not separated.
- **The misses fell by a net 12 and 6.** Seed 1 now misses on 100 of 310; read per seed, the sheet calls that partial. Seed 2 misses on 131, within 10 of 137, which it calls not the part. The sheet defines no outcome for a split between the seeds.
- **The moves went both ways.** 27 and 23 prompts went from miss to match, and 15 and 17 from match to miss. A site that was the whole difference, matched exactly and scored deterministically, would only fix. Three explanations fit, and this unit does not separate them (item 2).

The rounding stays: the code review found it is the integer session's own conversion, so it is a real mismatch. The next unit reads the remaining sites. The scorer also ran 2.1–2.9× slower than in claude-263; the cause is not separated.

## Tokens per second first

- **The integer export:** not read. No export or serve was redone (sheet step 6).
- **The float scorer**, GPU, serial decode, read from the `decode` records `score_rows` prints:
  - seed 1: 19,381 tokens in 456.4 s with memory, 42.5 tok/s; 19,381 in 602.1 s without memory, 32.2 tok/s;
  - seed 2: 18,997 in 618.5 s, 30.7 tok/s; 19,000 in 613.8 s, 31.0 tok/s.

  claude-263 ran the same command on the same checkpoints at 89.0 / 89.5 and 88.0 / 88.7 tok/s. These numbers time the instrument, not the served model. The slowdown is item 3.

## What did not work

1. **I-3 still fails at both seeds.** `compare-264.sh` printed:

   | Seed | First tokens differ, claude-263 | Now (bar 8) | Whole replies differ, claude-263 → now | Order / prompt mismatches | Sheet outcome, read per seed |
   |---:|---:|---:|---|---|---|
   | 1 | 112 / 310 | 100 / 310 | 308 → 303 | 0 / 0 | PARTIAL (fell by 12) |
   | 2 | 137 / 310 | 131 / 310 | 304 → 308 | 0 / 0 | NOT THE PART (fell by 6) |

   The sheet's partial wants both seeds to fall by more than 10, and seed 2 fell by 6. Its not-the-part wants both within 10, and seed 1 fell by 12. So it has no outcome for this split, and the table reads each seed alone. Both outcomes say the same next step: keep the rounding and read the remaining sites. So the split between the seeds does not change what follows.

2. **First tokens moved both ways.** Split of the prompts whose float first token moved, read from the stored files with the `jq` text below:

   | Seed | Moved | Miss → match | Match → miss | Miss → another miss | Misses not moved |
   |---:|---:|---:|---:|---:|---:|
   | 1 | 62 | 27 | 15 | 20 | 65 |
   | 2 | 77 | 23 | 17 | 37 | 77 |

   - If the projection rounding were the only difference, matched exactly and scored deterministically, it could only turn misses into matches. It also broke 15 and 17 prompts that the unrounded float had matched.
   - Neither net change is separated from moves going both ways at random. A two-sided sign test gives p ≈ 0.09 for 27 against 15, and p ≈ 0.43 for 23 against 17.
   - Three explanations fit, and this unit does not separate them:
     - **Other roundings.** Roundings the surrogate does not yet copy may decide many near-tie first tokens, which a few 2^-16 steps can flip either way.
     - **This rounding is inexact.** It does not match the integer on some inputs. One edge is known: `q16_residual` in f32 is one step high on odd grid points with |x| in [128, 256). That is the sheet's FIT note, narrowed by the code review. Whether any projection output here reaches 128 was not read.
     - **Run-to-run variation.** The GPU float scorer may not give the same first token run to run. This unit had no same-host repeat, and claude-263 and this unit scored on different hosts.
   - If the first is the cause, matching one site at a time will move the count both ways until the last site is matched. A count of first-token misses is then a weak guide to which site matters.

3. **The scorer ran 2.1–2.9× slower.** Each seed took 1,372 s and 1,560 s, against claude-263's 568 s and 532 s. Two causes are not separated:
   - **The host.** A different pod, at $0.908/h against $0.563/h. On it, seed 1's memory and no-memory arms differ by 32% (42.5 against 32.2 tok/s); claude-263's differed by under 1%. On this one pair of arms, this host varied far more than claude-263's.
   - **The rounding's kernels.** It adds several elementwise kernels to each of six projections per layer, in every decode step.

   If the kernels are the cause, served training pays the same cost. Its throughput was not measured.

4. **The pod cost more per hour than the sheet planned:** $0.908/h, against "under $0.80/h". The total was about $0.97 for about 1.07 h, under the sheet's $1.20 ceiling.

## What changed

- **Code.** `served_projection_q16` in `hamilton_sheet.rs`.
  - Under `with_served_surrogate_training` (its `BRIDGE_ACTIVATION_I16` flag), an FBS projection's output goes through `q16_residual`: floor(x·2^16 + ½) at 2^-16, clamped to i32, with a straight-through gradient. Outside it, the helper returns its input.
  - `linear`'s rank-3 FBS branch in `geometric_stack_native.rs` calls it on both returns: the SpiralCore/FBS arm and the plan forward.
  - Plain, Export and D11 forwards are unchanged, and so is the Q16 control alone.
- **Tests**, CUDA-gated and run on the pod:
  - `hamilton_sheet::tests::served_projection_rounding_is_the_integer_gather_on_cuda`. The helper equals `(held + 2^7) >> 8` at 2^-16 on exact halves of both signs. Outside the surrogate, under the Q16 control alone and in D11 mode, it is bit-identical.
  - `geometric_stack::tests::served_fbs_projections_round_to_the_residual_grid_on_cuda`. For all six projections of a fixed model, under the surrogate the output is on the grid, equals the unrounded output rounded half up, and carries the unrounded gradient. Outside it, plain, Q16-only and D11 outputs are bit-identical.
- **Docs.** The ORDER, this REPORT, the prep sheet with its FIT and pre-run lines, one notebook row in `float-training.md`, and the item's "does not yet serve as trained" line.

Training runs the same surrogate, so served training now takes this rounding too. No model has been trained with it yet.

## Measurements

- **The claim.** The ORDER is hand-written under the umbrella reading (cancel probe 2). `loop claim` refused on #3583's lock. #3583 is a draft at `90bfc731a`, and the only file it shares with this scope is the notebook, which takes added rows only.
- **The pod.** vast-58957379, 1 × RTX 5090 at $0.908/h, up about 11:39Z, bootstrapped at `5ce220ebc`, down 12:43Z, about $0.97. `job-264.sh` (SHA-256 `b100f632…fb6e`) and `linear-hunk.patch` (`b5be1874…f026`) were checked on the pod before the run.
  - **I-0a**, 11:46:31Z–11:47:40Z. With the `linear` hunk set aside, the model-level test ran alone and failed: "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 304 filtered out". Its failing assertion was on `rec.in.weight`, an unrounded 3.0229998. The hunk was restored.
  - **I-0**, 11:47:40Z–11:48:47Z, rebuilt after the restore. `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` gave 298 passed, 0 failed, 7 ignored, with both new tests ok. The source is `5ce220ebc1e52638c1dadcc60aca42d2caec600a`.
  - **The build.** `train-in-space` `9b8452ea2a173caf8e1908baf01027c74b54ce07be8480a19087a08fa49c138c`.
  - **The inputs.** The tokenizer, DEV `tokens.u16` and DEV `response_mask.u8` match the sheet's pins. TRAIN's `tokens.u16` (`3dee858f…`) and `response_mask.u8` (`b5e7352d…`) were fetched for the data root, as in claude-263. claude-262's seal and its members check (I-2).
  - **Scoring.** Seed 1 took 1,372 s and seed 2 1,560 s, one after the other. Each wrote 310 DEV samples (I-4). EXIT 0 came at 12:38:10Z.
- **The seal.** `/workspace/i3p` without the fetched inputs: the 48 files that `SHA256SUMS.files` seals, plus `DONE`, `EXIT`, `SHA256SUMS.err` and `SHA256SUMS.files` itself, 52 in the tar. The 48 checked on the laptop. The tar then went to the store and was read back.
- **The laptop.** `compare-264.sh` ran against claude-263's float files and integer replies. Its instrument check gave claude-263's 112 and 137 at both seeds. Item 2's split is the `jq` text below.
- **Not run:** export, serving, and R6 tok/s on an integer artifact. The sheet redoes none of them.

## The comparison script

The sheet's step 6 asks for its text and output here. It is `compare-264.sh`, SHA-256 `261f75b4913a157f6f13e7be359a0f8eeef6e353e58c8cfa37322d36582b1000`, run as `zsh compare-264.sh NEW OLD PAR OUT`:

```zsh
#!/bin/zsh
# claude-264: I-3 re-read. For each seed, compare the new float samples-memory.json (scored with the
# served projections rounded to 2^-16) with claude-263's stored integer replies, using
# claude-263's fields and bar (serve-i3.sh). Beside it: claude-263's own count from the same
# integer file against claude-263's float scores, which must reproduce 112 and 137 (the instrument
# check), and how many first tokens the rounding moved in float. Nothing is exported or served.
# usage: zsh compare-264.sh NEW OLD PAR OUT
#   NEW: this unit's pod outputs, with served-sN-scored/samples-memory.json and served-sN.score.exit
#   OLD: claude-263's pod outputs (its served-sN-scored/samples-memory.json)
#   PAR: claude-263's laptop parity folder (its served-sN-toks.json, the integer replies)
#   OUT: must not exist
set -u
set -o pipefail
NEW=${1:?usage: compare-264.sh NEW OLD PAR OUT}
OLD=${2:?usage: compare-264.sh NEW OLD PAR OUT}
PAR=${3:?usage: compare-264.sh NEW OLD PAR OUT}
O=${4:?usage: compare-264.sh NEW OLD PAR OUT}
[ -e "${O}" ] && { echo "refused: ${O} exists"; exit 2; }
mkdir -p "${O}"
# Counts over paired samples A and B: first tokens differ, whole replies differ, positions differ,
# prompts differ, and the first divergence position of each differing reply.
count() {
  jq -n -c --slurpfile a "$1" --slurpfile b "$2" '
    ($a[0]) as $a | ($b[0]) as $b |
    [range(0; $a|length)] as $i |
    { first: ([$i[] | select($a[.].reply[0] != $b[.].reply[0])] | length),
      whole: ([$i[] | select($a[.].reply != $b[.].reply)] | length),
      order: ([$i[] | select($a[.].position != $b[.].position)] | length),
      prompts: ([$i[] | select($a[.].prompt != $b[.].prompt)] | length),
      diverge: [$i[] | select($a[.].reply != $b[.].reply) | . as $k
        | ([$a[$k].reply, $b[$k].reply] | map(length) | min) as $m
        | ([range(0; $m) | select($a[$k].reply[.] != $b[$k].reply[.])] | first // $m)] }
    | .at0 = ([.diverge[] | select(. == 0)] | length)
    | del(.diverge)'
}
{
shasum -a 256 "${PAR}"/served-s[12]-toks.json "${OLD}"/served-s[12]-scored/samples-memory.json "${NEW}"/served-s[12]-scored/samples-memory.json
typeset -A before=(served-s1 112 served-s2 137)
for x in served-s1 served-s2; do
  se=$(cat "${NEW}/${x}.score.exit" 2>/dev/null)
  [ "${se}" = 0 ] || { echo "${x}: refused, score exit '${se}'"; continue; }
  f=${NEW}/${x}-scored/samples-memory.json
  g=${OLD}/${x}-scored/samples-memory.json
  t=${PAR}/${x}-toks.json
  n=$(jq length "${f}")
  [ "${n}" -eq 310 ] || { echo "${x} I-4: refused, ${n} float samples, not 310"; continue; }
  [ "$(jq length "${t}")" -eq 310 ] || { echo "${x}: refused, the integer file holds $(jq length "${t}") replies"; continue; }
  old=$(count "${g}" "${t}")
  echo "${x} claude-263 (old float vs integer): ${old}"
  [ "$(jq -r .first <<< "${old}")" -eq "${before[$x]}" ] \
    || { echo "${x}: refused, the stored files give $(jq -r .first <<< "${old}") first-token misses, not claude-263's ${before[$x]}"; continue; }
  moved=$(count "${g}" "${f}")
  echo "${x} the rounding in float (old float vs new float): ${moved}"
  new=$(count "${f}" "${t}")
  echo "${x} I-3 (new float vs integer): ${new}"
  first=$(jq -r .first <<< "${new}")
  order=$(jq -r .order <<< "${new}")
  prompts=$(jq -r .prompts <<< "${new}")
  bar=$(( n * 29 / 1000 ))
  b=${before[$x]}
  if [ "${order}" -ne 0 ] || [ "${prompts}" -ne 0 ]; then
    verdict="NOT READ (order mismatches ${order}, prompt mismatches ${prompts})"
  elif [ "${first}" -le "${bar}" ]; then
    verdict=PASS
  elif [ $(( first - b )) -le 10 ] && [ $(( b - first )) -le 10 ]; then
    verdict="NOT THE PART (${first}, within 10 of ${b})"
  elif [ "${first}" -lt "${b}" ]; then
    verdict="PARTIAL (${first}, below ${b}, above the bar ${bar})"
  else
    verdict="WORSE (${first}, more than 10 above ${b})"
  fi
  echo "${x} I-3: first tokens differ ${first} of ${n} (bar ${bar}, before ${b}); ${verdict}"
done
} 2>&1 | tee "${O}/compare.txt"
```

Its output, `compare.txt`:

```text
be36ef56dbd4c03e5c16913055097457ff387131c72c7598532c5dcaa1a3cc94  .../i3-262/serve-out/parity/served-s1-toks.json
dc5079eb0f08d91e7a49854c7956531bddcf7e7c01d8b836fb1d950e3fd4ca84  .../i3-262/serve-out/parity/served-s2-toks.json
2194f59831e86f968b02fce1c12dc3aa9aa04dd553202f5a59c870c344f4cbcd  .../i3-262/pull/i3/out/served-s1-scored/samples-memory.json
fb14944194bbe9a82ec82909705d972d07268977c6f7a08a9d5b2299527bbd82  .../i3-262/pull/i3/out/served-s2-scored/samples-memory.json
f20e68419e0b6ba8349d95687dc593b4ed8cb50d46892605adf7f862b7d5d60d  .../i3-264/pull/i3p/out/served-s1-scored/samples-memory.json
1855087376d7b7034b96e79e778e2982691e1d1b727c249c2c16e2e3ee7219d0  .../i3-264/pull/i3p/out/served-s2-scored/samples-memory.json
served-s1 claude-263 (old float vs integer): {"first":112,"whole":308,"order":0,"prompts":0,"at0":112}
served-s1 the rounding in float (old float vs new float): {"first":62,"whole":289,"order":0,"prompts":0,"at0":62}
served-s1 I-3 (new float vs integer): {"first":100,"whole":303,"order":0,"prompts":0,"at0":100}
served-s1 I-3: first tokens differ 100 of 310 (bar 8, before 112); PARTIAL (100, below 112, above the bar 8)
served-s2 claude-263 (old float vs integer): {"first":137,"whole":304,"order":0,"prompts":0,"at0":137}
served-s2 the rounding in float (old float vs new float): {"first":77,"whole":294,"order":0,"prompts":0,"at0":77}
served-s2 I-3 (new float vs integer): {"first":131,"whole":308,"order":0,"prompts":0,"at0":131}
served-s2 I-3: first tokens differ 131 of 310 (bar 8, before 137); NOT THE PART (131, within 10 of 137)
```

The paths are shortened to `...`; each is under `local/claude/`.

Item 2's split, one `jq` call per seed over the same three files (`O` the claude-263 float file, `N` the new one, `T` the integer replies):

```zsh
jq -n -c --slurpfile o "${O}" --slurpfile n "${N}" --slurpfile t "${T}" '
  ($o[0]) as $o | ($n[0]) as $n | ($t[0]) as $t | [range(0;310)] as $i |
  { moved: ([$i[] | select($o[.].reply[0] != $n[.].reply[0])] | length),
    fixed: ([$i[] | select($o[.].reply[0] != $t[.].reply[0] and $n[.].reply[0] == $t[.].reply[0])] | length),
    broke: ([$i[] | select($o[.].reply[0] == $t[.].reply[0] and $n[.].reply[0] != $t[.].reply[0])] | length),
    wrong_to_wrong: ([$i[] | select($o[.].reply[0] != $n[.].reply[0] and $o[.].reply[0] != $t[.].reply[0] and $n[.].reply[0] != $t[.].reply[0])] | length),
    unmoved_miss: ([$i[] | select($o[.].reply[0] == $n[.].reply[0] and $o[.].reply[0] != $t[.].reply[0])] | length) }'
```

It printed `{"moved":62,"fixed":27,"broke":15,"wrong_to_wrong":20,"unmoved_miss":65}` at seed 1 and `{"moved":77,"fixed":23,"broke":17,"wrong_to_wrong":37,"unmoved_miss":77}` at seed 2. At each seed, broke + wrong_to_wrong + unmoved_miss equals the new miss count, 100 and 131.

## Evidence

HF store `results/claude/one-model-end-to-end-i3-projection-2026-11-03/`, read back on 3 Nov:
- `claude-264.tar` SHA-256 `00aa1b86637c7fd34801db801026abed79114a17fb25f2f5548c619aada4e91a`. It is the pod's `/workspace/i3p` without the fetched inputs: the job log, every gate's output, the binary's hash and both seeds' scored folders. Its 48-file `SHA256SUMS.files` checks on the laptop.
- `SHA256SUMS`, the tar's line; `shasum -a 256 -c` on the read-back copy gives OK.

## Checks run at the head

- **On the pod**, at `5ce220ebc`: I-0a and I-0, as in Measurements.
- **On the laptop:**
  - `cargo check --offline --locked -p uor-r4-float-train --features cuda --all-targets`, with a stub `nvcc` on PATH. The crate's CUDA code type-checks; candle here refuses CPU tensor compute, so its tensor tests run only on the pod.
  - `cargo test --locked -p uor-r4-harness the_real_sheets`, the two sheet tests that failed in CI at claude-263.
  - `git diff --stat 5ce220ebc HEAD -- crates Cargo.lock` is empty: every commit after the scoring commit is documents only.
  - `uor-harness loop report --check` and `uor-harness loop pr-check --local` on this REPORT.
- **Before every push**, the no-overlap check against #3583.

## Skills used

None applied. The scripts are `job-264.sh`, `compare-264.sh` and `watch-264.sh`. They live in `local/claude/` and are not committed as files. Each was named in the reviewed sheet. `compare-264.sh`'s text is above. `job-264.sh` ran from `/workspace/job.sh`, outside the sealed folder, so it is recorded only by the SHA-256 checked on the pod.

## Still open

1. **The remaining parity sites: the next unit.** The sites the surrogate may still round differently from the integer session:
   - the RMS norm and its shift;
   - the recurrence state and its decay;
   - the Hamming reads and the turn memory;
   - the writer's Q12 query;
   - the convolution;
   - the embedding;
   - and `q16_residual`'s f32 edge on odd grid points with |x| in [128, 256).

   The method is claude-263's still open 1, with the existing tools: `fbs-snapshot --dev` and `fbs-compare` attribute the float-to-integer gap to each snap and to table rounding, and an integer serve with the turn memory off checks the memory path. Item 2 adds two cautions:
   - **Score the same commit twice on one host** before reading the next site's count. That separates run-to-run variation from what a change does.
   - **Read values per layer.** If first tokens flip both ways near ties, a site's effect on the first-token count can be small or hidden, so a value read per layer says more than a count.
2. **Two test gaps the code review found**, both MINOR and safe by construction. A later code unit adds both tests.
   - The model-level test asserts the plain, Q16-control and D11 forwards unchanged, but not the Export forward. Only `with_served_surrogate_training` sets the bridge flag, so Export cannot reach the rounding.
   - The SpiralCore/FBS arm's return in `linear` is rounded, but no test covers it.
3. **The scorer's slowdown.** Read the decode time with the served bridge on and off on one host, before any long served-training run. If the rounding's kernels are the cause, a fused rounding kernel is a candidate.
4. **Then the full-length served run**, by its FIT sheet, once I-3 passes.

## Plain summary

The laptop's integer model still gives a different first word from the trained model on about a third of 310 test prompts. This unit found one difference in how the two round numbers inside each layer and made the trained model round the same way. Mismatches fell from 112 to 100 at one seed and from 137 to 131 at the other, far from the bar of 8. Some matching prompts also became mismatches. That fits other rounding differences still in place, this rounding not matching exactly on some inputs, or scoring that varies from run to run, and this unit cannot tell which. The fix is kept, because it copies what the integer model does; the next unit finds the remaining differences. The scoring also ran about two to three times slower, for a reason not yet separated. GPU cost: one pod, about an hour, about $0.97.

References #3407.
