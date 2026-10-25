# REPORT: m1-bvg-k-block-apply-c2, claude-91

Lab: claude. Order: [ORDER.md](ORDER.md).
References #2835
References #2836

## What did not work, first

- **The cancel probe stopped the sheet. The binding reason is that no base lineage reaches its own admission gate** (the Stage T draft's precondition 2).
  - Registration has to fix the base, and a margin from that base's measured two-seed dispersion. With no qualifying base, a sheet written today would leave both blank, along with the cost, so no FIT could pass it.
  - On 19 Oct the injective pre-flight admitted 127 of 3,847 P_succ rows, below 500.
  - A8's address-structured cells, the draft's own candidate family, admitted 1,351 at the start but eroded by Q1. Their full-dose rerun (#2831, after the A8 qualification #2830) is what would produce a qualifying base.
- **The supporting reason: W2 does not yet point at the learning law.** The Stage T item (#2836) is "only warranted if W2 blames the learning law (H_law)". W2 came back **H_mixed** (#2845, #3177), whose row 3 next step is the code-utilization diagnostic, "do not change the learning law yet". Writing a sheet runs nothing, but that diagnostic supplies the evidence for the call.
- **The first draft of this record named the A8 qualification (#2830) as the base dependency.** The decision review caught that it produces no lineage. Had it gone in, #2830 passing would have re-offered the sheet with precondition 2 still unmet. The base dependency is now the full dose, #2831.
- **`loop next` offered this item anyway** (GO, "unblocks 13"), because neither item declares the two prerequisites. It will keep offering the sheet to every lab until they do. The routing fix is the separate unit claude-92.

## What changed

- This ORDER and REPORT only. No sheet was written.

## Measurements

None.

## The decision (owner log, D69; decision review below)

- **The C2 item (#2835) and Stage T (#2836)** now wait on two items:
  - `m1-a8-full-dose-rerun` (#2831), a base that passes its pre-flight and Q1 gates (cycle-free: #2831 depends only on #2830, and `m1-close-saved-model` already depends on both #2831 and Stage T);
  - `m1-w2-code-utilization` (#3179), the diagnostic W2's row 3 calls for.
- **Stage T stays on the M1 path (D79).** Only its order changes.
- **If the diagnostic shows the learning law is the limiter,** the Stage T sheet comes next, on the #2831 base. Its bar uses the then-current admission-matched next-word table. W2's Q4 numbers for reference: the bigram restricted to admitted cells gets 1,482 of 1,920, and the unigram gets 1,226 on the full panel.
- **If the diagnostic does not show that,** dropping Stage T from the M1 close would amend D79, which was Casey's call. That change is proposed in the owner queue, and is not decided by a lab.
- **To reverse:** drop the two dependencies.

## Checks run at the head

`uor-harness loop report --check` and `loop pr-check --local` were run before the PR went ready (see the PR).

## Skills used

None.

## Still open

- **claude-92:** add the two `depends_on` entries to both items and re-render.
- **The C2 item's done-when** (the sheet) stays open until both prerequisites are done.

## Plain summary

The next M1 step looked like "write the Stage T sheet", but there is no base model yet that keeps its shortlist through training, so the sheet could not name its base, margin or cost. The W2 result also says the learning rule is not the first thing to change. So the sheet waits for a base that holds (the A8 full dose, which follows Antigravity's A8 qualification) and for the code-utilization diagnostic. This unit records that, and claude-92 makes the roadmap say it.
