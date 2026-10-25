# REPORT: infra-loop-harness, claude-101

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2904. References #2851. References #2823.

## What did not work, first

- **`loop next` offered a blocked item as GO three times in a row.** It offered `m2-pin-train-mask-sheet` to claude-99, claude-100 and again after #3199, ranked as unblocking 8 items. Both branches of its done-when wait on other items:
  - the D37 branch on `float-training-module` and `d11-exporter-float-trunk`;
  - the discrete branch on a current discrete M2 trainer.

  A single `depends_on` cannot say "either", and the #3196 decision review rejected gating it on the float path alone. So the item is now `held`, and its text names both releases.
- **`float-training-module` could not have produced its deliverable.** Its done-when asks for a new crate with no dependency on `crates/uor-r4-training`, but its `scope:` held only that crate, `ratchet.rs` and docs. `loop guard` reads scope from `origin/main`, so the first write to the new crate would have been refused.
- **My first edit added a `notebook:` line for a page that does not exist,** and `roadmap check` refused it. The page is listed in `scope:` instead. The unit that creates it will add the `notebook:` line in the same PR. pr-check accepts a new notebook bound in the same PR, but the binding edits the item file, which it allows only for status edits. So the round-1 review had the item file added to its own `scope:` too.

## What changed

- **`docs/roadmap/items/m2-pin-train-mask-sheet.md`:** `status: held`, and a "Held (25 Oct)" note naming the two releases. One is `float-training-module` and `d11-exporter-float-trunk` done. The other is `m1-trunk-assembly-and-training` done, or a merged discrete trainer named by commit, so that a one-page discrete-law sheet can take a FIT at a current head. Round 1 made this release checkable. It also notes that `m2-train-grounded-replies` cannot be the route, because it depends on this item. Preparation that runs nothing may go ahead.
- **`docs/roadmap/items/float-training-module.md`:** `scope:` adds `crates/uor-r4-float-train/**`, the root `Cargo.toml`, `Cargo.lock`, `docs/research/notebook/float-training.md` and the item file itself. The note forbids changing other crates' entries in `Cargo.toml` or `Cargo.lock`. A dated note explains why. `crates/uor-r4-training/**` stays as before; the new crate must not depend on it.
- **`docs/roadmap/README.md`:** re-rendered (ready 50 → 49, held 3 → 4).

## Measurements

None. This is item-file maintenance.

## Checks run at the head

- `cargo xtask roadmap check`: 92 items OK. `roadmap render` was re-run.
- **Cancel probe:** at `b1cc33d03` the item was `ready` and `loop next` offered it as GO; the float module's scope held no new crate; no open PR touches either file.
- **Round 1 of the fresh review:** DECISION REVIEW: APPROVE on both changes, but VERDICT: CHANGES. The discrete release was not checkable, and it was circular through `m2-train-grounded-replies`; the notebook binding would have failed the float unit's own pr-check. Both are fixed. Round 2 is in the PR's review record and the owner logs on #2851 and #2823.

## Skills used

- scope-guard: the float module's scope now covers the crate its done-when names, so its unit needs no workaround.

## Still open

- **`float-training-module`** is now claimable with the right scope. It is claude's next unit.

## Plain summary

The loop kept steering labs to an M2 item that can't move until other work lands, so it's now marked held, with the two ways to release it. The float-training item can now create the new crate its own goal asks for.
