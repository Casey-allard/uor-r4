# REPORT: one-model-end-to-end, claude-263 (I-3 at the repaired head fails at both seeds; the full-length run did not start)

Lab: claude. [ORDER](./ORDER.md). PR #3592, item #3407.
Sheets: [part A](../../../../harness/prep/one-model-end-to-end-i3-repaired-2026-11-03.md), [part B](../../../../harness/prep/one-model-end-to-end-full-served-2026-11-03.md).

**In one paragraph.** A model trained as served still does not serve as trained. claude-262's two served step-256 checkpoints were scored in float at the pinned head `0073cd07b`, exported snapped, and served in integers on the laptop. The integer first token differs from the model's own float first token on **112 of 310** DEV prompts at seed 1 and **137 of 310** at seed 2, against a bar of 8. Whole replies differ on 308 and 304. claude-261 saw 295–305 before claude-262's repair, so the repair closed most of the gap but not this. Following the sheet, a fail is classed instrument: a parity fault in export or serving, not a model property. So part B, the full-length served run, did not start. The next unit localizes the fault to one layer.

## Tokens per second first

- **The integer export:** not read. The part A sheet reads R6 (tok/s) only on exports that pass I-3.
- **The float scorer**, GPU, serial decode, read from the `decode` records `score_rows` prints:
  - seed 1: 19,465 tokens in 218.6 s with memory, 89.0 tok/s; 19,465 in 217.4 s without memory, 89.5 tok/s;
  - seed 2: 18,688 in 212.3 s, 88.0 tok/s; 18,688 in 210.6 s, 88.7 tok/s.

  These numbers time the instrument, not the served model.

## What did not work

1. **I-3 failed at both seeds.** Per seed, `serve-i3.sh` printed:

   | Seed | `fbs-toks` exit | Order mismatches | First tokens differ (bar 8) | Whole replies differ | I-5 header `copy_key` | Verdict |
   |---:|---:|---:|---:|---:|---|---|
   | 1 | 0 | 0 | 112 / 310 | 308 / 310 | `[]` | FAIL |
   | 2 | 0 | 0 | 137 / 310 | 304 / 310 | `[]` | FAIL |

2. **So part B did not run** (ORDER done-when 4). That is the 8,192-step served run of (b) and (k2) at seeds 1–3. No R1–R7 reading exists for a served-trained export.

3. **The float model's first token barely depends on the turn memory.** This reading uses only the stored files, with no new tool.
   - At 256 steps the float model's first token with memory and without memory differs on only 3 of 310 prompts at seed 1 (prompts 32, 37 and 190) and 0 at seed 2.
   - The integer first token misses the float *no-memory* first token on 112 prompts at seed 1 and 137 at seed 2. At seed 1, 111 of those 112 are the prompts that miss the float memory first token; at seed 2 the two sets of 137 are the same.

   So the float side's memory is not needed to explain the gap. The trunk, the readout and the integer memory path all remain candidates. This does **not** show that the integer memory and reply-table path is right: a fault there would give the same counts. claude-264's first check is an integer serve with the turn memory off, if the existing tools allow it.

4. **Where the replies part.** Count each prompt at the first position where the integer and float replies differ.
   - At position 0: 112 prompts at seed 1 and 136 at seed 2. Seed 2's 137 first-token misses include prompt 242, whose float reply is empty, so it has no position 0 to part at.
   - At positions 1–4: 44 / 27 / 19 / 15 at seed 1, and 26 / 17 / 10 / 10 at seed 2.
   - 2 replies at seed 1 and 11 at seed 2 agree to their shorter length. Seed 1's 2 are identical. Of seed 2's 11, 6 are identical, one of them prompt 89, which is empty on both sides. The other 5 differ only in length, prompt 242 among them.

   The fault reaches the first token on more than a third of the prompts (36% and 44%), so it is not only in longer decodes. Where in the first token's computation it lies, including the integer memory read, is not known.

5. **Neither sheet's review records were on #3407 when the PR opened, and CI went red.** Part A's sheet had no `TEST FITNESS` line, so `prep::tests::the_real_sheets_are_readable` and `preprules::tests::the_real_sheets_pass_the_shape_rule` failed. Both sheets had passed their reviews earlier. `75304ef9c` adds the line. Both sheets' FIT and pre-run records are now on #3407 (comment 6600769545).

