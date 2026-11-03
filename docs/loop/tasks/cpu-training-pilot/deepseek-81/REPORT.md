# REPORT: bounded CPU table BVG-K pivot, deepseek-81

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `de9cfc84a`. Source commit: `8a0defae5`.

## What did not work

- **The registered matched control failed admission.** At seed 1, K1 with a
  four-Q12 step changed the 1,024-row panel from 278/384/396
  top-1/top-8/admitted to 278/383/395. It lost one admitted target.
- **The stop rule fired before K4.** `--require-admission` exited 2 after
  writing the complete K1 receipt. The K4 arm and seed 2 were not run.
- **Therefore this is not a K4 mechanism result.** It is a configuration stop
  for the four-Q12 one-coordinate control and cannot be called a BVG-K
  negative.

## What changed

Added an opt-in table BVG-K block to the exact tied-token trainer:

- `RankMarginConfig` defaults to one coordinate and accepts one through 64.
- K4 selects up to four distinct interior coordinates, patches only the tied
  writer table, reparses the complete artifact, and accepts only an exact
  changed-forward margin improvement with target admission retained.
- Checkpoint version 2 binds K; version 1 remains readable as K=1.
- `cpu-corpus-driver` reports accepted-coordinate totals and a histogram and
  can fail closed on a final admission rejection.

The K1 path retains the previous coordinate selection and has a fixed artifact
hash regression. The full CPU library and all driver tests passed before the
run.

## Measurements

All values are on the registered public seed-1 artifact, the same 1,024-row
document-disjoint panel, and the same 7,080-target training split.

| Arm | Steps | Accepted | Top-1 | Top-8 | Admitted | Result |
|---|---:|---:|---:|---:|---:|---|
| baseline | 0 | 0 | 278 | 384 | 396 | control |
| control-k1 | 4,096 | 432 | 278 | 383 | 395 | admission reject |
| pivot-k4 | 0 | 0 | — | — | — | not run |

K1 checked 262,144 Q12 coordinates, 1,024 tokens and 8,192 kept cells with zero
CPU/serving mismatches. Its step time was 15.938 ms, throughput 62.743 target
visits/s, wall time 127.504 s and peak RSS 51,952 KiB. The final artifact hash
was
`2f83d10d1db331e1f43d43c8ac3673f06c4642a923d1ceaeeea06816281ed930`.

The base and driver hashes, input pins and this result are in
[evidence/SHA256SUMS](evidence/SHA256SUMS).

## Checks run at the head

- Full CPU all-target suite: 128 library tests passed, 5 driver tests passed,
  all auxiliary binaries passed, 4 release benchmarks ignored.
- Strict Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Targeted formatting and `git diff --check`: passed.
- FIT review and `prep-check`: FIT, complete, `may_compute=true`.
- Preflight hashes: all five inputs plus the release driver matched the sheet.
- Run postflight: admission guard exited 2 as required; evidence checksums pass.

## Skills used

No available Codex skill applied. The project rules, CPU pilot records, trainer
and driver were read directly.

## Still open

- Run the narrower same-amplitude K1-versus-K4 pivot in deepseek-82.
- A completed K4 comparison on both seeds.
- Fresh-phrasing and complete-reply capability remain `NOT_RUN`.

## Plain summary

The first matched block comparison stopped on its own control before the new
four-coordinate arm ran. A four-unit change in one coordinate lost one admitted
target, so the preregistered admission stop correctly prevented a misleading
K4 comparison. The next unit compares one and four coordinates at the same
one-unit amplitude and requires the active K4 arm itself to preserve admission.
