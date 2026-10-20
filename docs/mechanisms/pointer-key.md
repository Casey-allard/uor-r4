# Pointer copy with an exact identity key

**Status: PARKED.** Every tested form is a softmax pointer inside a dense float model, and each missed its bar or cannot be served. A real prime-based key raised the pointer's hit rate from 0.377 to 0.545, yet the answers did not reach the pre-registered bar (one seed got 23 of 40 memory rows against a bar of 25, inside the noise). The version keyed by the slot's own name was never run, nor was the forced-copy test of it on a lawful pointer.

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## In plain words

When asked "what is Ana's locker?", the model should copy the stored answer from earlier text rather than invent one. A pointer is a small part of the model that chooses which earlier word to copy. This mechanism wants the pointer to choose by the exact name of the slot (Ana plus locker), not by the surrounding wording. Measured on 9 October on 13 rows, the pointer instead lands on the repeated sentence frame ("... locker is ..."), and almost never on the changing digits.

## What it replaces in a transformer

The copy head, also called an induction head. In a transformer, a head learns to look back at an earlier word and copy it. Here the choice of which word to copy is steered by prime-number addresses, so it does not depend on how similar two stretches of text look.

![A short passage "My locker is 47 ... What is my locker?" with a bar above every token showing the pointer's attention. One tall bar, 0.92, sits on a bare space token in the repeated frame and almost nothing on the digits 4 and 7. Beside it, the same passage with all attention forced onto the two digits. Below, two dials from the three seeds of one test: hit rate rising from 0.377 to 0.5452 while the memory score stays at 19, 21 and 23 of 40, against a control of 20.](../figures/mechanisms/pointer-key.svg)

*Left, where the pointer actually looks, and where the forced-copy test would put it. Right, a higher hit rate did not raise the memory score.*

## How it works

**The copy mixture.** The model's next-word distribution blends its ordinary scores with a copy distribution:

$$p(v)=(1-g)\,\mathrm{softmax}(z)_v+g\sum_{j:\,x_j=v}a_j ,$$

where $g$ is a learned gate, $x_j$ is the word at earlier position $j$, and $a_j$ is the pointer's attention over a window of earlier words. The first term uses a softmax, which matters for the rule check below.

**The project-specific part is how a source is chosen.**

- Every word has a registered prime. A source $j$ is admitted when the greatest common divisor of its atoms and the query's atoms exceeds 1:
$$\gcd(q,\pi_j)>1 ,$$
  or when an ordered n-let (a short ordered run of atoms) matches. These are `RouteAdmission::SharedAtom` and `Ngram`.
- Rarer atoms weigh more, by $\ln\gcd$. The route score is scaled by 4.0 (`ROUTE_SHARPNESS`), and the attention is a softmax over admitted sources.
- An identity term (`PointerIdentity`) adds the overlap of the six words before the source with the six before the query.
- A fixed table folds upper and lower case and one leading space to a canonical id ([`copy_identity.rs`](../../crates/uor-r4-training/src/copy_identity.rs)). It only decides matches and never changes the emitted word.

**The intended key** is the slot address: entity prime times attribute prime, written when the fact is stated. Not the wording around the value. That version was never built.

**Rule check.**

| Rule | Verdict |
|---|---|
| No softmax (D23) | Broken as built. Both the mixture and the pointer attention use softmax. The form is a softmax pointer and could not be served (project history, era 28) |
| No matrix multiplication (D30) | Not clean. The pointer sits inside a dense float model and its score is a dot product |
| Integer-only serving (D11) | Export serves only an unrouted, unselected pointer. A pointer with an identity term is refused at export. The fold table is one table read plus integer equality |
| Discrete path (D32) and offline float (D37) | No pointer exists on the discrete path. [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) would allow an offline float re-registration in a separately scanned module, with an integer export and a reported gap. Nothing is released |

## What has been tried

"Memory" is the memory half of a frozen 40-row chat panel. "Wrong-value" counts rows that gave a stated but wrong value. All 2026-10 runs fine-tune one 29M-parameter chat model.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| Sep (undated) | Earlier native engine: ordered occurrence correspondence between query and source words | Passed its gate; all 6,304 earlier traces preserved. The ancestor of the pointer. Authored panels | positive | [record](../../archive/native_geometric_correspondence_973.md) |
| 21 to 23 Sep | Typed session runners and a learned span-and-role decoder over exact owned spans | 128 of 128 primary answers; 24 of 24 final answers; 32 of 32 authored preflight. The two geometry arms tie, so the geometry is not credited | positive | [session](../../archive/integration/relational-session-result-2026-09-21.md), [argument binding](../../archive/integration/ordinary-form-argument-binding-result-2026-09-22.md), [lexical](../../archive/integration/ordinary-lexical-result-2026-09-23.md) |
| 1 Oct | Key-value recall at distances 16, 64, 200 with pointer arms | Without a pointer 1 of 109. Pointer arms scored 0.28 to 0.44, falling about as 1/N, matching an untrained "copy the latest" rule (0.36). Their misses are wrong-key answers | negative | [record](../../archive/integration/log-sieve-retrieval-design-2026-10-01.md) |
| 1 Oct | Native integer attention with capture-credit models against same-size controls | 128 of 128 answers for the capture-credit models. The controls fail. Authored key-value tasks only | positive | [record](../../archive/integration/geometric-attention-latch-2026-10-01.md) |
| 3 Oct | Zero-update audit of the final native artifact's copy targets | 54 of 74 copy targets lose to another copy action; 4 of 20 complete answers. Source boundary missing 60 of 114 target tokens | negative | [record](../../archive/integration/geometric-final-prefix-audit-2026-10-03.md) |
| 9 Oct | Diagnostic series on the 29M chat model's pointer over 13 numeric rows | The pointer attends the repeated frame, a bare space token (attention 0.9202 and 0.9064), and rarely the digits. Second digit selected on 0 of 13 rows, first digit on 3 of 13. Failing rows: 10 reader faults, 2 emitter faults, 1 partial | positive (diagnosis) | [record](../../archive/labs/pointer-selected-ids-decode-2026-10-09/README.md) |
| 9 Oct | Serving-time knobs: top-2 selection plus a gate floor of 0.9, weights unchanged | Both digits selected on 0 of 13 (control) to 3 of 13, target 13 of 13. Missed | negative | [record](../../archive/labs/inference-path-fix-result-2026-10-09/README.md) |
| 9 Oct | Token-identity copy stop on an 84-cell never-seen-value panel | 72 of 84 exact (0.8571; Wilson interval 0.7667 to 0.9163). The rest cannot be fixed by relaxing the rule, since the opened span emits the whole premise sentence | diagnostic stop | [record](../../archive/labs/token-identity-stop-open-population-2026-10-09/README.md) |
| 9 Oct | One-shot declared test of the 29M copy-pointer stack, no memory operator | 10 of 40 memory against a target of at least 34. Missed by 24 rows. A constant beat it on the "I do not know" half | negative | [record](../../archive/labs/v5-declared-acceptance-2026-10-09/README.md) |
| 10 Oct | Gate supervision, 55% recall data mix, one seed per arm | Memory 10 to 21 of 40, but an unsupervised control reached 22 of 40. The data mix, not the gate, caused it | negative | [record](../labs/pointer-gate-finetune-2026-10-10/README.md) |
| 10 Oct | Identity term keyed on the six words around the value | Wrong-value 17 to 12 of 40, memory 19 of 40 against an anchor of 20. The key was the context before the value, not the slot's name. Hit rate 0.003 | negative | [record](../../archive/labs/pointer-identity-2026-10-10/README.md) |
| 10 Oct | **Prime-ranked one-atom route** (the entity-address key), 3 data orders against a learned-key control | Hit rate 0.377 to 0.5452 at reachability 0.668. Memory 19, 21, 23 of 40 against 20. Wrong-value 17, 15, 9 against 17. Bits per byte 1.17387, 1.16947, 1.17109 against the anchor's 1.17425. The pre-registered bar (wrong-value at most 12 and memory at least 25 of 40 on all seeds) was not met | negative | [record](../../archive/labs/ptr-addr-2026-10-10/README.md) |
| 10 Oct | Recall-mixture doses of 10, 25 and 55 percent | 10 percent doubled memory without hurting replies; "I do not know" needed 25 percent and cost replies. Missed the keep bar by one row | negative | [record](../../archive/labs/mixture-dose-2026-10-10/README.md) |
| 11 Oct | Four serving-time tests on one model (23 of 40 normal) | Copy head off costs one row (22 of 40). Facts removed 0. Copy forced to half 7. The answers come from the attention read, not the copy head | positive | [record](../../archive/labs/answer-path-2026-10-11/README.md) |
| 12 Oct | Forced-copy oracle: replace the pointer's attention with a uniform one over a supplied answer span | Instrument built and unit-tested (919 library tests pass). The run on the bound model was not done | blocked | [record](../../archive/labs/copy-span-oracle-2026-10-12/README.md) |

**Reading these honestly.** The key did what a key should: it found the source more often, at no cost in likelihood. The answers did not follow. The record suggests why: the pointer branch carries only about 6 percent of the output mixture, and the answer-path test shows the answers come from the attention read. So the hit-rate measurements read a minor branch, and the bottleneck is after the source is found (hypothesis). The failures are specific. The identity runs keyed on the wrong thing. The gate run had a control that did as well. The v5 run had no memory operator at all. The inference knobs changed two decode settings on fixed weights. None of them tested a key on the slot's name, and the project's own rule says a negative closes a configuration, not a mechanism ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)).

