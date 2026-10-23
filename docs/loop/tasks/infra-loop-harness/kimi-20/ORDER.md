# ORDER: infra-loop-harness, kimi-20 (the loop page documents the render refresh recipe)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6 (the render chain this documents, as with kimi-17..19 and deepseek-44).
Base: `aac963833`.

## Goal

Card #3043: the render gate (#3035) requires every view's stamp to equal the PR's merge base with origin/main. After `git merge origin/main` on the render branch, the merge base is main's TIP, not the merge commit, so a plain `loop render` stamps the wrong commit and the gate refuses by name (verified live on #3038). The working recipe is `loop render --at "$(git merge-base origin/main HEAD)"` after the merge-forward. Add that recipe to the loop page's "The render PR" paragraph (`docs/loop/README.md`), which today says only "renders" without saying how the stamp is chosen.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base, `docs/loop/README.md`'s "The render PR" paragraph does not mention `--at` or the merge-base stamp; `grep -n -- '--at' docs/loop/README.md` finds no render recipe. Task stands.

## Files in scope

- `docs/loop/README.md` (a shared file; the one paragraph)
- this task folder

## Done when

- The paragraph names the recipe: merge `origin/main` forward, then `uor-harness loop render --at "$(git merge-base origin/main HEAD)"`, commit and push, never by force.
- The diff is that paragraph plus this task folder.
- `loop report --check` and `loop pr-check --local` pass; a fresh-context review from another vendor's lab is requested.

## Stop and ask if

- The recipe as verified on #3038 turns out to be wrong on a fresh check.
- Any file beyond the loop page and this folder needs to change.
