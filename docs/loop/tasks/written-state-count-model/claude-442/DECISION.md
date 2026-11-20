# DECISION: S5 takes the per-distance Pick split at its next refit, bundled (`CHARTER.md` §5.1, D68)

- **Lab:** `claude` (top session), 20 Nov 2026.
- **Order:** [ORDER.md](ORDER.md). **Decision review:** [DECISION-REVIEW.md](DECISION-REVIEW.md).
- **Evidence:** claude-441's [REPORT](../claude-441/REPORT.md) and its [`run1/`](../claude-441/run1/)
  packet (100% of C, wick's RTX 4090, build `aa36f9414`), and the S5 code at `47c0c82c8`. No new number
  is measured here.
- **Authority:** §5.1 and D68. A scope choice goes ahead when the lab's own whole-project decision
  review passes. It does not wait on the owner, and it is logged for Casey, who can reverse it.
  Nothing in §3 or the caps changes. [claude]

## The decision

S5's Pick buckets split by pair distance at **the next S5 refit**: the R-only fit that freezes the
full-C candidate, which codex-33's REPORT names as the next step. The split's rule is the one
claude-441 measured: class 0 when Pick did not fire, otherwise class 1 + min(d, 3), where d is the
read's pair distance. That gives 5 classes and 880 buckets at the highest order 10, against 3 classes
and 528. [claude]

The change rides with that refit. No unit is spent on the split alone, and codex-33's C1 artifact is
not refitted for it. If the freeze unit is otherwise ready and the split's support is not, the
candidate ships with the three-class rule and the split waits for the refit after it. The split never
delays P1(b). [claude]

A1 stays S5's copy path (claude-440's card), and every control that card keeps stays: A0, C1, C2, C3,
original-id ties and the writable, reloadable D11 carrier. [claude]

## Evidence (claude-441, DEV-tune at 100% of C, micro-BPB)

| Arm | DEV-tune all | DEV-tune novel |
|---|---|---|
| A1, three classes (the deployed rule) | 1,360,806 | 1,651,188 |
| **A1s, split** | **1,360,781** | **1,651,154** |
| C3, permuted codes, three classes | 1,360,932 | 1,651,364 |
| C3s, permuted codes, split | 1,360,913 | 1,651,338 |

Paired bands (2,670 records, 1,000 resamples): A1s − A1 [−32, −16] overall and [−44, −23] on novel
targets; C3s − C3 [−27, −10]; A1s − C3s [−241, −28], against A1 − C3's [−232, −24]. [claude-441]

