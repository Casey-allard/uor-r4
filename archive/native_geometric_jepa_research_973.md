# JEPA as a candidate native geometric training objective

September 12, 2026. Owner-requested research assessment; **NOT_IMPLEMENTED, NOT_RUN** in UOR-R4. The current coupled state/emitter experiment keeps its frozen method and acceptance. This note adds a research option, not a roadmap replacement or a native capability claim.

JEPA predicts a learned representation of another view from a context representation. [I-JEPA](https://arxiv.org/abs/2301.08243) applies this idea to image regions. [V-JEPA 2](https://ai.meta.com/blog/v-jepa-2-world-model-benchmarks/) extends latent prediction to video and action-conditioned planning. Its video-language results use additional alignment to a language model. These sources establish a research direction, not a working local geometric language model.

Language evidence exists: [LLM-JEPA, revision 2](https://arxiv.org/html/2509.14252v2) adds a representation-prediction term to token prediction, with related text/code views and other language tasks. Its tested backbones are transformers. Some pretraining evaluations accept an answer prefix because termination was unreliable; those results do not meet our complete-output acceptance. The objective warrants consideration, while its implementation and results are separate from UOR-R4.

The [authors' implementation at ea0017c6](https://github.com/galilai-group/llm-jepa/blob/ea0017c654ad917066ff32afc88276bea8ca5f7e/finetune.py) was inspected, not executed. It constructs a Hugging Face causal language model, reads final hidden states, and combines language loss with cosine-distance representation loss by default. It also contains alternative distance objectives. The observed implementation is not a transformer-free runtime, and no Python model dependency or weights were imported. A source copy/hash is preserved only as research evidence in the project-local handoff.

## Judgment for this project

A native analogue is plausible as an **auxiliary offline objective**: learn geometric state that predicts the state of a meaningfully related continuation or alternate view, while retaining byte/EOS learning. Conceptually:

`training objective = byte/EOS prediction loss + lambda * geometric representation prediction loss`

This is a proposal, not an implemented equation in the current learner. The new term could give state and context access a signal less directly tied to a particular byte decoder. It would not by itself define which occurrence to read, provide an attention normalization, prevent incorrect source selection or generate a sentence. Our source-selection mechanism and learned emitter would still need to work and pass causal interventions.

The first useful native design would need to answer these concrete questions:

- **Views:** use genuinely related views, such as a description and verified small program, or a prefix and a declared future span. A random pair, token-ID resemblance or identical encoding is not an informative prediction target. Keep the predictor's context free of the target view during prediction.
- **Representation:** compare signed geometric state through declared geometry, retaining orientation and necessary fiber information. Root-number differences or hash-bit distance cannot become a semantic metric by naming the loss JEPA. Exact names, numbers, occurrence/version identity, negation and order must remain distinguishable where outputs require them.
- **Learning and collapse:** specify how targets are trained or held stable and test against constant-state collapse. Agreement alone can be minimized by making everything identical. Retaining the output objective is necessary for our product, but its anti-collapse effect must be measured in this discrete model rather than assumed from a transformer experiment.
- **Causal value:** compare matched training budgets with and without the term, plus shuffled-view and context-disabled controls. Check whether state diversity is useful, not merely larger, and judge unseen complete outputs, exact-value preservation and source dependence. Better latent agreement alone is insufficient.
- **Execution boundary:** implement preparation and learning in Rust. Offline floating-point calculations are allowed; any predictor needed at serving must obey the existing bounded geometric/integer/table contract. No hidden transformer, provider or dense runtime is introduced.

Recommendation: retain JEPA as a concrete candidate for improving native state learning, rather than dismissing it as vision-only or adopting it based on popularity. Complete the current controlled state/decoder test first; its result informs whether the next useful change concerns optimization, representation or an auxiliary objective. A native JEPA experiment requires its own data, collapse controls, matched resource projection and acceptance before running. No additional experiment or spending follows from this research note.
