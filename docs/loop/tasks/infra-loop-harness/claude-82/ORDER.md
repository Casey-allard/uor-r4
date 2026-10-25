# ORDER: infra-loop-harness, claude-82 (one plan, held work released, owner calls of 24 Oct)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `460d1e413`.

## Goal

The owner, 24 Oct: "please reign in the project sprawl and get us back to a workable central source of truth we can run off of", and "our mandate should be enablement for moving forward and moving forward". Five files each call themselves the plan. `STATE`/`HANDOFF` are 12 commits stale. 26 items stay `held` although D67 retired the owner release, and no milestone moved in two weeks.

1. **`PLAN.md` is the one plan.** It gives the path to M1 ranked by how many items each step unblocks, the route M2–M8, and today's owner directions. It keeps the Premises and Owner directions tables that `loop next` reads.
2. **25 `held` items become `ready` (D67).** `m2-donor-corpus-generation` stays `held`: D41's donor pass still needs the owner at dispatch (CHARTER §5.1, §3.10).
3. **#2828 closes on its result (D78).** `m1-trunk-assembly-and-training` moves to M2, so M1 closes without the trunk (D79).
4. **The pointers name `PLAN.md`:** CHARTER §5.2–§5.8, `README.md`, `ROADMAP.md`, `docs/README.md`. The old "released by the owner" wording follows D67 in the README, `ROADMAP.md` and the loop README.
5. **D77–D79** record it. The roadmap view, `STATE` and `HANDOFF` are re-rendered.

## Cancel probe (run first)

At `460d1e413`, is any of this already done? No:
- `git grep -c 'status: held' docs/roadmap/items` gives 26;
- `docs/README.md` names `NOW.md` as the "live plan" three times;
- `PLAN.md` lists the trunk first for M1.

The unit stands. Class: instrument (records).

## Files in scope

- `PLAN.md`, `CHARTER.md`, `README.md`, `ROADMAP.md`, `docs/README.md`, `docs/loop/README.md`
- `docs/integration/DECISIONS.md`
- `docs/roadmap/items/*.md` (status and milestone lines), `docs/roadmap/README.md` (rendered)
- `STATE.md`, `HANDOFF.md` (rendered)
- `docs/mechanisms/legal-construction.md`, `docs/research/notebook/legal-construction.md`
- `docs/loop/tasks/infra-loop-harness/claude-82/**`

## Done when

- `PLAN.md` holds the ranked M1 path and today's directions, and the other plan files point to it.
- 25 items are `ready`, `m2-donor-corpus-generation` stays `held`, #2828's item is `done`, and the trunk item is M2.
- D77–D79 quote the owner.
- The roadmap check, `loop report --check`, `loop pr-check --local` and the quick gates pass.
- A fresh-context review at the exact head counts (D66).
- The PR says `Closes #2828` and `References #2904`.

## Stop and ask if

A held item turns out to need the owner for a reason other than the retired release.

## Owner approval

Owner paths changed: `CHARTER.md`, `docs/rules/changes.md` and `docs/integration/DECISIONS.md`.

Casey, in Claude's chat by 2026-10-25T00:03:59Z:
- "please reign in the project sprawl and get us back to a workable central source of truth we can run off of".
- For #2828: "Close it on its result (Recommended)".
- For the trunk: "M1 closes without it (Recommended)".
