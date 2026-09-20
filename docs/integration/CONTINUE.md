# Continue UOR-R4 geometric model research

Read the [exact-occurrence reader result](occurrence-reader-result-2026-09-20.md), its
[design note](occurrence-reader-design-2026-09-20.md) and
[evidence receipt](../evidence/native_geometric_occurrence_reader_2026-09-20.txt), then refresh
origin/main, canonical plan/current state, live issues and resources.

## Where the programme stands

`z_local(v) = z_E(v) + u_S(b)` is the retained local baseline: frozen E plus the separable reader's
**query row only**, with S's absence behaviour and **no history fold**. It reproduces the S-query-only
predictor exactly.

The [learned exact-occurrence reader](occurrence-reader-result-2026-09-20.md) is executed
(evaluation from base `1dae322c`, sealed at
`.uor-models/realtext-prior-2026-09-20/occurrence-reader-4`, 21 files / 3,385,009 bytes).

| Result | Value |
| --- | --- |
| Fresh `geom_slot_only`, reader-relevant, `z_local` | **0.000** |
| Fresh `geom_slot_only`, reader-relevant, **learned reader** | **0.825** [0.768, 0.880] |
| fixed latest-occurrence control | 0.533 |
| geometry-disabled (same weights, `w2=w3=w7=w8=0`) | 0.550 |
| fixed exact-role-match control | 0.008 |
| fresh synthetic all-position CE delta | **+0.6713** (tolerance +0.01) — FAIL |
| bounded raw-text all-position CE delta | **+2.7891** (tolerance +0.02) — FAIL |
| raw-text reads / false copies | 88 / **83** |
| occurrence artifact | **130 bytes** |
| uncached served path | 1,214 µs/position, **no measurable reader overhead** |

**Targeted selection works and the geometric feature carries real accuracy** (+0.275 over the
geometry-disabled control, on a family where exact identity cannot identify the correct source).
**Abstention does not.** The failure is precision on the admitted-but-uncovered stratum: 10/10 covered
raw-text positions were read correctly, and the damage is the 83 wrong reads where a key recurred but
its successor was not the answer. Three earlier versions are preserved sealed as near misses,
including one whose 4-bit threshold landed exactly on the correct candidate's score.

Controls all pass: read-disabled ≡ `z_local`; future-token causality; stale reference rejected after
reset; altered source payload changes the emitted payload (38 changed, 37 correct); export/reload
0 decision mismatches; cross-process continuation identical across three processes.

## The one next task

**Add exactly one declared abstention signal, learned from natural text, and re-evaluate the same
frozen panels and controls.**

The concrete gap: the selector has no training signal that separates "a key recurred and its successor
is useful" from "a key recurred". A construction-only population cannot supply that. The next task
should therefore:

1. Build a small training set of **natural-text positions with candidates** (the existing fit
   documents, admitted-but-uncovered positions) and learn a declared abstention feature or threshold
   from it — never from the raw-text probe windows or the fresh panel.
2. Re-run the **same** fresh synthetic panel, the **same** eight raw-probe windows, the same controls
   and the same uncached cost measurement, and report them against the **same** tolerances.
3. Leave the ring capacity, candidate bound, feature set, weights and amplitude frozen. This is an
   abstention fix, not a re-fit.

Do **not** re-fit the selector, widen the ring or the candidate bound, change width/precision/corpus,
or start a reset-only campaign. If the abstention fix still fails, report the smallest missing causal
condition rather than sweeping.

Also still open: CPQK production continuation is required **before any fit that uses `QueryTrainer`**
(it is neither used nor needed by the occurrence reader's ten-scalar trainer, which has a working
cross-process continuation). Reset/Continue maintenance, exact occurrence/version memory, shared
composition, useful conversation, executed Rust on one accepted artifact and complete consumer-machine
cost all remain unrun. `#973`/`#820`/`#963`/`#964` stay open.

## Resources

The standing-authorized **+10800000 ms** extension to **191300000 ms** was recorded in the live ledger
**before** consumption. This step charges **2700000 ms** once (four harness runs including three
superseded, three continuation checks, compile/test cycles and a documentation/delivery allocation).
**New cumulative 181238565 / 191300000 ms, remaining 10061435 ms (~167.7 min).** Retained:
3,385,009 bytes for `occurrence-reader-4` plus three superseded roots and three continuation roots;
whole-run peak RSS 142,458,880 B; physical energy UNAVAILABLE; the 128 MiB stop margin is intact; the
owner-authorized cleanup removed only regenerable debug caches. No paid external compute, no deletion
of unique material. Reuse the retained artifacts and vectors for derived reporting.

Deliver through a protected PR, verify the actual merged tree, and update the owning issues and any
existing project items.
