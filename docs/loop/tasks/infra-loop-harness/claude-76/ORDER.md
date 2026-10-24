# ORDER: infra-loop-harness, claude-76 (the umbrella's scope gains the five mechanism-board files)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `8a646977c`.

Claude runs solo (owner, 24 Oct: "fyi currently you are the only lab running so continue solo for now"). The next move by unblock value records the owner's 03:41Z rulings, which closes #2874, #2875 and #2876 (#2028 comment 6379430093).

## Goal

The record PR (claude-77) must edit the mechanism board for two of the rulings:
- "D11 READY, split Hopf (Recommended)";
- "Keep both READY", for spin-native weights and ternary/XOR.

No item's `scope:` covers `docs/mechanisms/README.md`. This unit adds it to the `infra-loop-harness` scope, which already holds `docs/integration/DECISIONS.md` and the item files, together with the four mechanism pages whose status lines change.

## Cancel probe (run first)

At `8a646977c`, does any item's `scope:` cover `docs/mechanisms/README.md`? No: `git grep -n "^scope:" -- docs/roadmap/items | grep -c -E "docs/mechanisms/README|docs/mechanisms/\*\*"` prints `0`. The unit stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line only)
- `docs/loop/tasks/infra-loop-harness/claude-76/**`

## Done when

- The item's `scope:` lists `docs/mechanisms/README.md`, `lut-serving.md`, `hopf-observation.md`, `spin-weights.md` and `ternary-xor.md`, and nothing else changes.
- The roadmap check, `loop report --check`, `loop pr-check --local`, `git diff --check` and the quick gates pass.
- The exact-head review is Claude's own fresh-context review (D66).

## Stop and ask if

Any other file has to change.
