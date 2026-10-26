# ORDER: m2-pin-train-mask-sheet, kimi-45

Lab: kimi. Item: [m2-pin-train-mask-sheet](../../../../roadmap/items/m2-pin-train-mask-sheet.md). Base: `e9cfa8873`. Origin: post-merge audit of #3300.

## Goal

Prevent a non-executable M2 recipe from authorizing compute. Preserve the valid pins and bars, but downgrade the discrete sheet to `NOT_FIT` unless a complete training and deterministic grounded-reply evaluation chain is shown to exist. Bind the missing implementation to `m2-train-grounded-replies`; do not invent commands or change the M2 bar.

The tracked repair #3303's done-when reads; the harness base is the existing #2851 item:

> The sheet no longer presents a non-executable chain as FIT. Record the exact CLI and missing-runner findings, replace the misleading recipe and verdict with `NOT_FIT` unless the complete chain exists, bind the remaining implementation to `m2-train-grounded-replies`, and obtain fresh D68 and non-author fitness reviews. No GPU work or model execution.

## Cancel probe (run first)

Source comparison found `stage-t-run step0` requires `--t16-fz`, `--t16-rel`, `--t1-fz` and `--out`; `train` additionally requires `--anchor-manifest`, `--sheet` and `--out-dir`. `trunk-run step0` requires `--fz-config`, `--fz-checkpoint` and `--out-dir`; each arm requires `--config`, `--checkpoint` and `--out-dir`; grade requires four report files. The sheet omits these, places trunk Step 0 before Stage T creates its checkpoint, and gives no M2 reply-evaluation command. `classification: instrument`. **Not cancelled; a flag-only FIT restoration is refused.**

## Files in scope

- `docs/harness/prep/m2-pin-train-mask-sheet-discrete-2026-10-26.md`
- `docs/labs/m2-pin-train-mask-sheet/**`
- `docs/loop/tasks/m2-pin-train-mask-sheet/**`
- `docs/research/notebook/m2-grounded-reply.md`
- `docs/roadmap/items/m2-pin-train-mask-sheet.md`
- rendered `docs/roadmap/README.md` and `docs/roadmap/issues.json`

No source, model, bar, hold, milestone-registry or other prep-sheet change. No pod and no host model evaluation.

## Done when

- D68 reviews the classification and records why #3300's FIT cannot stand; the decision is posted on the new item issue and linked from #2851.
- The sheet preserves pins/bars but says `TEST FITNESS: NOT_FIT`, removes the pseudo-executable recipe, and names the exact missing M2 runner/evaluator binding and CLI evidence.
- A fresh non-author review confirms `NOT_FIT` with no unresolved claim that compute is authorized.
- Lab record and one notebook row report the instrument negative; REPORT, roadmap render/check, full gates, local policy and fresh exact-head review pass; queue, fresh-main verification and cleanup complete.

## Stop and decide if

- Someone proposes restoring FIT without a complete M2 student training and deterministic reply-evaluation path.
- A source change, new GPU run, bar/hold change or out-of-scope edit appears necessary; use a new ORDER rather than expanding this repair.
- Evidence shows a command actually is valid at the bound source; verify the exact parser before retaining any criticism.
