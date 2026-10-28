# Exact addressed memory

**Status: PARKED.** Every trained form of this idea that was tested missed its bar or broke a project rule. The plain exact store itself worked whenever its address was right. The form that matters most, where the address is the model's read, was never built. See [what would reopen it](#what-would-reopen-it).

Labels used below: **proof** means derived from definitions or code, **measurement** means a recorded run, **hypothesis** means an untested idea. See [`docs/formal_vocabulary.md`](../formal_vocabulary.md).

## Stage 0 measurement (2026-10-26)

The T16_fz Seed 1 D11 artifact passed exact parity on all 14,287 DEV rows against the sealed Stage T GPU receipt, including prediction, top-8 and admitted count, with byte-identical repeat records and zero aggregate position-strip delta. That validates the integer instrument, not the address-as-read mechanism.

The same artifact failed the registered copy non-vacuity gate: exact-hint free-running recovered **0/40** exact values, so exact-minus-control had `b = c = 0` against both no-hint and random-hint. It did not meet either `d > 24 && d^2 > 4(b+c)` comparison. A hidden-fact panel is therefore not justified on this artifact/prompt layout. This is a scoped artifact/copy-instrument limitation, not a mechanism negative and not a model-oracle result. The unit is `References #2833`; BPB and training were not run. See [`docs/loop/tasks/m1-address-as-read/kimi-40/REPORT.md`](../loop/tasks/m1-address-as-read/kimi-40/REPORT.md).

## Object-store oracle measurement (2026-10-26)

The registered five-arm object-store oracle passed at `590ace9e`: true address recovered 40/40 current, 20/20 previous and 20/20 typed abstention with zero substitutions; no-memory recovered 0/80; deterministic sieve recovered 20/80. Sieve formed the true query on 60/80 rows but recovered only 20 values. True address beat no-memory on every family and beat sieve by 20/0 current and 20/0 abstention; previous sieve is descriptive current-value fallback and excluded from the contrast. D11 parity remained exact on 14,287/14,287 rows with byte-identical repeats.

This is an exact-read/oracle gain at the item's tested scope, not learned address formation, default serving or M3 durable-memory qualification. The mechanism stays **PARKED**. The next question is address formation on the 20 current rows the sieve misses, not storage. Evidence: [`kimi-42 REPORT`](../loop/tasks/m1-address-as-read/kimi-42/REPORT.md) and HF `results/kimi/m1-address-oracle-2026-10-26/results-v3/address-oracle-result-v3.tar.gz`.

## In plain words

Write each fact once, under an exact address, and read it back exactly. Do not smear it across weights, and do not hunt for it by fuzzy similarity. The address is built from prime numbers: every word has its own prime, so the same words always name the same slot. A new value replaces the old one, but the old one stays recoverable. A fact that was never written can answer "I do not know". In the one probe that checked, the store was never the cause of a failure: among 1,905 held-out misses, none was an eviction or a wrong stored value. What failed was the model's learned step of forming the address from a question.

## What it replaces in a transformer

The key-value memory. A transformer keeps a key and a value for every earlier word and finds the right one by comparing the question against all the keys. Here the question is turned into an address instead, and the address is looked up directly. No comparison against every key is needed.

![A conversation scrolling past a row of numbered slots. Each stated fact writes a tile labelled with a product of two primes, and a correction stacks a new version on the old tile. A question lights the matching tile and its latest version. An unwritten key shows Absent and an overwritten slot shows Evicted. Beside it, a bar chart of one test: perfect when the address is right, near zero on new phrasing, perfect again when the address is supplied.](../figures/mechanisms/exact-memory.svg)

*Left, how a fact is written, corrected and read, with the two honest failure answers. Right, why the tested models failed: the store was fine, the address was not formed.*

## How it works

Three built forms share one idea. In each, the address is exact.

