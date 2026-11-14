# ORDER: the mechanism board and pipeline map brought to 14 Nov 2026, claude-375

Lab: claude. Item: [docs-status-refresh](../../../../roadmap/items/docs-status-refresh.md) (#3380). Base: `085ff55d92c31ded738913ae8907251e4dcd8a21`.

On 14 Nov 2026 Casey asked, in claude's chat, that the repository documentation be brought up to date after a read-only audit of which vector-space mechanisms are trained, served and wired into the model. The board and the pipeline map stop at 31 Oct. The audit is `local/claude/one-vector-roadmap-2026-11-14.md` (not committed), made against this base. Its sections 1, 2 and 5 are the facts this unit reconciles. Its section 4, the V0–V5 roadmap, is Casey's open decision and is not written into any page here.

## Goal

Reconcile recorded facts only. Every claim is checked against the code at the base or the [float-training notebook](../../../../research/notebook/float-training.md) before it is written.

1. **`docs/mechanisms/README.md`:**
   - add the results since 31 Oct in a section of their own: the E8 address memory (claude-346, claude-366), the pair experts (claude-367, claude-368) and the pair rows (claude-370 Adds; claude-373 and claude-374 Do not add);
   - say which mechanisms are in claude-358's `p` recipe and which are not (Flock is ACTIVE and in no current recipe);
   - name the unused attention and zeta modules in `crates/uor-r4-core/src/`, say that the historical source of `geometric_stack_native.rs` is inactive, and that `helm_d_r4_attention.rs` is a softmax oracle.
2. **`docs/model/pipeline-map.md`:**
   - the bound artifact is `lut-stack/10`, and `/11` is the address-table research schema, exported with `--legacy-artifact` only;
   - `uor-chat --pair-experts` replaces a reply (`crates/uor-r4-integer/src/stack/chat.rs`);
   - training carries no state across windows (`Carry` is a scoring rule, `in_space_long.rs`);
   - the export refuses pair rows;
   - the units since claude-345.
3. **`README.md`:** only stale facts, worded as measurements with sources. `AGENTS.md` is an owner path; it changes only if one of its facts is stale.
4. **Redirect stubs** (`starling-read.md`, `flock-rank-reads.md`, `invariant-addressed-read.md`, `exact-addressed-memory.md`) stay redirects. Their target pages gain any content of the archived brief they lack.

## Scope

- `docs/mechanisms/README.md`
- `docs/model/pipeline-map.md`
- `README.md`
- `docs/mechanisms/flock-read.md`, `docs/mechanisms/route-read.md`, `docs/mechanisms/exact-memory.md`, only for content a redirect stub's archived brief holds and they lack
- `docs/loop/tasks/docs-status-refresh/claude-375/ORDER.md` and `REPORT.md`
- `docs/roadmap/items/docs-status-refresh.md`, only to add this unit's task folder to the item's scope

No other file may change. No ACTIVE / READY / PARKED status, bar, scope, stop rule or decision changes: status changes this unit would propose are listed in the REPORT's "Still open" as decision cards. No code, no `CHARTER.md`, no other lab's files, no rendered view (`STATE.md`, `HANDOFF.md`, `docs/roadmap/README.md`). No model run and no GPU.

## Cancel probe

GO at 14 Nov 2026 18:13Z.
- `origin/main` is `085ff55d9`, the audit's base.
- No open PR touches `README.md`, `docs/mechanisms/` or the item file.
- Two codex drafts touch `AGENTS.md` and `docs/model/pipeline-map.md`: #3967 (`codex/native-space-cycle`) and #3968 (`codex/cuda-native-train`). Their pipeline-map change is a D85 note near the top and a heading rename of "Two model paths". This unit leaves those lines alone and edits other sections, so whichever PR merges second merges forward. It is not a reason to stop, and it is named in the REPORT.

## Done when

- The diff is docs-only, inside the scope above.
- Every number and code location in the changed lines matches its source at the base.
- Every relative link in the changed files resolves.
- `uor-harness check` passes, `cargo run -q --locked -p uor-r4-claim-wording -- .` passes, and the docs-archive-and-links CI job is green at the head.
- A fresh-context exact-head review (docs-only, so a same-lab record is allowed) approves.
- The PR is queued with `uor-harness loop queue --pr N` and checked with `uor-harness loop close --verify --pr N` from the main checkout.

## Stop and report

Stop after the board, the map, the README, the target pages, the review and the merge verification. Do not write the V0–V5 roadmap into `PLAN.md`, the roadmap or any page.
