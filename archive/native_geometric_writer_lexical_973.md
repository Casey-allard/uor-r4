# Native contextual lexical writer correction for #973

Status: **retained at bounded writer-correction scope**. The [evidence receipt](evidence/native_geometric_writer_lexical_973.json) binds actual artifacts, source files, commands, results and resources. This is a UOR-R4 Geometric Language Model development result, not general instruction understanding or prose qualification.

## Observed defect and learned correction

The preceding [source-role cycle](native_geometric_source_roles_973.md) retained an exact feature collision:

```text
Where is selvi? Explain in a sentence. Answer:
Where is selvi? velra in Dusk Ridge. Answer:
```

At the first Assert proposal, the old writer masks both payload endpoints and composes word-local H4 context over the identical interior word `in`. Its dictionary also omits labelled payload words. Both prompts produce the same 21 keys, hash `875abdee54cf2b7b71c669e3bace99ba25ad14ba54111d749023ebde1b82d066`, and score 2. The instruction incorrectly becomes an `Explain → a sentence` fact. Weights on identical old keys cannot distinguish this pair.

The new [Rust writer extension](../crates/uor-r4-core/src/native_geometric/writer_lexical.rs) adds a separate lexical-prime dictionary and one ordered endpoint/context feature. Prime identities are packed exactly; magnitude and hash bits are not semantic distances. The dictionary selects the 128 most frequent-by-document words among 136 construction words, with deterministic lexical tie breaking and no payload exclusion. Existing masked features, H4 transport, orientation, fixed zeta context and boundary context remain active and unchanged.

Offline fitting uses 825 documents, 12,008 presented word boundaries and 3,518 distinct proposal frames. The 456 labelled instruction boundaries target NoWrite; other boundaries preserve the exact parent proposal. One epoch learns 13 coefficients of -8 and raises frame correctness from 3,482/3,518 to 3,518/3,518. All learned rows concern the observed endpoint pair in different contexts; their observed transfer remains limited to the tested forms.

In actual execution, the instruction residual is -8, changing its Assert score from 2 to -6 and selecting NoWrite. The matched fact residual is zero, retaining score 2 and the exact `velra / Dusk / Dusk Ridge` record. The old 21 keys stay unchanged. The runtime receives no supervision labels, instruction-word list, phrase parser, answer template or provider response.

## Artifact and runtime boundary

Retained CID: `blake3:169f23efd1babd314deed5cd523953179d94a8eb9dbf6e930892e57080a85828`; SHA256 `3594003cc8265a31f5fe512ab38cd68289f5255769f30cf254f1fc7028e00260`; 13,552,256 bytes. Local artifact: `.uor-models/native-typed-value-2026-09-05/writer-lexical/model.json`. Parent `82662f85` and every older candidate/receipt remain preserved.

The optional `writer_lexical` witness binds the parent CID, dictionary, coefficients, training configuration and document receipts. Every parent component, including writer rows, dictionary, cache, geometry, source selection, lexical emission and numeric operators, remains exact. Removing the extension reconstructs the complete parent byte-identically. Validation checks that parent recursively before checking the extension and current identity.

Each new coefficient must be nonpositive. Thus every proposal score can only decrease, and an old certified NoWrite window cannot become a write. The existing cache remains sound under that restricted condition; positive coefficients or changed old features/rows would require a different certification. This correction cannot introduce a previously missed positive write. Actual commits remain necessary checks because suppressing a winner can expose another proposal.

`WriterLexicalDisabled` reaches the actual input, response-entry and span paths and reproduces the restored parent's behavior. Serving adds bounded lexical address lookup and one score lookup per proposal, retains exact occurrence/version identity, and executes no matrix products or transformer backbone. Architectural preservation is distinct from measured comparative geometric advantage.

## Executed behavior and validation

