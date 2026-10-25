# ORDER: infra-loop-harness, claude-90 (two item fixes: C2's scope, address-as-read's dependency)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance, as in kimi-27). Base: `a4eaaf238`.

## Goal

Two item files block or mislead `loop next`.

1. **`m1-bvg-k-block-apply-c2` (#2835).** This is step 3 of the M1 path. Its remaining work is only the one-page Stage T pre-registration sheet: the C2 finite controls already passed (receipt `2fff942f…`). But its scope still claims all of `crates/uor-r4-discrete/**`, the default #3171 gave every M1 item.
   - That overlaps Antigravity's open A8 order (#3180), so `loop claim` refuses it. Two labs that could work in parallel are serialized for no file reason.
   - Narrow the scope to the item's docs paths.
   - Also bring its text up to date: Stage T now runs on the lab's own pre-run review after its FIT (D67, D77), not on a typed owner release.
2. **`m1-address-as-read` (#2833).** `loop next` offers it as ready with no dependencies. But its done-when needs a registered three-arm GPU evaluation on the held #2844 checkpoint, which is the trunk (`m1-trunk-assembly-and-training`, M2 under D79).
   - Declare that dependency, so `loop next` stops sending labs to an item they cannot finish.

Record the decision as a `[claude] owner log:` note on #2835 and #2833, with its review record (D69). `DECISIONS.md` is an owner path, so a lab's own decision never goes there.

## Cancel probe (run first)

At `a4eaaf238`:
- `UOR_LAB=claude uor-harness loop next` skipped #2835 and went GO on `m1-address-as-read`. Its item file has `depends_on: []`, and its "What is missing" names the #2844 checkpoint.
- `m1-bvg-k-block-apply-c2.md` has `scope: [crates/uor-r4-discrete/**, …]`. Antigravity's #3180 holds `m1-a8-finite-cuda-qualification`, whose scope is the same glob.
- `uor-harness loop claim infra-loop-harness` refused: "already claimed by PR #3169 (codex-1)". #3169 is Codex's draft, idle since 01:07Z and conflicting with main. Its files are `PLAN.md`, `DECISIONS.md` and its own task folder.
  - This unit shares no file with #3169. So this ORDER is written by hand, as claude-88's was.
  - The umbrella item acting as a single lock for every harness unit is recorded under Still open.

The unit stands. Class: instrument (roadmap bookkeeping).

## Files in scope

- `docs/roadmap/items/m1-bvg-k-block-apply-c2.md`
- `docs/roadmap/items/m1-address-as-read.md`
- `docs/roadmap/README.md` (rendered by `cargo run -p xtask -- roadmap render`)
- `docs/loop/tasks/infra-loop-harness/claude-90/**`

## Done when

- After the merge, `loop next` no longer offers `m1-address-as-read`.
- `loop claim m1-bvg-k-block-apply-c2` is no longer refused for overlap with #3180.
- The render shows both changes.
- A fresh-context review at the exact head counts.
- The PR says `References #2835` and `References #2833`. Neither item closes here.

## Stop and ask if

Never stop and wait. If a check shows the C2 sheet does need crate code, keep the crate glob, record why, and take other work.
