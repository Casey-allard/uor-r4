# REPORT: cpu-training-pilot, deepseek-108 (point vote complete replies)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Prep:
[point-vote replies](../../../../harness/prep/cpu-training-pilot-point-vote-replies-2026-11-03.md).
Base: `a66028b0e`.

## What did not work, first

- **The first FIT review rejected the decoder contract.** It used EOS token 1,
  while the project's integer panel path stops at BOS id 0 through
  `reply_stop`. The driver now uses that established contract.
- **The second FIT review rejected the reachability test.** Reachability was a
  supplied boolean, not a tested computation. A pure boundary helper now tests
  exact cap/context, cap plus one and context plus one.
- **The point vote's measured correctness gain does not propagate into
  complete replies.** Both seeds have vote opening top-1 `3/60`, but all four
  arms have zero exact-reference and zero BOS-complete replies. All 60 replies
  in every arm hit the 16-token cap.
- **Seed 1 gets worse under the vote on some reply statistics.** Vote produces
  4 distinct non-empty replies and 57 repeated-4-gram replies, against nearest
  row's 10 and 55 and point-off's 2 and 22. Fewer distinct replies with more
  repetition is not a chat improvement.
- **Exact-reference coverage is bounded.** Nine of 60 stored references are
  unreachable under the combined 16-token/remaining-context rule, leaving 51
  reachable references. Even among those, no arm matches an exact complete
  reply.

## What changed

`cpu-point-transfer-driver --reply-cap 16` now performs deterministic greedy
complete replies over each existing matched arm:

- point-off;
- nearest point row;
- eight-row point vote;
- eight-row shuffled-target vote.

It feeds each generated token back through the CPU FBS session, stops through
the shared integer `reply_stop` contract, and records:

- BOS, cap and context stops;
- exact-reference and opening matches;
- reachable reference count;
- non-empty and distinct replies;
- overlapping repeated 4-grams;
- generated/reference tokens, wall time and aggregate generated tokens/s.

The finite suite is 13 tests. The source change is confined to the existing
point-transfer driver and task records.

## Measurements

Both seeds used 16 Rayon workers concurrently on the Ryzen 9 9950X.

| Seed | Arm | Exact | BOS | Cap | Opening | Distinct | Repeat 4-gram | Generated tok/s |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | point-off | 0 | 0 | 60 | 0 | 2 | 22 | 20,303 |
| 1 | nearest | 0 | 0 | 60 | 2 | 10 | 55 | 20,358 |
| 1 | vote | 0 | 0 | 60 | 3 | 4 | 57 | 20,074 |
| 1 | shuffled vote | 0 | 0 | 60 | 0 | 11 | 46 | 20,220 |
| 2 | point-off | 0 | 0 | 60 | 0 | 2 | 40 | 20,904 |
| 2 | nearest | 0 | 0 | 60 | 1 | 12 | 37 | 20,739 |
| 2 | vote | 0 | 0 | 60 | 3 | 5 | 22 | 20,189 |
| 2 | shuffled vote | 0 | 0 | 60 | 3 | 8 | 6 | 19,203 |

The two registered seeds ran concurrently and alone; the seed-1 repeat ran
afterward in a separate process. Registered wall times were 1.616 s and
1.550 s. The tokens/s figures are aggregate
generated tokens divided by each arm's wall interval over all 60 prompts; they
are not single-stream decode rates.

The seed-1 repeat reproduced every complete-reply reading and every per-arm
ordered-token SHA-256 exactly after timing fields were removed. Digests are:

- seed 1 point-off `ea9a314d…8eeb`, nearest `b4fff629…38e4`, vote
  `7bf63e47…d1ea9`, shuffled vote `7e2e32b4…b134`;
- seed 2 point-off `11f8e912…d878`, nearest `faa0386b…467e`, vote
  `69e4fd0e…9d7b`, shuffled vote `6767a039…e5c3`.

Every teacher-forced arm retained zero query-coordinate, token and kept-cell
mismatches over 2,110 positions, 540,160 Q12 coordinates, 2,110 decisions and
16,880 kept cells.

Artifact hashes remain identical to deepseek-106:

- seed 1 point `ecd2c616…7ec56`, vote `0dd5a835…8fac`, random `85ed6785…40569`;
- seed 2 point `5f127f79…08ece`, vote `cd76559c…eee6`, random `48b4e621…b52ff`.

The detailed records are [comparison.json](evidence/comparison.json).

## Reading

The eight-row vote remains a served opening mechanism. It fails this screen's
propagation bar because exact-reference complete replies do not strictly
exceed any control on either seed `(0 = 0)`. The result does not reject all
point votes or all complete-reply training; it closes this fixed 60/60,
16-token, greedy configuration.

## Checks run at the head

- Focused point-transfer driver tests: 13 passed before public compute.
- FIT: non-author review, FIT before public compute.
- Both public seeds and a seed-1 exact repeat: passed.
- Full CPU release tests: 163 library tests passed with 4 release benchmarks
  ignored; all binary suites passed, including point transfer `13/13`.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- Package formatting, diff whitespace, FIT schema, evidence checksum and JSON
  validation: passed.

## Skills used

No available Codex skill applied.

## Still open

- No tested point-vote configuration produces a complete reply.
- This panel shares prompt templates and is not fresh-phrasing or semantic
  correctness evidence.
- A materially different training mechanism, not a larger point-vote width, is
  needed for reply content.

## Plain summary

The previous unit made an eight-nearest-neighbour vote part of the model file.
It improved which token appeared first in a held-out reply on three of 60
prompts. This unit fed every generated token back and asked whether that small
gain became a whole reply.

It did not. Every arm generated 16 tokens without reaching BOS, matched no
stored reference reply, and produced very few distinct texts. The vote is not
ready for the chat path; the next work should target reply content and
termination rather than another point-vote variant.