**1. Exact log and prime sieve (measurement on fixtures; the gcd arithmetic itself is proof).** Every user turn is logged as word atoms. Each word has a prime from a registry table. A clause's address is the product $A_c=\prod_{p\in S_c}p$ over its set $S_c$ of distinct content primes, plus its ordered pairs of neighbours (n-lets). A query $q$ is built the same way. A clause is admitted when

$$\gcd(q, A_c) > 1,$$

that is, when they share at least one prime. The greatest common divisor is computed as a sorted merge of the two factor lists, so no division is needed. Ranking is a separate step: most shared primes first, then latest occurrence, then ordered-pair match. A record counts as evicted only when its ring slot was actually overwritten.

**2. The exact relation store (AERM, "G v1").** Learned heads tag each token as entity, relation, value or other. An integer store maps (entity, relation) to a value. It also keeps the previous distinct value and a version counter. A read answers one of four ways for the current and the previous version: `None`, `Hit`, `Absent` or `Evicted`. A prime-route variant keys by the semiprime $k=p_a p_b$ of two atoms. By unique factorization (proof), $k$ names the unordered pair exactly. The value is routed through the expert so that direction survives.

**3. Product-key memory.** The query splits into two halves. Each half is scored against $S$ sub-keys and the top $k$ on each side are kept. That gives $k^2$ candidate slots, scored by the sum $s_1(i)+s_2(j)$. The best $k$ slots are read from an $S\times S$ table. With $S=256$ the table has 65,536 rows and a token reads 32 of them. The sub-keys are learned, or fixed to the 120 unit icosians or the 240 E8 roots. The built form mixes the rows with softmax weights.

**Rule check.**

| Rule | Verdict for the exact log, sieve and store | Verdict for the tested trained forms |
|---|---|---|
| No softmax anywhere (D23) | Clean | Broken. The product-key form mixes rows with softmax, and the tag and trigger heads trained with cross-entropy |
| No matrix multiplication (D30) | Clean | Not clean. They sit inside a dense float stack |
| Integer-only serving (D11) | Fits by design, but nothing is built. The integer serving crate has no gcd, and it refuses product-key memory | No served form |
| Discrete path (D32) and offline float (D37) | The integer store matches the discrete shape. No memory store exists in the discrete crate yet | A learned tagger would be float with an integer export and a reported gap. No hold is released |