| Check | Actual result | Scope |
| --- | --- | --- |
| Writer construction | 825/825 | Record correction plus parent output/EOS preservation; includes the 64 balanced cases |
| Original collision | 2/2 | Instruction suppressed, matched fact retained; disabled control restores parent |
| Retained histories | 12/12 versus parent 8/12 | Four extra instruction records removed; all legitimate records, selected sources and output bytes retained |
| Post-selection fixed-form comparisons | 32/32 | 26 new-prompt identity cases and 6 repeated factual cue controls; no refit after draw |
| Retained independently judged answers | 663/663 | Prior construction population |
| Word construction / open / prior transfer | 132/132, 28/28, 24/24 | Exact source commitments and generated answers |
| Source positions / original abstention | 40/40, 10/10 | Prior source correction preserved |
| Earlier semantic/control reports | 14/14 exact matches | Includes retained negative interventions |
| Native API | 176/176 | Answers, independent turns, checkpoint import, artifact identity and corruption rejection |
| Focused tests | 32 unique passing | Kernel source guard and seven actual-artifact allocation/checkpoint checks included |

The [Rust behavior driver](../crates/uor-r4-core/examples/native_writer_lexical.rs) observes prompt tokens and freely generates candidate, restored-parent and disabled-feature responses. For instruction annotations, its offline reference removes only parent records whose two payload endpoints lie within the labelled instruction, then checks legitimate payloads, IDs, previous links, conflict state and directory. All comparison cases preserve the parent's actual bytes/EOS; that is not an independent answer-quality label.

The factual control `Record: Explain in a sentence. Where is Explain? Answer:` retains the actual `Explain → a sentence` record in both models. Both still emit:

```text
 a sentence . The conversation does not give ae.
```

This remains a generation negative. Passing this writer control rules out a global lexical blacklist, not malformed prose. The two existing 33-step targets remain 0/2 under the 32-step response-entry cap, with exact prior wrong text/EOS preserved. General prose, flexible instruction following, reasoning, Rust synthesis, alpha, frontier capability and complete-path energy savings remain unqualified. New-artifact browser/WASM/HTTP, blanket full suite and Clippy are NOT_RUN. #973 remains open.

Actual generated Copy/Add Rust equalities and four retained identifier-return fragments were extracted, compiled and executed. The writer allocation check restores checkpoints across 81 input positions and eight output positions and measures zero allocations during observe/begin/predict/observe. Artifact load took 25.663 seconds there. The separate three-case word check measured a 145.583-microsecond warm predict/observe median and 215.708-microsecond maximum, with 26.180-second loading; encoding, session creation, BOS, checkpoints, JSON, decoding and reporting are excluded from the warm measurement. Complete-path energy advantage is not established.

## Resources and successor

This cycle used **1748.183 model seconds** and **552.355 engineering-command seconds**. Cumulative model use is **22232.042 / 25050.000 seconds**. The 2,700-second and 2 GiB extensions were recorded before use under standing authorization. Five unused fitting minutes were reassigned to interface/allocation verification without increasing any total. The preserved 196.620-second reservation leaves 2621.338 seconds. Conservative storage is **36,408,483,840 / 39,158,181,888 bytes**, including the retained copy and 16 MiB metadata reserve, with 2,615,480,320 bytes remaining after the 128 MiB stop margin. Limits remain one model process, two threads, 4 GiB model RAM, 6 GiB build RAM, 512 configured context tokens and 96 output tokens; the inherited five-turn replay explicitly permits 1,056 padding tokens per turn and up to 8,192 total input tokens. Sampled model/build process-tree peaks are 3,041,673,216 / 2,557,296,640 bytes. No unique deletion or paid external compute occurred. Small metadata/document/Git commands are outside the engineering stopwatch.

**Next: compositional owner/value emission through the shared selector.** Refresh the actual selected relation/version, first emitted field and transition between fields. The current word adapter enters suffix emission only after the value copy completes, so suffix weights cannot place an owner before that value. If this structural limit remains, learn an artifact-bound field choice over the selected relation: owner, connective, value/span and stop, preserving exact byte anchors and all value-first behavior. Use explicitly different owner-first requests; do not assign incompatible answers to the already retained sentence prompts.
