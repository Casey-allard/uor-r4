# ORDER: cpu-training-pilot, deepseek-163 (longest-32 copy depth)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `7fe55fdbf`.

## Goal

Test whether exact suffix copy matches deeper than four improve complete-reply
body accuracy. Deepseek-149 selected depth four from a depth-one-through-four
search; depths 8-32 have not been tested. This is a content-selection screen,
not a termination or language result.

## Cancel probe

Deepseek-153's selected longest-4 arm reached 409/4,661 train and 556/4,614
dev correct body positions with zero complete replies. No prior unit evaluated
longest exact suffix depths above four on generated replies.

## Fixed construction

Add an isolated `longest_32_to_1` arm to the existing 192-token exact copy
decoder. Search depths 1 through 32, retain the deepest available match, and
fall back through shorter depths exactly as longest-4 does. Keep the existing
decoder, writer fallback, memory and stopping path unchanged.

Controls are longest-4, `k2>k1`, fixed depth 1, random available depth in
1..32, and the unmodified writer. The same artifact seeds and generation cap
are used.

Fit the rule on sealed train panel 2711 as an in-sample diagnostic. Evaluate
unchanged on a fresh `cell=dev,dev` panel seed 2752 generated only after FIT
and GO. Each artifact seed must have at least 200 reference-reachable replies
and a non-vacuous random control.

## Bar

For both artifact seeds, `longest_32_to_1` must beat `longest_4_to_1`,
`k2>k1`, fixed depth 1, random depth and writer on correct reference body
positions by the paired two-sigma rule. It must:

- select at least one depth greater than four on the fresh panel;
- preserve all 200 reachable rows;
- preserve zero premature BOS stops and zero ungated token mismatches;
- repeat byte-identically in a fresh process after timing fields are removed.

Zero exact and BOS-complete replies are expected because this arm changes
content selection only. A body-position gain is a bounded content result; it
does not qualify complete replies or chat.

## Stop rules

Stop on input, artifact, tokenizer or panel drift; inaccessible source or
context; fewer than 200 reachable rows; a vacuous random control; a missing
deep match; reference or token mismatch during replay; nondeterminism; or any
need to change serving, CUDA, float training or Claude-owned source.

## Files in scope

- `crates/uor-r4-cpu-train/**`
- `docs/harness/prep/cpu-training-pilot-copy-depth32-2026-11-05.md`
- `docs/loop/tasks/cpu-training-pilot/deepseek-163/**`
- `docs/labs/cpu-training-pilot-copy-key-reply-2026-11-05/**`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`
- `docs/integration/current-state.md`
- `docs/harness/claims.json`

## Done when

1. Longest-32 selection, exact replay and depth accounting have finite tests.
2. FIT and GO precede the fresh panel draw.
3. Both artifacts run on the fresh panel with all controls and repeat.
4. Evidence reports body-position gain or rejection without claiming complete
   replies.
5. Exact-head review, protected merge and fresh-main verification complete.
