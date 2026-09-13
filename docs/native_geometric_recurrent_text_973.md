# Recurrent geometric reads and grounded text — #973

**PASS_RECURRENT_GROUNDED_TEXT.** One shared learned action policy now executes the earlier typed reads and variable-length text through the same recurrent runtime. Final development:64/64 exact answers including EOS,32/32 complete changed-source pairs,64/64 correct source paths and8/8 four-source answers absent from the new training curriculum. All earlier one-read, dependent-read, adaptive-read and text controls pass through this path. Retain `15baec48`; no promotion or general-prose qualification.

## What is integrated

The [recurrent_text source](../crates/uor-r4-core/src/native_geometric/recurrent_text/mod.rs) uses the retained learned geometric reader, byte/EOS writer, cursor advance and discrete query update. A new shared policy selects Emit, Read or Stop. Each actual emitted byte updates a pending query through the learned update. The current source stays selected while its text is emitted; a boundary Read commits the pending query and recomputes subsequent source selection. A silent Read while a byte is present incorporates that selected byte without emitting it, supporting the earlier typed link tasks. There is no historical-policy dispatch or answer-copy override.

The schema2 policy has2,048 rows indexed by the current/last byte, payload presence, whether the supplied context contains any multi-byte spans, and positive learned compatibility at the proposed next query. The context-length feature is observable structure, not a task ID or supplied depth, but it does distinguish byte-only from text contexts. A redundant current-span-length feature was removed after the first candidate regressed. Source spans, exact bindings and the initial two-byte query remain supplied.

Every observation performs two full geometric scoring passes over the four supplied candidates: current read and proposed next read. Scoring reuses signed H4 directed relations, structured Hamming distances and fixed phase differences. Zero winning compatibility offers no payload; it is a model decision, not an exact proof of absence from memory. No transformer, runtime provider or serving matrix contraction is introduced. Bounded allocations and repeated validation/scoring remain; complete optimized latency/energy qualification was not run.

Inference accepts only records and query. It receives no answer, source path, depth, stop marker or action label. Stop can execute only at a span boundary; the policy must select it. Four source reads,128 emitted bytes and160 transitions are resource bounds, with exhaustion reported as failure rather than forced correct termination.

## Learning and frozen test

Only the shared action policy is fitted. Bounded search through the actual frozen writer, update and geometric reads finds trajectories whose complete output matches the target, including EOS. It uses at most512visited states per example and supplies action credit without an expected-query or path oracle. Search prioritizes emission/stopping and selects the first successful trajectory; label conflicts would require checking alternative trajectories, not declaring an impossibility theorem.

Training combines512new examples with historical **training portions only**, totaling13,312 rows. The new corpus has256causal pairs; changing only the first source text changes a one-source answer into a two-/three-source composition. Development has32pairs/64rows, with depths1/2/3/4 represented by32/12/12/8rows. Initial source slots are balanced. New complete answers, contexts and families are disjoint across splits; characters are covered. Development initial queries use EFGH, while later byte-derived queries are unrestricted. Exact authored address transitions define this typed task; they do not establish a semantic metric.

Preparation finds every required trajectory, no conflicting training-row targets, and no unseen new-development policy rows. Final preparation visits at most32states per training example and33per new development example. It exercises332policy rows. This is compositional trajectory transfer across known feature rows, not unseen-feature generalization.

The original128epoch proposal was reduced **before the first fit** to one row-balanced categorical cross-entropy update at rate0.1. Each exercised row has a unanimous action target, so later updates would only change unexported confidence. The full behavioral gate was retained:61/64 exact new answers,31/32 complete pairs,95% at every depth,61correctsourcepaths, zero longer-answer successes under every intervention, and all historical panels preserved. Final training accuracy is13,312/13,312;254exported action rows change. Primitives stay byte-identical. This is output-derived action supervision, not joint end-to-end learning of the primitive operators.

| Intervention | All64 development answers | Longer32 answers |
| --- | ---: | ---: |
| Untrained shared policy | 0 | 0 |
| Learned Full | 64 | 32 |
| Read disabled | 0 | 0 |
| Emitted-byte feedback disabled | 9 | 0 |
| Query update disabled | 9 | 0 |
| Cursor advance disabled | 0 | 0 |
| Boundary read disabled | 32 | 0 |
| Initial query reversed | 0 | 0 |
| Stop disabled | 0 | 0 |

