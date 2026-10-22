# REPORT: infra-loop-harness, claude-46 (give legal-construction-exact-solve its notebook page)

Lab: claude. Base: `18c03cebc`. Order: [ORDER.md](ORDER.md), committed first (`d16a22476`). The cancel probe found that the item declared no `notebook:`, so the task stands.

## What did not work, first

- **The ORDER's stop condition fired.** `cargo xtask roadmap check` refuses a `notebook:` field that names a page which does not exist yet: "notebook `docs/research/notebook/legal-construction.md` does not exist". This item needs a **new** page, because no notebook covers optimizers, so the #2997 route (a `notebook:` field) cannot work before the page exists. And this unit's own scope (`infra-loop-harness`) cannot create a page under `docs/research/notebook/`.
- **The fallback is the #2991 pattern.** The new page's path joins the item's `scope:`, so the closing unit of #2828 can create the page, with its one row, inside the item's own scope. The effect is the same as the ORDER's goal: the item can record its notebook row. The literal done-when 1 ("declares that `notebook:`") is not met, and this REPORT says so.
- **`main`'s roadmap README was stale** (ready 35, expected 34), from the render race between two status-flip merges. It is re-rendered here as a shared file.

## What changed

| Commit | What |
|---|---|
| `d16a22476` | This ORDER, alone. |
| `438ce619f` | `docs/roadmap/items/legal-construction-exact-solve.md`: `docs/research/notebook/legal-construction.md` is appended to `scope:`. `docs/roadmap/README.md` is re-rendered. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK. `roadmap render --check`: current, after the re-render.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one scope entry, plus the shared README render.

## Still open

- **The #2828 closing unit** creates `docs/research/notebook/legal-construction.md` with its one row. It can then add `notebook:` once the page exists.
- **A harness improvement for later:** `roadmap check` could allow `notebook:` for a page that the same PR creates. That is not needed here.

## Plain summary

The solver task must log its result on a brand-new notebook page, but the tools would not let it name a page that does not exist yet. This lets the task create the page itself.