By class (A1s's DEV-tune targets, and EM's Pick weight over R's buckets): 5,340 targets where Pick
did not fire; distance 0 holds 4,111,759 at 190‰; distance 1 holds none; distance 2 holds 36,362 at
37‰; 3 or more holds 6,653,956 at 10‰. The five classes sum to the 10,807,417 scored targets. A copy at distance 2 hits about 17% of the time, one at 3 or more about 8%. [claude-441]

## Why the split, and why only bundled

- **Its gain clears 0, but it is tiny.** A1s beats A1 by 16 to 32 micro-BPB, about 2 parts in
  100,000 of A1's bits. That is below anything P1's bars resolve: bar (b) is a 3% margin over R\*. The
  split earns its place only because it costs almost nothing. [claude]
- **It costs nothing at run time.** The serving read already returns the distance. `code_pick`
  returns `(p, d(p), ties)`, and `runtime/rank.rs` uses `d` only to tell exact (0) from near (above 0).
  The split changes that mapping and the bucket index. It adds no read, no lookup and no per-session
  state. [claude, read at `47c0c82c8`]
- **It costs about 39 KB in the artifact.** `FrozenPickMixture` stores bucket_count × classes ×
  (experts + 1) Q32 weights as u64 words: 176 × 3 × 14 = 7,392 words now (codex-31's 59,180-byte file
  with its header), and 176 × 5 × 14 = 12,320 words split. That is 39,424 bytes more, about 17 parts in
  a million of codex-33's 2,316,604,155 model and tokenizer bytes. [claude, from the code]
- **It costs one format change and one replay field.** `word_count` in `frozen_pick.rs` fixes 3
  classes, and the `UORS5W01` header carries no class count. A split fit needs a format version that
  records its class count, with the three-class file still loading. The CUDA fit port (codex-31's
  `fit/device.cu`, "the full 528×14 Q32 state") has to run at 880×14. Each of these is a change the
  refit unit already makes to the same files when it refits; on its own, each would be a unit's
  overhead for a 2-in-100,000 gain. Hence bundled, never alone. [claude]
- **It credits no geometry.** C3s − C3 is nearly as large as A1s − A1, and the codes' edge over a
  permuted map is the same band before and after the split. The split refines how far Pick trusts a
  read by its distance; it does not show that the D11 codes are what makes the read good. [claude]

## What the refit unit carries

1. **One class rule, shared.** The serving runtime and the fit use one function for the class.
   claude-441's `pick_split_class` is at `kn_ceiling.rs:179` today. The refit unit moves it into
   `runtime/mixture.rs`, which `count_runtime.rs:23` and `kn_ceiling.rs:25` both include, and
   `runtime/rank.rs:23-27` calls it in place of its three-arm match. Two copies of the rule would
   break §3.7's "same typed operator definitions" in training and serving (D85).
2. **Both arms on R and DEV-tune.** The unit fits A1 (three classes) and A1s (split) on the same
   reserved TRAIN rows and reports A1s − A1 on DEV-tune before it freezes either. Measured at C1
   (codex-31): 270 s of feature preparation, shared by both arms, and 2.37 s of GPU optimizer per
   arm. At full C the cost is unmeasured [est: larger, and still shared].
3. **The replay covers the class.** The unit's bit-identical CUDA-to-CPU replay checks route, tie,
   read choice and class on ≥ 64 prompts (claude-409's bar B count, as claude-440's card requires),
   with a planted negative: a target whose distance is moved across a class boundary must fail it.
4. **The three-class format keeps loading**, so codex-33's artifact and its 192/192 client replay
   stay valid.

## Reversal rule

A new card, or the refit unit's own REPORT, returns S5 to three classes if any of these holds:

1. **Bits.** At the refit, the paired A1s − A1 band on DEV-tune does not lie wholly below 0. Here
   DEV-tune only selects the arm; it is not the held score. The
   sealed DEV-test gets an A1s − A1 arm only if the candidate's prep sheet registers it before its one
   sealed run (§4.5); this card does not require one.
2. **Cost.** The split's support makes the refit unit miss its own sheet's budget or delays the
   frozen candidate. This defers the split to the next refit; it is not a finding against it. The
   refit unit's REPORT records the deferral and PLAN's P1 row says so. If that refit was P1(b)'s
   last, P1(b) is three-class, and the split waits for the first S5 refit after it (P2's equal-byte
   arms or a later freeze).

Speed and memory need no clause of their own. The split adds no read and no per-session state, and
39 KB of weights; the refit unit's bar B checks (claude-409: ≥ 1,000 tok/s warm, ≤ 4 GB resident)
time and size the served model with whichever rule it packs. A failed replay is a bug to fix, not a
reason to switch. [claude]

## Alternatives turned down

- **Keep three classes for good.** It gives up a measured gain whose band clears 0 for 39 KB and a
  format version. Turned down while the split can ride a refit that happens anyway; it is the fallback
  under the reversal rule. [claude]
- **A unit of its own to refit codex-33's C1 artifact now.** That artifact is P1(a)'s count-identity
  test on the 1% slice, not the candidate. A unit spent there buys 2 parts in 100,000 on an artifact
  no bar scores (L-008). [claude]
- **Four classes, dropping distance 1.** No target fell at distance 1, and claude-441 reads that as
  the D11 code set having no two codes 1 bit apart; nobody has checked the code set. `code_pick`'s d
  is the sum of two Hamming distances between codes, one per token pair, so d = 1 needs one pair at 0
  and the other at exactly 1. If no two distinct codes are 1 bit apart, each pair adds 0 or at least
  2, and d = 1 cannot occur. Then the four-class rule addresses the
  same weights for every target and scores the same, saving 176 × 14 × 8 = 19,712 bytes. The refit unit
  may use it only after it reads the code set and shows that. Otherwise the measured five-class rule
  stands. [claude]
- **A finer split of 3 or more.** That class holds 6,653,956 targets at 10‰ and is where most of
  Pick's near reads sit. `code_pick` returns the full d, so a finer split is buildable, but it is
  unmeasured. A unit may test it under its own sheet. [claude]

## What this card does not decide

- **The geometry label.** claude-441's reading stands: `split_gain_geometric`, tiny, and mostly not
  from the codes. P2's bar still decides geometry. [claude]
- **Any change to codex's units.** The card asks the refit unit for the four items above. The note
  goes to #4063, not into codex's folders. [claude]

## Cost to undo

Low. A1 and A1s are both fitted at the refit (item 2), the three-class format keeps loading (item 4),
and returning to three classes is a choice of which fitted file to pack. [claude]

## Index row: not written

`docs/integration/DECISIONS.md` is an owner path (`docs/harness/owner-paths.json`), and this unit has
no `## Owner approval` section, so the card lives here, as claude-409's and claude-440's did. [claude]
