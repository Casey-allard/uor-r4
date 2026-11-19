# ORDER: written-state-count-model, claude-439 (P1: the S5 Pick expert, fitted on reserved TRAIN, against its controls)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `9db62fe7b` (main, after claude-438's PR 4103 merged as `f72e0ba86`).

**Bar moved:** P1's S5 item, "the S5 Pick and floor experts", and the P1 control "without S5".
This is also the next step that codex-14's DECISION names:

> fit the normalized S2/S5/S6 distribution on reserved TRAIN records and rank all contenders
> before argmax. Preserve pure count, geometric read disabled, exact-token-pair and rotated-read
> controls, original-ID ties […].

This unit takes the S5 and S6 part of that step. S2 here is the existing ceiling experts. No
other lab's unit holds it: on 19 Nov ~05:35Z, codex-15, 16 and 17 (PRs 4104 and 4105 merged, 4106
in draft) all work the dialogue copy path's source selection; `claims.json` and #4063 name nothing
on the count mixer.

## The question

Does a session-copy expert, read by the Pick XOR + popcount over geometric codes, lower D87
DEV-tune bits beyond the count experts? And does it do so beyond the same expert with its geometry
removed (exact token pairs) or scrambled (rotated codes)?

## What changes

1. **Fit on reserved TRAIN, score on DEV-tune.** claude-433 and claude-438 fitted the ceiling's
   weights on DEV-tune, the set they scored, so their U_count is an in-sample ceiling.
   - Here the weights are fitted on a reserved TRAIN slice R. R is disjoint by document from the
     TRAIN slice C that builds the counts.
   - Every arm and control is scored, with its weights frozen, on DEV-tune.
   - The pure-count arm is refitted on R the same way, so every comparison is like for like.
   - The R/C split, by document id, is fixed before any score is read.
2. **The Pick expert.** For a target at position t of a session (one D87 document, within the
   read's context window), Pick takes the latest earlier position p whose (p − 1, p) block codes
   are nearest, by XOR + popcount, to (t − 2, t − 1)'s. Its gate decides whether it fires.
   - When it fires, the expert puts its whole Q48 mass, 2^48, on the token at p + 1. The target's
     mass is then 2^48 or 0.
   - The expert is present only where Pick fires. Elsewhere it is dropped from the bucket's
     simplex, as `present` already drops orders above h.
   - Ties go to the latest position, then the lowest original token id. The tie rule is recorded
     and is identical in every arm.
3. **Buckets gain a Pick dimension:** (did not fire, fired with distance 0, fired with
   distance > 0), crossed with the existing (highest order, total class) rule. That is
   3 × 11 × 16 = 528 buckets, inside S6's 4,096.
4. **One mixture, normalized by construction.** Each expert is a distribution over the vocabulary,
   so the bucket's Q32 mix is one too. Bits are read from the target's mass, as before. No argmax
   enters the score.

## Arms and controls, all fitted on R, scored on DEV-tune

| Arm | Pick expert | What it rules out |
|---|---|---|
| A0 pure count | absent | the base every arm must beat |
| A1 Pick, geometric | D11 block codes, gate from the snapshot | the arm under test |
| C1 read disabled | the expert's column built, then forced absent | a bucket-split gain with no copying |
| C2 exact token pair | codes replaced by the token ids; distance 0 only | copying with no geometry |
| C3 rotated read | the token → code map permuted by a seeded rotation, same code multiset | geometry that does no more than any code map |
| P+ positive control | A1 on DEV-tune documents each followed by a verbatim repeat of itself | whether the instrument can show a copy gain at all |

- **P+ must cut bits** on the repeated halves by a wide margin; its threshold is fixed in the
  sheet. If it does not, the instrument is broken and A1's result is not read (L-007). The owner's
  direction, as codex-16 quotes it: "check the test to make sure it is actually able to be right".
- **A geometric gain counts only if A1 beats C2 and C3,** each by the paired prompt bootstrap
  band, as well as A0 and C1. Beating A0 alone is a copy gain, not a geometry gain.

## Step 0, before any build: can A1 run on D87 at all?

The Pick read needs 16-bit block codes per token from a D11 snapshot. S0 sets V = 4,096 for both,
but nothing on main records that D87 was tokenized with the D11 trunk's tokenizer.

- If the tokenizer hashes match, A1 uses the newest saved D11 snapshot's token table and level-0
  gates. The snapshot is named, with its sha256, in the sheet.
- If they differ, A1 cannot run. The unit then ships C2 alone, labelled as a token-level copy
  control and not as S5. The REPORT says the geometric arm was not runnable, and why.

## What it builds

- `crates/uor-r4-discrete/src/kn_ceiling.rs`:
  - an optional Pick column in `CeilingTable` (mass and fired-class per target);
  - the 3× bucket rule;
  - `present` extended to the Pick expert.
  - The existing modes, starts and `gap_bound_q12` are unchanged.
- The Pick column on GPU, through the existing CUDA `MODE_PICK` read (`discrete.cu`, nvcc PTX in
  its own module):
  - checked word for word against the CPU oracle `CpuDiscretePick` on a slice;
  - its traces `[pick, d(pick), ties, gate + 1, out root]` compared as in `pick_read_cuda.rs`.
- `docs/labs/kn-reference/instrument`:
  - the R/C split;
  - the five arms and P+;
  - JSON with each arm's DEV-tune micro-BPB (overall and novel), its fit bits on R, its bound, and
    the bootstrap bands of A1 − A0, A1 − C1, A1 − C2 and A1 − C3.

## Gates

- **G1, laptop:**
  - host tests: an absent Pick expert reproduces claude-438's table and weights exactly;
  - the 528-bucket rule on fixtures;
  - a fixture where copying is right, where A1's bits drop, and one where it is wrong, where EM
    drives the Pick weight to its floor;
  - ties resolved latest-then-lowest-id;
  - every claude-433 and claude-438 test still passes;
  - stub-nvcc `cargo check`s are clean, and the D32 grep of the added lines is clean.
- **G2, negatives,** each set aside against the base, run, expected to fail, then restored:
  - Pick's tie rule taking the earliest position fails the tie test;
  - the Pick column ignoring the gate fails the wrong-copy fixture.
- **G3, wick's RTX 4090, after a sheet and a non-author FIT:**
  - the CUDA Pick = CPU oracle on a 1‰ slice;
  - then the five arms and P+ at 1%, 10% and 100% of C, with a completion watcher armed and
    results copied off wick.
  - DEV-test is not read: it stays sealed for gate (b).

## Decisions recorded here

- **Scope is codex-14's next step, with codex-14's controls.** The result then answers their
  DECISION instead of running beside it. The draft PR and a #4063 comment go up at this ORDER,
  before any code. [claude]
- **The Pick expert is a point mass, not a softened one.** EM sets how far to trust it per bucket.
  Any spreading of its mass would be a hand-set rate, and the per-bucket weight already is that
  rate, fitted. [claude]
- **wick, not a rented pod.** The run is hours of lookups at most, and wick is Casey's own GPU at
  no rental. [claude]

## Out of scope

- S3 geometric-address rows, E8 snap and every other P2 item;
- the output Pick expert (scoring candidates by their S0 codes);
- the error-write (P3);
- any change to the 13 count experts or to `EmStart`.
