# D41 — one-time offline donor corpus exception (2026-10-19)

**Status:** owner-ratified decision record. No compute, $0. The exception activates only
when a `distill-d1` generation packet is dispatched after the driver math optimization.

The owner ratified D41 in the Kimi PM chat on 2026-10-19 ("if we can do that then I ratify
d41 and confirm gpt-oss-120b") and confirmed the donor `openai/gpt-oss-120b`. The decision
text is [DECISIONS.md D41](../../integration/DECISIONS.md#d41--one-time-offline-donor-corpus-exception-owner-2026-10-19).

Scope in one paragraph: one donor, one corpus (`distill-d1`), one offline generation pass on
one declared pod, exempt from D23/D25/D27/D28/D30 only while generating; the donor never
executes in student preparation/training/evaluation/serving/grading; the student consumes
hard targets only on the unchanged D32 discrete path; the exception expires when the corpus
is sealed. Sequence (owner-directed): driver math optimization first (occupancy, BVG-K driver
integration), then a measured ~1M-token pilot on the same donor (~$2), then production corpus
sizing. The injective attention test line is unaffected and runs first.

Drafting analysis, donor candidate review (licenses, pod fit, cost) and the FIT skeleton are
retained by the Kimi PM seat; key points: gpt-oss-120b is Apache-2.0 (no naming or derivative
conditions; Llama 3.3 and Gemma 3 were excluded on license grounds), ~30M-token production
corpus ≈ $52–63 on one PRO 6000, Qwen3-30B-A3B (~$15 on one 5090) remains the cheaper
fallback should the owner redirect.
