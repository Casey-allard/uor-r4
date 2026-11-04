# REPORT: generated-query admission training, deepseek-135

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `27317ddb8`.

## What did not work

- **The registered readiness bar failed on both seeds.** True training raised
  K=8 positive-margin rows only **30→36** on seed 1 and **32→49** on seed 2,
  against 640. Same-cell pairs reached only 14 and 21, against 64.
- **Held-out admission did not improve.** Seed 1 fell **53→47**; seed 2 rose
  only **48→55**, against a required +128.
- **The target permutation was not distinguishable.** Seed 1 true/permuted
  held-out admission was **47/51**. On seed 2 both arms accepted the same two
  moves and produced byte-identical final-gain artifacts.
- **The equal-move random control was no weaker.** Seed 1 random remained at
  53 admitted targets versus true 47; seed 2 tied frozen at 48.
- **Generated depth was non-falling but teacher-forced top-1 fell on both
  seeds:** 438→435 and 385→334 of 3,765.
- No reply became exact or BOS-complete. This closes the fixed 32-coordinate,
  ±32, 12-move final-gain admission objective; it does not reject all query
  geometry or another coordinate family.

## What changed

Added a CPU-only generated-query admission trainer:

1. every candidate changes one final-gain coordinate in the fixed
   `0,8,...,248` family;
2. the candidate artifact is reparsed and actual generated trajectories are
   rescored;
3. the lexicographic objective maximizes admitted target cells, then lowers
   target-cell rank sum, then adds distinct cells and same-cell pairs;
4. fixed-horizon missing rows may not increase;
5. accepted artifacts are audited against integer serving;
6. a deterministic cross-cell target permutation reports changed/cross-cell
   assignments and a mapping hash; and
7. an equal-move random control records every coordinate and before/after
   code.

All three changed arms independently prove gain-table-only byte isolation.

## Measurements

| Seed | Frozen eligible | True eligible | True pairs | Frozen held-out | True held-out | Permuted | Random |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 30 | **36** | 14 | 53 | **47** | 51 | 53 |
| 2 | 32 | **49** | 21 | 48 | **55** | 55 | 48 |

True accepted 1 move on seed 1 and 2 moves on seed 2. Seed-1 generated depth
was 0/973 frozen and true; seed 2 was 4/973. Teacher-forced top-1 was
438/435 and 385/334.

The permutation covered 894 fixed-horizon assignments, changed 893, and moved
886 across writer cells. Its mapping SHA-256 is
`527a7c30ae654e65a12faf448021905c8eb343633961000b169f86c5f5a2ca1a`.

Final-gain artifact hashes:

| Seed | Frozen | True | Permuted | Random |
|---:|---|---|---|---|
| 1 | `ed06a88b…d7daffc` | `4f8d1a82…bb5a7a1` | `d59e8bef…2b91af3` | `ea4644ef…439ef90` |
| 2 | `2c071845…eaf7e842` | `dbd6104b…c3ef6b71` | `dbd6104b…c3ef6b71` | `d6aaa57c…30dc028c` |

Every arm retained zero mismatches over **4,328 positions, 1,107,968 Q12
coordinates, 4,328 token decisions and 34,624 kept cells** per seed.

The build took **18 s**, peaked at **1,211,880 KiB** and occupied
**264,131,283 bytes**. Public runs took **107.80/109.75/109.85 s** at
**400,200-409,532 KiB**. The binary SHA-256 is
`28e516cf6be48bfbeceb4aced5c0e260255e5828a0923b9134cd4e2c24bdbdea`.
The retained pipeline is **28,021,704 bytes**.

Report and evidence hashes:

- seed 1:
  `90dc72331cb4a4a5162b5973a7d006de5c704e9f0a22d8ce11a8aa8449c77ac0`;
- seed 2:
  `9897e1a1b3e266d54b9770db351739b2fccb0ad035cb64d93fc2800bd4c7b2db`;
- pipeline `SHA256SUMS`:
  `4ffd4ce084ca2a2a41c2cb4cd5c58a3e571b25fdf5546326cf90e08c3a22e312`.

## Checks run at the head

- The first FIT/code reviews rejected an unauditable random control,
  unverified cross-cell movement, missing objective/refusal tests and
  true-only gain isolation. These were repaired before compute.
- Fresh FIT and code reviews passed at `5f65d260f`; a fresh pre-run review
  returned GO at `346667e70` after a 0.16 s synthetic scheduling probe.
- Full `uor-r4-cpu-train` release suite: **233 library cases, 229 passed and 4
  expected ignores**, plus **57 binary tests passed**.
- Focused self-prefix tests: **20/20**; query-admission driver: **1/1**.
- Strict CPU Clippy, formatting, prep shape, evidence checks and
  `git diff --check`: passed.
- Both seeds and the repeat exited 0. Normalized reports and every artifact
  matched the repeat; recursive pipeline checksums passed.

## Skills used

No system-provided Codex skill applied. Read-only subagents performed the two
FIT/code review rounds and pre-run review.

## Still open

- A different query-coordinate family, trunk-level admission path, or direct
  vector-serving objective is required; this final-gain schedule is closed.
- Complete replies, fresh phrasing, useful local chat, integer integration and
  GPU qualification remain open.
- Exact-head delivery review, protected merge and fresh-main verification.

## Plain summary

The final gain can change only one or two accepted moves under the fixed
objective and does not approach writer-training readiness. Training rows rise
to only 36/49, held-out admissions do not beat the matched controls, and
teacher-forced top-1 falls on both seeds. Exact parity and deterministic
repeats rule out an instrument fault. Direct final-gain admission training is
therefore negative for this schedule.
