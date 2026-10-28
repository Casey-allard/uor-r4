# REPORT: pipeline-map-revision, claude-159

Lab: claude. Order: [ORDER.md](ORDER.md). Doc-only: no code, no run, no GPU.

The pipeline map is revised. Every stage now leads with its cost per token, measured where claude-157 and claude-158 measured it. claude-149 to claude-158 are folded in, the gaps and the order of work are re-ranked, and a new section reads the archived prime and angular router in full. The owner's "golden ratio complex – Riemannian bridge" is ADR-0003 §5.

## What did not work, first

- **Two of three archive readers stopped early.** The subagents started at about 129k tokens of context.
  - The Rust-router reader ran two tool calls before its hand-back.
  - The docs reader read all six documents and stopped at 200k.

  The top session read the rest itself:
  - the three router commits' messages;
  - #958's evidence;
  - ADR-0003 §1–§5;
  - the ancestor's README and function list;
  - the prime-pointer and Hopf mechanism pages.
- **claude-156's items fail the roadmap check.** `cargo xtask roadmap render --check` fails on main with 11 problems, all in the three items claude-156 wrote:
  - missing `## How to start`, `## Rules that apply` and `## Sources`;
  - `route-plus-memory`'s 85-character title;
  - `route-plus-memory`'s area `memory`.

  The loop gates did not catch this. This unit fixes its own item, leaving 8 problems. The other two items are outside this scope, and the umbrella that could fix them overlaps this unit's open claim. So **the next unit is that fix**, through `infra-loop-harness`. `roadmap render` refuses until it lands, so `docs/roadmap/README.md` is re-rendered there too.
- **claude-156's item was wrong about the archive.** It said the search "found no record of … a 'complex–Riemannian bridge' by that name". ADR-0003 §5 is that bridge. The item now carries a dated correction.
- **The first map's citations had drifted.** Its float-training notebook "line N" references now point at other rows, because the notebook adds rows at the top. Its two copy-form commits (`87ade4920`, `6d8bac253`) are not on main after #3332's squash merge. claude-146's REPORT also gives those findings under different commits. The revision names notebook rows by unit and date, and cites claude-146's REPORT lines 29–30.

## What changed

- **`docs/model/pipeline-map.md`:**
  - **cost first:** every stage table now starts with "Cost per token", measured where profiled (stages 3, 4, 5, 7, 8, 9, 11), and the chain table gains a served-cost and a training-cost column;
  - **a new table**, "What changed since the first map", for claude-149 to claude-158;
  - **stages updated:** the trunk (fit `9,328 / 10,573` and `10,152 / 11,140`, the storage diagnosis), the look-back (the Hamming read merged; the type spin, the opening read and the register negative; the bag works), the long memory (607 of 1,226 by the turn's first 8 tokens), the readout (claude-150's decision to serve nearest, at 21 times the writer), training (the profile; rates up to `41,295`) and serving (98.6% in the trunk);
  - **a new section,** "The archived prime and angular router": what it computed, what was measured, its cost per token today, which piece fits which stage, and the owner's memory checked line by line;
  - **the scale arithmetic** re-run on the measured rates: 1T tokens in about 70 to 91 days on 4 GPUs, if scaling held;
  - **the gaps re-ranked**, with storage first, the served decoder second, serving kernels third and training cost fourth;
  - **the order of work** rebuilt as R1–R8, with `route-plus-memory` first.
- **`PLAN.md`:** the 27 Oct refocus direction is set to `done`, with its note.
- **`docs/roadmap/items/pipeline-map-revision.md`:** the three required sections, the correction, and `status: done`.

## Measurements

None: this unit runs nothing. Every number in the revision cites its REPORT, notebook row, mechanism page, ADR or PR. Numbers derived from them are labelled proof by arithmetic.

