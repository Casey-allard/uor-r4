# Geometric selected-record occurrence consumer — October 3

This implements the source-consumption seam selected by the
[executed source-binding contrast](geometric-chat-source-binding-2026-10-03.md):
correct recalled values reached all twenty read-on inputs, but four complete
answers followed the recent distractor. It preserves the existing compiler,
store, checkpoint and tokenizer. It does not replace Claude's emitter/control
lane or DeepSeek's distance/surface-form lane.

## Mechanism and numerical boundary

The integer `geometric_occurrence_read` reader processes the selected record's
original BPE occurrences, query and preceding response tokens through the
existing causal q4 H4 context. Only original record occurrences are candidates.
Each candidate retains exact record, commit, token offset and token ID; equal
tokens remain separate occurrences. Query and response tokens never become
copy candidates. Scope/entity/relation/view/status remain caller metadata;
relation and view do not yet condition numerical scoring. Unsupported status
and oversized inputs reject, without silently truncating or changing the last
valid public read.

Directed relative-H4 context potentials score each occurrence. Content-address
channels are absent, age is zero and held-span channels are inactive in this
initial seam. These choices are explicit, rather than omitted learned features
being described as complete attention. The reader returns actual integer Q24
scores and Q31 occurrence/NoRead weights through the existing native reducer.
Its admitted sequence is at most 128 tokens, H2/L4 in the construction driver.

For each head, the floating development adapter computes

\[
 P_h(v)=\frac{w_{\mathrm{NoRead}}P_{\mathrm{parent}}(v)
 +\sum_{j:\,\mathrm{token}_j=v}w_j}{w_{\mathrm{NoRead}}+\sum_jw_j}.
\]

It averages the heads. NoRead preserves the **entire** pointer-aware parent
distribution; it does not substitute trunk logits. Disabled mixing returns
the original score bits. Exact zero mixture mass remains negative infinity
in log scores; a zero/underflowing target probability rejects the construction
loss. The adapter cannot independently change the relative probabilities of
two tokens absent from the source frame. Generation and EOS therefore remain
dependent on the retained parent and its reaction to the generated prefix.
There is no separately learned Stop action in this deliverable.

The offline ordinary-answer loss uses the native integer weights and denominator
as floating normalized probabilities in forward,
with the declared softmax-score adjoint and existing quarter-coefficient/context
surrogates in backward. Coefficient and frozen-context credit are connected
without duplicating coefficient gradients. Targets enter only the loss;
preceding teacher-forced tokens are training prefix information. This is a
declared surrogate, not the derivative of finite hard choices.

Native export binds the tokenizer, full parent checkpoint identity, source
coefficients, configurations, H4 algebra and exponential table. Reload verifies
those bindings and constructs the returned artifact from the exported bytes.
The construction preserves a deliberately corrupted export and requires its
rejection. Source/native reload here is independent construction in one
process; it is not a fresh-process generated-chat test.

## Construction evidence

The retained actual protocol-2 parent has vocabulary 4096 and context 384.
The consumer starts from fresh legal seeded quarter coefficients; no authored
answer weights or retained synthetic-language fit is transferred. Its H2/L4
context has 8,963,136 source coefficients, so neither training nor storage is
assumed cheap merely because occurrence access is bounded. The construction
uses a complete accepted development answer including EOS, at most eight
individual target-token forward/backward batches, and zero optimizer updates.
Eight token batches are not eight B8 optimizer updates.

At source `b3fcbc4a8268056756624804860e22e274a1ec1d`, the
[construction report](../evidence/geometric-source-consumer-construction-2026-10-03.json)
records **one complete teacher-forced answer, four BPE tokens plus EOS**. All five
prediction rows have finite, nonzero context, potential and NoRead gradients,
exact native trace replay after payload reload, and bit-identical disabled parent
scores. Loss matches the native target NLL to at most 1.1921e-7. The changed-payload
negative rejects with `consumer native payload digest differs`. There are zero
optimizer updates and no autonomous generated-answer measurement.

The worker takes **55.44 seconds real**, with maximum RSS **1,020,280,832 bytes**
(about 973 MiB); report-internal wall time is 53.277 seconds. This is construction
cost in the test build profile, not optimized serving speed. Context expanded
tables occupy 44,564,480 logical bytes, alongside packed coefficients and the
other components. Reported component payload sums to 50,356,018 logical bytes
(about 48 MiB), including packed and expanded tables, the duplicate potential
table, exponential table and algebra; this excludes reader scratch and allocator
overhead. Context access is 6,552 reported coefficient/table reads per input token
at this shape, 78,624–104,832 per constructed prefix. Complete process RSS includes parent, source shadows, backward
graphs and wrapper allocations. The native wrapper creates a reader and trace
vectors per call, so this does not establish allocation-free complete serving.

Two integer and two training focused tests pass, including occurrence/rejection
state and mixture/null/zero-mass arithmetic. Independent source/outcome review
approves this bounded result. The driver seals and verifies the complete report
file set; independent review checks inventory/lengths and report SHA, without
claiming its own cryptographic seal recomputation. Report SHA-256 is
`4729ea02663094468f865884a460fa7fe6d5b187a74663fa9479854672ccf000`;
executable SHA-256 is
`6673ea0451948ec60cb936a409e187604190b5d7c3249b13b6ec6e8a143b7670`.
The retained root is
`~/uor-r4-local/workspace/research/geometric-source-consumer-20261003/construction-2`.
Construction-1 remains sealed as an initialization error before answer batches:
the existing potential-source constructor needed its Held-latch admission flag.
The repaired constructor is only saved, never evaluated; held-span scoring is
still inactive in the actual consumer.

The untrained uniform reader is a same-information diagnostic, not a learned
capacity-matched comparator. No fitted-language, general-chat, geometry
advantage, complete native serving or energy result follows from construction.

## Next causal decision

The owner's October 3 correction puts **making geometric chat work first**.
Beating a transformer is a later goal. It is not a threshold for admitting this
development fit, keeping a mechanism, or preserving a bounded negative.
Measure complete short/long B8 cost witnesses, then admit one bounded geometric
consumer continuation whose primary outcome is source-bound complete responses.
No automatic scale/dose sweep follows from a failure.

Implement a learned ordinary reader control later for attribution, with identical
candidates, context, NoRead, parent inputs and answer objective. It is not a
precondition for the first working geometric path. A full directed bilinear reader provides
a stronger useful control than a symmetric diagonal score. Its sixteen active
directional coefficients per lane exceed the geometric reader's four: report
the extra 96 coefficients and actual table/read cost rather than padding with
inactive parameters or claiming exact capacity equality. This comparison
isolates read relation within shared H4 context; a wholly ordinary recurrence
remains a different comparison.

Use fixed selected-value×recent-distractor examples and separately reserved
complete values/wording, preserving the twenty exposed cases as regression.
Both arms need actual free-running complete responses, EOS/length outcomes,
independent exported-artifact reload and row-by-row comparison with the
disabled parent and initial consumer. A copy-token CE improvement is not
complete answering. If both learned readers improve, attribute the result to
the connected consumer. If copying improves but non-copy words or stopping
remain wrong, examine the demonstrated mixture limitation instead of adding
an unchanged dose. A negative does not retire the geometry family.

The current parent and mixture still use floating arithmetic. This is an
integer geometric reader connected to a floating language reference. Native
vocabulary emission, explicit conditional generation/stopping, complete-path
compiled instruction/allocation qualification and laptop energy remain open.
