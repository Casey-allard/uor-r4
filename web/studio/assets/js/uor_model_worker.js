// Browser Web Worker connecting Studio (`web/studio/`) to the `crates/uor-r4-integer`
// (`--features serving-wasm`) integer execution engine:
// `init`, `tickClock`, `loadStack`, `step`, `saveSession`, `restoreSession`,
// `weightsPerToken`, `allocEvents`, `leakedModels`.
//
// Zero remote network calls are made by this model worker. Every capability field
// comes directly from the loaded `UORLUT01` stack header or the WASM exports.
// Prompt text is mapped to token IDs by a deterministic UTF-8 byte-mod stand-in
// (`rawBytes[i] % vocabSize`) until a stack-bound tokenizer artifact is bundled.

const ENGINE_TAG = 'uor-r4-integer/serving-wasm';
const LUT_MAGIC = 'UORLUT01';
const LUT_SCHEMA = 'uor-r4.lut-stack/1';
const PROMPT_TOKENIZER_LABEL = 'UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize)';
const MAX_SESSIONS = 32;

let wasmModule = null;
let wasmInitPromise = null;
let capabilities = null;
let activeHandle = null;
let initialSessionBytes = null;
let activeJob = null;
const sessions = new Map();
const sessionEpochs = new Map();

function bumpSessionEpoch(conversationId) {
    if (!conversationId) return 0;
    const next = (sessionEpochs.get(conversationId) || 0) + 1;
    sessionEpochs.set(conversationId, next);
    return next;
}

async function ensureWasmLoaded() {
    if (!wasmInitPromise) {
        wasmInitPromise = (async () => {
            const mod = await import('../../pkg/uor_r4_integer.js');
            if (globalThis.__UOR_WASM_BYTES__) {
                await mod.default({ module_or_path: globalThis.__UOR_WASM_BYTES__ });
            } else {
                await mod.default();
            }
            wasmModule = mod;
            return mod;
        })().catch((error) => {
            wasmInitPromise = null;
            throw error;
        });
    }
    return wasmInitPromise;
}

function parseUorLutHeader(bytes) {
    if (!(bytes instanceof Uint8Array) || bytes.byteLength < 16) {
        throw new Error('Expected a binary UORLUT01 stack file (at least 16 bytes).');
    }
    const magic = new TextDecoder('ascii').decode(bytes.subarray(0, 8));
    if (magic !== LUT_MAGIC) {
        throw new Error(`Unrecognized stack header magic "${magic}"; expected "${LUT_MAGIC}".`);
    }
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const jsonLenBig = view.getBigUint64(8, true);
    const jsonLen = Number(jsonLenBig);
    if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > bytes.byteLength) {
        throw new Error(`Invalid UORLUT01 header length (${jsonLen} bytes).`);
    }
    const headerJson = new TextDecoder('utf-8').decode(bytes.subarray(16, 16 + jsonLen));
    const parsed = JSON.parse(headerJson);
    if (!parsed || parsed.schema !== LUT_SCHEMA || typeof parsed.shape !== 'object' || !parsed.shape) {
        throw new Error(`Unsupported stack schema "${parsed?.schema || 'unknown'}"; expected "${LUT_SCHEMA}".`);
    }
    const synthetic = Boolean(parsed.source && parsed.source.synthetic);
    return {
        schema: parsed.schema,
        shape: {
            vocab: Number(parsed.shape.vocab) || 0,
            width: Number(parsed.shape.width) || 0,
            heads: Number(parsed.shape.heads) || 0,
            mlp: Number(parsed.shape.mlp) || 0,
            pattern: String(parsed.shape.pattern || ''),
            read: String(parsed.shape.read || ''),
            rotation: Boolean(parsed.shape.rotation),
            context: Number(parsed.shape.context) || 0,
        },
        group: typeof parsed.group === 'number' ? parsed.group : null,
        synthetic,
        status_label: synthetic ? 'synthetic stand-in' : 'unqualified stack artifact',
    };
}

function requireModel() {
    if (!capabilities || activeHandle === null || !initialSessionBytes) {
        throw new Error('Load a UORLUT01 stack file (.lut / .bin) before generating. No model is bundled.');
    }
}

function ensureSessionEntry(conversationId) {
    requireModel();
    if (!conversationId) {
        throw new Error('A conversation identity is required.');
    }
    if (!sessions.has(conversationId)) {
        if (sessions.size >= MAX_SESSIONS) {
            throw new Error('32 sessions are open. Export and close an unused conversation.');
        }
        if (!sessionEpochs.has(conversationId)) {
            sessionEpochs.set(conversationId, 1);
        }
        sessions.set(conversationId, {
            bytes: new Uint8Array(initialSessionBytes),
            lastToken: null,
        });
    }
    return sessions.get(conversationId);
}

