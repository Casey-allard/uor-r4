# Prep sheet: M-World ratio reply-SFT intervention (one-model-end-to-end, deepseek-31)

Lab: deepseek. Item: [one-model-end-to-end](../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [deepseek-31](../../loop/tasks/one-model-end-to-end/deepseek-31/ORDER.md).

TEST FITNESS: FIT - fresh non-author review at `53f8f4f2bdc194efb921622aea720ae3537fbcdf`, reviewed-runtime `c8f9937a197573af83f34d167c246f14167840f738361429fe4c1896e0c1147d`, reviewer session `pr3850_fit_exact`.

PRE-RUN: GO - fresh D67 review at `53f8f4f2bdc194efb921622aea720ae3537fbcdf`, reviewed-runtime `c8f9937a197573af83f34d167c246f14167840f738361429fe4c1896e0c1147d`, reviewer session `pr3850_d67_verify`.

## Why

The closed 2,048-step reply-SFT dose selected step 1,280 with 52/256 free EOS
stops, but zero EOS top-1 and zero complete accepted replies. Its mixed corpus was
41.45% M-World by total tokens but only 20.32% by scored reply tokens. This is
one distinct data-distribution intervention: repeat the same excluded M-World
store three times while keeping OASST2 once.

## Question

At 2,048 steps from the same `(b64L)` seed-1 base, with the same geometry, reply
mask, optimizer and selector but OASST2 once and excluded M-World three times,
does the selected checkpoint produce complete accepted replies while retaining
stopping and human continuation?

## Bar

This is a staged development run, not a milestone bar. Outcome row 3 requires EOS
top-1 >= 16/256, median EOS rank <= 3, free EOS stops >= 16/256, complete accepted
rows >= 4/256 and OASST2 continuation top-1 >= 93/512. The M2 bar remains
>= 256/512 saved questions, >= 52/128 complete-and-distinct pairs and >= 104/256
single-row-complete questions on the frozen fresh panel. `m2-reply512` remains
unavailable, so this unit cannot claim an M2 pass.

## Base artifacts

Use the exact base, tokenizer, held-out guard, panel and hashes from
[the original reply-SFT sheet](./one-model-end-to-end-reply-sft-2026-11-07.md).
The fixed base is seed-1 `(b64L)` checkpoint
`oh/run/b64L-s1/models/step-016384`.

## Data split

The new store is the original OASST2 reply-masked TRAIN followed by the exact
excluded M-World v2 store three times:

`results/deepseek/one-model-end-to-end-content-ratio-2026-11-08/inputs/`

| file | SHA-256 |
|---|---|
| `tokens.u16` | `3a645e736f96d2e37a267ba3826512001c41b92d8c689fae1451784b2fa2da34` |
| `response_mask.u8` | `83911dc7488c13475469fa743e0b6fbca15fcd20e76fc2fee33cb89877673baf` |
| `manifest.json` | `8b43ff038133a7dc9744bee71dd8aa7e3f6c5e59a664d19690b991a69f0be9fd` |

The store was uploaded and read back with those hashes. It has 65,071 documents,
11,786,293 tokens, 5,710,018 scored targets, 223,469 scored EOS (39,136 per
million) and median reply length 11. The M-World sources are three exact copies;
OASST2 remains 32.02% of total tokens and M-World rises to 67.98%. By scored
reply targets, OASST2 is 56.65% and M-World is 43.35%, versus 20.32% for
M-World in the old mix.

`corpus-panel-overlap` decoded every document and mask-1 assistant response:
exact panel strings, exact accepted replies, and shared word 6-grams are all zero
in both document and assistant-response classes. Audit SHA-256:
`4db5d14f49ca7edaa1d44944cec842f3876c6f23a801ceacc4181bd59c14d5ee`.

## Recipe

The original reply-SFT recipe, changing only the data root and keeping 2,048
steps:

```text
--steps 2048
--eval-every 256
--seed 1
```

Training starts at step 0 from the same base with a fresh optimizer and schedule.
All other geometry, width, context, batch, micro-batch retry, scoring and selector
settings are unchanged.

## Readings and selector

Record the base controls and every checkpoint from 256 through 2,048. Apply the
original deterministic `per-turn/v1` selector unchanged.

## Outcome

Apply original outcome rows 1-5 to the selected checkpoint. Because this is a
new data intervention after the closed dose, another row 6 closes this
configuration and does not authorize a second ratio extension. Row 7 also closes
it. Only row 3 unlocks seed 2 and another intervention.

- Row 1: continuation failure; no seed 2.
- Row 2: no ranking movement; close this data/recipe pair.
- Row 3: positive; unlock seed 2 and a longer run.
- Row 4: termination moved but content did not; inspect answer content.
- Row 5: ranking moved but free generation did not; inspect the mismatch.

No result claims M2, seed reproducibility, a mechanism effect or useful chat.

## Steps

1. Record fresh FIT and D67 reviews of the closed recipe pair and exact runtime.
2. Check pod admission; use one DeepSeek-lab RTX 5090 within the project caps.
3. Build the exact merged pin, fetch and hash every input, reproduce the base
   rank, free-generation, continuation and overlap controls, and record CPU
   hardware before grading.
4. Retrain from the base at step 0 for 2,048 steps, exporting and scoring every
   256-step checkpoint.
5. Apply the original outcome rows to the selected checkpoint, seal, read back
   and delete the owned pod.
6. Record one result row. Do not run seed 2 or a second ratio extension after a
   non-positive result.

## Stop rule

- Instrument, pin, overlap, merge or input failure: train nothing; fix and use a
  new root.
- Out of memory: retry micro-batch 8 once; a second failure is `NOT_RUN`.
- Non-finite loss, missing checkpoint, failed export/panel or changed hash:
  stop this intervention.
- Never cancel a healthy run for a self-set timer. Never touch another lab's
  pods, leases, files or GPU devices.
- A second row 6 or row 7 closes this data intervention.

## Minimum test power and cost

One seed is exploratory. Seed 2 remains conditional on row 3 and a fresh decision.
The prior 2,048-step run used 1,485 seconds of training and 2,210 seconds of pod
uptime. This run projects to roughly the same 45-50 minutes and $0.45-$0.55 at
$0.563/hour, under $1.30 with one OOM retry. The launcher stays inside four pods
and $8/hour.

## Instrument checks

The job records the CPU model, architecture and logical processor count before
grading, and binds every base and checkpoint grade to that hardware. Both CPU
files and all sidecars must enter the seal. The completed seal must be at most
100 MiB, read back with every `SHA256SUMS` entry valid, and report `EXIT=0`.
After training and all checkpoint scoring finish, the successful job re-hashes
the raw training token and mask files, records their sizes and hashes in
`control/post-use-inputs.json`, and then removes only those two raw payloads.
The fixed source store and the retained corpus manifest keep the input
recoverable; this prevents the larger mix from pushing the completed seal over
its size cap.
