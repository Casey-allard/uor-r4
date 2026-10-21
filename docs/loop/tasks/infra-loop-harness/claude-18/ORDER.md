# ORDER: infra-loop-pr06-review-records, claude-18 (loop PR 6: loop review with tagged review records)

Lab: claude. Item: [infra-loop-pr06-review-records](../../../../roadmap/items/infra-loop-pr06-review-records.md) (#2909). Setup row: 6. Base: `987f72259`.

## Goal

The item's done-when:
- `loop review` writes and parses the `uor-review v1` comment.
- The policy check needs an APPROVE at the current head, from a lab whose vendor differs from the author's, with no later CHANGES. A doc-only or generated-only PR may carry a same-lab `fresh-context` record.
- Under D59, a setup PR counts only `claude` records. A setup PR is one whose `Setup row:` names a row in the build-plan table, and whose item is `infra-loop-harness` or that row's own item.
- The end of D59 is an owner switch.

The planted fault: **an untagged record, a same-vendor record on a code PR outside D59, or a setup-PR record from a lab other than Claude is refused.**

## Cancel probe (run first)

Does `loop review`, or any `uor-review v1` parser, exist? Result at `987f72259`: no. Review records are free text, and `pr-check` and `next` parse the `[<lab>] Exact-head review record` line. The task stands. Class: instrument.

## Architecture calls (Claude, D52)

1. **One module, `loopreview.rs`.** It renders and parses records, decides setup-PR status, reads the D59 switch, and runs the gate as a pure function. `pr-check`'s `review-record` rule and `loop next` both call it. After this lands, only `uor-review v1` records count.
2. **The record.** The first line is `[<lab>] uor-review v1`. Below it are `key: value` lines: `pr`, `head`, `lab`, `model`, `launch`, `verdict` (`APPROVE` or `CHANGES`), `order`, an optional `context: fresh`, and a `done-when` line holding either `host: …` or `gpu: sha256:…`. Free text may follow after a blank line. The tag must equal `lab:`.
3. **The proposed switch.** `docs/harness/d59.json`, holding `{"schema":1,"d59":"in force"|"ended","owner_words":…,"time":…}`. This PR proposes the file and does not add it.
   - The gate treats D59 as in force unless both hold: `owner-paths.json` at the base lists the path, and the file reads `"ended"`.
   - Listing the path is the owner's step.
4. **Writing.** `loop review --pr <n> --verdict APPROVE|CHANGES` posts a comment. That is the only GitHub write, and it goes through a trait with a fake. It refuses to post a record that would not count.

## Files in scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**`
- `docs/harness/README.md` and `docs/loop/README.md`: usage, and what is built.
- `docs/loop/tasks/infra-loop-harness/claude-18/**`

## Done when

- Each case of the planted fault has a fixture test. This includes a Claude-built code PR with no setup row, and `Setup row: 99`.
- A Claude setup PR with a Claude APPROVE passes.
- The switch defaults to D59 in force.
- The full harness suite passes, as do `pr-check --local`, claim-wording, links and the copyright check.

## Stop and ask if

- The switch would need to be added to owner paths in this PR.
- A GitHub write other than the review comment is needed.
- Any file outside scope needs to change.
