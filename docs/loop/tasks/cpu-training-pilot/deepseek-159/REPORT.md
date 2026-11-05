# REPORT: cpu-training-pilot, deepseek-159

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `330185de6`.

## Status

Decision: **REJECT** for the fixed exact-body generated-stop construction on a
registered fresh panel. Both artifact seeds fit **7/11** exact training bodies
with no selected-row false stops, but on the unopened seed-2722 dev panel they
produced **0/10** oracle-exact replies. Premature stops were **5** and **9**
against the four-reply limit. Every control produced zero exact replies.

This closes exact-body filtering with this eight-prototype sign-signature gate
and threshold family. It does not reject geometric termination, copy-boundary
learning or terminal-token construction.

## What did not work

- The first exact-body run, attempt 7, opened dev before an independent FIT
  verdict was recorded. It is preserved as exploratory evidence, not used as
  the registered decision. It produced 1/13 exact replies on the previously
  opened dev panel.
- The registered fresh confirmation had adequate power: 458 reachable replies,
  a 10-reply oracle ceiling and 11 distinct terminal train signatures on each
  artifact seed.
- Fresh-dev true exact replies were **0/10** on both artifact seeds, below the
  seven-reply floor and below the paired two-sigma gain over controls.
- Premature stops were **5/458** on seed 1 and **9/458** on seed 2, both above
  the four-stop limit.
- The shuffled, deepseek-157 all-reference and no-gate controls each produced
  zero exact fresh-dev replies.
- Exact-body training is in-sample separable, but the endpoint state does not
  transfer to unseen copy trajectories.

## What changed

`cpu-copy-generated-stop` now supports:

1. exact-body train-row selection through `--exact-body`;
2. separate all-reference, shuffled-label and no-gate controls;
3. a signature census and mandatory distinct-terminal-signature floor;
4. machine-enforced train/dev exact floors, reply power and oracle power;
5. replay of an exact-cap-length reference without requiring a trailing token;
6. per-reply fired step, stop class, exact status and unchanged trace hash.

The first firing emits BOS; otherwise the sealed longest-first token is fed
through exact `CpuFbsForward`. The decoder never reads the reference.

## Registered confirmation

Fit used train panel 2711. Evaluation used a newly generated `cell=dev,dev`
panel, seed 2722, after FIT `64fac2483` and GO `3641b70f2`. The panel was
absent before the run and was validated for its seed, split, protocol,
conversation count, world and tokenizer hash.

## Measurements

Selected training census and fit:

| Artifact seed | Exact terminal | Exact nonterminal | Distinct terminal | Terminal stops | False stops / limit | Margin |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 11 | 308 | 11 | 7/11 | 0/308 (limit 3) | 69 |
| 2 | 11 | 308 | 11 | 7/11 | 0/308 (limit 3) | 67 |

Complete-reply results:

| Artifact seed | Panel | Oracle exact | True exact | True premature | True at reference | True no fire |
|---:|---|---:|---:|---:|---:|---:|
| 1 | train | 11 | **7** | 1 | 7 | 445 |
| 1 | fresh dev | 10 | **0** | 5 | 1 | 452 |
| 2 | train | 11 | **7** | 2 | 7 | 444 |
| 2 | fresh dev | 10 | **0** | 9 | 0 | 449 |

Fresh-dev controls:

| Artifact seed | Shuffled exact | All-reference exact | No-gate exact |
|---:|---:|---:|---:|
| 1 | 0 | 0 | 0 |
| 2 | 0 | 0 | 0 |

Both registered runs repeated byte-identically in fresh processes. Gate runs
took **9.01 / 9.02 s** for seed 1 and **9.00 / 9.08 s** for seed 2. Peak run
RSS was **201,168 KiB**; generator-build peak RSS was **1,269,380 KiB**.

The analyzer binary is **2,025,288 bytes**, SHA-256
`bd97d293a660061538f2c500d02449d9b39073b4f8be1c447b01d7e9976127b1`.

Fresh copy-report hashes:

- seed 1: `a2ce9d5a00f8e581856eb1d1835d24f1c1bf8981262edcea838c940cb17b173a`;
- seed 2: `31f1474b22b5ea952b674ac01ffadc4b7604c5b2580059e3204fdb6b0079d246`.

Local confirmation-run manifest SHA-256:
`e936ec3bd7cbb7c42ee97ee6ce46b4ee933a3f7f558f12c72d1062f81bbdd595`.
Published attempt-8 raw manifest SHA-256:
`f7bef7aa2c61a82f4f382815b8722ea3edc6acc5f04b09246e2b80e4a7ebf2be`.
Published evidence manifest SHA-256:
`a6634328d115b945ce6a8e14c9c7daaac256d9cbe7ab6cc7da7ac5aa76600ab0`.

Post-merge note: fresh-main verification found the ignored attempt-8
`tokens.u16` file named by the published panel manifest. Deepseek-161 removes
the corpus payload from the public manifests, records its hash and original
local-run seal in `LOCAL-RUN-SEAL.txt`, and verifies the manifest-only evidence.
It does not alter this result or rerun the experiment.

## Checks run at the head

- **286** CPU library tests passed with 4 expected ignores.
- **82** binary tests passed; the focused generated-stop binary passed its
  **8** tests.
- Scoped package Clippy with `--all-targets --no-deps -- -D warnings`,
  rustfmt, D23/D25/D32 ratchets, claim wording, report and policy checks pass.
- The registered command block passed `bash -n` before the run.
- The confirmation `SHA256SUMS`, attempt-8 `RAW-SHA256SUMS` and published
  evidence manifest verify.
- Both fresh-process gate repeats are byte-identical.
- Every input, selected-trace and source-report hash matched.
- The generated panel matched its registered seed, split, protocol,
  conversation count, world and tokenizer.
- No serving, integer runtime, CUDA, float-training or Claude-owned source
  changed.

## Size

Against base `330185de6`, this unit changes one CPU-train binary (**+307 /
-48**), 11 task, preregistration, notebook, roadmap, state and lab-record
files (**+905 / -0**), and 38 published evidence files (**+264,409 / -0**):
**50 files, +265,621 / -48**. No binary, corpus or scratch artifact is
tracked.

## Skills used

No system-provided Codex skill applies. Project CPU research, evidence, FIT and
GO review procedures were used.

## Still open

- Do not retry the same eight-prototype gate with another label filter derived
  from the reference horizon.
- The next termination unit should train on copy phrase boundaries or use a
  continuation objective with an independent boundary signal.
- Improve terminal-token reconstruction before expecting a stop gate to
  recover even the 10-13 reply oracle ceiling.
- Complete replies, fresh phrasing, useful chat and integer-served model
  capability remain open.

## Plain summary

Filtering to exact bodies lets this gate fit its small successful training set,
but the state does not transfer: on a fresh panel it completes no reply and
stops too early. This closes the exact-body filter with this prototype gate;
the next attempt needs a better copy-boundary signal or terminal objective.
