# REPORT: one-model-end-to-end, deepseek-26

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Execution complete; one pod, seed 1.

## What did not work, first

- **Teacher-forced EOS still never reached top-1.** Every scored checkpoint had
  `eos_top1 = 0/256`. Top-8 moved from the base's 198/256 to 256/256 at every
  scored checkpoint, and the median rank fell from 7 to 2.
- **Free generation produced EOS stops without complete accepted replies.**
  The selected checkpoint emitted 23 EOS stops, but `complete = 0` and
  `pairs_both_complete_distinct = 0`.
- **The final checkpoint moved backwards.** Free EOS stops fell from 23 at step
  512 to 1 at step 1,024, and the continuation guard fell from 142 to 137.
- **This is not a capability or M2 result.** `m2-reply512` remains unavailable;
  no substitute was scored under it. One seed cannot establish reproducibility
  or attribute movement to M-World versus the reply mask.

## What changed

- Executed the reviewed 1,024-step reply-masked SFT on `vast-59644755`, one
  RTX 5090, from the exact merged pin.
- Reproduced the exact `9a370566…` base artifact and all base rank, generation
  and continuation controls before training.
- Scored all four checkpoints and applied the deterministic selector.
- Sealed 87 files, read the complete manifest back, and deleted the pod.

## Measurements

| checkpoint | OASST2 guard | EOS top-1 | EOS top-8 | EOS median | free EOS stops | complete rows |
|---|---:|---:|---:|---:|---:|---:|
| base | 132/512 | 0/256 | 198/256 | 7 | 0/256 | 0/256 |
| 256 | 137/512 | 0/256 | 256/256 | 2 | 16/256 | 0/256 |
| 512 | **142/512** | 0/256 | 256/256 | 2 | **23/256** | 0/256 |
| 768 | 141/512 | 0/256 | 256/256 | 2 | 21/256 | 0/256 |
| 1,024 | 137/512 | 0/256 | 256/256 | 2 | 1/256 | 0/256 |

The registered selector chose step 512: every checkpoint was guard-safe, all
tied at EOS top-1 zero and zero complete rows, and step 512 had the most
`per-turn/v1` free EOS stops. The result is outcome row 6, **Partial movement**:
continuation stayed above the floor and EOS ranking/generation moved, but no
complete accepted reply moved with them.

Per the registered row 6, seed 2 does not run. The next unit is one 2,048-step
dose extension at a new root, followed by outcome rows 1-5.

Training reached 1,024 steps in 721 seconds. Pod uptime was 1,457 seconds
(24.3 minutes) at `$0.563/hour`, costing approximately **$0.228**.

## Evidence

- reviewed pin: `8903c5551a5e9c3953bd6a2eab1322c7e6c276c2`
- reviewed runtime:
  `044120d4bc348aec934028a7fc77f70831b41a745cf601f1639abc516541089a`
- store:
  `hf://datasets/caseyallard/uor-r4-store:results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-20261108t005743z-a849820fba9cfbf4`
- seal manifest SHA-256:
  `14626e60220a45d56498a1ed40952ba54bd7a788d05c855531b25e7aa6208f27`
- seal size: 65,925,169 bytes across 87 files; `READBACK=verified`, `EXIT=0`
- `best-checkpoint.json` SHA-256:
  `51acc9edb7b816ed1b480b10e333d57b9fc7c46991177216fbeb763f7bc41096`
- selected step-512 artifact:
  `30c527cf184b36c48d7fd1729687122a48f4a1af78caca27d30029e77175dc7f`
- selected step-512 model:
  `3b51fcda4e13dc45c489e20b49549358d4e1573f79212475adcab3f7b3623094`
- selected step-512 Hamming read:
  `e68e085b57780eb8d75fd8eb4845e3c4e30fa618d456745631004ae9e9d0c057`

The downloaded seal passed every `SHA256SUMS` entry. The base overlap audit
reported zero findings in all four categories, and every input/checkpoint-facing
hash matched the reviewed sheet.

## Checks run

- repository policy hook and exact-pin/runtime preflight: passed;
- remote CUDA build, the two pinned trainer tests, exporter, grader and panel
  builds: passed;
- base artifact, ranks, free-generation counts, 512-row fingerprint and
  continuation guard: passed;
- finite progress values, checkpoint ready markers, export/reload, all three
  panels and scored markers at 256/512/768/1024: passed;
- complete store readback and teardown: passed.

No failure retry occurred. The run completed all 1,024 steps naturally.

## Skills used

No system-provided Codex skill applied. The repository's loop, compute and
geometry rules were used directly.

## Still open

- One 2,048-step dose extension at a new root under a new execution unit.
- Seed 2 remains conditional and is not authorized.
- `m2-reply512` remains unavailable unless exact bytes are recovered.

## Plain summary

Reply-masked SFT improved EOS ranking and produced some EOS stops while
preserving human continuation, but generated no complete accepted replies and
regressed by the final checkpoint. This is partial movement, not a model result.
