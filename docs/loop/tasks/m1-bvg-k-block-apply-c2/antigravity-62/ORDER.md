# ORDER: m1-bvg-k-block-apply-c2, antigravity-62

Lab: antigravity. Item: [m1-bvg-k-block-apply-c2](../../../../roadmap/items/m1-bvg-k-block-apply-c2.md). Base: `dc5f409b0`.

## Goal

The item's done-when reads:

> Already met: Stage C2 finite controls PASS (block apply exact) and are recorded, with the receipt above.
>
> Still to do: a one-page Stage T pre-registration sheet is posted on the issue, complete enough for a non-author TEST FITNESS: FIT review. Nothing in it runs until its FIT and the lab's own pre-run review (D67). Its bar is the Stage 2 bar: "beat the unigram table, then the admission-matched next-word table (the version W2 produces, not the 6,300 full-panel table), on two seeds." The estimate to carry is about $12 for two seeds.

## Cancel probe (run first)

Check that (1) prerequisites `#2831` (`m1-a8-full-dose-rerun`, merged as `dc5f409b0`) and `#3179` (`m1-w2-code-utilization`, PR `#3193`) are `status: done`, (2) the Stage C2 qualification receipt `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json` matches SHA-256 `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`, and (3) `~/.cargo/bin/cargo check -p uor-r4-discrete` compiles cleanly on host.

**Probe result (`instrument` — PASS):**
1. `#2831` (`dc5f409b0`) verified the A8 address-structured lineage (`address-structured-v1.json`, `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) under active D40 rollback (`AuditPolicy::registered()`), passing `B3_init` (`1,351 / 3,847` DEV `P_succ`) and `Q1` on `FrozenZero` (`DEV b_3(P_succ) = 1,106 / 3,847`, `DEV b_3(Full) = 4,292 / 14,287`, `DEV top1_correct = 871 / 14,287`), and diagnosed the `K = 1` 64-row greedy proposal + 64-batch rollback coupling motivating Stage T (`BVG-K` at `K = 16`). `#3179` (`PR #3193`) established the W2 admission-matched comparators (`1,138 / 14,287` restricted code table, `1,482 / 14,287` restricted bigram on `1,920` admitted rows, vs `1,226 / 14,287` unigram and `6,300 / 14,287` full-panel `T2`).
2. `shasum -a 256 .../bvgk-qualification.json` matches `2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3` (`8/8` C2 controls pass).
3. `~/.cargo/bin/cargo check -p uor-r4-discrete` succeeds (`exit 0`).

## Files in scope

- `docs/harness/prep/m1-bvg-k-block-apply-c2*`
- `docs/harness/prep/m1-bvgk-stage-t*`
- `docs/harness/prep/m1-bvgk-staget*`
- `docs/labs/m1-bvg-k-block-apply-c2/**`
- `docs/loop/tasks/m1-bvg-k-block-apply-c2/**`
- `docs/research/notebook/m1-discrete-attention.md`

## Done when

1. A one-page Stage T pre-registration sheet (`docs/harness/prep/m1-bvgk-stage-t-training-2026-10-25.md`) is written, passes `prep::tests::the_real_sheets_are_readable` and `uor-harness loop pr-check --local`, receives a fresh-context non-author `TEST FITNESS: FIT` review and D68 decision review, and is posted on `#2835` with the Stage 2 bar, `~$12` two-seed cost estimate, pinned A8 base lineage and D40 controller, the three C2 uncovered items (`AKd`, `V1d`, `KId`/`V8d`/`V9d`) registered as first-batch GPU controls, and an explicit "What this leaves uncovered" section.
2. `docs/labs/m1-bvg-k-block-apply-c2/README.md` records the C2 qualification receipt and Stage T registration summary, one row is appended to `docs/research/notebook/m1-discrete-attention.md`, and `docs/roadmap/items/m1-bvg-k-block-apply-c2.md` is set to `status: done`.

## Stop and ask if

- Any file outside the declared scope needs editing (re-scope or run a D68 decision review, never wait).
- The fresh-context non-author FIT review returns `NOT FIT` on a defect that cannot be fixed inside the sheet scope.
