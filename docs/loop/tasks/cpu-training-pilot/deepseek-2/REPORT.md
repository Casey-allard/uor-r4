# REPORT: execute the bounded public CPU rank-margin pilot, deepseek-2

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506). Base: `374c7bdd3`. FIT sheet: [cpu-training-pilot-public-rank-margin-2026-11-02.md](../../../harness/prep/cpu-training-pilot-public-rank-margin-2026-11-02.md).

## What did not work, first

- **The public development ranking did not move.** Both artifact seeds kept exactly their baseline top-1, top-8 and target-admission counts after 256 update attempts.
- **The tiny margin reduction was not a capability gain.** Margin loss fell by 1,039 and 1,069 units on seeds 1 and 2, but the registered primary ranking bar required increases of 43/47 and 40/44 rows respectively. A loss-only change is retained as a negative.
- **The seed-2 timing probe accepted no update.** All 16 probe attempts were refusals, so the 256-step arm could not be predicted to find updates from the probe alone. The probe's role was cost projection, not an outcome gate; both projections remained below one CPU hour.
- **Public-run restart is not executable with this CLI.** The driver reloads its final checkpoint, while two-resume equality is covered by trainer-level tests. Public-corpus restart from an interval checkpoint remains `NOT_RUN`.
- **No complete-reply or genuinely fresh-phrasing panel is available without credentials.** The sealed inputs return HTTP 401 and were not opened.

## What changed

- Executed the registered zero-update and 256-step arms on both public FBS dual-c131 seeds with the same 7,080-sample training set and 1,024-row document-disjoint internal evaluation set.
- Captured source, binary, toolchain, hardware, OS, RAM and filesystem identity under the run root and copied the evidence into [evidence](evidence/).
- Preserved each probe, baseline and trained JSON receipt, the checkpoint hashes and the final artifact hashes.
- Applied the FIT rules without changing the bar after observing the result.

## Pre-run review

A fresh non-author review returned **GO** at order/claim source `5b1bb0a1b44ec7a10f9e65a2ae2f55d69877646e`.

- FIT sheet complete and unchanged.
- Public FBS SHA-256: seed 1 `a9df5263e8b170931d876e7fc3ccbbf1b41805e8a07f90248528c62d57f56cfd`; seed 2 `da56fe96b14a4a0bf383996873f10f3b149f63ebcc22d14e3dfaa260a8db51d1`.
- Public tokens `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`; mask `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`.
- Binary SHA-256 `694fa06c3dda4bd18ad91e6a4a2950dab4766401c7206cf44feab7f58a5197cc`; Rust/Cargo 1.97.1.
- Machine: AMD Ryzen 9 9950X, 16 cores / 32 threads, 60 GiB RAM, Ubuntu kernel 7.0.0-42-generic.
- Focused corpus/checkpoint suite 6/6. Public FBS parity 192 complete positions per seed with exact CPU/serving outputs.

## Measurements

Panel denominator: 1,024 rows. All reported export checks compare the CPU evaluator to `IntegerStackSession`.

| Seed | Arm | Top-1 | Top-8 | Admitted | Margin sum | Target rank sum |
|---|---|---:|---:|---:|---:|---:|
| 1 | baseline | 278 | 384 | 396 | 11,739,473 | 319,318 |
| 1 | 256 steps | 278 | 384 | 396 | 11,738,434 | 319,266 |
| 2 | baseline | 224 | 285 | 292 | 29,717,075 | 342,478 |
| 2 | 256 steps | 224 | 285 | 292 | 29,716,006 | 342,466 |

| Seed | Top-1 delta / bar | Top-8 delta / bar | Admission delta | Verdict |
|---|---:|---:|---:|---|
| 1 | 0 / +43 | 0 / +47 | 0 | fail |
| 2 | 0 / +40 | 0 / +44 | 0 | fail |

- Exact export: seed 1 and seed 2 each checked 1,024 tokens, 262,144 Q12 query coordinates and 8,192 kept-cell entries with zero mismatches, at baseline and final.
- Seed 1 training: 32 accepted and 224 refused steps; 20,705 context tokens in 4.291 s of update work, 4,824.742 context-tokens/s, 59.654 target visits/s, 16.763 ms mean update; 66.024 s total run wall time; 52,060 KiB peak RSS.
- Seed 2 training: 19 accepted and 237 refused steps; 20,705 context tokens in 4.075 s of update work, 5,081.373 context-tokens/s, 62.827 target visits/s, 15.917 ms mean update; 64.993 s total run wall time; 51,956 KiB peak RSS.
- Timing probes projected 16.492 minutes (seed 1) and 16.382 minutes (seed 2) for 256 steps from complete 16-step wall time, both below the one-hour stop.
- Final reloadable artifacts: seed 1 `db58f4efb3f4279e9c06f1667e1b014b70edf62ba43f0faaf37869c04886e50c`; seed 2 `6781fe807a1a1e47d386b50a78d7f0367e405da58c2cc5038b6b9967e0d23570`.
- Final checkpoints: seed 1 `d9e0f9d7858d222cb8b27fbc5750f7b0a8cfb3fba9ace214e2e88d895e042d0a`; seed 2 `85235546615a2a94954f41631c6dbdcc543e5230730a0ea94cc0bca972d0da9c`.

## Decision

**FAIL on the registered primary ranking bar for both seeds.** The tested 256-step configuration is a negative for public document-disjoint next-token gain. It preserved exact export and admission and improved the continuous margin sum slightly, but it did not produce the registered top-1/top-8 increase. There is no evidence here that this schedule is useful as a CPU language-training replacement.

This result is configuration-specific. It does not reject rank margin, the FBS trunk or CPU training generally, and it does not justify an unregistered longer run or a third seed.

## Checks run at the head

- Fresh non-author pre-run review: GO.
- Focused corpus/checkpoint tests: 6/6.
- Public FBS trained-artifact parity: both seeds, exact.
- Six pilot runs completed: two 16-step probes, two zero-update baselines and two 256-step arms.
- Raw JSON and machine identities are in [evidence](evidence/); `SHA256SUMS` covers every committed evidence file.

## Skills used

No repository skill was invoked. The pilot used the merged D84 CPU program, its FIT sheet and the synthetic-free public corpus path.

## Still open

- A configuration that clears the public ranking bar.
- Public-corpus restart from an interval checkpoint.
- A genuine trained FBS float shadow.
- Complete-reply and fresh-phrasing correctness on credentialed sealed panels.
- The final program-wide decision on whether CPU training is useful for development; this single failed public configuration is not enough to answer it.

## Plain summary

The CPU ran the full registered pilot cheaply and exactly, but the model's top-1, top-8 and admission counts did not move on either seed. The training mechanism worked; this tested learning configuration did not produce a measurable ranking gain.