function cancelActive() {
    if (activeJob) {
        activeJob.cancelled = true;
    }
}

function encodePromptTokens(payload, vocabSize) {
    if (Array.isArray(payload.tokenIds) && payload.tokenIds.length > 0) {
        return payload.tokenIds.map((t) => {
            const id = Number(t);
            if (!Number.isInteger(id) || id < 0 || id >= vocabSize) {
                throw new Error(`Token ID ${t} is outside stack vocab range 0..${vocabSize - 1}.`);
            }
            return id;
        });
    }
    const userMessage = (payload.messages || []).filter((m) => m.role === 'user').pop();
    const text = typeof userMessage?.content === 'string' ? userMessage.content.trim() : '';
    if (!text) {
        throw new Error('A non-empty prompt or tokenIds array is required.');
    }
    const rawBytes = new TextEncoder().encode(text);
    const out = new Array(rawBytes.length);
    for (let i = 0; i < rawBytes.length; i++) {
        out[i] = rawBytes[i] % vocabSize;
    }
    return out;
}

function formatTokenChunk(tokenId, index) {
    return index === 0 ? `${tokenId}` : ` ${tokenId}`;
}

self.onmessage = async ({ data }) => {
    const { action, id, payload = {} } = data || {};
    try {
        if (action === 'cancel' || action === 'stop_generation') {
            cancelActive();
            return;
        }
        if (action === 'get_storage_status') {
            const storage = self.navigator?.storage?.estimate
                ? await self.navigator.storage.estimate()
                : null;
            self.postMessage({
                action: 'storage_status_result',
                storage: storage
                    ? {
                        usageMB: (storage.usage / 1048576).toFixed(1),
                        quotaMB: (storage.quota / 1048576).toFixed(1),
                    }
                    : null,
            });
            return;
        }
        if (action === 'purge_all_caches') {
            throw new Error('The loaded stack is held in worker memory. Reload the page to unload it; saved conversations are preserved.');
        }
        if (action === 'prewarm') {
            await ensureWasmLoaded();
            self.postMessage({
                action: 'runtime_loaded',
                engine: ENGINE_TAG,
                ready: Boolean(capabilities),
                capabilities,
            });
            return;
        }
        if (action === 'load_model') {
            if (activeJob) {
                throw new Error('Stop generation before loading a stack artifact.');
            }
            await ensureWasmLoaded();
            const raw = payload.bytes instanceof Uint8Array
                ? payload.bytes
                : new Uint8Array(payload.bytes || []);
            if (!raw.byteLength) {
                throw new Error('The selected stack file is empty.');
            }
            const headerInfo = parseUorLutHeader(raw);
            wasmModule.tickClock(1n);
            const handle = wasmModule.loadStack(raw);
            const blankState = wasmModule.saveSession(handle);
            const wpt = Number(wasmModule.weightsPerToken(handle));
            const allocCount = Number(wasmModule.allocEvents());
            const leaked = Number(wasmModule.leakedModels());

            activeHandle = handle;
            initialSessionBytes = new Uint8Array(blankState);
            sessions.clear();
            sessionEpochs.clear();

            capabilities = {
                engine: ENGINE_TAG,
                handle,
                file_name: payload.fileName || null,
                artifact_bytes: raw.byteLength,
                schema: headerInfo.schema,
                shape: headerInfo.shape,
                group: headerInfo.group,
                synthetic: headerInfo.synthetic,
                status_label: headerInfo.status_label,
                tokenizer_label: PROMPT_TOKENIZER_LABEL,
                weights_per_token: wpt,
                checkpoint_bytes: initialSessionBytes.byteLength,
                alloc_events: allocCount,
                leaked_models: leaked,
            };

            self.postMessage({
                action: 'model_loaded',
                capabilities,
                bytes: raw.byteLength,
            });
            return;
        }

        requireModel();

        if (action === 'close_session' || action === 'reset_session') {
            if (activeJob) {
                throw new Error('Stop generation before closing or resetting a session.');
            }
            const convId = payload.conversationId;
            if (convId) {
                sessions.delete(convId);
                bumpSessionEpoch(convId);
            }
            wasmModule.restoreSession(activeHandle, initialSessionBytes);
            self.postMessage({
                action: 'session_reset',
                id,
                conversationId: convId,
            });
            return;
        }

        if (action === 'export_session') {
            if (activeJob) {
                throw new Error('Stop generation before exporting a session checkpoint.');
            }
            const entry = ensureSessionEntry(payload.conversationId);
            self.postMessage({
                action: 'session_exported',
                id,
                conversationId: payload.conversationId,
                bytes: new Uint8Array(entry.bytes),
                lastToken: entry.lastToken,
            });
            return;
        }

        if (action === 'import_session') {
            if (activeJob) {
                throw new Error('Stop generation before importing a session checkpoint.');
            }
            const convId = payload.conversationId;
            if (!convId) {
                throw new Error('A conversation identity is required to import a checkpoint.');
            }
            const rawCheckpoint = payload.bytes instanceof Uint8Array
                ? payload.bytes
                : new Uint8Array(payload.bytes || []);
            if (!rawCheckpoint.byteLength) {
                throw new Error('Checkpoint payload is empty.');
            }
            wasmModule.restoreSession(activeHandle, rawCheckpoint);
            const verifiedBytes = wasmModule.saveSession(activeHandle);
            bumpSessionEpoch(convId);
            sessions.set(convId, {
                bytes: new Uint8Array(verifiedBytes),
                lastToken: typeof payload.lastToken === 'number' ? payload.lastToken : null,
            });
            self.postMessage({
                action: 'session_imported',
                id,
                conversationId: convId,
                bytes: verifiedBytes.byteLength,
            });
            return;
        }

        if (action !== 'generate') {
            throw new Error(`Unknown worker action: ${action}`);
        }
        if (activeJob) {
            throw new Error('Another generation is active. Wait for its completion or cancellation.');
        }

        const convId = payload.conversationId || 'default';
        const entry = ensureSessionEntry(convId);
        const startEpoch = sessionEpochs.get(convId) || 1;
        wasmModule.restoreSession(activeHandle, entry.bytes);

        const promptTokens = encodePromptTokens(payload, capabilities.shape.vocab);
        const maxTokens = Math.min(4096, Math.max(1, Number(payload.options?.max_new_tokens) || 32));
        const job = { id, conversationId: convId, epoch: startEpoch, cancelled: false };
        activeJob = job;

        const started = performance.now();
        try {
            let tok = promptTokens[0];
            for (let i = 1; i < promptTokens.length; i++) {
                wasmModule.tickClock(1n);
                wasmModule.step(activeHandle, tok);
                tok = promptTokens[i];
            }

            const allocBefore = wasmModule.allocEvents();
            const tokenIds = [];
            let fullText = '';

            for (let stepIdx = 0; stepIdx < maxTokens && !job.cancelled; stepIdx++) {
                if ((sessionEpochs.get(convId) || 0) !== startEpoch || !sessions.has(convId)) {
                    job.cancelled = true;
                    break;
                }
                wasmModule.tickClock(1n);
                const [nextTok, marginQ16, stepCount] = wasmModule.step(activeHandle, tok);
                tok = nextTok;
                tokenIds.push(tok);
                const chunk = formatTokenChunk(tok, stepIdx);
                fullText += chunk;

                const durationSec = (performance.now() - started) / 1000;
                self.postMessage({
                    action: 'stream_token',
                    id,
                    chunk,
                    fullText,
                    tokenId: tok,
                    marginQ16,
                    stepCount,
                    tokenCount: tokenIds.length,
                    tps: durationSec > 0 ? tokenIds.length / durationSec : null,
                });

                if ((stepIdx + 1) % 8 === 0) {
                    await new Promise((resolvePromise) => setTimeout(resolvePromise, 0));
                }
            }

            const allocAfter = wasmModule.allocEvents();
            const allocDelta = Number(allocAfter - allocBefore);
            if ((sessionEpochs.get(convId) || 0) === startEpoch && sessions.has(convId)) {
                const savedBytes = wasmModule.saveSession(activeHandle);
                sessions.set(convId, {
                    bytes: new Uint8Array(savedBytes),
                    lastToken: tok,
                });
            }

            capabilities.alloc_events = Number(allocAfter);
            capabilities.leaked_models = Number(wasmModule.leakedModels());

            const durationSec = (performance.now() - started) / 1000;
            const stoppedBy = job.cancelled ? 'cancelled' : 'length';
            self.postMessage({
                action: 'generate_complete',
                id,
                fullText,
                tokenIds,
                tokenCount: tokenIds.length,
                durationSec,
                tps: durationSec > 0 ? tokenIds.length / durationSec : null,
                stoppedBy,
                weightsPerToken: capabilities.weights_per_token,
                allocDelta,
                allocEvents: capabilities.alloc_events,
                statusLabel: capabilities.status_label,
                tokenizerLabel: capabilities.tokenizer_label,
            });
        } finally {
            if (activeJob === job) {
                activeJob = null;
            }
        }
    } catch (error) {
        self.postMessage({
            action: action === 'generate' ? 'generate_error' : 'operation_error',
            id,
            error: String(error),
        });
    }
};
