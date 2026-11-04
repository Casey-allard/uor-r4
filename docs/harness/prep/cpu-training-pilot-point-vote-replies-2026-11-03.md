# CPU point-vote complete replies

**Status:** FIT

TEST FITNESS: FIT

## Question

Does the artifact-bound eight-row point vote affect full greedy replies after
the first token, or is its effect limited to the teacher-forced opening row?

## Base artifact

| Input | Identity |
|---|---|
| seed 1 FBS artifact | `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd` |
| seed 2 FBS artifact | `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1` |
| DEV tokens | `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` |
| DEV response mask | `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` |
| corpus manifest | `ab269c9818a713ef41f9aff0fdd8995386d5eb0ebaf8a98636f7766e99f83eab` |
| hardware | AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads |

## Data split

Use the fixed deepseek-104/105 selection without change:

- 60 training examples and 60 evaluation examples;
- eight training and eight evaluation first-eight-token keys;
- zero key overlap;
- 48 production training-derived point rows;
- random-target seed 1.

## Steps

1. Build a fresh release `cpu-point-transfer-driver`.
2. Run finite tests for bounded greedy replies before reading public data.
3. Run both public seeds with `--neighbor-vote 8 --reply-cap 16`.
4. Record point-off, nearest, vote and shuffled-vote complete-reply readings.
5. Preserve all teacher-forced opening, full-sequence and parity arms.

## Candidate

For each evaluation prompt:

1. step every prompt token through the artifact;
2. mark reply start before the final prompt token;
3. take the model's first generated token from that step;
4. feed it back and repeat through the integer `reply_stop` contract until BOS,
   16 generated tokens, or context end;
5. normalize the held-out reference by removing one trailing BOS token, then
   compare the complete remaining id sequence.

The candidate is the artifact whose point table records `vote_k = 8`.

## Bar

Instrument and exactness:

- finite controls prove prompt tokens are not fed twice;
- BOS, cap and context stopping are deterministic and use `reply_stop`;
- exact-reference normalization removes one trailing BOS and does not trim any
  other token; a reference longer than the cap or remaining context is counted
  as unreachable rather than as a model miss;
- exact, distinct and repeated-4-gram counters are independently tested;
- all teacher-forced arms retain zero query, token and kept-cell mismatches.

Candidate:

- on each seed, report exact-reference complete replies, BOS-complete replies,
  opening top-1, distinct non-empty replies and repeated-4-gram replies;
- report per-arm wall time and generated tokens/s;
- the candidate carries to a larger run only if exact-reference complete
  replies strictly exceed point-off, nearest and shuffled-vote on both seeds;
- report the paired complete-reply and opening differences separately. No
  three-sigma claim is made from 60 prompts.

Definitions:

- a distinct reply is one exact generated token vector; the empty vector is not
  counted;
- a repeated-4-gram reply contains the same overlapping four-token window in
  two positions inside that reply;
- generated and reference token totals exclude the trailing BOS stop.

## Minimum test power

Two public model seeds, 60 held-out unseen-key prompts per seed, four matched
arms and a 16-token generated cap. This is a bounded readiness screen, not a
decisive capability test; no three-sigma or useful-chat claim is made from 60
prompts. A replicated propagation result must still be carried to a larger
complete-reply panel.

## Cost and stop

Local Ryzen 9 9950X only; no GPU, pod or paid API. The run is four greedy arms
over 60 prompts for each of two seeds, capped at 16 generated tokens each.
Expected cost is seconds on the local CPU. Stop on input drift, nondeterministic
generation, counter overflow, split or cap changes, or any attempt to call an
exact-reference match semantic correctness.

## Outcomes

- **No propagation:** the vote may remain a served opening mechanism, but it
  does not enter the chat path as a reply-quality candidate.
- **Replicated propagation:** carry the exact same vote into a larger complete
  reply panel. This 60-prompt screen is not capability evidence.
- **Instrument fault:** stop, repair and rerun; do not record a mechanism
  negative.

## Stop

No tuning after observing the two seeds. Do not change the split, `vote_k`,
cap, random seed or metrics. Do not call exact-reference agreement semantic
correctness or useful chat.

## Verdict

TEST FITNESS: FIT - M2 #2030
sheet: docs/harness/prep/cpu-training-pilot-point-vote-replies-2026-11-03.md
reviewer: Codex fresh-context non-author review
note: fixed deepseek-104/105 60/60 split, four matched arms, established
reply_stop/BOS semantics, one-trailing-BOS reference normalization, exact
cap/context reachability, per-arm cost, distinct/repeated-4-gram counters,
12/12 finite tests at FIT, 13/13 including the post-review sequence digest;
readiness screen only
date: 2026-11-03

A FIT verdict clears compute for the configuration in that sheet only (D22).
