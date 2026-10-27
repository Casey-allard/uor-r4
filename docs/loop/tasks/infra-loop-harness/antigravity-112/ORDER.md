# ORDER: infra-loop-harness, antigravity-112

Lab: antigravity. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in antigravity-100, antigravity-107, and antigravity-109). Base: `3b0c0059ef15286eab91e41df9b1ecbf5ce13b79`.

## Goal

Unblock `uor-harness loop next` and `uor-harness loop claim` on `m4-lawful-likelihood-bpb` (`#2861`, M4) and `object-store-consumer-m3` (`#2873`, M3) now that M1 (`#2029`) and all 8 steps of the M1 ranked path (`PLAN.md` Steps 1–8, including `#2837` and `#2841`) are `status: done` on `origin/main` with two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`, `uor-r4.lut-stack/3`) artifacts (`04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d` and `1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de`):

1. **`docs/roadmap/items/m4-lawful-likelihood-bpb.md` (`#2861`, M4):**
   - Narrow `scope:` from `[crates/uor-r4-discrete/**, crates/uor-r4-integer/**, docs/mechanisms/lut-serving.md, ...]` to `[docs/model/serving-and-export.md, docs/research/findings.md, docs/research/open-questions.md, docs/labs/m4-lawful-likelihood-bpb/**, docs/harness/prep/m4-lawful-likelihood-bpb*, docs/loop/tasks/m4-lawful-likelihood-bpb/**, docs/research/notebook/m4-serving.md]` now that `crates/uor-r4-integer/src/lawful_likelihood.rs` (`#2945`) is merged and only the docs-only sheet registration and FIT remain (`#2861#issuecomment-6374580394`), clearing the false overlap with `d11-exporter-float-trunk` (`PR #3320`) and `infra-loop-harness` (`PR #3319`).
2. **`docs/roadmap/items/object-store-consumer-m3.md` (`#2873`, M3):**
   - Under a fresh-context D68 decision review (`DECISION.md` and owner log on `#2904`, `#2861`, and `#2873`), release `status: held` -> `status: ready` and narrow `scope:` to `[crates/uor-r4-object-store/**, docs/mechanisms/exact-memory.md, docs/mechanisms/object-store.md, docs/labs/object-store-consumer-m3/**, docs/harness/prep/object-store-consumer-m3*, docs/loop/tasks/object-store-consumer-m3/**, docs/research/notebook/m3-durable-memory.md]` (since `crates/uor-r4-integer/src/session_codec.rs` interop is already merged in `#2960` / `#3106` and M1 is now `CLOSED_PASS`), and re-render `docs/roadmap/README.md`.

## Cancel probe (run first)

1. `uor-harness loop claim infra-loop-harness --lab-topic post-m1-scopes` refused on `PR #3319` (`kimi-50`). Checked `gh pr view 3319 --json files`, `gh pr view 3320 --json files`, and `gh pr view 3321 --json files`: `0` overlapping files (`#3319` touches only `AGENTS.md`, `HANDOFF.md`, `STATE.md`, `docs/harness/LESSONS.md`, and `kimi-50/{ORDER,REPORT}.md`). Hand-written `ORDER.md` per PART 3 Step 2 umbrella exception.
2. Confirmed (`instrument` classification on `3b0c0059e`): `uor-harness loop next` outputs `WAIT` because `m4-lawful-likelihood-bpb.md` claims `crates/uor-r4-integer/**` (overlapping `PR #3320`) and `object-store-consumer-m3.md` is still `status: held`.

## Files in scope

- `docs/roadmap/items/m4-lawful-likelihood-bpb.md`
- `docs/roadmap/items/object-store-consumer-m3.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/infra-loop-harness/antigravity-112/**`

## Done when

1. `docs/roadmap/items/m4-lawful-likelihood-bpb.md` narrows its `scope:` to its docs/prep/lab/task/notebook paths, `docs/roadmap/items/object-store-consumer-m3.md` is updated to `status: ready` with non-overlapping scope, and `docs/roadmap/README.md` is re-rendered.
2. Fresh-context D68 decision review (`DECISION.md`) passes and is logged as `[antigravity] owner log:` on `#2904`, `#2861`, and `#2873`.
3. `xtask roadmap check`, `uor-harness loop pr-check --local`, and `uor-harness loop gates --full` pass.

## Stop and ask if

- Never stop to wait on Casey; log the D68 decision review on `#2904`, `#2861`, and `#2873`.
