# REPORT: one-model-end-to-end, deepseek-6

Lab: deepseek. Order: [ORDER.md](ORDER.md).
Evidence:
[public-chat post-smoke baseline](../../../labs/one-model-end-to-end/public-chat-post-smoke-2026-11-02/README.md).

## What did not work, first

- The first registration was rejected because `conv-talk-01` and `conv-mem-01`
  had already been observed before it was finalized. It was preserved as
  `TEST FITNESS: NOT_FIT`.
- That smoke used `uor-chat`'s default protocol 1. TRAIN and the panel use
  protocol 2, so its `" Whilk from you."` output was an instrument mismatch,
  not a model reading.
- The corrected protocol-2 panel is still negative: neither seed recalled
  `Pickle` or `tomato` in either fixed memory probe.
- The REPL prints ids/s but not its internal per-turn seconds. The transcript
  preserves the displayed rate; the evidence record derives per-turn seconds
  from `id count / displayed rate` and retains the 24 process-level timing
  records. This timing-precision limitation does not affect the memory verdict.
- Post-run analysis rewrote the first root's aggregate in place. Its raw outputs
  are preserved but its aggregate is not authoritative. A pre-recorded fresh
  sibling rerun reproduced all 33 reply structures exactly and is now the only
  aggregate cited.
- The public artifact's own README explicitly says it is not an assistant and
  is not chat-capable. This run does not contradict that warning.

## What changed

Docs only. The task now contains a rejected pre-registration, an honest
post-smoke FIT sheet, the exact protocol-2 commands, and a two-seed evidence
record. No model, serving, tokenizer or training code changed. The change is
within the ORDER's documentation-only estimate.

## Measurements

At source head `ffe7e5491`, the pinned executable
`4f7458c4bfe3b5129033564e97f7cd7d781d838c24f79edf62c92a26109f45a2` ran the
eight fixed panel entries at protocol 2, greedy, 16 threads and 16 maximum
replies.

- 24 processes, 33 replies: 11 for seed 1, 11 for seed 2 and 11 for the
  seed-1 repeat.
- All processes exited 0 and every reply stopped by `eos` or
  `max_new_tokens`.
- All returned protocol schemas were `uor-r4.literal-role-dialogue/2`.
- Reply text independently decoded from reply ids in 33 of 33 cases.
- The seed-1 repeat matched all 11 first-pass replies in text, ids, stop
  reason and history depth.
- Memory anchors: 0 of 4 passed. Seed 1 and seed 2 both failed the `Pickle`
  and `tomato(s)` anchors.
- Peak per-process RSS: 11,592 KiB. The 24 process timing records sum to
  2.82 s; no separate orchestrator timer is asserted.
- Observed reply rates ranged from 144.47 to 350.35 ids/s.
- External spend: `$0.00`.

Exact reply text, ids, stops, histories, timings and rates are in the evidence
record linked above. Raw transcripts and `verification.json`
(`45e28a60e49369bbd46d877b9e06a429d98ad6ac64aa640a09928216626e8a4b`) are
under `local/deepseek/public-chat-post-smoke-2026-11-02-rerun/`.

## Checks run at the head

- Fresh-reviewer `TEST FITNESS: FIT`, `PRE-RUN: PASS` at `fd57d3ba8`; no
  model invocation in that review.
- `cargo test --locked -p uor-r4-tokenizer --lib`: 13 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin
  uor-chat`: 3 passed.
- Artifact-directory `sha256sum -c SHA256SUMS`: 5 of 5 passed.
- `uor-harness prep-check`: this sheet was complete and FIT.
- `uor-harness check-formats`, `ratchet d25`, and `claim-wording`: passed.
- The registered panel's preflight validated every artifact, tokenizer,
  panel, executable, source, lockfile and toolchain hash.
- Independent extraction and verification: 33 of 33 reply texts matched their
  ids; the repeat was exact; the memory anchors failed as reported.
- `git diff --check` passed for this docs-only change.
- `cargo fmt --all -- --check` reports pre-existing formatting drift in
  untouched crates on this base; it is not a source gate for this branch.

## Skills used

`experiment-prereg` governed the FIT and pre-run structure.
`cost-and-compute-admission` governed the local zero-cost projection.
`panel-scoring` confirmed this ungraded panel does not support a capability or
headline claim.

## Still open

The public dual-c131 artifacts remain connected product smoke targets, not
conversational models. Training and qualification of a chat-capable model stay
on Claude's independent track. This ungraded run does not establish memory
capability, answer correctness, transfer or language quality. The serving
client also reports that its retained-bundle contract expects 256 context
positions while these artifacts declare 192; no fixed-panel turn reached the
artifact limit.

## Plain summary

The packaged integer model can be loaded and run on this workstation, and its
output is repeatable. But when asked to remember a puppy's name or what
Grandpa grows, both copies produce generic word fragments instead of the
requested word. The serving path works; this model is not ready to chat.
