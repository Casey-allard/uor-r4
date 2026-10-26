# ORDER: m1-address-as-read, kimi-40 (Stage 0 copy and artifact parity)

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `6f38dd393865a0b4320a2e28bee3be675130dc16`.

## Goal

Determine whether the pinned T16_fz D11 artifact can support a measurable serving-time address oracle at all. Deliver a finite GPU copy probe and artifact parity/determinism receipt, then a scoped decision on whether a hidden-fact panel is justified. This unit says `References #2833`; it does not close the item or claim a model gain.

## Cancel probe (run first)

The address consumer and 6 finite controls are delivered. Panel-v2 replays the store but has answers in model-visible dialogue on 70/80 rows, so it cannot measure model gain. The 19/40 baseline is from another model/panel, no lawful BPB evaluator exists for this stream, and `trunk-model.d11` contains the position-stripped Stage T snapshot, not Level 3 trunk tables. A full three-arm evaluation would therefore be an instrument defect, not evidence.

`classification: instrument`. The smallest useful next step is the Stage 0 copy probe and artifact parity check, not panel-v3 or training.

## Files in scope

- `Cargo.toml` and `Cargo.lock` (workspace exclude and generated dependency pins only)
- `crates/uor-r4-object-store/**`
- `docs/mechanisms/exact-memory.md`
- `docs/labs/m1-address-as-read/**`
- `docs/harness/prep/m1-address-as-read*`
- `docs/loop/tasks/m1-address-as-read/kimi-40/**`
- `docs/research/notebook/m3-durable-memory.md`
- `docs/roadmap/items/m1-address-as-read.md`
- `docs/roadmap/README.md` (rendered by `cargo xtask roadmap render` only)

## Done when

- A Rust instrument validates the pinned T16_fz Seed 1 D11 artifact (`cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd`) and tokenizer (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`).
- The D11 scorer agrees with the sealed original GPU receipt on all 14,287 DEV rows, with top-1/top-8/admission and any aggregate position-strip difference from retained 455/1,484 recorded; every row has status 0 and two passes are byte-identical. The receipt hash is `a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e`; no host model forward is allowed.
- A 40-pair copy probe reports teacher-forced and free-running exact recovery against no-hint and random-hint controls. It passes only if both primary comparisons meet `d > 24 && d^2 > 4(b+c)`; otherwise the instrument is not sufficient for a hidden-fact oracle and no v3 follows.
- Host finite controls and full gates pass; the GPU sheet has non-author FIT and the lab's D67 pre-run review before dispatch. REPORT records exact costs, negative scope and the next branch. No BPB, training or issue close.

## Stop and decide if

- Any artifact, tokenizer, panel or identity check fails; this is an instrument stop, not a mechanism negative.
- The copy probe fails its registered non-vacuity floor: stop without building panel-v3 and record the artifact/capability limitation.
- Parity differs, outputs are nondeterministic, or the D11 shows a position-strip gap: record it before interpreting copy results.
- A required change leaves this scope, or the work would require a new model/training run rather than this zero-training probe.
