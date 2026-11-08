# REPORT: one-model-end-to-end, deepseek-34

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Source run:
`results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/runs/s1-content-ratio-20261108t055107z-adb12718d6c9ea88/`.

**The remaining content failure starts at the response opening.** On the selected
step-512 artifact, the first target token is never in the top 8 or even top 32
for any of 256 rows. After a correct opening is supplied, 1,614 of 2,576 later
targets are top-1. The model can continue a copy once aligned; it cannot choose
the first token from the query and context.

## What did not work, first

- The aggregate teacher-forced top-1 of 1,614/2,832 hides the opening failure.
  Every one of the 1,614 hits is after the first reference token.
- Under `per-turn/v1`, first-target ranks are min 67, median 956, max 4,055 of
  4,096 tokens: 0/256 top-1, top-8 or top-32.
- Under `one-user-turn/v1`, the first-target result is the same: 0/256 top-1,
  top-8 or top-32; min 43, median 1,189, max 4,060.
- More SFT or another ratio extension therefore does not address the first
  measured divergence; those configurations are already closed.

## What changed

- `served-reply-panel` teacher-forced reports now include a separate
  `first_target` rank tally: target count, top-1/top-8/top-32, rank minimum,
  median and maximum.
- The reply-SFT job records `first_target_top1`, `first_target_top8` and
  `first_target_rank_p50` for every checkpoint. The deterministic selector and
  outcome thresholds are unchanged.
- A focused unit test separates first-target ranks from the aggregate.

## What ran

- Local CPU scorer tests: 15 passed.
- The sealed content-ratio step-512 artifact was rescored in both registered
  renderings. No training, pod, GPU or spend was used.

## Measurements

| Rendering | First target top-1 | top-8 | top-32 | Rank min / p50 / max |
|---|---:|---:|---:|---:|
| `per-turn/v1` | 0/256 | 0/256 | 0/256 | 67 / 956 / 4,055 |
| `one-user-turn/v1` | 0/256 | 0/256 | 0/256 | 43 / 1,189 / 4,060 |

After the first target:

| Rendering | Later targets | Later top-1 | Later top-8 |
|---|---:|---:|---:|
| `per-turn/v1` | 2,576 | 1,614 | 2,574 |
| `one-user-turn/v1` | 2,576 | 1,600 | 2,574 |

The run's `progress.jsonl` independently records opening query spread 0.26249 at
step 512, against 2.06252 over all rows: the prompt changes the opening point
very little. This is consistent with the failed first-target selection.

## Evidence

- Source artifact:
  `406c9a5c05d63c8f2b6044ee6de6ea78d9fa91582dc03c4aa3e4fe877846e384`.
- Per-turn report:
  `bf6bbf5506731221f8c7d877453aca81c356290c85edb35ff54e1a00f760bd3c`.
- One-user report:
  `bca570ab7e0cd2b16f04e942ad8bf57682dbdb1533c7c611cc0677165acdb5d0`.
- Both reports use the sealed tokenizer `d36d3e87…`, panel inputs
  `1de2fc28…`, labels `3b5c5477…`, plan `0ed1ab98…`, and manifest
  `7c33f3f7…`.
- Full reports and summary:
  `results/deepseek/one-model-end-to-end-opening-diagnostic-2026-11-08/`;
  readback verified every entry in `SHA256SUMS`, whose SHA-256 is
  `5ec62761cbbb7dc7126ad73a75a1b3a3c10fa3c4e660a09321b1d10b6ffa249d`.
- The complete source seal and same-scorer parity reports remain at
  `results/deepseek/one-model-end-to-end-packet-parity-2026-11-08/runs/20261108t064446z-5825453f4aa453b4/`.

## Checks run at the head

- `served-reply-panel` tests: 15 passed.
- Reply-SFT gate: 302 checks passed.
- Reply-SFT mutation suite: 126 of 126 mutations caught.
- `bash -n`, ShellCheck and `git diff --check`: passed.
- Both CPU rescoring runs exited 0 and retained the registered free-generation
  totals: 0 complete rows; 145 EOS stops under `per-turn/v1` and 67 under
  `one-user-turn/v1`.

## Skills used

No system-provided Codex skill applied. The repository's D22, serving,
geometry and delivery rules were used directly.

## Still open

- The next training or architecture unit should target query-conditioned
  first-target selection. Extending the closed content ratio or dose is not
  indicated.
- `m2-reply512` recovery is still blocked by access to the historical Runpod
  volume; the current machine has the config file but `runpodctl` reports no
  API credential, so the volume cannot be listed or mounted.
- No useful-chat, capability, M2 or mechanism claim follows from this
  diagnostic.

## Plain summary

The model can copy most answer tokens once the first token is supplied, but it
never selects the first token in the top 32. The next work should make the
prompt and query control that opening decision; more of the same SFT cannot.
