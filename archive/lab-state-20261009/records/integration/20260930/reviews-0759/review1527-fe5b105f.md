Codex exact-head delta review: source corrections APPROVED at fe5b105fe3590a97b0ce56bb2e21b7713926e1ae, compared with 067a5fac, reusing the earlier complete diagnostic-source review.

The ordinary development path again calls model.target_nll through the shared accumulator, preserving the active float/served head; the explicit diagnostic path alone passes the supplied head. Tensor is imported, the test configurations enumerate actual StackConfig fields, and exact head dimensions are rejected before row scoring. Logits are detached. S1 uses the retained runs' actual [1,1] mixture. Distortion retains per-intervention replies already computed, including component/layer/map only-quantized and leave-out cases, and prints unavailable activation data honestly. The obsolete survives_export aliases and isolated-kernel attribution are removed.

Executed: direct rustfmt --check on all four changed Rust files and git diff --check pass. No Cargo, diagnostic execution, training or historical rerun was performed. This is source approval, not a merge-ready validation receipt.

Remaining delivery validation: compile both diagnostic binary targets and the changed library tests; execute the explicit-head and malformed-head witnesses, and retain a focused served-mode development equivalence check against target_nll (the previously requested regression is not added in this delta). Use the shared runner after valid admission, and check the combined exporter/saved-transport candidate where it interacts. Preserve historical metrics under their original evaluator identity. The report's 93%/7% split remains an arithmetic decomposition, not an independently identified optimization cause.

No additional broad source audit or historical training replay is required by this review.