## What changed

Docs only; no code changed. The files:
- this unit's ORDER and REPORT;
- the two prep sheets, as they passed FIT, with part A's verdict lines added;
- one notebook row in `float-training.md`;
- a new bullet under the item's "What is missing".

## Measurements

- **The claim.** `loop claim` refused, citing #3583's stale file list. The ORDER was written by hand under the run file's umbrella exception, after a fresh decision review returned ALLOW with six conditions. The cancel probe records the refusal, the no-overlap check and the evidence. The call is logged for Casey on #2028 (6600465605). The coexistence note is on #3583 (6600465061).
- **The pod.** vast-58946756, 1 × RTX 5090 at $0.563/h, created 09:53Z, ready 09:57Z, down 10:21Z, about $0.26. `job-i3.sh` (SHA-256 `d350e94a…ed64`, checked on the pod) ran at the pin:
  - **I-0:** `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1` gave 296 passed, 0 failed, 7 ignored, including `pair_halves_puts_the_first_token_in_each_copy_heads_low_half` and the copy-key tests. The source is `0073cd07bd5d16021639041aa30c55132d2c6dc9`. The binaries are `train-in-space` `47c866c2…993d` and `fbs-snapshot` `9c529875…65a5`.
  - **The inputs:** the sheet's three data pins match: `tokenizer.json` `d36d3e87…`, DEV `tokens.u16` `0919e90f…` and DEV `response_mask.u8` `a838bb1a…`. TRAIN's `tokens.u16` (`3dee858f…`) and `response_mask.u8` (`b5e7352d…`) were fetched for the data root; the sheet pins neither. claude-262's seal and its members check.
  - **I-1:** `git diff fac0131bc 0073cd07b` over float-train and the integer crate is #3579's nine files. The hunks were read on 3 Nov, when the sheet was written, and the file list was re-run at review. Every hunk of `in_space_long.rs`, `train-in-space.rs` and `fbs_export.rs` is copy-key only:
    - `hamming_copy_key` defaults to 1 and is not written to `config.json` when it is 1;
    - `with_copy_key(1)` changes nothing;
    - the other additions are a refusal, a test and help text.

    `git diff 4465685531 fac0131bc` over both crates is empty. Both runs' `config.json` carry `train_as_served: true` and `hamming_copy_key: null`, so neither has a copy key.
  - **Scoring:** seed 1 took 568 s and seed 2 took 532 s, one after the other. Each wrote 310 DEV samples, and each snapshot exited 0. EXIT 0 came at 10:18:18Z.
- **The laptop.** `serve-i3.sh` ran with `fbs-export` `b2d9aff9…` and `fbs-toks` `79f49ff2…`, built in a tree whose integer and tokenizer crates and Cargo.lock equal the pin. A deviation from the sheet's step 6: that tree was not this unit's worktree but `.worktrees/claude-copy-key-k2` at tree `afd99dd94`, as `serve-out/binaries.sha256` records. The script's equality guard passed on it, so the served code is the pinned code. It made the snapped exports (format `uor-r4.lut-stack/7`, seed 1 `607511e3…`, seed 2 `80567113…`), then the I-3 reading above.
- **Not run:** E, the batched scorer's parity. #3583 was not merged at claim time, so the sheet drops it. claude ran it outside this unit as review evidence for #3583: PASS, store `results/claude/pr-3583-parity-2026-11-03`.

## The serving script

The part A sheet's step 6 asks for its text here. It is `serve-i3.sh`, SHA-256 `4ef237ef1c799e30514bdff287f59d4561ec551ff6ff826a99744b35d95cd79d`, run as `zsh serve-i3.sh ROOT TREE OUT`:

