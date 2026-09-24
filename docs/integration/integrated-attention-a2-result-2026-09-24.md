# Integrated attention A2: evidence learning improves; hard routing remains unresolved

September 24, 2026. References #973 and #820. Open development within D7 Milestone A. Protected delivery: [PR #1387](https://github.com/UOR-Foundation/uor-r4/pull/1387), open for review; A2 is not yet merged.

## Delivery and decision

The owner-requested PR [#1386](https://github.com/UOR-Foundation/uor-r4/pull/1386) merged through the protected queue at `2026-09-24T18:15:16Z`. Main commit `b020f34a09816ba235984cedde1c50f4c8e9cae2` has tree `c68b8b332080d09884b27191731b56296dc4a1f4`, exactly equal to reviewed head `d1f5524c1d2b302cf2c07fea6b47309460b85f66`. The incorporated component PRs #1380–#1385 are closed with provenance comments. Queue checks remain compatibility acknowledgements, not executed tests.

The [A2 continuation](integrated-attention-a2-plan-2026-09-24.md) is implemented, fitted, exported, reloaded and independently replayed. **Useful attention and complete language generation remain unsuccessful.** The corrected C120 and 2I models each admit the right source on only **1/24 new first-answer decisions**, rank it first on **0/24**, and answer **0/12 complete read-enabled correction variants** correctly. The model remains a research candidate and does not replace the application default.

The evidence narrows the next work. The head can now assign more probability to the right decision when supplied its source offline. However, the corrected address training leaves **zero net changes in the dedicated exported context-token address bank**. Its soft objective has not produced useful hard routing. Train an integer routing-margin objective in the same integrated model next; do not infer a failed representation family from an effectively unchanged context bank. Keep learning complete evidence-conditioned language output alongside routing.

## Model change and first-principles basis

A1 wrote a source key before later entity context arrived, allowed identical candidate codes, and did not teach the head how to use correct evidence. A2 changes these coupled interfaces:

- Raw events are still captured immediately. Searchable records are committed after 16 further observed tokens, with the original source occurrence and the later commit time both retained. A query can see only records committed before its cutoff. Up to 32 observed context tokens form each key; the source anchor token is masked.
- Coarse Key/Query codes share contextual selected rows in a dedicated address bank. Admission probes that one coarse code, while fine codes can retain value/state information for relative-energy ranking. The State encoder retains its previous rows. Context is initially an unordered bag; one masked BPE token is not a masked complete value.
- Offline contrastive learning compares a query with causal positive/negative keys, with an explicit utilization term. Fine codes receive truncated relative-energy credit. The language head learns marginal Generate/Copy probability using actual reads, a fit-only positive source, and NoRead; source evidence is not a command to copy its value.
- Natural prose/Rust next-token training is joined with paired permission corrections whose answer requires an uncopied Yes/No consequence. Raw identity, learned address, source ranking, gate selection and language output remain separately observable.
- A2 has an explicit `UORIA02` envelope and payload version 2. The reader retains A1 support. An actual preserved A1 executable rejects an A2 artifact with `invalid artifact envelope`; that expected rejection is format evidence, not model-quality failure.

The learning ideas were checked against primary descriptions of [DPR](https://aclanthology.org/2020.emnlp-main.550/), [REALM](https://proceedings.mlr.press/v119/guu20a.html) and [wav2vec 2.0](https://arxiv.org/abs/2006.11477). They motivate competing sources, downstream language credit and discrete-code utilization. Their dense/Transformer backbones are not part of this implementation; those papers do not validate this model's learning rule.

## Data, attempts and reproducibility

Every arm uses the same seed `71679248566564`, V4096 tokenizer, four lanes, 64 candidate bound, 512 examined-posting bound and 4,096 raw/record capacities. The fit has **294 episodes / 140,597 tokens per epoch**: 102 pinned natural source files plus 192 grounded worlds, interleaved deterministically. Development has **38 episodes / 14,421 tokens**: 12 separate natural files, 24 new grounded worlds and two inherited A1 cases. All are exposed development material, not final acceptance data. Each of four fits completed eight epochs, **1,124,776 observed tokens**; total **4,499,104**, within the 6,000,000 cycle cap.

The first pair trained address contrast on both weak natural recurrence hints and grounded first decisions. Natural hints use the eventual target to choose a source; many are not identifiable from the query. They contributed 123,888 of 125,424 contrastive updates per arm, versus 1,536 grounded updates. One [prospective correction](../evidence/integrated-attention-a2-label-correction-2026-09-24.json) removes only the weak natural **address** supervision. Natural token/evidence training, data, seed, rates and eight epochs remain unchanged. Both initial failures are preserved.

| Attempt | Source commit | Model SHA256 |
|---|---|---|
| `c120-dev1` | `3b7d2f5811337483c2276ef77e4c9fe27c6a10ba` | `f99615164ed177225306c8f51436aac6526373d1c110da60f27a69554fa96d3c` |
| `geometry-dev1` | same initial source | `5de21a03e9dcb5297df9dd64dd454bb0d636506d864da1da22b1a0165c966d38` |
| `c120-dev2` | `40542ede834fe223b5575feeeeae1b46677b3801` | `91ed02613d89c470757d6ec5be6a933c6796ebcc88437520448478f7740f95bd` |
| `geometry-dev2` | same corrected source | `9a09c51b24ecbc65f12a3072e2b63aac409cdc59372e4a526db046351ebdd9f7` |

Retained container: `/Users/casey.allard/uor-r4-investigations/integrated-attention-a2-20260924/`. All fit, replay and inspection roots were exclusively claimed, sealed and verified by the Rust report API. All four `data.json` files are byte-identical. The combined data manifest BLAKE3 is `68db614fbeb35fee6068dfb4753fd2ec347733a2fd7a58d817fe0335890cf809`; tokenizer SHA256 is `a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f`.

For each artifact, separate-process replay produces byte-identical `evaluation.json`, `generations.json` and `source-diagnosis.json`. All **112 generated rows** also reproduce after snapshot restore within the tested short episodes. The [checked-in evidence](../evidence/integrated-attention-a2-result-2026-09-24.json) preserves complete generated text/token IDs, prompts, source rows, component digests, model/executable identities and report manifests. Full step traces and fit logs remain in the sealed local roots.

## Loaded observations

Lower bits/token is better. These are one-seed development measurements, without a final confidence or promotion claim.

| Measurement | Initial C120 | Initial 2I | Corrected C120 | Corrected 2I |
|---|---:|---:|---:|---:|
| Read-enabled bits/token | 6.877009 | 6.966861 | 6.854878 | 6.798124 |
| NoRead bits/token | 7.136560 | 7.202779 | 7.155441 | 7.101787 |
| New first-answer source committed | 24/24 | 24/24 | 24/24 | 24/24 |
| New first-answer source admitted | 1/24 | 1/24 | 1/24 | 1/24 |
| New first-answer source ranked first / selected | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |
| New first-answer gate opens | 0/24 | 0/24 | 22/24 | 22/24 |
| Complete correct answers, reads enabled | 0/12 | 0/12 | 0/12 | 0/12 |
| Net changed dedicated address-bank coefficients | 99 | 99 | 0 | 0 |

Each generation panel includes 12 read-enabled correction variants, their 12 NoRead counterparts and four natural prose/code continuations. All 24 correction rows per artifact fail the unchanged complete-answer equality rule. For example, the corrected C120 output for a positive Orion permission prompt is `:::::::::::::).len();_d::::::::::)`; corrected 2I produces `::::::::::::::(&self, Ser::::::::::::`. Their corresponding NoRead outputs are colon loops. The full outputs, including source edits, are retained without selective omission.

A1 and A2 use different data and training doses: their absolute likelihoods are not a matched improvement estimate. Corrected 2I's lower aggregate likelihood here also does not establish geometric advantage: useful generated behavior is still absent, this is one exposed seed, and a competent ordinary learned router plus identifiable frame/transport comparison remains required.

### Source-to-language connection

The new 24 worlds contribute 36 BPE decision-token positions. On corrected C120, one-step forced-source NLL is **61.806803 nats**, versus **115.862965** when evidence is removed at the same actual state. On corrected 2I it is **58.884029**, versus **114.491525**. The actual hard-read NLL is **119.330910 / 115.052391**: opening the gate more often with the wrong source does not deliver the offline benefit.

The two inherited probes contribute 28 full-answer positions and must remain separate. Corrected C120 forced-source/NoRead NLL is **186.648055 / 187.202234**; corrected 2I is **178.598839 / 178.404147**, a slight worsening. Their source admission also falls from 21/28 to 12/28 across the correction. Thus the new decision-word result cannot be promoted to a general source-conditioned language claim. One-step forced evidence is a diagnostic only; no gold source enters serving.

### Hard address and output diagnosis

The dedicated context bank has 2,097,152 stored coefficients. Both corrected artifacts end with exactly the initialized integer values. A shared Key/Query role row belongs to a different bank and is outside this count. Its global bias cannot by itself learn entity-specific distinctions. Floating masters persist across updates: it would be incorrect to say every gradient is discarded by per-step rounding. Master trajectories were not retained, so sub-bin movement, cancellation and oscillation are not distinguished.

The soft overlap loss at temperature 8 and hard single-page admission are different objectives. The correct source's code ranks in the query's top four on only 2/24 new first decisions. On frozen paired value edits, all 12 pairs preserve the query coarse code, but only four initial and three corrected pairs preserve the source coarse code. Remaining BPE pieces and the shifted 32-token context can affect the key. This establishes the current artifact's failure, not mathematical impossibility of the encoder class.

A read-only exhaustive V4096 surface-probability scan separates head quality from local greedy decoding. In the initial pair, colon is the actual surface MAP on all 24 new first decisions. In the corrected pair, the actual target is MAP on **0/24** for both arms; an independent NoRead session has colon as MAP on **24/24**. Forced correct evidence makes the target MAP on **11/24** for each arm. Greedy choice can differ from MAP, but wider decoding cannot by itself repair the learned colon preference. Stop plus all surface-token probability mass normalizes to one within `8e-15`; these floating scans are offline diagnostics, not served computation.

## Mathematical and engineering qualification

The precise implemented claim is **contextual finite-code retrieval with a learned empirical relative-energy score**. For each lane the ranker uses a relative group element `(q_l a_l)^-1 k_l`, with identity local actions in this implementation, then four unary and four pair factors. The 2I table represents quaternion-group composition. There are no canonical position–momentum variables, symplectic flow or physical Hamiltonian theorem. Energy coefficients remain fixed after loading; state, queries and stored records change which energy entries are read. Nontrivial learned transport and useful mutable-Hamiltonian behavior remain research objectives.

The coarse contextual bank is algebra-independent under matched inputs. Geometry can affect state and fine ranking here, but it cannot explain a coarse-address gain directly. All-write warmup is not learned selective storage. A committed record can be excluded from a saturated posting page; source commitment and admission are separate counts. A 4,096-event ring is exact bounded retention, not unbounded exact history.

Each artifact stores **42,135,302 parameter bytes**. The complete generated step, including query/key/state code formation, uses approximately **35,858–36,009 selected learned coefficient reads** in the observed panels, plus measured table/page/posting accesses. This is roughly 4.6 times A1's observed access level, caused chiefly by context rows; it is a real architectural cost. Logical counters omit raw-byte copying, index maintenance and allocation. No full numerical-kernel machine-code or electrical-energy qualification was performed. Peak model-fit resident memory was **431,570,944 bytes** (about 412 MiB); reported fit-command elapsed times were 47–60 seconds per arm.

Independent engineering review also found a **pre-existing long-session snapshot limitation**: `ExactMemory::restore` rebuilds posting membership from surviving records. After page saturation followed by record-ring eviction, that rebuild can admit a formerly excluded record and displace a previously admitted one. The tested short episodes do not wrap the ring. Explicit posting membership must be preserved and validated before claiming persistent-session replay at that boundary. Snapshot parity here covers serving, not training resumption (`last_transition` is not saved). This is recorded for the compositional-session milestone rather than hidden by the successful short replay count.

## Executed verification and resource record

- The changed module's 28 focused tests passed during integration, before the final envelope/source-boundary edits; 1,129 unrelated tests were filtered out. This is not reported as a full-suite or final-commit test result.
- Offline touched-package check and release builds succeeded. Final source-bound release executables performed all corrected fits, reloads, short snapshot generation, independent replays and probability inspections described above. Only known dead-code warnings were emitted.
- Final formatting, whitespace and claim-wording checks are recorded in the delivery closeout. Compatibility queue statuses do not substitute for these local checks.
- Total training exposure was 4,499,104 tokens; no paid compute or artifact deletion occurred. The [budget](../evidence/integrated-attention-a2-budget-2026-09-24.json) includes the entire recovery/build/fit/review/delivery cycle, not just the roughly four minutes spent in fit commands.
- A [recorded projection discrepancy](../evidence/integrated-attention-a2-context-projection-correction-2026-09-24.json) remains explicit: the original 512-token context projection applied to authored episodes, while the natural loader actually used up to 1,024 fit / 768 development tokens. This was discovered after the first C120 fit and corrected prospectively for the remaining arms. It is not retroactively relabeled as preauthorized measurement. Other cycle ceilings were unchanged.

The [cycle closeout](../evidence/integrated-attention-a2-closeout-2026-09-24.json) charges **4,008,466 ms**, including an explicit five-minute delivery reserve, to the shared ledger: **473,475,252 / 477,000,000 ms**. The retained attempt container occupies **194,756,608 allocated bytes**. Physical free space at closeout is **30,118,309,888 bytes**, leaving **4,080,070,656 bytes** above the preserved reserve and stop margin. No additional cleanup was necessary. The whole shared build-cache delta was not independently isolated and is not reported as measured per-attempt storage.

## Next: make hard admission learn in the integrated model

The next A3 mechanism is a **structured integer routing margin**, developed with the same source-conditioned language head, exact memory, ordinary/geometric arms and complete-output endpoint. It is a proposed learning mechanism, not an executed success.

1. For a causal paired source-value edit, learn one latent coarse bucket shared by the query and both source variants. Require that bucket to beat competing codes by an integer margin. It is a learned code, not an exact occurrence label or serving-time gold field. Same-entity versions are co-positive for coarse access; wrong-entity committed records supply exclusion pressure, with typed version selection still a separate future ranking obligation.
2. Include actual 64-posting occupancy and competing-source constraints. A globally collapsed code cannot sustain reliable admission across a stream that exceeds the 64-posting page. Skip identical selected-row multisets that the present feature map cannot distinguish.
3. Start with bounded deterministic signed-four-bit coordinate updates on active content rows, accepting only edits that reduce the hard balanced-batch objective. An accepted edit changes the served artifact directly. Soft/straight-through gradients can be a declared warm start, but exported integer margins and page admission decide the result. Record hard row/code changes and any retained master residuals; no arbitrary rate/epoch sweep is justified by A2.
4. Require an actual hard-admission improvement on fit and then paired held-out development over A2's 1/24 before attributing remaining failure to ranking. Continue through actual pre-gate ranking, selected source and complete uncopied answer generation against NoRead/source edits in that same run. A coefficient-change count alone is not progress to language acceptance. Reject a routing treatment that merely overloads a page or worsens the retained outputs unnoticed.
5. If integer optimization moves useful content rows yet cannot generalize, replace the unordered bag with learned ordered span/role descriptors. Reuse the existing observed-text segmenter's byte/BPE identity and ordered-feature contracts, not its old clause assumptions, lexical-membership oracle or unqualified weight format. Full-span evidence also needs an owned copy cursor and output training; storing a longer payload alone is insufficient.

The model remains on the adopted prime/zeta/R4/shared-operator programme. This continuation connects optimization to the actual serving decision and keeps full language behavior as the endpoint. It does not reopen a broad architecture survey, introduce a transformer backbone, or promote a Hamiltonian/geometric/energy claim from a small fixture.
