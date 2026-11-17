---
id: point-key-integer
title: Serve the point key's opening in the integer artifact on the laptop
milestone: research
area: serving
status: held
open_to: lab
needs_gpu: false
depends_on: [one-model-end-to-end, turn-memory-integer, integer-retention-rates, integer-nearest-readout]
notebook: docs/research/notebook/float-training.md
scope: [crates/uor-r4-integer/src/stack/session.rs, crates/uor-r4-integer/src/stack/format.rs, crates/uor-r4-integer/src/stack/memory.rs, crates/uor-r4-integer/src/stack/mod.rs, crates/uor-r4-integer/src/stack/point_key.rs, crates/uor-r4-integer/src/stack/tests.rs, crates/uor-r4-integer/src/stack/synthetic.rs, crates/uor-r4-integer/src/bin/fbs-export.rs, crates/uor-r4-integer/src/bin/fbs-toks.rs, crates/uor-r4-integer/src/bin/fbs-point-key.rs, crates/uor-r4-integer/src/wasm_serve.rs, crates/uor-r4-integer/src/wasm_bindgen_exports.rs, crates/uor-r4-float-train/src/in_space_long.rs, crates/uor-r4-float-train/src/bin/train-in-space.rs, docs/roadmap/items/point-key-integer.md, docs/harness/prep/point-key-integer*, docs/labs/point-key-integer/**, docs/loop/tasks/point-key-integer/**, docs/research/notebook/float-training.md]
---

## Why it matters

The project's first hurdle is one working model end to end: token → vector → vector → integer → token (`PLAN.md`). The integer table, search and `/8` record are built, and the point-key override serves the integer session's own opening. Claude-232's end-to-end greedy parity check then missed every cell, and the owner subsequently stopped the Q5 investigation. The original integer configuration is held.

- **claude-219 found the point key.** At a reply's opening where the turn memory's exact first-8 key misses, the trunk's opening point is named by its L1-nearest TRAIN openings.
- **claude-220 serves it inside the float model's decode** ([REPORT](../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md), merged as `c3d5cb684`, #3443). Its registered outcome is row 2: the served key carries the reply at all three seeds on the fresh 1,200-conversation panel p-w.
  - Templates against greedy: (50, 9), (44, 4), (47, 5), against bands of 15.4, 13.9 and 14.4.
  - Every served token is the measured key's name, with a distance drift of 0.0.
  - The negatives: the gain is recall of TRAIN replies, and it closes about 18% of the gap to a forced right first token.

The integer artifact cannot hold the float table, since no floating point is allowed in discrete-model computation (D32). claude-220's decision review set two conditions for its integer form:
- **(b)** its bar is measured with the integer trunk's own points, for both the query and the table, never with float points quantized after the fact;
- **(d)** the export needs its own item. This is that item.

## What is missing

**Disposition, 17 November 2026.** Claude-229 decoded and graded p-x once at seeds 1–3. The integer session used the native writer in place of the trained whole-table nearest readout. Both arms recorded zero discordant template and exact-reply pairs throughout. The lab recorded row 0, an instrument fault, and reported neither a mechanism verdict nor a qualified throughput number. [Original run and rerun rule](../../loop/tasks/point-key-integer/claude-229/REPORT.md)

Claude-232 then missed greedy parity in every tested cell. No three-seed p-x rerun with a repaired readout followed. The owner stopped the Q5 readout investigation (#3462), and its incomplete replay was archived. This configuration closed as not planned. Reopening requires an explicitly selected consumer and a registered parity repair for this exact artifact. The original sheet keeps p-x as the deciding panel and fixes the rerun bar before reading its outputs. Native replay and the written-state experiment continue under #3973 and #4063. [Parity comparison](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) · [Q5 stop and archive](../../loop/tasks/integer-nearest-readout/claude-257/REPORT.md)

**Completed units:** claude-222 (#3452) built the integer point table, its least-L1/commonest-target search and the `/8` record, with 380 library tests, 18 negative gates and a full serving audit. Claude-224 (#3454, [REPORT](../../loop/tasks/point-key-integer/claude-224/REPORT.md)) then built the panel prompts, integer decode and grade, with 383 integer-library tests, 185 float-library tests, a full serving audit and 13 negative gates. These are synthetic/host instruments, not a real TRAIN export or panel evaluation. Claude-227 completed [`integer-retention-rates`](integer-retention-rates.md) with GPU parity inside the 3% bar: 1,531 / 1,536 sequences and 4,023 / 4,096 single tokens. Three full snapshots are stored. Claude-228 built the run's reference/read-back/BOS instruments, with 390 integer-library tests with one ignored and four negative gates. Claude-229's real run found row 0, an instrument fault: the integer session uses the native writer instead of the trained whole-table nearest readout, so every grade pair is (0, 0) and no throughput is reported. [Claude-231](../../loop/tasks/integer-nearest-readout/claude-231/REPORT.md) built the nearest readout and export on native and browser paths; [claude-232](../../loop/tasks/integer-nearest-readout/claude-232/REPORT.md) then missed greedy parity in every cell, so the mismatch must be localized before the p-x rerun.

The original registration below preserves the table, search, artifact, instruments and untested integer bar. A revived run would first have to localize the parity mismatch and reproduce the trained readout.

- **A table built by the integer engine.**
  - It holds TRAIN's opening points at the writer query's representation (i16 Q12: 212,968 × 256 × 2 B, about 109 MB) and their targets.
  - It is taken with the memory off, by an export pass that runs the integer session over TRAIN's openings (`fbs-export --point-key`). It and the panel arms run on the bit-exact laptop engine, by the owner's allowance (below).
  - claude-220's float `opening-table.safetensors` is never quantized into it. Neither is any float snapshot, which is why the float exporter and snapshot files are outside this item's scope.
- **The served search in the integer session.**
  - **Where:** at `stack_step_fbs`'s first reply step, and its timed twin. Where the turn memory's opening read misses its exact key, the memory moves nothing, so `b.query_q12` is the trunk's own point. The search overrides the native token there.
  - **How:** the `-L1` scan of `stack_whole_table_nearest`, plus a second pass for the commonest target among all rows at the least distance, ties to the smaller id. That is float's `l1_named` rule.
  - **The miss record** (`moved`) is kept in every build, not only under test.
- **The artifact:** `uor-r4.lut-stack/8`, or a new optional record that older engines refuse, since `StackTurnMemory` denies unknown fields.
  - The new section is `memory.point.{points,targets}`.
  - The restore refusal covers its state.
- **Later, for speed, not for this item's bar: an exact integer evaluator on the GPU.**
  - The integer engine is CPU code, and `uor-r4-integer` has no CUDA path.
  - The owner wants it built ("eventually we want the performance boost that we get by building our custom CUDA drivers"). It would be a CUDA form of the integer session's step and the table search, built ahead with nvcc (never NVRTC), and bit-exact against the laptop engine on a fixed prompt set.
  - It needs its own item, scope and decision card. This item's done-when does not wait for it.
- **Optionally, the sign code** (6.8 MB, the Hamming form that `integer-hamming-read`'s popcount fits). It is reported only, since claude-219 read it too thin to tell.

## Done when

The model throughout is (r-drop)'s sealed step-16,384 checkpoints at seeds 1–3, the ones claude-217 to claude-220 scored, exported to the integer artifact.

1. **The integer table and search are built and unit-tested.**
   - The search names a missed opening by the commonest target at the least L1 distance, ties to the smaller id.
   - An opening the exact key fires on is left to the memory.
   - The table round-trips through the artifact byte for byte.
   - A restore with the table is refused until its state is saved.
2. **The instrument holds, exactly, on every panel run.** Any one of these failing is a fault, and the unit stops:
   - at every served opening, the integer search's name equals a test-side reference scan of the same integer query against the same integer table;
   - the table read back from the artifact equals the one built, byte for byte;
   - no served name is BOS.
3. **Parity against the float key, reported, deciding nothing.**
   - **What is compared:** on p-w, p-v and DEV at seeds 1–3, the integer served names against claude-220's float served names (its `samples-{split}-served-memory.json`), at every opening both serve.
   - **The classes, fixed here before any panel is read:**
     - **Within the rounding bound:** the integer table holds a row with the float's name within 512 Q12 units of the integer least distance.
       - Half a unit of rounding on each value of the query and of a row moves their L1 sum over 256 values by at most 256 units.
       - Two rows can move in opposite directions, so 512 bounds any order that rounding alone can flip.
       - The integer points come from the integer trunk itself (condition (b)), not from rounded float points. So this class says a difference lies inside the bound; it does not prove that rounding caused it.
     - **Trunk drift:** every other difference.
   - **Why it decides nothing:** the integer form's names differ from the float key's by design (claude-220's DECISION.md, form C). The agreement share and both classes are reported, beside the replies' agreement on the same prompts.
4. **The deciding bar: form C's own, on a fresh panel** (DECISION.md: "Its own bar on a fresh panel"), measured as condition (b) requires.
   - **The panel:** a new held-out-phrasing panel, p-x. It has 1,200 conversations, is drawn by `heldout-panel` at a seed no panel has used, and has its own census. Nothing reads it before the sheet registers the bar.
   - **The comparison:** the integer served arm against integer greedy, both with the memory on.
   - **The arm carries at a seed** when its templates gain b − c > 2·√(b + c), with b + c ≥ 10, and its exact replies, its loops and DEV's templates are each no worse beyond their bands. That is claude-220's `carries` and `harmless`.
   - **The rows:** the sheet registers them for the three seeds (carries at all three, at some, at none), with a fresh FIT and a pre-run review. A seed below the bar is reported under its row, and no panel is redrawn.
5. **Tokens per second first:** the integer decode against context on the laptop, with the table search counted, and its resident memory. Serving the exported artifact on a client is the product (CHARTER §3.6).
6. **Each result is one notebook row, and each REPORT gives the negatives first.**

## How to start

- **Read:**
  - claude-220's REPORT and DECISION.md (conditions (b) and (d));
  - `turn-memory-integer`'s item, and DeepSeek's integer turn memory (#3446, the `/7` turn memory in the integer session);
  - `crates/uor-r4-integer/src/stack/{memory.rs,session.rs,format.rs}` at the plug-in point.
- **Before claiming:** check that neither `integer-hamming-read` nor `fbs-serving-kernels` holds a claim on `session.rs`, and that no Studio worktree plans edits to `session.rs` or `wasm_serve.rs`.
- **Which run goes where:**
  - **The laptop's bit-exact integer engine:** everything that runs the integer artifact. That covers builds, the finite unit tests, the export pass over TRAIN, the panel arms, the scoring of done-when 3 and 4, and tok/s and resident memory.
  - **A pod, briefly:** turning (r-drop)'s checkpoints into snapshots. `fbs-snapshot` loads the float checkpoint on CUDA.
  - **No new run:** the float references are claude-220's sealed served files.
- **The owner's allowance (31 Oct 2026, in claude's chat).**
  - **The question:** CHARTER §3.6 keeps "panel scoring, qualification and milestone evaluation of an exported artifact" as GPU research compute, and the integer engine has no GPU path. Asked how this item's deciding panel is scored, the owner answered: "I want both. If the laptop bit-exact will allow us to proceed right now I think we should do that, but eventually we want the performance boost that we get by building our custom CUDA drivers".
  - **What it covers:** this item's scoring, on the bit-exact laptop engine, and nothing more. It is the owner's own answer, recorded here. `CHARTER.md` is an owner file, and its text is unchanged.
- **Settle in the sheet:** the export pass's cost over TRAIN's 212,968 openings, and how it is split across the laptop's cores.

## The units (31 October 2026)

1. **Done: the integer table, its search, the step, the `/8` record and the builder** (claude-222, #3452, merged as `f4beaaf79`). This is done-when 1, on the synthetic stack: 380 library tests, a full pass of the serving audit, and 18 negative gates.
2. **Done: the panel decode and the grade** (claude-224, #3454, merged as `18c4c3da7`), on the laptop with finite tests and no real-model run.
   - **The decode:** `fbs-toks --replies` decodes a panel's prompts with the memory on, under greedy, from a `/7` or `/8` artifact and writes claude-220's sample shape.
   - **The grade:** `train-in-space --integer-grade` reads the integer arms by their own tokens, compares served names with claude-220's float served files (done-when 3), and reads the bar of done-when 4.
   - **Checks:** 383 integer-library tests, 185 float-library tests, 13 negative gates and a full serving audit.
   - **Limit:** both integer arms stop at BOS, 64 reply tokens or the shared 192-token context. Long-prompt replies are capped identically, and the real grade must report that count.
3. **Done: the run's instruments** (claude-228, #3459, merged as `c71e85788`), done-when 2, on the laptop with finite tests and no real-model run.
   - The independent reference scan checks every served opening in `panel_reply`; `fbs-export --point-key` reads the table back as built; a BOS name is refused at the step.
   - **Checks:** 390 integer-library tests with one ignored, four negative gates and a full serving audit. These checks stop the run on mismatch.
   - [`integer-retention-rates`](integer-retention-rates.md) is complete (claude-226 and claude-227), so the integer artifact serves (r-drop)'s retained recurrence.
4. **Ran: the real run (claude-229), on the laptop: row 0, an instrument fault.**
   - (r-drop)'s full snapshots at seeds 1–3 are in the store at `results/claude/integer-retention-rates-parity-2026-10-31/out/full/`, so no pod is needed.
   - The steps ran under a fresh FIT and pre-run review: `fbs-export`, and `fbs-export --point-key` over TRAIN; the fresh panel p-x drawn with its census; the arms on p-x, p-w, p-v and DEV at seeds 1–3.
   - Done-when 2's instruments held (8,295 served openings), but integer greedy is not the trained model's decode: its first token equals claude-220's float greedy on 0 of 310 DEV prompts and 2 of 1,202 p-w prompts at seed 1.
   - The likely cause: (r-drop) is a cell-objective model, decoded as the whole table's nearest token, while the integer session serves the native writer.
   - Nothing was counted and no throughput is reported. p-x stays the deciding panel ([lab record](../../labs/point-key-integer/README.md)).
5. **Built: the trained model's nearest readout** (claude-231, #3463, merged as `62308fcbc`), on native and browser paths, with 395 integer-library tests with one ignored, seven negative gates and full serving/WASM audits.
6. **Parity MISS: localize the mismatch** (claude-232, #3464, merged as `4195ce2bf`). First-token agreement on DEV is 291, 285 and 284 of 310 at seeds 1–3; on p-w it is 911, 777 and 64 of 1,202. Every cell missed its bar, and whole-reply agreement remains incomplete. The trunk, Hamming read and readout are suspects, not located causes. The next step registered at the time was to localize the mismatch, recheck parity and rerun unit 4's p-x sheet. That rerun was still pending when Q5 stopped.

## Rules that apply

- **Serving stays integer** (CHARTER §3.6, as amended by D80 and D81): no float or transcendental instruction. D32 applies to the table and its search: integer only, with adds, subtracts, absolute values and compares.
- **Geometry only, no squashing** (§3.1–§3.2).
- **Deterministic exact or membership checks only,** with no neural graders (§3.10, §4.4).
- **A negative closes a configuration, never a mechanism** (§4.1). An integer rounding difference is classed before it is called anything.

## Sources

- [claude-220's REPORT](../../loop/tasks/one-model-end-to-end/claude-220/REPORT.md), [DECISION.md](../../loop/tasks/one-model-end-to-end/claude-220/DECISION.md) and its [lab page](../../labs/one-model-end-to-end/served-key-2026-10-31/README.md).
- [claude-219's REPORT](../../loop/tasks/one-model-end-to-end/claude-219/REPORT.md): the point key named in free generation.
- [turn-memory-integer](turn-memory-integer.md): the integer turn memory this builds on.
