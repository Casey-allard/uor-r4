# REPORT: direct final-gain BOS training, deepseek-131

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[BOS gain](../../../harness/prep/cpu-training-pilot-bos-gain-2026-11-04.md).
Base: `7a77bce37`.

## What did not work

- **The registered termination bar failed on both seeds.** Held-out terminal
  BOS top-1 remained **0/103** against a required 26/103. Shuffled-label and
  equal-move random controls also scored zero, so both paired gains were zero
  against threshold two.
- **The trainer moved 8/8 proposed final-gain coordinates on both seeds, but
  changed distance rather than rank.** Seed 1 reduced terminal BOS distance
  from 116,475,330 to 115,987,630 over training and from 123,149,944 to
  122,624,236 on held-out decisions. Seed 2 reduced them from 118,425,828 to
  117,946,250 and 124,406,100 to 123,896,520. BOS admission/top-8 and terminal
  top-1 did not move.
- **Seed 2 lost nonterminal content.** Frozen nonterminal content top-1 was
  288/3,662; trained was 286/3,662. The training objective improved local
  content by two rows, but that gain did not transfer.
- **Free generation did not terminate.** Every arm stopped all 103 replies at
  the 16-token cap: zero BOS-complete and zero exact-reference replies on both
  seeds. Seed 1 produced two distinct non-empty texts per arm with 24 repeated
  4-grams; seed 2 produced two in the controls and three trained, with 43
  repeated 4-grams.
- **This does not reject all final-gain or termination training.** It closes
  this eight-move, one-code lexicographic objective on this repeated-key,
  document-disjoint panel. It does not test a direct serving-level BOS
  candidate, longer schedules, other geometric objectives or GPU training.

The repeat's first shell diff still included nested per-arm token-generation
timing fields. The registered definition excludes timing and resource fields;
recursive non-timing normalization left every report field equal and all four
seed-1 artifacts byte-identical. No run data or artifact changed.

## What changed

Added `CpuTerminationGainTrainer` and `cpu-termination-gain-driver`.

Training marks the decision whose final reply token is corpus EOS as terminal
and all other decisions as nonterminal. It caches the exact pre-final-gain
normalization, fixed memory movement, content target and writer query path.
Each candidate changes one final-gain coordinate by one signed code and is
scored through Q12 conversion, memory movement, writer admission, point
fallback and the complete CPU/integer audit.

A candidate must not add nonterminal BOS selections or reduce nonterminal
content top-1. Among safe candidates, the objective improves lexicographically
by terminal BOS top-1, admission, top-8, lower BOS rank sum, lower BOS distance
sum and nonterminal content top-1.

The run compares four arms: frozen memory baseline, true-label training,
class-preserving shuffled-label training and an equal-move random final gain.
Shuffled and random controls fail closed if unchanged or vacuous.

## Measurements

The fixed split selected 16 eight-token key families. Even documents supplied
98 training samples / 3,969 decisions (98 terminal, 3,871 nonterminal); odd
documents supplied 103 evaluation samples / 3,765 decisions (103 terminal,
3,662 nonterminal). Key families repeat by design; this is not unseen-key or
fresh-phrasing evidence.

The true and shuffled trainers each accepted 8 moves on both seeds; the random
control applied 8 matching `+/-1` moves.

| Seed | Arm | Terminal BOS top-1 | BOS admitted | BOS top-8 | Nonterminal BOS | Content top-1 |
|---:|---|---:|---:|---:|---:|---:|
| 1 | frozen | 0/103 | 1/103 | 0/103 | 0/3,662 | 341/3,662 |
| 1 | trained | 0/103 | 1/103 | 0/103 | 0/3,662 | 341/3,662 |
| 1 | shuffled | 0/103 | 1/103 | 0/103 | 0/3,662 | 341/3,662 |
| 1 | random | 0/103 | 1/103 | 0/103 | 0/3,662 | 341/3,662 |
| 2 | frozen | 0/103 | 0/103 | 0/103 | 0/3,662 | 288/3,662 |
| 2 | trained | 0/103 | 0/103 | 0/103 | 0/3,662 | **286/3,662** |
| 2 | shuffled | 0/103 | 0/103 | 0/103 | 0/3,662 | 288/3,662 |
| 2 | random | 0/103 | 0/103 | 0/103 | 0/3,662 | 288/3,662 |

Every arm on both seeds and the repeat retained zero mismatches over
**103 samples, 3,868 positions, 990,208 Q12 query coordinates, 3,868 token
decisions and 30,944 kept cells**. All CPU/integer parity checks were exact.

Run walls were **47.200 / 46.884 s**, peak RSS **341,000 / 340,276 KiB** and
32 Rayon workers. The repeat took 46.348 s at 346,212 KiB. The fresh release
build took 18 s, peaked at 1,201,512 KiB and occupied 260,319,453 target bytes.
The pipeline occupies **26,784,364 bytes**, below 64 MiB.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the binary, build receipt, three reports
and all twelve artifact copies. Raw roots remain under
`local/deepseek/termination-gain/2026-11-04-attempt-1/`.

## Checks run at the head

- Fresh FIT review 1 returned NOT_FIT for a per-candidate trajectory rebuild,
  incomplete launch/resource enforcement and incomplete control non-vacuity.
- After repair, fresh FIT review 2 returned FIT at `2b27e6383`.
- Pre-run review 1 returned NOT_GO because repeated-key scope and exact machine
  identity were not explicit; both were corrected.
- Pre-run review 2 returned NOT_GO because final-token EOS was not enforced and
  one BOS-distance field was misnamed; both were corrected with a negative test.
- Final FIT review returned FIT at `39ac7ccf3`; the final pre-run review returned
  GO at `43ac62aa0` before public compute.
- Full `uor-r4-cpu-train` release suite: **204 library and 48 binary tests
  passed, 0 failed, 4 expected ignores**. Focused termination-gain tests:
  **11/11**; driver tests: **3/3**.
- Strict no-deps all-target CPU Clippy with `-D warnings`, package formatting,
  and `git diff --check`: passed.
- D23, D25 and D32 ratchets: passed.
- All five public input hashes matched; source matched implementation pin
  `c5f9472ac`; binary SHA-256 was stable across all three runs.
- Both public seeds and the cross-run seed-1 repeat produced passing per-root
  `SHA256SUMS`; the normalized report SHA-256 is
  `3dda35a2899ca1d8afdff6073a2ff24bd0989b0941f746ee92de3c811473b82b`.
- Full loop gates were **11/11 green**. One `loop-drive` test hit a transient
  `Text file busy`; the immediate rerun passed without a source or gate change.

## Skills used

No system-provided Codex skill applied. Read-only subagents were used for the
mechanism design audit and independent adversarial FIT, pre-run and final
reviews.

## Still open

- A termination representation that changes held-out BOS rank, not only query
  distance. A direct serving-level stop candidate requires coordination because
  it changes the integer-session contract.
- Complete replies, fresh phrasing, corpus-scale training and GPU qualification.
- Claude's owned long-window P2b run and later CPU audit of its materialized
  export.

## Plain summary

This unit trained the served final gain toward BOS at the end of training
replies and preserved content during every accepted move. Both seeds accepted
all eight updates and retained exact CPU/integer parity, but held-out BOS top-1
stayed zero; seed 2 also lost two nonterminal content rows. The mechanism
changed distance, not ranking, and free generation never terminated. This is a
clean negative for the fixed coordinate objective, not a rejection of
geometric termination or a useful-chat result.