```zsh
#!/bin/zsh
# claude-263: I-3 at the repaired head. claude-261's serve-snap.sh, adapted to claude-262's two
# served checkpoints: the snapped export (no --raw-gates, no --raw-transition, as
# --train-as-served 1 trains), I-5 (no copy_key in either header), then I-3 against each seed's
# float samples-memory.json. No transcripts are written.
# usage: zsh serve-i3.sh ROOT TREE OUT
#   ROOT: the unpacked pod outputs, with served-sN/config.json (the run's config, from the seal's
#         claude-262/r/served-sN/config.json), served-sN-snapshot.json,
#         served-sN-scored/samples-memory.json, served-sN.score.exit and served-sN.snap.exit, N = 1, 2
#   TREE: the checkout whose local/claude/target/release holds fbs-export and fbs-toks, built with
#         --features serving-wasm; its integer and tokenizer crates and Cargo.lock must equal 0073cd07b's
#   OUT:  must not exist
set -u
set -o pipefail
ROOT=${1:?usage: serve-i3.sh ROOT TREE OUT}
TREE=${2:?usage: serve-i3.sh ROOT TREE OUT}
O=${3:?usage: serve-i3.sh ROOT TREE OUT}
HEAD_PIN=0073cd07bd5d16021639041aa30c55132d2c6dc9
B=${TREE}/local/claude/target/release
[ -e "${O}" ] && { echo "refused: ${O} exists"; exit 2; }
git -C "${TREE}" diff --quiet "${HEAD_PIN}" -- crates/uor-r4-integer crates/uor-r4-tokenizer Cargo.lock \
  || { echo "refused: ${TREE}'s integer and tokenizer crates or Cargo.lock differ from ${HEAD_PIN}"; exit 2; }
mkdir -p "${O}/served" "${O}/parity"
{ shasum -a 256 "${B}/fbs-export" "${B}/fbs-toks"; echo "tree $(git -C "${TREE}" rev-parse HEAD), integer crates equal to ${HEAD_PIN}"; } \
  | tee "${O}/binaries.sha256"
for x in served-s1 served-s2; do
  se=$(cat "${ROOT}/${x}.score.exit" 2>/dev/null)
  sn=$(cat "${ROOT}/${x}.snap.exit" 2>/dev/null)
  [ "${se}" = 0 ] && [ "${sn}" = 0 ] || { echo "${x}: refused, score exit '${se}', snapshot exit '${sn}'"; continue; }
  "${B}/fbs-export" --run-config "${ROOT}/${x}/config.json" "${ROOT}/${x}-snapshot.json" "${O}/served/${x}.uorlut" \
    > "${O}/served/${x}.export.out" 2>&1
  echo "${x} export: exit $?"
  [ -s "${O}/served/${x}.uorlut" ] || { echo "${x}: no export"; continue; }
  shasum -a 256 "${O}/served/${x}.uorlut" | tee "${O}/served/${x}.uorlut.sha256"
  len=$(od -An -t u8 -j 8 -N 8 "${O}/served/${x}.uorlut" | tr -d ' ')
  dd if="${O}/served/${x}.uorlut" bs=1 skip=16 count="${len}" 2>/dev/null > "${O}/served/${x}.header.json"
  keys=$(jq -c '[.. | objects | select(has("copy_key")) | .copy_key]' "${O}/served/${x}.header.json")
  fmt=$(jq -c '.format // .schema // empty' "${O}/served/${x}.header.json" 2>/dev/null)
  echo "${x} I-5: copy_key values in the header ${keys} (wants []); format ${fmt}"
  f=${ROOT}/${x}-scored/samples-memory.json
  n=$(jq length "${f}")
  [ "${n}" -eq 310 ] || { echo "${x} I-3: refused, ${n} float samples, not 310"; continue; }
  jq -c 'map({position, prompt, reference})' "${f}" > "${O}/parity/${x}-prompts.json"
  "${B}/fbs-toks" "${O}/served/${x}.uorlut" --replies "${O}/parity/${x}-prompts.json" \
    --out "${O}/parity/${x}-toks.json" --reply-tokens 64 > "${O}/parity/${x}-toks.out" 2>&1
  rc=$?
  first=$(jq -n --slurpfile f "${f}" --slurpfile t "${O}/parity/${x}-toks.json" \
    '[range(0; $f[0]|length) | select($f[0][.].reply[0] != $t[0][.].reply[0])] | length')
  whole=$(jq -n --slurpfile f "${f}" --slurpfile t "${O}/parity/${x}-toks.json" \
    '[range(0; $f[0]|length) | select($f[0][.].reply != $t[0][.].reply)] | length')
  order=$(jq -n --slurpfile f "${f}" --slurpfile t "${O}/parity/${x}-toks.json" \
    '[range(0; $f[0]|length) | select($f[0][.].position != $t[0][.].position)] | length')
  bar=$(( n * 29 / 1000 ))
  echo "${x} I-3: fbs-toks exit ${rc}; first tokens differ ${first} of ${n} (bar ${bar}); whole replies differ ${whole}; order mismatches ${order}"
  if [ "${keys}" = "[]" ] && [ "${rc}" -eq 0 ] && [ "${order}" -eq 0 ] && [ "${first}" -le "${bar}" ]; then
    echo "${x} I-3: PASS"
  else
    echo "${x} I-3: FAIL"
  fi
done
```

