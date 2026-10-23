# M4 served UORLUT01 quality, export gap and CLI — Pre-Registration (DRAFT)

TEST FITNESS: PENDING

- **Line:** M4 (#2032), `m4-export-uorlut01-cli` (#2859), host preparation in PR #3087 (`kimi-31`).
- **Question:** can one chosen model be exported as `uor-r4.lut-discrete/1`, served through `uor-chat`, and meet the registered held-out quality bar under D11?
- **Status:** draft only. No artifact is chosen, no held-out chat stream or lawful likelihood is registered, and no GPU run, FIT, spend or owner release is requested.

## Blocking inputs before FIT

1. **Chosen checkpoint:** a saved trained discrete checkpoint accepted by `m1-close-saved-model` and `m3-qualify-memory`, with exact path, SHA-256, snapshot identity and source commit. Its normalization mode must be `ScaleMatchedSnapV1`; the current exporter refuses control arms. **PENDING.**
2. **Frozen held-out chat stream:** exact file identities, SHA-256, byte count, eligible target positions and denominator. The `ft/dev` M1 panel is not silently substituted for this M4 held-out stream. **PENDING.**
3. **Lawful likelihood and scorer:** the implementation and geometric formula delivered by `m4-lawful-likelihood-bpb` (#2861), including non-negativity and exact unit mass over the full alphabet, bits calculation without a banned function, source SHA-256 and count-table control. **PENDING.** No BPB formula or score is invented here.
4. **Audit coverage:** a named native and WebAssembly audit root for the discrete CLI/evaluator. The existing 44 native and 7 WebAssembly symbol scopes cover their named legacy/stack roots, not the new `--discrete` CLI root. Extending an audit script is outside kimi-31's scope and remains **PENDING.**

## Base artifact

- Container: `UORLUT01`, schema `uor-r4.lut-discrete/1`, `matrices = []`, with `discrete.table` (`i32`), `discrete.cells` (`u32`), `discrete.routes` (`u32`) and `discrete.gates` (`i32`). Snapshot/source identity is bound in the header.
- Serving entry point: `uor-chat --discrete MODEL.lut --prompt-ids CSV`, calling `IntegerDiscreteModel::evaluate_prefix` greedily. Text chat and `UR4SESS1` persistence are outside this sheet's first CLI control.
- Pin the exact merged source commit, checkpoint SHA-256, exported artifact SHA-256, tokenizer/panel identities and scorer commit before FIT. **All currently PENDING.**

## Data split

Register the held-out chat stream before any scoring: path/object identity, SHA-256, bytes, documents, target rows, context policy and byte denominator. Keep development selection separate from the sealed evaluation; do not open a fresh slice during instrument development. Record contamination/exclusion checks and the count-table baseline on exactly the same positions.

## Bar

1. **Held-out quality:** served score **at most 0.90 bits per byte** on the registered stream. The #2861 scorer must fix the numerator, denominator and token-to-byte mapping before FIT; this draft does not redefine the existing milestone bar.
2. **Export gap (CHARTER §3.7):** evaluate the offline candidate and served integer artifact with the same evaluator, positions and tie order. For a lossless table-native discrete export, scorer inputs and row outputs must be identical and the export gap exactly zero; any difference is an instrument fault. For any D37 float-trained candidate, report served-minus-float BPB and its two-seed noise band; that band is **PENDING** and cannot be invented from the old M1 top-1 bands.
3. **Static instructions:** zero float, softmax/squashing, dense matmul and mul/div in the declared serving kernel. Run both existing audits and the resolved discrete-root coverage from blocking input 4; absence outside those named roots is not claimed.
4. **Fresh-process identity:** two fresh CLI processes loading the same exported bytes and prompt must emit identical generated ids, equal to direct `evaluate_prefix` predictions. Record artifact SHA-256 and all ids.
5. **Throughput:** report prompt-reading (prefill) and reply-writing (autoregressive decode) separately on the client laptop: 1 warmup + 20 timed runs, all raw samples, median and nearest-rank p95, hardware metadata and parameter accesses. Energy is UNAVAILABLE unless measured.

## Controls

- Wrong-schema, bad-token, over-context, nonzero-temperature and unsupported text/session inputs must fail closed.
- A toy finite artifact must match direct library predictions in two fresh processes before any trained-artifact run.
- Score the lawful count-table baseline with the same #2861 instrument. A constant or copy control must not pass a non-vacuity check.
- Flip one nonzero route action or writer cell in an in-memory control; the parity/hash check must detect it.

## Stop

- Missing or mismatched checkpoint, panel, scorer, source or artifact identity: **NOT_RUN**, instrument incomplete.
- Any nonzero forbidden instruction in an audited serving root, any nonzero lossless-export mismatch, or a failed planted-fault control: diagnostic stop and instrument repair, never a mechanism negative.
- A served BPB above 0.90 with a valid instrument closes that model/export configuration only. It does not close D11 serving or M4.
- Stop before allocation if projected cost exceeds the approved sheet; preserve receipts and never cancel a healthy leased run for a self-set timer.

## Cost

Proposed cap, requiring the normal FIT and owner release: one RTX 5090 pod for at most 15 minutes (about $0.89/hour, **at most $0.25**) for export and registered evaluation. Laptop builds, static checks and product serving timing are $0 external compute, projected at most 30 minutes. No pod or spend is authorized by this draft.

## Minimum test power

Use at least 200 fresh held-out rows (CHARTER §4.3); the exact denominator is fixed only with blocking input 2. Report every row, denominator, uncertainty and any two-seed dispersion used for an export-gap decision. A finite CLI fixture is an instrument control, not language evidence.

## Steps

1. Resolve all four blocking inputs; pin the sheet hash and obtain a non-author TEST FITNESS: FIT review.
2. Obtain Casey's approval in this lab's chat for this exact sheet, then run `scripts/pod/uor-pod status` and lease only the declared GPU.
3. On the GPU, verify identities, export the checkpoint, and evaluate the offline candidate and integer artifact on the same registered rows. Save artifacts, scores and checksums off the pod before shutdown.
4. On the client laptop, rerun the native/WebAssembly audits, the two-fresh-process CLI control, and separate prefill/decode timing on the final artifact.
5. Record the export gap, BPB, raw timings, costs and exact negative scope in `docs/research/notebook/m4-serving.md` and the item REPORT. M4 remains held until its full acceptance is independently checked.

## Outcomes

- **Pass:** all registered bars and audits pass; submit the exact artifact and evidence for M4 acceptance.
- **Quality miss:** close only that checkpoint/export configuration and select the next registered experiment.
- **Instrument stop:** repair the named instrument or audit gap; do not claim a model result.

TEST FITNESS: PENDING
