# ORDER: number-aware-tokenizer, antigravity-16

Lab: antigravity. Item: [number-aware-tokenizer](../../../../roadmap/items/number-aware-tokenizer.md). Base: `686365c65`.

## Goal

Repair order for PR `#2966` following Kimi's `CHANGES` review (`https://github.com/UOR-Foundation/uor-r4/pull/2966#issuecomment-6341920536`):
1. Merge fresh `origin/main` into `antigravity/number-aware-tokenizer`.
2. Re-render [`docs/roadmap/README.md`](../../../../roadmap/README.md) with `cargo xtask roadmap render` so that flipping [`docs/roadmap/items/number-aware-tokenizer.md`](../../../../roadmap/items/number-aware-tokenizer.md) to `status: done` matches the rendered roadmap index (`40 ready / 18 done`) and passes the `docs archive and links` CI gate.

Estimated diff size: ~55 lines across `docs/roadmap/README.md` and `docs/loop/tasks/number-aware-tokenizer/antigravity-16/**`.

## Cancel probe (run first)

Ran `gh pr view 2966 --comments` and `git diff origin/main -- docs/roadmap/README.md`:
- `docs/roadmap/README.md` was omitted from `antigravity-14`, causing `cargo xtask roadmap --check` in the `docs archive and links` job to fail (`README.md is stale ... line 23`).
- Result: repair is needed; continue.

## Files in scope

- `crates/uor-r4-tokenizer/**`
- `models/**`
- `docs/mechanisms/tokenizer.md`
- `docs/harness/prep/number-aware-tokenizer*`
- `docs/labs/number-aware-tokenizer/**`
- `docs/roadmap/items/number-aware-tokenizer.md`
- `docs/roadmap/README.md`
- `docs/loop/tasks/number-aware-tokenizer/antigravity-14/**`
- `docs/loop/tasks/number-aware-tokenizer/antigravity-16/**`

## Done when

- Fresh `origin/main` is merged and `docs/roadmap/README.md` is re-rendered via `cargo xtask roadmap render` (`cargo xtask roadmap --check` passes).
- `uor-harness loop gates --full`, `uor-harness loop report --check`, and `uor-harness loop pr-check --local` pass at the repair head.

## Stop and ask if

- Any file outside the scope list above would need to be modified.
