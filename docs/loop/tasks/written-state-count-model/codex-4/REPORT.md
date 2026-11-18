# [codex] Content-based entity selection through the frozen exact writer

## What did not work, first

The learned query-context score missed all three registered bars on both seeds. On held-out prompts it bound both roles in 18/144 matched pairs. The frozen offset selector bound 26/144 and switching off the new query score bound 24/144. Individual correct starts improved by six over the offset selector, but the new rule lost on the role reversals. Only 52/288 first errors moved later than offset (bar 145); 188 tied and 48 moved earlier. No arm, including the oracle, produced the complete value followed immediately by EOS. This closes this selector configuration. The wider P3 and conversational model gates remain open.

## What changed

A separate integer state captures requested, excluded and earlier fact contexts from annotated TRAIN on CUDA. Serving compares the local packed E8 root contexts of earlier/later occurrences that share an exact token ID. The winning fact position anchors a second comparison against the learned ordered tokens between fact and value. Only the resulting position goes to the #4086 writer at the first answer step. Gold span length and target IDs are unavailable to both learned paths. Each arm consumes its own emitted IDs. The prototype state is 92 u32 (368 bytes of bank payload including padding; 939 bytes for the trained JSON). Training yielded 6 requested, 6 excluded, 7 fact contexts and 1 relation. The trained banks are identical across the two released seeds; the state envelopes and native continuations are seed-specific.

## Measurements

First wrong is a zero-based index against the requested value plus EOS. Both seeds have the following counts on the same 288 held-out texts. Half the targets consist of one value token; half consist of three.

| Source supplied to writer | Correct value start | Both roles correct / 144 | Correct first ID | Whole value | First wrong index 0 / 1 / 2 / 3 |
|---|---:|---:|---:|---:|---:|
| Content selector | 74 | 18 | 74 | 74 | 214 / 74 / 0 / 0 |
| Query score disabled | 96 | 24 | 96 | 96 | 192 / 96 / 0 / 0 |
| Frozen offset selector | 68 | 26 | 70 | 68 | 218 / 40 / 2 / 28 |
| Oracle start | 288 | 144 | 288 | 288 | 0 / 144 / 0 / 144 |
| Copy disabled | 0 enabled | 0 | 0 | 0 | 288 / 0 / 0 / 0 |

All 74 content successes copied the full value, then free generation emitted period (16) where the target was EOS (1). Oracle showed the same release error on all 288. Every paired improvement over offset moved 0→1. The 48 regressions moved 1→0 (18), 2→0 (2) or 3→0 (28). All 214 held-out source misses selected a different query token position than the annotated terminal name piece. Of these 44 chose the excluded mention and distractor value. The other 170 selected neither target source. Among those 170, 119 chose a leading name piece, 49 the token for “ took” and two the opening piece of “ silver”. On 73 rows the selected query and fact were both the **correct person's first piece**, two IDs before the annotated terminal anchors. The value address then landed inside the name. Every correct terminal fact selection reached its correct value in this split.

| Held-out factor | Both-correct matched pairs, content arm |
|---|---|
| Name early / late within question | 18/72 / 0/72 |
| First / reversed written fact order | 9/72 / 9/72 |
| Exclusion absent / before / after question | 12/48 / 0/48 / 6/48 |
| No / short / long trailing text | 6/48 / 6/48 / 6/48 |
| Kim–Mia / Ada–Noah lexical families | 18/72 / 0/72 |

The worst marginal levels were below the registered 50% pair bar. Single-ID Kim/Mia supplied all 74 successes; all 144 Ada/Noah rows failed. The second lexical family changes values and names together. The first-piece traces localize a failure on those specific rows; the family contrast alone cannot isolate tokenization as the cause. The question-position intervention also changes wording. An exact-ID/same-context categorical comparator would be required to isolate the value of the E8 metric.

**Two held-out traces.** In the Kim prompt with late exclusion and a long trailer, the trained choice (query 15, fact 4, source 6) produced [1074,16,1] for [1074,1]. Offset began with comma (14). In the matched family with Mia requested and Kim excluded before the question, offset produced [1730,16,1]. Content selected the excluded Kim at 14 instead of the requested Mia at 21 and produced [1074,16,1]. A correct opening became an immediately wrong value.

**Original regression.** Content still reached 0/16 correct sources at either seed. First wrong stayed at index 0 on 15; on one shared-red example it moved to index 1 because the selector copied the other value: target [677,1710,1] (“ red fox”), output [677,1999,3,1] (“ red kite!”). On nine original rows the content selector found both correct name occurrences, then handed the writer the final “ose” piece of “ chose” one token early. On the remaining seven it already had the wrong query anchor. Each TRAIN and held-out fact had a one-token connector and required fact+2. The original grammar requires fact+3. Source-offset transfer therefore failed even when the two mentions were correct.

## Checks run at the head

The shielded inputs, finite tests, raw state identities and controls are in [FIT](FIT.md), [pre-run review](PRE-RUN.md), [panel design](PANEL.md) and [selector receipts](selector/outputs.sha256). The Rust and SM89 selector agreed on all 1,216 blind triples. Across the five writer arms the Rust serving path and the AOT device agreed on 9,640 and 9,622 held-out transitions, plus 497 and 484 original transitions. The writer cubin remained b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28. All four outputs were byte-identical on independent process replay. Against #4086 all 96 original control trajectories and all 665 steps matched. Training and blind prediction took 54 seconds altogether; one five-arm held-out rollout took 3.09 s on seed 1 and 3.08 s on seed 2, including CPU differential checks. These timings describe the bounded research harness.

After review, the runner binds both trained states to their frozen digests. The tokenizer generator rejects empty or invalid original spans, and the predictor checks its row bound before padding. The revised generator reproduced all six input arrays byte for byte. Both malformed original cases and a modified state were rejected. The rebuilt selector and scorer reproduced all eight blind prediction files and all four complete rollouts byte for byte without refitting. [Verification digests](verification/verified.sha256)

The private store bundle contains generated input arrays, both fits and blind predictions, raw closed-loop steps, independent repeats, binary identities and run logs. The local download of the pinned bundle matched SHA-256 50c70fcedd914d850a450abb320532480875ebdcbc798dd07702abc50a723921. [Immutable Wick bundle](https://huggingface.co/datasets/caseyallard/uor-r4-store/blob/3587647de1f11436e01135f003752be8b5e2135c/results/codex/content-selector-20261118/content-selector-wick.tar.gz) · [Raw writer digests](WRITER-SHA256SUMS) · [Seed 1 held-out case traces](rollouts/seed1-held-cases.tsv) · [Seed 2 held-out case traces](rollouts/seed2-held-cases.tsv)

## Skills used

The native-space cycle governed seed binding, training state, same-prefix serving replay and serialization. The tokenizer-only corpus census and a non-author FIT preceded empirical GPU work.

## Still open

The most discriminating next experiment is to carry both ends of the matched entity mention, then learn where the relation ends from facts with one-, two- and three-token connectors. Cross names sharing initial pieces, a verb/value reused elsewhere in the prompt, and exclusion position. Grade requested role, entity endpoint, value start and first emitted token separately. The common-prefix and repeated-verb failures can survive a span-length fix. After an oracle address, the observed next error is always the native period on these held-out prompts, so immediate-EOS behavior requires its own control.

## Plain summary

The exact writer copied every correctly addressed value. The learned selector mixed up requested and excluded mentions, sometimes matched the first piece of the right name, and learned only one connector length. The released generator then added a period after every correctly copied held-out value. These are three separately measurable problems.
