# DECISION: P1's S5 copy path is A1, Pick over the D11 trunk codes (`CHARTER.md` §5.1, D68)

- **Lab:** `claude` (top session), 19 Nov 2026.
- **Order:** [ORDER.md](ORDER.md). **Decision review:** [DECISION-REVIEW.md](DECISION-REVIEW.md).
- **Evidence:** claude-439's [REPORT](../claude-439/REPORT.md) and its [`run2/`](../claude-439/run2/)
  packet (100% of C, wick's RTX 4090, code head `932234a4b`). No new number is measured here.
- **Authority:** §5.1 and D68. A scope choice goes ahead when the lab's own whole-project decision
  review passes. It does not wait on the owner, and it is logged for Casey, who can reverse it.
  Nothing in §3 or the caps changes. [claude]

## The decision

P1's S5 session-copy expert is **A1**: the Pick read over the D11 trunk codes with Identity
transport, latest position on a tie and then the lowest original id, as fitted on reserved TRAIN in
claude-439. P1(b)'s deployable count model carries A1's S5 column. [claude]

codex-28 binds a reserved-TRAIN S5 fit on claude-439's split (its `count/INPUTS.md` checks
claude-439's split hashes), and its REPORT names the genuine reserved-TRAIN refit as still open.
That REPORT does not say which arm's Pick column its packer carries. So the unit that does the refit
and packs the mixture adopts A1's column, or reconciles its own with A1's on the record. The
question goes to #4063, not into codex's files. [claude]

Every unit that measures or changes S5 keeps these controls, the ones codex-14's DECISION lists:
**A0** (pure count), **C1** (read disabled), **C2** (exact token pairs), **C3** (the same read over
a permuted token → code map), original-id ties, and the writable, reloadable D11 carrier. [claude]

## Evidence (claude-439, DEV-tune at 100% of C, micro-BPB)

| Arm | DEV-tune all | DEV-tune novel | Read cost at 100% |
|---|---|---|---|
| A0 pure count | 1,432,620 | 1,747,432 | none |
| A1 Pick, D11 codes | 1,360,806 | 1,651,188 | about 9 min device read per pass |
| C2 exact pair | 1,365,580 | 1,657,701 | about 1,300 ms on the host |
| C3 rotated codes | 1,360,932 | 1,651,364 | about 9 min device read per pass |

Paired bands (2,670 records, 1,000 resamples): A1 − C2 [−5,232, −4,318] overall and
[−7,141, −5,889] on novel targets; A1 − C3 [−232, −24]; A1 − A0 [−77,388, −66,248]. [claude-439]

## Why A1

- **It is the better measured path, by a band that clears 0 widely.** A1 beats C2 by 4,318 to 5,232
  micro-BPB out of sample, about 3½‰ of A1's bits, and by more on novel text. Of the measured arms
  only C3 comes close, and C3 costs the same device read. So C3 is never the cheaper choice. [claude]
- **The measured cost is small.** A1's measured extra cost over C2 is offline: about 9 min of device
  reads per pass on wick, inside an hour-scale run. P1's budget is ≤ 1 GPU-day. At serving time the
  read is per token over one session's context. Its cost on the M1 has not been measured, so the
  reversal rule below covers it. [claude, est]
- **It reuses the track's operator.** claude-409's card records the gap order's step (1) as "the
  Pick read | done (claude-408, #4045); now S5, the output expert and the pointer copy". P2's row
  carries "the output Pick expert, claude-408's CPU Pick applier and test gaps". A1 reuses the
  operator that P2 already carries. Choosing C2 as S5 would keep that operator for P2 and add a
  second copy operator to hold bit-identical across train and serve (D85). [claude]
- **Its footprint is not measured.** At serving time A1 needs each session token's D11 code through
  the transport, plus a per-position code history to search. C2 needs only token ids. Both add S5's
  fitted weights. Neither path's deployed bytes or per-session state were measured [est]. codex-14
  asks to "record the entire deployed footprint", so the P1(b) unit records S5's deployed bytes and
  per-session state for A1 and for C2. [claude]

## What this card does not decide

- **The geometry label.** claude-439's reading stands: `geometric_gain`, small. The D11 codes beat a
  permuted code map by only 24 to 232 micro-BPB, and that edge narrowed at every size of C. C3
  suggests that most of A1's lead over C2 comes from firing at distance > 0 under any code map, not
  from the codes. This card chooses A1 for its measured bits. It does not credit the D11 codes with
  that lead. P2's bar (≥ 1% fewer DEV-test bits than the best of three equal-byte controls) still
  decides geometry. [claude]
- **A cheaper nearest-pair path.** A read that fires on the nearest earlier pair under a cheap
  injective map might hold most of A1's lead at C2's cost. That is an untested hypothesis
  (claude-439, "Still open"). A unit that tests it keeps A1 as the incumbent and C2, C3 as controls.
- **Per-distance weights.** EM gives Pick about 10‰ of the mixture in the distance > 0 class,
  against 190‰ at distance 0 (target-weighted over R's buckets, claude-439). A split by distance (1, 2, 3+) may change that. That is a future unit, not a choice made
  here.

## Reversal rule

A new card switches S5 to C2 if any of these holds:

1. **Bits.** At P1(b)'s deployable size, the paired A1 − C2 band does not lie wholly below 0. That
   covers a band that includes 0 and one wholly above 0. The band is read on DEV-tune, and on the
   sealed DEV-test only if P1(b)'s prep sheet registers a C2 arm before its one sealed run. A sealed
   read cannot gain an arm afterwards (§4.5).
2. **Speed.** The served model with A1 misses claude-409's bar B speed (≥ 1,000 tok/s warm at 2k
   context on one M1 performance core), and the same model with C2 meets it.
3. **Memory.** The served model with A1 misses claude-409's bar B state limit (≤ 4 GB resident),
   and the same model with C2 meets it. The P1(b) footprint record above feeds this rule.

**Replay is required, not optional.** The unit that packs A1 into the D11 artifact carries a
bit-identical CUDA-to-CPU Pick replay: route, tie and read choice, on ≥ 64 prompts, the count in
claude-409's bar B. It also carries the gate negative that claude-439 left unrecorded (its REPORT,
"Still open"). A failed replay is a bug to fix, not a reason to switch. [claude]

## Alternatives turned down

- **C2 now.** It is cheaper offline, but 4,318 to 5,232 micro-BPB worse. Its saving is minutes of
  wick time per pass. Revisit only under the reversal rule. [claude]
- **C3.** Same read cost as A1, slightly worse, and its code map is a control by design. [claude]
- **Both A1 and C2 as separate experts.** C2 fires on 4,108,760 DEV-tune targets and A1 fires at
  distance 0 on 4,111,759, so a C2 column would presumably repeat most of A1's distance-0 class (not
  checked). No arm measured the pair. A unit may test it under its own sheet. [claude]

## Cost to undo

Low. The choice is one column in the S5 fit and the packer. C2's column can be recomputed on the host
in about 1,300 ms from the same split, and claude-439's code keeps every arm. [claude]

## Index row: not written

`docs/integration/DECISIONS.md` is an owner path (`docs/harness/owner-paths.json`), and this unit has
no `## Owner approval` section, so the card lives here, as claude-409's did. [claude]
