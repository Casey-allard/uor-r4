// Native model worker. Every emitted byte comes from the explicitly loaded artifact.
let wasmModule;
let wasmInitPromise;
let capabilities;
let activeJob = null;
const sessions = new Map();
const MAX_SESSIONS = 32;

async function ensureWasmLoaded() {
    if (!wasmInitPromise) {
        wasmInitPromise = (async () => {
            const module = await import('../../pkg/uor_r4_wasm_router.js');
            await module.default();
            wasmModule = module;
            return module;
        })().catch(error => { wasmInitPromise = null; throw error; });
    }
    return wasmInitPromise;
}
function requireModel() {
    if (!capabilities) throw new Error('Load a retained native model JSON file before generating. No model is bundled.');
}
function sessionFor(conversationId) {
    requireModel();
    if (!conversationId) throw new Error('A conversation identity is required.');
    if (!sessions.has(conversationId)) {
        if (sessions.size >= MAX_SESSIONS) throw new Error('32 sessions are open. Export and close an unused conversation.');
        sessions.set(conversationId, wasmModule.native_geometric_create_session(conversationId, 'studio-user', 'project-uor'));
    }
    return sessions.get(conversationId);
}
function cancelActive() {
    if (activeJob) {
        activeJob.cancelled = true;
        wasmModule.native_geometric_cancel(activeJob.handle);
    }
}
function closeSession(conversationId) {
    const handle = sessions.get(conversationId);
    if (handle !== undefined) {
        if (activeJob?.handle === handle) cancelActive();
        wasmModule.native_geometric_free_session(handle);
        sessions.delete(conversationId);
    }
}
self.onmessage = async ({ data }) => {
    const { action, id, payload = {} } = data || {};
    try {
        if (action === 'cancel' || action === 'stop_generation') { cancelActive(); return; }
        if (action === 'get_storage_status') {
            const storage = self.navigator.storage?.estimate ? await self.navigator.storage.estimate() : null;
            self.postMessage({ action: 'storage_status_result', storage: storage ? { usageMB: (storage.usage / 1048576).toFixed(1), quotaMB: (storage.quota / 1048576).toFixed(1) } : null });
            return;
        }
        if (action === 'purge_all_caches') throw new Error('The loaded model is held in memory. Reload the page to unload it; saved conversations are preserved.');
        if (action === 'prewarm') {
            await ensureWasmLoaded();
            self.postMessage({ action: 'runtime_loaded', ready: Boolean(capabilities), capabilities });
            return;
        }
        if (action === 'load_model') {
            if (activeJob) throw new Error('Stop generation before loading a model.');
            await ensureWasmLoaded();
            const bytes = new Uint8Array(payload.bytes || []);
            if (!bytes.length) throw new Error('The model file is empty.');
            const loaded = JSON.parse(wasmModule.native_geometric_init(bytes));
            sessions.clear();
            capabilities = loaded;
            self.postMessage({ action: 'model_loaded', capabilities, bytes: bytes.length });
            return;
        }
        requireModel();
        if (action === 'close_session' || action === 'reset_session') {
            closeSession(payload.conversationId);
            self.postMessage({ action: 'session_reset', id });
            return;
        }
        if (action === 'export_session') {
            if (activeJob) throw new Error('Stop generation before exporting a checkpoint.');
            const bytes = wasmModule.native_geometric_export_session(sessionFor(payload.conversationId));
            self.postMessage({ action: 'session_exported', id, conversationId: payload.conversationId, bytes });
            return;
        }
        if (action === 'import_session') {
            if (activeJob) throw new Error('Stop generation before importing a checkpoint.');
            wasmModule.native_geometric_import_session(sessionFor(payload.conversationId), new Uint8Array(payload.bytes));
            self.postMessage({ action: 'session_imported', id });
            return;
        }
        if (action !== 'generate') throw new Error(`Unknown worker action: ${action}`);
        if (activeJob) throw new Error('Another generation is active. Wait for its completion or cancellation.');
        const handle = sessionFor(payload.conversationId);
        const userMessage = (payload.messages || []).filter(m => m.role === 'user').pop();
        if (typeof userMessage?.content !== 'string' || !userMessage.content.trim()) throw new Error('A nonempty text prompt is required.');
        const maxTokens = Math.min(4096, Math.max(1, payload.options?.max_new_tokens || 256));
        const job = { id, handle, cancelled: false };
        activeJob = job;
        let fullText = '', tokenCount = 0;
        const started = performance.now();
        try {
            wasmModule.native_geometric_ingest(handle, userMessage.content);
            let stoppedBy = 'length';
            while (tokenCount < maxTokens && !job.cancelled) {
                const result = JSON.parse(wasmModule.native_geometric_generate_step(handle, Math.min(8, maxTokens - tokenCount)));
                const chunk = result.text || '';
                fullText += chunk;
                tokenCount += result.token_count;
                stoppedBy = result.stopped_by;
                const durationSec = (performance.now() - started) / 1000;
                self.postMessage({ action: 'stream_token', id, chunk, fullText, tokenCount, tps: durationSec > 0 ? tokenCount / durationSec : null });
                if (stoppedBy !== 'length' || result.token_count === 0) break;
                // Yield between actual native token batches so cancellation can be received.
                await new Promise(resolve => setTimeout(resolve, 0));
            }
            if (job.cancelled || stoppedBy === 'length') {
                const finished = JSON.parse(wasmModule.native_geometric_finish_generation(handle));
                fullText += finished.text || '';
                if (job.cancelled) stoppedBy = 'cancelled';
            }
            const durationSec = (performance.now() - started) / 1000;
            self.postMessage({ action: 'generate_complete', id, fullText, tokenCount, durationSec, tps: durationSec > 0 ? tokenCount / durationSec : null, stoppedBy: job.cancelled ? 'cancelled' : stoppedBy });
        } finally {
            if (activeJob === job) activeJob = null;
        }
    } catch (error) {
        self.postMessage({ action: action === 'generate' ? 'generate_error' : 'operation_error', id, error: String(error) });
    }
};
