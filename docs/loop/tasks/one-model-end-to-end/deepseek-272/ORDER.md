# ORDER: one-model-end-to-end, deepseek-272 (aligned scenario readout)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Base: `8b818936c`.

## Goal

Measure the current served geometric artifact on the aligned scenario corpus
without disturbing Claude's E8 work. The readout must use the exact dialogue
encoder and CPU served-reply path, then report complete replies, exact
references, asked values, distractors and abstentions by scenario archetype.

Run the same readout before and after a bounded CPU research dose using the
existing rank-margin corpus driver. This is development evidence, not GPU
qualification or a capability claim.

## Cancel probe

1. `scenario-corpus check` on the production TRAIN, DEV and Fresh files.
2. A zero-step `cpu-corpus-driver` run over TRAIN with the public trained FBS
   artifact, 1,024 held-in evaluation rows and its exact CPU/serving audit.
3. A 256-target CPU update smoke, recording accepted updates, throughput and
   panel movement.

Result: GO. The corpus regenerates byte for byte; the trainer accepts all
20,000 documents and 185,023 training targets; zero-update parity is exact.
The 256-target smoke ran at 51.9 targets/s with 36 accepted updates, but top-1,
top-8 and target admission did not move. A longer blind run is therefore not
justified until the scenario readout exists.

## Files in scope

- `crates/uor-r4-cpu-train/Cargo.toml`
- `crates/uor-r4-cpu-train/src/lib.rs`
- `crates/uor-r4-cpu-train/src/scenario_grade.rs`
- `crates/uor-r4-cpu-train/src/bin/cpu-scenario-grade.rs`
- `docs/loop/tasks/one-model-end-to-end/deepseek-272/**`
- `docs/research/notebook/cpu-training.md`
- `docs/research/notebook/data-tokenizer.md`
- `docs/roadmap/items/one-model-end-to-end.md`

Corpus files, trained artifacts and generated reports remain under ignored
`local/` paths. No float-trainer, CUDA, serving-format, bar, milestone or
Claude-owned file is in scope.

## Done when

1. A Rust binary validates the artifact, tokenizer, scenario JSONL and split,
   and refuses mismatched or malformed inputs.
2. Prompts are created only through `DialogueProtocol::literal_roles_v2` and
   `encode_assistant_prefix`; generation uses the existing CPU served path.
3. Reports give complete/EOS, exact-reference, asked-value, distractor,
   abstention and prompt-ineligible counts overall and by archetype, with
   bounded deterministic case selection.
4. Focused tests cover split/hash refusal, deterministic sampling, value and
   distractor scoring, abstention, and a non-vacuous generated reply.
5. The public trained FBS artifact is graded on DEV and Fresh before training.
6. One bounded CPU rank-margin dose is selected from the smoke throughput and
   pre-registered here before it starts; the final artifact is graded on the
   same DEV and Fresh cases.
7. The REPORT records negatives first, exact commands, hashes, timings, CPU
   utilisation, accepted/refused updates and the before/after result. It makes
   no GPU, language or useful-chat claim.
8. Focused tests, release build, formatting, clippy for changed code, report
   checks and local PR checks pass, or a pre-existing warning is named.

## Bounded dose

The pre-registered dose is 16,384 rank-margin steps with the driver defaults
(one coordinate, margin 1 Q12, step 1 Q12), seed 272, evaluation stride 2 and
a 1,024-row internal panel. At the measured 51.9 updates/s this is about
5.3 minutes. No second dose runs in this unit. A negative result closes this
configuration, not the mechanism.

## Stop and ask if

- the scenario cases cannot be encoded without changing the generator;
- the CPU trainer needs an artifact shape or source change outside the listed
  CPU crate files;
- a run would overlap, interrupt or modify Claude's active E8 work;
- the bounded dose exceeds 20 minutes on this workstation;
- the before/after case sets or artifact identities differ.

STOP and report when done. Do not start an unregistered GPU run or another
mechanism experiment.