## What has never been tested

- The forced-copy oracle: supply the true slot or span and see whether the answers follow. It has been named and left unrun since 12 October. It must not be run as a new execution of the softmax pointer artifact (D23, D30): it needs a lawful pointer, or stored outputs.
- A pointer keyed on the slot's name (entity prime times attribute prime), as opposed to the six words around the value or a one-atom window.
- Admitting a whole value span, such as the full two-digit run, instead of one token.
- Exact admission with a learned ranking over addresses.
- Any pointer on the discrete path, or with a copy distribution that has no softmax.
- A pointer served as integers by gcd, with the export gap measured.
- At least 3 seeds with at least 120 memory rows. Every pointer run was one fine-tune of one 29M model on a 40-row panel (the address-key test used three data orders).

## What would reopen it

1. **Run the forced-copy oracle on a lawful pointer.** Supply the true slot address or span to bound the possible gain, using a softmax-free, matmul-free pointer or stored outputs only. A new execution of the existing softmax pointer artifact is not allowed (D23, D30; see the [mechanism board](./README.md)).
2. **Then train a slot-address key.** A parsed or learned key (entity prime times attribute prime), trained in from the start of the dialogue fine-tune, with matched controls, at least 3 seeds and at least 120 memory rows. The bar is a gain of at least 5 rows of 40 over the control.
3. **Remove the softmax.** Give the copy distribution a rank table, a hard select, or exact admission plus a learned ranking, so the head can be registered under the no-softmax and no-matrix-multiplication rules. That is a new offline float sheet (D37) or a discrete sheet (D32), with a fresh `TEST FITNESS: FIT` review and an owner release.
4. **Serve it as integers.** Route by gcd or n-let equality with no float score, and report the export gap.