[D23](../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project) bans softmax. [D11](../integration/DECISIONS.md#d11--native-multiplier-free-transformerless-serving-for-every-lab-d10-superseded) fixes the served form as integer and table lookups. [D37](../integration/DECISIONS.md#d37--floating-point-is-legal-in-offline-training-only-d32s-learning-path-float-ban-is-revised) allows float in offline training only. D30 (no matrix multiplication) is in the same file.

## What has been tried

Verdicts are this page's labels. "Authored panel" means a small test set written by the project, not natural language.

| Date | What was run | Result | Verdict | Record |
|---|---|---|---|---|
| 26 Aug | Prime-route mechanics with no model: prime routes, ordered n-lets, gcd overlap, 7-row query closure | Mechanics pass. The product probe was not run, because no compiler from prompt tokens to addresses existed. 32 of 32 canary matches | inconclusive | [record](../../archive/prime_route_attention_qualification_958.md) |
| Sep (undated) | Learned selector over exact versioned records (initial, previous, current), authored panels | 976 of 976 construction and fresh targets on the reassertion repair. Owner revision 60 of 60, against 24 of 60 for its parent. Authored-panel scope only | positive | [record](../../archive/native_geometric_reassertion_chain_repair_973.md) |
| Sep (undated) | Learned read of the previous version, fresh text gate. Also a 64-update addressed-attention pilot (13 Sep) | Selected exact old records but 44 of 64 complete, so not promoted. The pilot gave no useful generated answers | negative | [record](../../archive/native_geometric_historical_selection_973.md) |
| 24 Sep | Synthetic key-value overwrite recall (KVAR): overwrite store with hard-select readout | Held-out accuracy 0.8235 and 0.7157 on two seeds. Recurrence-only 0.0098, gate-only 0.0196. A hand-written overwrite table scores 1.0000 | positive | [record](../../archive/integration/kvar-recall-result-2026-09-24.md) |
| 24 Sep | Exact previous-and-current token-pair cache on 24 project documents | Lowers a count reference by 0.2376 bits per target (95% interval −0.2928 to −0.1907) over 87,564 positions. Not general prose | positive | [record](../../archive/integration/realtext-sparse-recall-result-2026-09-24.md) |
| 28 Sep | AERM probe: 1.41M-parameter model, exact integer store, learned read trigger, 3 seeds, new templates and names | The store scored 1.000 in distribution, but the frozen margin gate failed even there (0.146, 0.122 and 0.102 against 0.30, because the control reached 0.85 to 0.90). On new phrasing "Updated" accuracy was 0.000, 0.000 and 0.015 against a control at 0.39 to 0.47. The trigger fired on 0 of 826 gold reads in seeds 1 and 2 (158 of 826 in seed 3) | negative | [record](../../archive/integration/d2-aerm-probe-result-2026-09-28.md) |
| 28 Sep | G v1: always read the current and previous register | Frozen gate failed in all three seeds: 0.1575, 0.0733 and 0.0293 against a 0.90 bar. Text loss and parameter parity passed | negative | [record](../../archive/integration/g1-always-on-read-result-2026-09-28.md) |
| 29 Sep | Failure attribution on the G v1 misses, no training | Of 1,905 failures, 1,480 (77.7%) were the query entity tagged wrongly, 242 (12.7%) the write key, 142 (7.5%) the query relation. Eviction: 0. Wrong stored value: 0 | positive | [record](../../archive/integration/g1-decomposition-result-2026-09-29.md) |
| 29 Sep | Mask the tag and trigger losses on text (arms A, B, C) | Arm A lifted held-out binding to 0.967 (264 of 273) but text loss rose 0.1188 against a 0.05 limit. Arm C reached 0.553 | inconclusive / negative | [A, B](../../archive/integration/g-binding-result-2026-09-29.md), [C](../../archive/integration/arm-c-result-2026-09-29.md) |
| 1 Oct | Exact log and sieve feeding an emitter, key-value recall at distances 16, 64, 200 | 106 of 109. With recall switched off 16 of 109. One seed, one development draw. The recall dispatch was chosen by the instrument category and earlier assistant replies were supplied as references, so this is not a closed-loop conversation result. Same-relation-in-new-words: sieve alone answered 1 of 52 | positive / inconclusive | [record](../../archive/integration/grounded-memory-evidence-2026-10-01.md) |
| 10 Oct | Product-key memory (64 sub-keys, top 16) at layer 4 of the 29M chat model, 2 seeds | 19 of 40 on both seeds against a 27 of 40 bar. No-memory anchor 20 of 40 | negative | [record](../../archive/labs/addressed-memory-2026-10-10/README.md) |
| 10 Oct | Saved model on fresh unexposed role and value combinations | 437 of 512 on development, 0 of 128 on fresh. Recorded as panel fitting | negative | [record](../../archive/labs/m2-fresh-437-2026-10-10/README.md) |
| 11 Oct | Product-key memory replacing the layer-8 feed-forward block, 3 seeds. Fat and lean codebook | Fat: wrong-value failures 47 to 34 of 120, memory flat at 57 of 120, 20 times slower per step, and 1.65 times slower generation in a separate float GPU test ([record](../../archive/labs/tokens-per-second-2026-10-11/README.md)). Lean: bits per byte worse by 0.0949 and 0.1066 against 0.02 allowed | inconclusive / negative | [fat](../../archive/labs/addressed-read-2026-10-11/README.md), [lean](../../archive/labs/addressed-read-lean-2026-10-11/README.md) |
| 11 Oct | Serve product-key memory on the integer path | The integer path refuses it, so speed is unmeasured where it matters. The earlier "12 times" fused-kernel claim was a fat-versus-lean codebook difference and is withdrawn ([withdrawal](../../archive/labs/fused-memory-kernel-2026-10-11/README.md)) | blocked | [record](../../archive/labs/native-address-port-2026-10-11/README.md) |

**Reading these honestly.** The pattern is consistent. Where the address was given or written by a fixed rule, the store worked: the 1.000 in distribution, the zero eviction failures, and the 106 of 109 recall. Where a learned head had to form the address from new phrasing, it failed, and the failures sit in the tagging, not the storage. The product-key runs tested something else: they replaced a feed-forward block with a table, not the read. The project's own correction note says so, so they do not test "address instead of search" at all. The recorded failures belong to *configurations* (a 1.4M-parameter probe with a learned trigger; 2,000-step fine-tunes of one 29M model on a 40-row panel with softmax weights). They do not close the mechanism ([D22](../integration/DECISIONS.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)).

A caution on weight. Retrieval caused only 2.2 percent of the failing rows in the era-22 chat panel (issue #1708), so it is open whether memory is the binding constraint on the first milestone at all (hypothesis).

## What has never been tested

- The address as the model's actual read, with the store and read trained in from the start (the order D22 section 5(c) asks for; [full text](../../archive/integration/DECISIONS-2026-10-19.md#d22--a-negative-closes-a-configuration-never-a-mechanism-five-closures-reopened)). Every "addressed read" run replaced a feed-forward block instead.
- A true gcd sieve on the integer path. As of 1 October only a pairwise membership scan had been built.
- Same relation in different words, using a learned channel that feeds the exact channel, at the 0.9 relation-accuracy gate.
- Ordered n-let keys for atoms that are not role-disjoint.
- Eviction proofs driving "I do not know" at long context in a trained model.
- Parameters actually touched per token, for any exact memory on the served path.
- E8 or icosian codebooks as the write address of facts, with prime semiprimes as the exact check.

## What would reopen it

1. **Build the address as the read.** Add memory fields to the dialogue settings and train the store and read in from the start, with 2 to 3 seeds and a frozen panel of at least 80 rows. The brief's bar is at least 8 more rows of 40 than the 19 of 40 seen, with bits per byte within 0.01.
2. **Serving-time oracle is complete at the T16_fz scope.** The registered five-arm result is above. The next measurement is address formation on the 20 current sieve misses, not another storage run.
3. **Re-register lawfully.** Use an offline float sheet (D37) or a discrete sheet (D32) with a softmax-free value mix (a rank table or a hard select), a separately scanned module, an integer export and a reported export gap. This needs a fresh `TEST FITNESS: FIT` review and an owner release.
4. **Build an integer read kernel**: gcd or sorted-merge admission plus a table value read, in the integer serving crate.
5. **Report address formation apart from read accuracy**, since 77.7 percent of the held-out failures were query-entity tagging.

## Also known as

Exact addressed memory; prime-keyed exact log and gcd sieve (log-sieve retrieval); semiprime expert or prime-route store; AERM exact relation store ("D2", "G v1"); product-key memory (learned or E8/H4 sub-keys); exact-pair cache; KVAR overwrite memory; "address instead of search".

## Code

- [`address_read.rs`](../../crates/uor-r4-object-store/src/address_read.rs): exact prime/semiprime address directory (`PrimeAddress`, `AddressDirectory`, `ObjectTarget`, `AddressReadOutcome`) and sorted-merge GCD sieve consumer over `uor-r4-object-store`, with finite CPU controls in [`tests/address_read.rs`](../../crates/uor-r4-object-store/tests/address_read.rs) and the final serving-time oracle sheet [`m1-address-as-read-oracle-2026-10-26.md`](../harness/prep/m1-address-as-read-oracle-2026-10-26.md).
- [`manifold_sketch.rs`](../../crates/uor-r4-object-store/src/manifold_sketch.rs): 4,096-bit multiplier-free ARX hyperdimensional sketch (`Sketch4096`), reversible `i16` bipolar accumulator (`SketchAccumulator`), hierarchical parent-scope blending (`ScopedAddressDirectory`), two-stage hybrid exact-sieve + sketch candidate fallback (`read_hybrid_sieve`, $+3\sigma$ admission gate `>= 2144` bits), and integer epistemic hazard gating (`HaloHeader`, `EpistemicState`), with L-010 paired controls in [`tests/manifold_sketch.rs`](../../crates/uor-r4-object-store/tests/manifold_sketch.rs).
- [`m6_ablation.rs`](../../crates/uor-r4-integer/src/m6_ablation.rs): integer prime-addressed memory (`PrimeFactorKey`, `IntegerAddressedMemory`) with sorted-merge GCD sieve admission, `[initial, previous, current]` versioning, eviction proofs, and the four-arm M6 causal intermediate-state ablation instrument ([lab record](../labs/m6-causal-ablation-instrument/README.md)).
- [`durable_consumer.rs`](../../crates/uor-r4-object-store/src/durable_consumer.rs): M3 scoped durable-session persistence (`export_scope` / `import_scope`, `UORMEM01` + `UR4SESS1`) and occurrence-bound fact quotation over `uor-r4-object-store`, with finite CPU controls in [`tests/durable_consumer.rs`](../../crates/uor-r4-object-store/tests/durable_consumer.rs) and the M3 durable-memory bar pre-registration in [`docs/labs/object-store-consumer-m3/README.md`](../labs/object-store-consumer-m3/README.md).
- [`stack_aerm.rs`](../../crates/uor-r4-training/src/stack_aerm.rs): the exact relation store simulator and the G v1 always-on read.
- [`stack_prime_route.rs`](../../crates/uor-r4-training/src/stack_prime_route.rs): the prime-route and semiprime store, with `ReadPolicy::{Tagged,Sieve}`.
- [`stack_store.rs`](../../crates/uor-r4-training/src/stack_store.rs): the token-level bridge to the exact versioned store (version order, predecessor chain, conflict marking, capacity eviction).
- [`stack_memory.rs`](../../crates/uor-r4-training/src/stack_memory.rs): sparse product-key memory with H4 or E8 codebooks.
- [`cache_memory.rs`](../../crates/uor-r4-training/src/cache_memory.rs): the exact-pair cache over a frozen model.
- [`whole_path_learning.rs`](../../crates/uor-r4-training/src/whole_path_learning.rs): the whole-path source (CPU fixtures only).
- [`durable_memory.rs`](../../crates/uor-r4-core/src/native_geometric/durable_memory.rs) and [`historical_read.rs`](../../crates/uor-r4-core/src/native_geometric/historical_read.rs): scoped durable memory and the learned previous-record selection.
- [`bin/kvar-recall.rs`](../../crates/uor-r4-core/src/bin/kvar-recall.rs) and [`bin/realtext-sparse-recall.rs`](../../crates/uor-r4-core/src/bin/realtext-sparse-recall.rs): the two recall harnesses.
- Evidence, newest context first: [`attention-state-2026-10-18`](../labs/attention-state-2026-10-18/README.md), [log-sieve design](../../archive/integration/log-sieve-retrieval-design-2026-10-01.md), [`kimi-direction-2026-10-12`](../../archive/labs/kimi-direction-2026-10-12/README.md).

## Related mechanisms

[pointer-key](./pointer-key.md): copies the value token the address names. The pointer found the source but not the slot. [read-binding](./read-binding.md): teaches an attention read which stated value belongs to the question, the learned alternative to an exact address. [object-store](./object-store.md): exact byte storage of source material beneath the model, with no addressing of facts. [e8-orbit-memory](./e8-orbit-memory.md): the E8-addressed product-key variant, replacing a feed-forward block. [relation-read](./relation-read.md): reads retained structure, where exact edges would come from this store. [admission](./admission.md): the measured ceiling on what any read can reach. [prime-zeta-codebook](./prime-zeta-codebook.md): supplies the prime registry the sieve depends on. [route-read](./route-read.md) and [flock-read](./flock-read.md): similarity-free alternatives that read by address or by a small kept set.
