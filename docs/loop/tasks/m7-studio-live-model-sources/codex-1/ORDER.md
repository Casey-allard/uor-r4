# Restore a verified public M1 prediction in Studio

Lab: codex. References [m7-studio-live-model-sources](../../../../roadmap/items/m7-studio-live-model-sources.md), #3440. Base: 9465b92d6e11ca0ee913ebe34aaaddff0cdf97f5.

## Cancel probe

The live catalog contains only the synthetic demo. A pinned M1 seed 1 URL loads the 19-table trunk with the published digest, but the prediction panel refuses text without its matching tokenizer. The default composer routed a request to the browser task fixture. Both defects reproduced in a fresh Chromium session.

## Scope and done check

Trace the existing publication gate, match public weights and tokenizer to their release evidence, and exercise text prediction through the normal rendered Studio controls. Repair the smallest necessary Studio and publishing surfaces, with focused regression checks. Record the artifact identity, tokenizer binding, a real top-eight result, the compute boundary and the publication state. The parent session owns review and release.

## Stop list

Use only the already released M1 artifact. Preserve existing gates and evidence. No research training, panel scoring, private artifacts, paid resources, changes to owner rules, workflow edits or direct public writes.

## Scope amendment after exact-head review

The parent Codex session reviewed #3440, M1 #2029, the M7 client path, the CPU-training release gate (#3609), the 19-table parity inventory and Charter §§3–5. It approved the decision recorded in [DECISION.md](DECISION.md) for this single prior release. The amendment covers a discrete served-forward receipt, validator and negative tests, browser download-origin checks, and repairing the existing positive staging fixture in `scripts/tests/test_release_served_forward_gate.py`. The done check now also requires raw matched CPU and native integer decision words, including the eight ordered candidates rendered in Chrome.

## Fifth review amendment

The top Codex session approved fixing the two findings at `20077ede6`. In the existing model-source loader, public M1 verifier and model-source tests, bind release verification to the card selected at load time, including a catalog refresh that drops its tokenizer pin. In the dedicated predictor and its component tests, bind ranked tokens to the submitted text even if the operator edits the field before the worker replies. Verify the staged candidate with Chrome and the pinned native receipt.

## Sixth review amendment

The parent approved the two findings at `449d87fbe`. Carry the page's generation guard into typed source changes before they mutate the selection. Exercise the bundled, published, file, URL and API paths and hold a prediction across the visible switch attempt. Check the public verifier's entire request log through the offline result, including the interval spent reading the tokenizer bytes. Use existing Studio adapter, UI, browser-verification and regression surfaces.
