// r4_worker.js — Web Worker for WASM event-loop isolation
// Offloads uor-r4 candidate scoring and generation off the main DOM thread.

let router = null;
let wasmInitialized = false;

// #790 item 5: try to install a static-mode R4G1 bundle (scored graph
// preferred, converter carryover fallback — the same preference order the
// CLI and server use) so r4g1/transformerless selections can actually
// serve through generate_r4g1_response. Missing files keep the honest
// geometric fallback; a typed installer refusal is logged, never hidden.
async function tryInstallStaticR4g1Bundle(scope, wasmModule) {
    if (typeof wasmModule.set_r4g1_bundle !== 'function') return;
    try {
        for (const graphPath of ['./graph/score.r4g1', './compiled.r4g1']) {
            const graphRes = await fetch(graphPath);
            if (!graphRes.ok) continue;
            const tokRes = await fetch('./tokenizer.bin');
            if (!tokRes.ok) break;
            const graph = new Uint8Array(await graphRes.arrayBuffer());
            const tokenizer = new Uint8Array(await tokRes.arrayBuffer());
            wasmModule.set_r4g1_bundle(graph, tokenizer);
            console.log(`[${scope}] R4G1 bundle installed from ${graphPath}`);
            return;
        }
        console.log(`[${scope}] no static R4G1 bundle found; geometric fallback stays active`);
    } catch (err) {
        console.warn(`[${scope}] R4G1 bundle install refused:`, err);
    }
}

self.onmessage = async function (e) {
    const { type, id, payload } = e.data || {};
    
    switch (type) {
        case 'INIT_ENGINE': {
            try {
                const wasmModule = await import('./pkg/uor_r4_wasm_router.js');
                const { default: init, UorR4Router } = wasmModule;
                await init();
                // #790 item 5: this global was never assigned before, so
                // the r4g1/transformerless branches below could never run.
                self.wasm_module = wasmModule;
                await tryInstallStaticR4g1Bundle('r4_worker', wasmModule);
                router = new UorR4Router(1.2);
                wasmInitialized = true;
                if (router.get_vocab_size() === 0) {
                    router.index_default_corpus();
                }
                self.postMessage({
                    type: 'ENGINE_READY',
                    id,
                    success: true,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            } catch (err) {
                console.error("[r4_worker] Failed to init local WASM:", err);
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: String(err)
                });
            }
            break;
        }

        case 'GENERATE_RESPONSE': {
            if (!wasmInitialized || !router) {
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: "WASM Engine not initialized in worker"
                });
                return;
            }

            try {
                const { text, identity, max_tokens, temperature, gamma, selectedEngine } = payload;
                let responseText = "";
                let generationMode = "geometric-local-worker";

                if ((selectedEngine === "transformerless" || selectedEngine === "r4g1") && self.wasm_module?.generate_r4g1_response) {
                    try {
                        const r4g1Res = self.wasm_module.generate_r4g1_response(text, max_tokens);
                        if (r4g1Res) {
                            responseText = r4g1Res;
                            generationMode = selectedEngine === "r4g1" ? "r4g1-zero-multiply-wasm" : "transformerless-r4g1-wasm";
                        }
                    } catch (err) {
                        console.warn("[r4_worker] R4G1 WASM evaluation failed, falling back to geometric:", err);
                    }
                }

                if (!responseText) {
                    const geomResult = router.generate_geometric_response(
                        text,
                        identity || "null_dev_00",
                        max_tokens || 25,
                        temperature || 0.7,
                        10.0,
                        4.0,
                        gamma || 0.5
                    );
                    responseText = geomResult.text || "Manifold resonance too sparse for synthesis.";
                    generationMode = (selectedEngine === "transformerless" || selectedEngine === "r4g1") ? "geometric-fallback-wasm" : "geometric-local-worker";
                }

                router.index_sentence(responseText, identity || "null_dev_00");

                self.postMessage({
                    type: 'GENERATION_COMPLETE',
                    id,
                    responseText,
                    generationMode,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            } catch (err) {
                console.error("[r4_worker] Generation error:", err);
                self.postMessage({
                    type: 'ENGINE_ERROR',
                    id,
                    error: String(err)
                });
            }
            break;
        }

        case 'INDEX_SENTENCE': {
            if (router && payload?.text) {
                router.index_sentence(payload.text, payload.identity || "null_dev_00");
                self.postMessage({
                    type: 'INDEX_COMPLETE',
                    id,
                    indexedSentences: router.get_total_indexed_sentences()
                });
            }
            break;
        }

        default:
            console.warn("[r4_worker] Unknown message type:", type);
            break;
    }
};
