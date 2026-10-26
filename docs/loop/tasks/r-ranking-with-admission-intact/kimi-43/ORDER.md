# ORDER: r-ranking-with-admission-intact, kimi-43

Lab: kimi. Item: [r-ranking-with-admission-intact](../../../../roadmap/items/r-ranking-with-admission-intact.md). Base: `87024eca8`.

## Goal

The item's done-when reads:

> A run with admission demonstrably intact reports ranking results (top-1 among admitted rows) against the count tables.

## Cancel probe (run first)

Source and item inspection found the registered FIT at `docs/harness/prep/r-ranking-with-admission-intact-2026-10-25.md` (`TEST FITNESS: FIT`, #2884 comment `6396898060`), both dependencies `done`, the delivered Stage T trainer, `w2-assemble` count-table assembler, and the sealed #2831 Q1 archive at `results/antigravity/a8-full-dose-rerun-2026-10-25/`. The Q1 admission-intact baseline is `4,292` admitted rows, `1,106 / 3,847` `P_succ`, `871` top-1 and `1,455` top-8. The remaining measurement is the FIT's three-step ladder, not another baseline audit or a new read law. `classification: instrument`. **Not cancelled.**

## Files in scope

- `crates/uor-r4-discrete/**`
- `docs/harness/prep/r-ranking-with-admission-intact*`
- `docs/labs/r-ranking-with-admission-intact/**`
- `docs/loop/tasks/r-ranking-with-admission-intact/**`
- `docs/research/notebook/m1-discrete-attention.md`

Always allowed: `docs/roadmap/items/r-ranking-with-admission-intact.md` and rendered `docs/roadmap/README.md`. Prefer the existing scorer and trainer; a new scorer or model change requires demonstrated necessity, failing-first tests and no change to driver source pins.

## Done when

- Reproduce #2831 FrozenZero Q1 on one RTX 5090: `b_3(Full)=4,292`, `b_3(P_succ)=1,106`, `top1=871`, `top8=1,455`, and compute admission-matched Unigram and T2 scores on exactly those `4,292` rows.
- Complete the registered 256-batch `Arm-Rel-K8` seed-1 probe with no `consecutive_rejections` and projected full-run wall time at most 35 minutes.
- Only after both gates pass, run `Arm-FZ-K8`, `Arm-Rel-K8` and `Arm-Rel-K16` on seeds 1 and 2 through 16,404 batches. Seal checkpoints, evaluations, configurations, binary identity and checksums.
- Report full-panel and conditional admitted top-1/top-8 against full-panel and admission-matched count tables, with decode tokens/s, complete cost and exact negative scope. Apply the FIT decision table; keep `fresh_v2` sealed unless its DEV gate passes.
- Add a REPORT and one notebook row, pass full gates and local policy, obtain a fresh exact-head code/evidence review, queue, verify the merge and clean up.

## Stop and decide if

- Q1 reproduction mismatches any registered count or cannot score the same admitted row set: `INSTRUMENT_STOP`; repair the instrument before the throughput probe or full runs.
- The 256-batch probe emits `consecutive_rejections`, cannot establish a wall-time projection, or projects above 35 minutes/run: `INSTRUMENT_STOP`; repair before Step 3.
- Identity, data, config, checkpoint or binary hashes drift, an output is nondeterministic, or admission controls/ratchets fail: preserve the evidence and classify instrument versus mechanism before continuing.
- Any model or parity evaluation would run on the host, or a host fallback would replace the GPU evaluator.
- Work would require out-of-scope files, overlap another lab, change a bar/scope/stop rule, or add a new read law; use a fresh ORDER or D68 review rather than expanding this unit silently.
- Disk remains below the 30 GiB stop band after Kimi-owned merged build cleanup, or the fleet exceeds four pods / `$8/h`; do not dispatch until the actual constraint clears.
