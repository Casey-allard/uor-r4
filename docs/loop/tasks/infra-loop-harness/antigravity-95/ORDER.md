# ORDER: infra-loop-harness, antigravity-95 (release `m1-export-gap-and-toks` and Stage-T-gated discrete items from `held`)

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file status and scope maintenance, as in claude-114, claude-120, claude-121, and antigravity-83). Base: `bd54e2a184a06216544af5e6bd9d24544e6b0fc6`.

## Goal

With `m1-bvg-k-stage-t-training` (#2836, merged in #3239), `m1-d11-artifact-scorer` (#3248, merged in #3249), and `m1-within-cell-writer` (#3267, merged as `bd54e2a18`) all `status: done` on `origin/main`, `uor-harness loop next` returns `WAIT` because the downstream items that were placed on `status: held` waiting for `#2836` (and `#3248`) still read `status: held` in `docs/roadmap/items/`.

If a fresh-context D68 decision review approves, this unit releases four items whose documented hold prerequisite (`#2836` complete, plus `#3248` for Step 6) is now met on `origin/main`:

1. **`docs/roadmap/items/m1-export-gap-and-toks.md`** (`#2839`, Milestone M1, Step 6 of `PLAN.md`, unblocking `#2841` and `#2837`):
   - set `status: ready` and `depends_on: [m1-d11-exporter-discrete, m1-bvg-k-stage-t-training, m1-d11-artifact-scorer]`;
   - update `scope:` to `[docs/labs/m1-d11-artifact-scorer/instrument/**, docs/harness/prep/m1-export-gap-and-toks*, docs/labs/m1-export-gap-and-toks/**, docs/loop/tasks/m1-export-gap-and-toks/**, docs/research/notebook/m4-serving.md]` (pointing to the detached `d11-scorer` instrument built in `#3248` with no crate lock, and dropping `docs/mechanisms/lut-serving.md` so idle `#3169` does not block `loop claim`, as `claude-121` did for `#2820`);
   - update `What is missing` to record that `#2836`'s saved two-seed checkpoints and `#3248`'s `d11-scorer` instrument are on `main` / HF store, leaving `Δloss`, planted-fault control `B4`, the GPU panel export-gap run, and laptop `tok/s`.
2. **`docs/roadmap/items/r-ranking-with-admission-intact.md`** (`#2884`), **`docs/roadmap/items/r-rank-objective-keeps-cells.md`** (`#2883`), and **`docs/roadmap/items/r-finite-relational-head.md`** (`#2877`):
   - set `status: ready`, add `m1-bvg-k-stage-t-training` to `depends_on:`, add `notebook: docs/research/notebook/m1-discrete-attention.md`, and add `crates/uor-r4-discrete/**` and `docs/research/notebook/m1-discrete-attention.md` to `scope:`.

## Cancel probe (run first)

Class: instrument. Run at `bd54e2a184a06216544af5e6bd9d24544e6b0fc6`:
- `uor-harness loop claim infra-loop-harness --lab-topic release-step6-and-stage-t-items` refused: `item infra-loop-harness is already claimed by PR #3169`.
- Checked open PRs (`#3284`, `#3283`, `#3169`) via `gh pr view --json files`: zero file overlap with the four item files, `docs/roadmap/README.md`, or this task folder. Hand-written under the umbrella exception (`run-antigravity.txt` Part 3.2).
- Verified on `origin/main` that `#2836`, `#2838`, `#3248`, and `#2831` all read `status: done`.
- **Result (`instrument`):** All hold; proceed to the D68 decision review.

## Files in scope

- `docs/roadmap/items/m1-export-gap-and-toks.md`
- `docs/roadmap/items/r-ranking-with-admission-intact.md`
- `docs/roadmap/items/r-rank-objective-keeps-cells.md`
- `docs/roadmap/items/r-finite-relational-head.md`
- `docs/roadmap/README.md` (generated render only)
- `docs/loop/tasks/infra-loop-harness/antigravity-95/**`

## Done when

- A fresh-context D68 decision review approves the status and scope updates, logged as `[antigravity] owner log:` on `#2904`, `#2839`, `#2884`, `#2883`, and `#2877`.
- The four item files and rendered `docs/roadmap/README.md` are updated, `xtask roadmap check`, `loop report --check`, `loop pr-check --local`, and `loop gates --full` pass, and a fresh-context D66 exact-head review approves.

## Stop and ask if

- Never stop and wait. If the D68 reviewer rejects releasing any of the four items, keep that item `held` and record the finding in `DECISION.md` and `REPORT.md`.