## Also known as

Pointer-copy head (A1); copy head; token-identity pointer (`pointer_identity`); prime-route pointer (`pointer_route=prime-ranked`); entity-address pointer; copy-identity fold table; forced-copy oracle (copy-span oracle).

## Code

- [`geometric_stack.rs`](../../crates/uor-r4-training/src/geometric_stack.rs): `PointerConfig`, `PointerSelect::TopK`, `PointerIdentity`, `PrimeRoute`, the gate loss and the copy-span oracle switch.
- [`copy_identity.rs`](../../crates/uor-r4-training/src/copy_identity.rs): the fixed fold table.
- [`stack_prime_route.rs`](../../crates/uor-r4-training/src/stack_prime_route.rs): the prime-route addressing instrument, shared with exact memory.
- [`examples/bind-probe-exact.rs`](../../crates/uor-r4-training/examples/bind-probe-exact.rs): the exact-recall probe behind the 84-cell panel. [`bin/heldout-memory-panel.rs`](../../crates/uor-r4-training/src/bin/heldout-memory-panel.rs): the held-out memory panel builder.
- [`word_copy_runtime.rs`](../../crates/uor-r4-core/src/native_geometric/word_copy_runtime.rs): the September engine's exact word-occurrence copy, the pointer's ancestor.
- Evidence: [`attention-state-2026-10-18`](../labs/attention-state-2026-10-18/README.md), [`kimi-direction-2026-10-12`](../../archive/labs/kimi-direction-2026-10-12/README.md).

## Related mechanisms

[exact-memory](./exact-memory.md): the store whose slot address the pointer is meant to use as its key. [read-binding](./read-binding.md): steers the attention read toward the bound value; the answer path turned out to be that read. [route-read](./route-read.md): a different way to read by address, with no copy mixture. [relation-read](./relation-read.md): its copy-successor capability is the pointer's analogue on the discrete path. [prime-zeta-codebook](./prime-zeta-codebook.md): the prime registry behind gcd and n-let admission. [tokenizer](./tokenizer.md): the 4,096-token vocabulary has no pure multi-digit tokens, so a two-digit value is two tokens, which interacts with copying.
