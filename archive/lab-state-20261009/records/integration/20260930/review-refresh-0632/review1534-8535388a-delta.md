Codex independent source delta review at `8535388a9a48d221d26d7ec98885dd1ada0cc15e`, relative to `85b7d6b9`. No Cargo/model execution.

The uncalibrated CLI provenance now uses the same served-codec quantizer helper as `export_stack`, and the served scope text names the restored codec. The prior hardcoded-RTN inconsistency is resolved at source level. The added calibrated-QAT refusal assertion is consistent with the existing contract. No further required source correction found in this delta.

Validation boundary: the new test constructs CLI-like provenance and an export JSON object in the test; it does not invoke `geometric-stack export` or read that command's produced `export.json`. Label it helper/artifact metadata consistency, not an executed production-CLI end-to-end test. The CLI caller was inspected in this review; current-head compilation/execution is still NOT_RUN under the hold. Do not infer whole CLI execution from this source review or the manually assembled JSON assertions.

Before protected delivery, complete the applicable admitted checks and ensure the integration candidate composes with #1533/#1538, which touch the same export callers/signature region. This review resolves the named provenance delta; it is not an unconditional merge receipt.