## Evidence

HF store `results/claude/one-model-end-to-end-i3-repaired-2026-11-03/`, read back on 3 Nov:
- `claude-263-a.tar` SHA-256 `3a364efdc5c0bd633142d3ef21c3ef6a6d2404e00308e4f46e0f3d08f9cbaaf5`: the pod's `/workspace/i3` without the fetched inputs. Its 46-file seal checks on the laptop.
- `claude-263-a-serve.tar` SHA-256 `7216cd08cd118c02b5bd4eee9abe60a9aac1c44a93b3a5baf2a3f3eb0d8eb96e`: the exports, headers, prompts and `fbs-toks` replies.

## Checks run at the head

- On the pod at `0073cd07b`, I-0: `cargo test --release --locked -p uor-r4-float-train --features cuda --lib -- --test-threads=1`. Result: 296 passed, 0 failed, 7 ignored.
- On the laptop:
  - `cargo test --locked -p uor-r4-harness --no-fail-fast`: 677 lib tests passed, 0 failed, 1 ignored, and every other test target passed. That covers both sheet tests that failed in CI.
  - `uor-harness loop report --check` and `uor-harness loop pr-check --local` on this REPORT.
- Before every push, the no-overlap check against #3583. No file of #3583's six is in this ORDER's scope except the notebook.

## Skills used

None applied. The scripts are `job-i3.sh`, `serve-i3.sh` and `watch-i3.sh`. They live in `local/claude/` and are not committed as files. Each was named in the reviewed sheet. `serve-i3.sh`'s text is in this REPORT. `job-i3.sh` ran from `/workspace/job.sh`, outside the sealed folder, so it is recorded only by the SHA-256 checked on the pod (Measurements).

## Still open

1. **Localize the parity fault: the next unit, claude-264.** The part A sheet names the method: run the existing `fbs-toks` and `--score-rows` on a truncated config until the first layer where float and integer part, with no new trace tool. Its named candidates are claude-262's two one-unit edges in the float stand-in (claude-262 REPORT, still open 5):
   - `floor(x + 0.5)` rounds the f32 tie at `0.5 − 2^−25` up;
   - the shift is chosen from the unrounded row maximum.

   This REPORT adds three pointers:
   - the float side's memory is not needed to explain the gap (item 3);
   - the integer memory path is unchecked, so an integer serve with the turn memory off comes first;
   - the first token parts on more than a third of prompts (item 4).

   One more existing chain fits the method: `fbs-snapshot --dev` writes the float trunk's plain, export and D11 readouts on a DEV sample, and `fbs-compare` attributes the float-to-integer gap to each snap and to table rounding. Whether those readouts follow the as-served forward is its first question.
2. **Then the full-length served run**, by its FIT sheet, once I-3 passes at a repaired head.
3. **A rule proposal for Casey:** name `one-model-end-to-end` in the run file's umbrella exception (#2028, 6600465605).

## Plain summary

claude-262 fixed how the model trains "as served". This unit checked whether the integer version the laptop runs gives the same answers as the trained model. It does not yet. On 310 test prompts, the laptop's first word differs from the trained model's own first word on 112 at one seed and 137 at the other. Before claude-262's fix it was about 300, on claude-261's longer runs, a different configuration. The trained model gives the same first word with or without the memory that recalls replies, so that memory is not needed to explain the gap; the laptop's own memory path is still to be checked. The full-length training run therefore waits. The next unit finds the one layer where the two versions part and fixes it. GPU cost: one pod, about 28 minutes, about $0.26.

References #3407.