All64Full trajectories reproduce exactly after export/reload. One actual pair keeps query `[70,69]` and all keys fixed: `the small fish waits.` comes from source1. Changing only that first span to `the small ` yields `the small fish waits softly.` through sources1→2→3→0, with queries `[70,69]`→`[64,67]`→`[116,119]`→`[44,47]`. Every byte and EOS comes through the learned writer/policy loop.

The same new runtime retains8,256/8,256one-read,2,176/2,176dependent,2,240/2,240adaptive and576/576text examples. Old byte records are converted to one-byte spans without extra action annotations. These are panel counts, not necessarily unique underlying cases.

## Demonstrated failure and bounded correction

Attempt1 produced64/64new answers but failed overall: only568/576retained text examples passed. All eight failures emitted the correct one-byte `?` or `!` and omitted EOS. The policy separated single-byte from longer-span endings; the short punctuation-ending rows were untrained, while corresponding longer-span rows already learned Stop.

Removing that redundant input halves the table and shares parameters across those lengths. Attempt2 uses byte-identical training/development data and acceptance. It recovers all eight failures and preserves all13,240previously successful retained rows plus all64new Full answers. No punctuation output override, evaluation-label training, relaxed expectation or extra training-data row was added. Both attempts, source versions and executables remain preserved. Final fit/report/controls took6,682ms, including2,554ms internal fitting and training-output checking.

Final candidate SHA256:`acad5821d77361aba1b752789f4f5324f27719201e5ab9ba0fdf8cd3e1e0c4c5`. Negative candidate:`ca40b30552942ab327eee6580218a9ca28723031ffe3eb94280f694ea561916c`. Parent:`22b1df1c6b5721ea71d0bad40ab968434b63397c7819213aec046c7def6af63e`. Independent final holdout: **NOT_RUN**.

Four new focused tests and21retained focused tests pass. They cover actual emitted-byte query feedback, its disabled control, artifact corruption/export, action/read bounds and data causality. Formatting, claim wording and whitespace checks accompany delivery. The two completed attempts are exclusively claimed, sealed and Rust-verified. All37priorsealedroots/1,080files, prior source/executable and original dirty checkouts verify unchanged; the negative attempt is preserved separately. [Exact evidence](evidence/native_geometric_recurrent_text_973.json) binds results, comparisons and sources.

Complete charged builds/preparation/fits/controls/checks:397,229/1,800,000ms. Shared125,384,987/132,950,000ms; parent5,634,414/7,710,000ms. No extension, paid compute, cleanup or V3–V7 replay. Projection retains6GiBRAM,512MiBnewstorage,128MiBmargin,two buildthreads/one modelprocess. Final storage, delivery and restart receipts stay at `/Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/recurrent-text-1/`; original artifacts remain in `attempt-1/` and `attempt-2/` there.

## Next responsibility toward the geometric language model

The emitted-byte accumulator is the retained XOR address operator. Its whole-span result is order-insensitive, although exact source bytes and cursor order are retained. Successful supplied-address composition therefore does not establish adequate language state or semantic transport. The owner explicitly reaffirmed the geometric language-model goal while permitting relaxation of obsolete workflow restrictions.

Next implement and train an **order-sensitive geometric state update in this recurrent path**, with a small contextual task requiring different generated answers for reordered relations using the same tokens. Reuse the existing signed finite group-composition interface in [Hamming refinement](../crates/uor-r4-core/src/native_geometric/hamming_refinement/runtime.rs) and the [H1 research design](integration/geometric-attention-research-2026-09/attention-and-learning.md); preserve exact occurrence identity and orientation, and measure collisions before fitting. Train state/query decisions against output, retain this integrated control, and distinguish a noncommuting geometric operator from demonstrated language learning. Do not grow a table of punctuation or task-specific exceptions. Raw-language binding, general prose, common primitive learning and final replacement qualification remain open under #973 and #964.