**The archive read, in brief** (the map's new section has the sources):
- **The bridge.** ADR-0003 §5 is the direction-preserving golden radial bridge: R_φ(z) = φ·E(z), with E : ℂ² → ℝ⁴. Exact in ℤ[φ], a shell step is one integer add. Its algebra is built and tested, with no language result.
- **The ancestor.** `projects/research-archives/code/gemini-dev/prime-router/server.py` is a GloVe hypersphere router with an Ollama voice. Its next word is a dense product and an argmax.
- **Hopf sector routing:** negative, MRR 0.0045 against 0.0743.
- **The prime-ranked pointer:** the hit rate rose from 0.377 to 0.545, but the answers stayed at 19–23 of 40, against a bar of 25.
- **PR #455:** averaging over all (25.91%) beat top-3 (24.54%) and hard selection (22.81%).
- **`prime_router_package.py`** is a zeta-zero spectral test on ψ(x) − x, not a router.

## Checks run at the head

- **`cargo xtask roadmap render --check`:** `pipeline-map-revision.md` passes. The 8 problems left are in `fbs-cost-profile.md` and `route-plus-memory.md`, outside this scope (above).
- **`cargo xtask links --json local/claude/links.json`:** no broken link has `pipeline-map.md`, `PLAN.md` or this item as its source. The 1,197 broken links counted repo-wide predate this unit.
- **`uor-harness loop report --check`:** PASS. **`uor-harness loop gates --full`:** 11 of 11 green.
- **The fresh in-unit review: two claims checks at `81be3903e`, each against the sources.** Both gave CHANGES; all findings are fixed in the next commit.
  - **Stages and tables (map lines 39–223):** every claude-150 to claude-158 number matches, and all 39 new link targets resolve. Two medium findings:
    - openings "never passed `302 / 228`" was false at seed 2, so it is now "never beat it by the sheet's margin (bar `347.3`)", also in the gaps;
    - the writer's admission as the fix is now labelled a hypothesis, with the reason it fails stated as not known.

    Seven low findings were fixed too:
    - claude-153's copies in seed order;
    - the read rates labelled a bound across hosts;
    - the step (3.73 ms) told apart from the decoded token (3.77 ms), and the stated prefill rate removed;
    - the loop's 1% marked as a share of the plain token;
    - the layer right after the read (1.69 ms host time) noted, and "host-bound" in place of "waits on launches";
    - the type spin's seed-2 openings pass noted;
    - a wrong cross-reference.
  - **The archive section, the scale arithmetic and the rankings (lines 224–356):** the arithmetic recomputes exactly, and the bridge matches §5 on every point. Three medium findings:
    - the ℤ[φ] algebra is built and tested (`IMPLEMENTED_SUBSTRATE`, `PASS_SOURCE_FREE`), not "never measured". It has no language result;
    - the 4.44% figure is #951's, recorded on #958;
    - the pointer's "because" is a hypothesis.

    Four low findings were fixed too:
    - one add per shell step, not two;
    - the normalized exponential samples above temperature 0;
    - "recorded `NOT_RUN`", not "measured";
    - "hard coded" is read two ways. #455's fixed baseline (26.42%) beat its learned arms.

## Skills used

- **holding-the-goal.** The owner's criterion is cost per token, and the owner asked for the archived bridge. Every stage leads with its cost, and the bridge is read from the ADR's own lines, not from a summary.

## Still open

- **The roadmap check's 8 problems**, in `fbs-cost-profile.md` and `route-plus-memory.md`, and the re-render. That is the next unit.
- **R1, `route-plus-memory`**, the decisive run, after that fix.
- **The ancestor's `server.py`**, read here by its function list and README, not line by line. The map uses only what the README, the evidence note and the reader's cited lines state.

## Done when

1. **Met.** The map is revised as the item's done-when 1 and 2 quote.
2. **Met.** PLAN's 27 Oct refocus direction is `done`.
3. **Met.** `pipeline-map-revision.md` passes the roadmap check's rules, and it is `done`.
4. **Met on merge,** with the `[claude, for Casey]` note on #2028.

## Plain summary

The map of the whole model, from training data to the laptop's reply, now says what each part costs per token, and the order of work follows that.
- **The model cannot store what makes a reply fit the question.** The next run gives it an exact memory, keyed by the user's first words.
- **Serving on the laptop is slow in the model's own layers.**
- **Training is slow in its backward pass and its search of the whole vocabulary.**

The old prime router's "golden ratio bridge" exists: it is ADR-0003 §5. It changes only a vector's length, with integer arithmetic. Its algebra is built and tested, but it has never been tried in a language model.
