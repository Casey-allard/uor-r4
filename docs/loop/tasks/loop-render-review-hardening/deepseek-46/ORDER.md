# ORDER: loop-render-review-hardening, deepseek-46

Lab: deepseek. Item: [loop-render-review-hardening](../../../../roadmap/items/loop-render-review-hardening.md). Base: `0af044c7f`. `References #3048` (draft; the item closes when all five follow-ups land).

## Goal

Close the five follow-ups from the render-path reviews: (1) a `context()` fake-git fixture whose negative gates fail when the render check is removed or `stamp == base` is disabled, and which covers cross-repo, fallback-removed, AGENTS-outside-block and hand-edited-block; (2) `docs/roadmap/README.md` gated by an xtask render comparison at the stamp or explicitly exempted with a recorded reason; (3) the queue mirror (`ReviewContext::of`) must require `isCrossRepository == false`; (4) `loop render` must default its stamp to the merge base on `kimi/render` (so the documented merge-forward refresh passes) or the loop-page wording must require `--at`; (5) tidy the old deepseek-42 REPORT nit.

## Cancel probe

Class **instrument** (harness code + tests; no GPU). At the base: `looprender::default_at` does not exist (render stamps HEAD); `loopprcheck::Pr` has no cross-repository field and `ReviewContext::of` takes the sentinel on the branch name alone; no fixture drives `context()`; the README exclusion has only a code comment; the doubled-prefix nit is present. Task stands.

## Files in scope

- `crates/uor-r4-harness/src/loopreview.rs`, `crates/uor-r4-harness/src/looprender.rs`
- `crates/uor-r4-harness/tests/loop_review*.rs`
- `docs/roadmap/items/*.md`
- this task folder

## Done when

- (3) and (4) implemented as above; `loop render` on a `kimi/render` branch defaults its stamp to `git merge-base origin/main HEAD`, unit-tested offline; the queue/PR metadata carries `isCrossRepository` and the sentinel path requires a known `false` (unknown and fork both refuse), tested.
- (1) a `context()` fixture with negative gates (remove the verify call; disable the stamp rule) plus the four cases; (2) a recorded decision for the roadmap README (xtask comparison or written exemption); (5) the nit fixed.
- `cargo test -p uor-r4-harness`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass; a non-author record at the head.

## Stop and ask if

- The fixture needs network or a dependency (it must stay offline).
- A shared registration file needs more than small additive edits (allowed, but say why).