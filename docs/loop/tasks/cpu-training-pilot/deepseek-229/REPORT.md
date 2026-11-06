# REPORT: cpu-training-pilot, deepseek-229

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[continuation guard sheet](../../../../harness/prep/cpu-training-pilot-continuation-guard-2026-11-06.md).
Base: `31974240d`.

## Status

Complete as **REJECT**.

## What did not work

- A confident full-sequence continuation suppressed 23 boundary decisions, but
  exact replies fell from **17 to 14** on both artifacts. It changed the late
  content trajectory and broke previously exact completions.
- Premature BOS improved only from **70/81** to **65/80**. Only **202/223** of
  467 reachable replies completed by guarded BOS; the remaining rows hit the
  cap or context.
- The shuffled continuation control made zero exact replies, so the mechanism
  was active, but the candidate failed exact and completion retention.

## What changed

Added a generated-state boundary guard. When BOS is proposed, suffix memory
may suppress it only if a strict-majority continuation with two observations
is deeper than the local copy candidate. Otherwise BOS is accepted.

## Measurements

| Arm | Exact | Body positions | Premature BOS | Guarded BOS |
|---|---:|---:|---:|---:|
| continuation guard | 14 | 661 / 660 | 65 / 80 | 202 / 223 |
| original boundary | 17 | 661 / 660 | 70 / 81 | 0 / 0 |
| shuffled continuation guard | 0 | 426 / 426 | 8 / 8 | 18 / 18 |
| full retrieval | 69 | 1,053 / 1,053 | 0 / 0 | 0 / 0 |

The guard changed 23 steps from boundary and made 2,730/2,929 deeper-global
decisions. Both fresh-process repeats matched byte-for-byte. Panel pins:
tokens `614120412a4161554ebf263d79e8ad6f3126463d0745e4209dc5fb89738e2c71`,
mask `0398275d4fa01eb1f321246dd7bceaba919a4cdb294caebe12db812877f437ac`,
manifest `19d9629273ea423f5d39bef81740ef34c018a440ff4206359575290e1e8b773b`.
Raw manifest: `bfa7b6103a1ec247dacd4bfaf4f216ff07a609799ececd391f5f33dd955d9edf`.

## Interpretation

Confident continuation is real branch evidence, but it is not sufficient
termination evidence: suppressing a stop can change later exact content.
Future stopping must be trained jointly with content or decide among complete
completion branches, rather than extending a trajectory after its correct end.

## Checks run at the head

- focused continuation-guard test and driver test passed;
- full CPU library: 408 passed, 4 expected ignores;
- both artifacts repeated byte-identically;
- independent delegation was unavailable; review was local exact-head.

## Skills used

No system-provided Codex skill applied.

## Still open

- joint content-and-termination objectives;
- native stopping without post-end continuation;
- Claude's independent end-to-end serving track.

## Plain summary

The generated continuation guard is active and reduces premature BOS, but it
breaks exact content and still misses most completions. The fixed guard is
rejected.
