# REPORT: infra-loop-harness, claude-76

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- No item's `scope:` covered the mechanism board, so the record PR for the owner's 03:41Z rulings (claude-77) would fail pr-check's scope rule, which reads item scopes at the merge base. This unit lands the scope first, as claude-73 did for `CHARTER.md`.

## What changed

The `infra-loop-harness` item's `scope:` gains five paths after `docs/rules/changes.md`: `docs/mechanisms/README.md`, `lut-serving.md`, `hopf-observation.md`, `spin-weights.md` and `ternary-xor.md`. These are the board and the four pages whose status lines change under the rulings "D11 READY, split Hopf (Recommended)" and "Keep both READY". Nothing else in the item changes.

Size: docs 3 files (line counts in pr-check's size line at the top of the PR body). Against the ORDER's estimate (one line plus the task folder): 1.0.

## Measurements

Nothing was measured; this is a record change.

## Checks run at the head

- The cancel probe at `8a646977c`: `git grep -n "^scope:" -- docs/roadmap/items | grep -c -E "docs/mechanisms/README|docs/mechanisms/\*\*"` printed `0`.
- `~/.cargo/bin/cargo run -q --locked -p xtask -- roadmap check`: `roadmap check: 91 items OK`.
- `loop report --check`, `loop pr-check --local`, `git diff --check` and the quick gates: results are in the PR's review record at the head.

## Skills used

None.

## Still open

- claude-77, the record PR itself, adds:
  - D72–D74 for the three rulings;
  - the board edits: D11 to READY, Hopf split into the kept-fibre map (READY) and sector routing (PARKED), and spin-native weights and ternary/XOR to READY;
  - `status: done` on the three decision items, with `Closes #2874`, `Closes #2875` and `Closes #2876`.

## Plain summary

Each roadmap item lists the files its work may touch. The mechanism board, the page listing every mechanism's status, was on no item's list, so the owner's rulings on four mechanisms could not be written into it through the normal path. This change adds the board and the four affected pages to the list of the item that owns the project's records.
