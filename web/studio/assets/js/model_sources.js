/**
 * Studio model-source manager.
 *
 * Owns the "where does generation run" choice for the Studio:
 *   - Local WASM engine: a UORLUT01 stack artifact loaded into the existing Web Worker
 *     (bundled synthetic demo, a local .lut/.bin file, or a user-initiated URL fetch).
 *   - OpenAI-compatible API: an HTTP endpoint the user configures. Only used when the user
 *     explicitly selects it; requests are user-initiated and never happen at page load.
 *
 * The page makes no external/off-origin request at load: it reads only the same-origin
 * ./models.json catalog and its bundled assets (the static scan and runtime trap enforce
 * external []). Ordinary API keys are never bundled and are
 * stored only in this browser's localStorage; the native bootstrap's ephemeral launch token stays in memory instead. The exported object is also available at
 * window.UORModelSources for other Studio modules; every state change dispatches a
 * "uor-model-source-changed" CustomEvent on the document.
 */

import { normalizeNativeApiUrl } from './desktop_origin.js';
import { sha256Hex, validateTokenizerArtifact } from './uor_tokenizer_artifact.js';

/**
 * Public interface:
 *   list()                              -> { active, wasm, api }
 *   active()                            -> { kind, id }
 *   apiConfig(id?)                      -> config | null
 *   loadLocalFile(file)                 -> Promise<{ bytes, fileName, header, meta, source }>
 *   loadFromUrl(url)                    -> Promise<{ bytes, fileName, header, meta, source }>
 *   loadPublished(idOrEntry)            -> Promise<{ bytes, fileName, header, meta, source }>
 *
 * Published catalog entries are the only sources allowed to be cross-origin. They are
 * fetched only after an explicit user click on their Load button (user-initiated network
 * access to the host named on the card), optionally with a pinned sha256/size and a
 * stack-bound tokenizer artifact; the bytes are then posted to the worker, which remains
 * the only WASM backend. Bundled/demo entries must stay same-origin.
 *
 *   save()                              -> persisted list()
 *   testApi(id?)                        -> Promise<{ ok, detail, models }>
 *   setActive({ kind, id })             -> list()
 *   addApi(config)                      -> config
 *   updateApi(id, patch)                -> config
 *   removeApi(id)                       -> list()
 *   mount(containerElement)             -> { render, unmount }
 */

const STORAGE_KEY = 'uor.studio.modelSources.v1';
const EVENT_NAME = 'uor-model-source-changed';
export const LOAD_PROGRESS_EVENT = 'uor-model-load-progress';
const MAX_WASM_BYTES = 128 * 1024 * 1024; // matches the worker's MAX_ARTIFACT_BYTES (M1 trunk is 86.5 MB)
let latestOperationId = 0;
let selectionEpoch = 0;

function beginOperation(operation) {
    if (!operation || !Number.isInteger(operation.operationId) || typeof operation.sourceId !== 'string') return null;
    latestOperationId = Math.max(latestOperationId, operation.operationId);
    return { operationId: operation.operationId, sourceId: operation.sourceId, selectionEpoch };
}

function isStaleOperation(operation) {
    return Boolean(operation && (operation.operationId < latestOperationId || operation.selectionEpoch !== selectionEpoch));
}

function emitLoadProgress(detail) {
    const progress = {
        loaded: Math.max(0, Number(detail.loaded) || 0),
        total: Number(detail.total) > 0 ? Number(detail.total) : null,
        phase: detail.phase,
    };
    if (Number.isInteger(detail.operationId)) progress.operationId = detail.operationId;
    document.dispatchEvent(new CustomEvent(LOAD_PROGRESS_EVENT, { detail: progress }));
}
const BUNDLED_DEMO_URL = './pkg/synthetic-stack.lut';
const BUNDLED_DEMO_LABEL = 'Synthetic demo (seed 5)';
const MODELS_CATALOG_URL = './models.json';
const MODELS_CATALOG_SCHEMA = 'uor-r4.studio-models/1';

// Optional same-origin publish catalog written next to index.html by the stage helper.
// Shape: { schema: "uor-r4.studio-models/1", bundled?: { label?, url? },
//          models?: [{ id?, label?, url, note?, sha256? }] }.
// A missing or invalid catalog is not an error: the hardcoded bundled demo stays.
const catalog = {
    bundled: { label: BUNDLED_DEMO_LABEL, url: BUNDLED_DEMO_URL },
    models: [],
};

export const API_PRESETS = Object.freeze([
    {
        preset: 'local-uor',
        label: 'Local uor-r4 API',
        baseUrl: 'http://127.0.0.1:8000/v1',
        model: 'uor-r4',
        note: 'An OpenAI-compatible gateway running on this machine.',
    },
    {
        preset: 'ollama',
        label: 'Ollama (local)',
        baseUrl: 'http://127.0.0.1:11434/v1',
        model: 'llama3.2',
        note: 'Ollama exposes an OpenAI-compatible endpoint at /v1.',
    },
    {
        preset: 'custom',
        label: 'Custom OpenAI-compatible API',
        baseUrl: '',
        model: '',
        note: 'Any endpoint that implements /chat/completions (and optionally /models).',
    },
]);

function emptyState() {
    return {
        wasm: {
            kind: 'bundled',
            id: 'bundled-demo',
            label: BUNDLED_DEMO_LABEL,
            fileName: 'synthetic-stack.lut',
            schema: null,
            synthetic: true,
            byteLength: null,
        },
        api: [
            apiEntry({ preset: 'local-uor', label: 'Local uor-r4 API', baseUrl: 'http://127.0.0.1:8000/v1', model: 'uor-r4', apiKey: '' }),
            apiEntry({ preset: 'ollama', label: 'Ollama (local)', baseUrl: 'http://127.0.0.1:11434/v1', model: 'llama3.2', apiKey: '' }),
        ],
        active: { kind: 'wasm', id: 'bundled-demo' },
    };
}

let idCounter = 0;
function nextId(prefix) {
    idCounter += 1;
    return prefix + '-' + Date.now().toString(36) + '-' + idCounter.toString(36);
}

function requireApiBaseUrl(value) {
    let url;
    try { url = new URL(String(value || '').trim()); } catch (_error) { throw new Error('Only HTTP(S) API endpoints are supported.'); }
    if (url.protocol !== 'http:' && url.protocol !== 'https:') throw new Error('Only HTTP(S) API endpoints are supported.');
}

function apiEntry(input) {
    return {
        id: input.id || nextId('api'),
        preset: input.preset || 'custom',
        label: String(input.label || 'API endpoint').trim(),
        baseUrl: String(input.baseUrl || '').trim(),
        model: String(input.model || '').trim(),
        apiKey: typeof input.apiKey === 'string' ? input.apiKey : '',
        temperature: Number.isFinite(Number(input.temperature)) ? Number(input.temperature) : undefined,
        ephemeral: input.ephemeral === true,
        createdAt: input.createdAt || Date.now(),
    };
}

function clone(value) {
    if (typeof structuredClone === 'function') {
        try {
            return structuredClone(value);
        } catch (_error) {
            /* fall through to JSON copy */
        }
    }
    return JSON.parse(JSON.stringify(value));
}

function loadState() {
    try {
        const raw = globalThis.localStorage && globalThis.localStorage.getItem(STORAGE_KEY);
        if (!raw) return emptyState();
        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== 'object') return emptyState();
        const fallback = emptyState();
        const wasm = Object.assign({}, fallback.wasm, parsed.wasm || {});
        if (!wasm.id) wasm.id = fallback.wasm.id;
        const api = Array.isArray(parsed.api) ? parsed.api.filter((item) => item && item.baseUrl !== undefined).map(apiEntry) : fallback.api;
        let active = parsed.active && (parsed.active.kind === 'wasm' || parsed.active.kind === 'api')
            ? { kind: parsed.active.kind, id: parsed.active.id || null }
            : fallback.active;
        if (active.kind === 'api') {
            const selected = api.find((item) => item.id === active.id);
            try { requireApiBaseUrl(selected && selected.baseUrl); } catch (_error) { active = fallback.active; }
        }
        return { wasm, api, active };
    } catch (_error) {
        return emptyState();
    }
}

const state = loadState();
let mounted = null; // { container, render }
// The Studio may move rendered source groups out of the mount container (the
// Models page regroups them into labeled sections). Delegation is therefore
// document-level, scoped to the manager's own containers.
const MODEL_SCOPE_SELECTOR = '#modelSourceMount, #modelsSections, #modelSourcesPanel, #inspector';
let message = { tone: 'muted', text: '' };
let editing = null; // { id: string|null, values: {...} }

function persist() {
    try {
        const activeEntry = state.active.kind === 'api' ? findApi(state.active.id) : null;
        const persisted = Object.assign({}, state, {
            api: state.api.filter((entry) => !entry.ephemeral),
            active: activeEntry && activeEntry.ephemeral ? { kind: 'wasm', id: state.wasm.id || 'bundled-demo' } : state.active,
        });
        globalThis.localStorage.setItem(STORAGE_KEY, JSON.stringify(persisted));
        return true;
    } catch (_error) {
        setMessage('error', 'Could not persist model sources: browser storage is unavailable or full.');
        return false;
    }
}

function setMessage(tone, text) {
    message = { tone, text };
    updateMessageNode();
}

function updateMessageNode() {
    if (!mounted) return;
    const node = mounted.container.querySelector('[data-role="status"]');
    if (node) {
        node.textContent = message.text || '';
        node.className = 'source-status source-status-' + (message.tone || 'muted');
    }
    // The regrouped Models page moves the cards out of the mount; mirror the
    // status into its visible verification row so errors are never silent.
    const visible = document.getElementById('modelsVerifyRow');
    if (visible) {
        visible.textContent = message.text || '';
        visible.dataset.tone = message.tone || 'muted';
    }
}

function emit(detail) {
    const source = publicList();
    const event = new CustomEvent(EVENT_NAME, { detail: Object.assign({ source }, detail) });
    document.dispatchEvent(event);
}

function publicList() {
    return { active: clone(state.active), wasm: clone(state.wasm), api: clone(state.api) };
}

function findApi(id) {
    if (!id) return null;
    return state.api.find((item) => item.id === id) || null;
}

function activeApi() {
    if (state.active.kind !== 'api') return null;
    return findApi(state.active.id);
}

function publicConfig(entry) {
    if (!entry) return null;
    return { id: entry.id, label: entry.label, preset: entry.preset, baseUrl: entry.baseUrl, model: entry.model, apiKey: entry.apiKey, temperature: entry.temperature, ephemeral: entry.ephemeral };
}

function setActive(next) {
    if (!next || (next.kind !== 'wasm' && next.kind !== 'api')) {
        throw new Error('Unknown model source kind.');
    }
    if (next.kind === 'api' && !findApi(next.id)) {
        throw new Error('That API endpoint no longer exists.');
    }
    if (next.kind === 'api') requireApiBaseUrl(findApi(next.id).baseUrl);
    const previous = clone(state.active);
    const activeEntry = next.kind === 'api' ? findApi(next.id) : null;
    selectionEpoch += 1;
    state.active = { kind: next.kind, id: next.kind === 'api' ? next.id : (next.id || 'bundled-demo') };
    if (!persist() && !(activeEntry && activeEntry.ephemeral)) {
        state.active = previous;
        selectionEpoch -= 1;
        throw new Error(message.text);
    }
    setMessage('muted', '');
    if (mounted) mounted.render();
    emit({ kind: state.active.kind, id: state.active.id, apiConfig: state.active.kind === 'api' ? publicConfig(activeApi()) : null });
    return publicList();
}

function setWasmMeta(meta) {
    state.wasm = Object.assign({}, state.wasm, meta);
}

/** Accept only http(s) or same-origin relative URLs for user-initiated model downloads. */
function resolveModelUrl(raw) {
    const value = String(raw == null ? '' : raw).trim();
    if (!value) throw new Error('Enter a model file URL.');
    let url;
    try {
        url = new URL(value, document.baseURI);
    } catch (_error) {
        throw new Error('That model URL is not valid.');
    }
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
        throw new Error('Model URLs must use http(s).');
    }
    return url;
}

function parseUorLutHeader(input) {
    const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
    if (bytes.byteLength < 16) {
        throw new Error('That file is too small to be a UORLUT01 stack artifact.');
    }
    const magic = new TextDecoder('ascii').decode(bytes.subarray(0, 8));
    if (magic !== 'UORLUT01') {
        throw new Error('Unrecognized file header; a stack artifact starts with the ASCII magic UORLUT01.');
    }
    const view = bytes.byteOffset === 0 && bytes.byteLength === bytes.buffer.byteLength
        ? new DataView(bytes.buffer)
        : new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const jsonLenBig = view.getBigUint64(8, true);
    const jsonLen = Number(jsonLenBig);
    if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > bytes.byteLength) {
        throw new Error('The UORLUT01 header length is invalid for this file.');
    }
    let header;
    try {
        header = JSON.parse(new TextDecoder('utf-8').decode(bytes.subarray(16, 16 + jsonLen)));
    } catch (_error) {
        throw new Error('The UORLUT01 JSON header could not be parsed.');
    }
    const schema = header && typeof header.schema === 'string' ? header.schema : '';
    if (!/^uor-r4\.lut-(stack|discrete)(-trunk)?\/\d+$/.test(schema)) {
        throw new Error('Unsupported artifact schema "' + (schema || 'unknown') + '"; expected a uor-r4.lut-stack or uor-r4.lut-discrete container.');
    }
    return {
        schema,
        shape: (header && header.shape) || null,
        group: typeof header.group === 'number' ? header.group : null,
        synthetic: Boolean(header && header.source && header.source.synthetic),
        header,
    };
}

async function readArtifact(bytes, fileName, origin, options) {
    const extra = options || {};
    if (isStaleOperation(extra.operation)) return { stale: true };
    const parsed = parseUorLutHeader(bytes);
    const meta = {
        kind: origin.kind,
        id: origin.id,
        label: origin.label,
        fileName,
        schema: parsed.schema,
        synthetic: parsed.synthetic,
        byteLength: bytes.byteLength,
        shape: parsed.shape,
        format: extra.format || 'UORLUT01',
        sizeBytes: Number.isInteger(extra.sizeBytes) ? extra.sizeBytes : bytes.byteLength,
        sha256: extra.sha256 || '',
        expectedSha256: extra.expectedSha256 || '',
        host: origin.host || '',
        tokenizerBound: Boolean(extra.tokenizerArtifact),
    };
    setWasmMeta(meta);
    state.active = { kind: 'wasm', id: meta.id };
    persist();
    if (mounted) mounted.render();
    emit({
        kind: 'wasm',
        id: meta.id,
        bytes,
        fileName,
        header: parsed.header,
        headerMeta: { schema: parsed.schema, shape: parsed.shape, synthetic: parsed.synthetic, group: parsed.group },
        expectedSha256: extra.expectedSha256 || '',
        tokenizerArtifact: extra.tokenizerArtifact || null,
        sizeBytes: meta.sizeBytes,
        format: meta.format,
        meta: clone(meta),
    });
    return { bytes, fileName, expectedSha256: extra.expectedSha256 || '', header: parsed.header, tokenizerArtifact: extra.tokenizerArtifact || null, meta: clone(meta), source: { kind: 'wasm', id: meta.id } };
}

function getBundledDemo() {
    return { label: catalog.bundled.label || BUNDLED_DEMO_LABEL, url: catalog.bundled.url || BUNDLED_DEMO_URL };
}

function catalogModelEntries() {
    return catalog.models.slice();
}

function sameOriginModelUrl(value) {
    var raw = String(value || '').trim();
    if (!raw) return null;
    try {
        var url = new URL(raw, document.baseURI);
        if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
        return url;
    } catch (_error) {
        return null;
    }
}

/** Published entries may name any http(s) host (the user's click is the consent to fetch it). */
function remoteModelUrl(value) {
    var raw = String(value || '').trim();
    if (!raw) return null;
    try {
        var url = new URL(raw, document.baseURI);
        if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
        return url;
    } catch (_error) {
        return null;
    }
}

function formatBytes(bytes) {
    var n = Number(bytes);
    if (!Number.isFinite(n) || n <= 0) return '';
    if (n >= 1024 * 1024) return (Math.round((n / (1024 * 1024)) * 10) / 10) + ' MB';
    if (n >= 1024) return Math.round(n / 1024) + ' KB';
    return n + ' B';
}

function shortSha(value) {
    var s = String(value || '');
    return /^[0-9a-f]{64}$/i.test(s) ? s.slice(0, 12) + '…' : s;
}

async function loadModelCatalog() {
    try {
        var response = await fetch(MODELS_CATALOG_URL, { cache: 'no-store', credentials: 'omit' });
        if (!response.ok) return;
        var data = await response.json();
        if (!data || data.schema !== MODELS_CATALOG_SCHEMA) return;
        if (data.bundled && typeof data.bundled === 'object') {
            var bundledUrl = sameOriginModelUrl(data.bundled.url || data.bundled.path || '');
            if (bundledUrl) {
                catalog.bundled = {
                    label: typeof data.bundled.label === 'string' && data.bundled.label.trim() ? data.bundled.label.trim() : BUNDLED_DEMO_LABEL,
                    url: bundledUrl.href,
                };
            }
        }
        // The publisher's stage helper writes models[] entries:
        //   kind "bundled" -> the bundled demo (label/url override the hardcoded default,
        //                     and it MUST stay same-origin);
        //   any other kind ("published"/"url"/...) -> a user-initiated model card whose
        //                     http(s) URL may be cross-origin (for example Hugging Face),
        //                     carrying optional tokenizer_url/sha256/size_bytes/format pins.
        var entries = Array.isArray(data.models) ? data.models : [];
        var published = [];
        entries.forEach(function (entry, index) {
            if (!entry || typeof entry !== 'object') return;
            var kind = typeof entry.kind === 'string' ? entry.kind.toLowerCase() : '';
            var label = typeof entry.label === 'string' && entry.label.trim()
                ? entry.label.trim()
                : (typeof entry.name === 'string' && entry.name.trim() ? entry.name.trim() : 'Model ' + (index + 1));
            if (kind === 'bundled' || kind === 'demo') {
                var bundledUrl = sameOriginModelUrl(entry.url || entry.path || '');
                if (bundledUrl) catalog.bundled = { label: label, url: bundledUrl.href };
                return;
            }
            var url = remoteModelUrl(entry.url || entry.path || '');
            if (!url) return;
            var tokenizerUrl = remoteModelUrl(entry.tokenizer_url || entry.tokenizerUrl || '');
            published.push({
                id: typeof entry.id === 'string' && entry.id ? 'published-' + entry.id : 'published-' + index,
                label: label,
                url: url.href,
                tokenizerUrl: tokenizerUrl ? tokenizerUrl.href : '',
                note: typeof entry.note === 'string' ? entry.note : '',
                schema: typeof entry.schema === 'string' ? entry.schema : '',
                format: typeof entry.format === 'string' ? entry.format : 'UORLUT01',
                sha256: typeof entry.sha256 === 'string' ? entry.sha256.toLowerCase() : '',
                sizeBytes: Number.isInteger(entry.size_bytes) && entry.size_bytes > 0 ? entry.size_bytes : 0,
                tokenizerSha256: typeof entry.tokenizer_sha256 === 'string' ? entry.tokenizer_sha256.toLowerCase() : '',
                primary: entry.primary === true,
                host: url.host,
            });
        });
        catalog.models = published;
        document.dispatchEvent(new CustomEvent('uor-model-catalog-changed'));
        if (mounted) mounted.render();
    } catch (_error) {
        // A publish catalog is optional; a missing or malformed one is ignored.
    }
}

function getCatalog() {
    return { bundled: getBundledDemo(), models: catalogModelEntries() };
}

/**
 * Fetch one artifact with a byte cap and progress callback. The page verifies the pinned
 * sha256/size after the download; the worker re-verifies expectedSha256 before load.
 */
async function fetchArtifactWithProgress(rawUrl, options) {
    var opts = options || {};
    var url = remoteModelUrl(rawUrl);
    if (!url) throw new Error('The model URL is not a valid http(s) URL.');
    var response = await fetch(url.toString(), { cache: 'no-store', credentials: 'omit' });
    if (!response.ok) {
        throw new Error('The model download from ' + url.host + ' failed with HTTP ' + response.status + '.');
    }
    var expected = Number(opts.expectedBytes) > 0 ? Number(opts.expectedBytes) : 0;
    var declared = Number(response.headers.get('content-length') || 0);
    if (declared > MAX_WASM_BYTES) {
        throw new Error('That model file is larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser limit for this Studio.');
    }
    if (expected > MAX_WASM_BYTES) {
        throw new Error('The catalog pins a model larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser limit for this Studio.');
    }
    var total = expected || declared || 0;
    var report = function (loaded, complete) {
        if (typeof opts.onProgress !== 'function') return;
        var pct = total > 0 ? Math.min(100, Math.floor((loaded * 100) / total)) : 0;
        opts.onProgress(complete ? 100 : Math.min(99, pct), loaded, total);
    };
    if (!response.body || typeof response.body.getReader !== 'function') {
        var buffer = await response.arrayBuffer();
        var whole = new Uint8Array(buffer);
        if (whole.byteLength > MAX_WASM_BYTES) throw new Error('That model file is larger than the browser load limit for this Studio.');
        if (expected && whole.byteLength !== expected) {
            throw new Error('Downloaded size ' + whole.byteLength + ' does not match the pinned size_bytes ' + expected + '.');
        }
        report(whole.byteLength, true);
        return whole;
    }
    var reader = response.body.getReader();
    var chunks = [];
    var received = 0;
    for (;;) {
        var read = await reader.read();
        if (read.done) break;
        var chunk = read.value instanceof Uint8Array ? read.value : new Uint8Array(read.value);
        received += chunk.byteLength;
        if (received > MAX_WASM_BYTES) {
            try { reader.cancel(); } catch (_error) { /* ignore */ }
            throw new Error('That model file is larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser load limit for this Studio.');
        }
        chunks.push(chunk);
        report(received, false);
    }
    var merged = new Uint8Array(received);
    var offset = 0;
    for (var i = 0; i < chunks.length; i += 1) {
        merged.set(chunks[i], offset);
        offset += chunks[i].byteLength;
    }
    if (expected && merged.byteLength !== expected) {
        throw new Error('Downloaded size ' + merged.byteLength + ' does not match the pinned size_bytes ' + expected + '.');
    }
    report(merged.byteLength, true);
    return merged;
}

/** Fetch and validate the stack-bound tokenizer named by a published entry. */
async function fetchTokenizerArtifact(entry, stackSha256, vocabSize) {
    var url = remoteModelUrl(entry.tokenizerUrl);
    if (!url) throw new Error('The catalog tokenizer_url is not a valid http(s) URL.');
    setMessage('info', 'Loading the stack-bound tokenizer for ' + entry.label + '…');
    var response = await fetch(url.toString(), { cache: 'no-store', credentials: 'omit' });
    if (!response.ok) {
        throw new Error('The tokenizer download from ' + url.host + ' failed with HTTP ' + response.status + '.');
    }
    var text = await response.text();
    var tokenizerHash = await sha256Hex(new TextEncoder().encode(text));
    if (entry.tokenizerSha256 && tokenizerHash !== entry.tokenizerSha256) {
        throw new Error('Tokenizer sha256 ' + shortSha(tokenizerHash) + ' does not match the catalog pin ' + shortSha(entry.tokenizerSha256) + '; refusing to load.');
    }
    var artifact;
    try {
        artifact = JSON.parse(text);
    } catch (_error) {
        throw new Error('The tokenizer_url did not return a JSON tokenizer artifact.');
    }
    if (!validateTokenizerArtifact(artifact, stackSha256, vocabSize)) {
        throw new Error('The tokenizer artifact does not bind to this stack (sha256/vocab mismatch); refusing to load the trained model without its bound tokenizer.');
    }
    return artifact;
}

/** Load a published catalog entry: user-initiated download, hash/size check, tokenizer binding. */
async function loadPublished(idOrEntry, operation) {
    if (fullDesktopHost()) throw browserOnlyError();
    const currentOperation = beginOperation(operation);
    var entry = typeof idOrEntry === 'string'
        ? catalog.models.find(function (item) { return item.id === idOrEntry; })
        : idOrEntry;
    if (!entry) throw new Error('That published model is not in the catalog.');
    var host = entry.host || (new URL(entry.url)).host;
    setMessage('info', 'Downloading ' + entry.label + ' from ' + host + '…');
    var bytes = await fetchArtifactWithProgress(entry.url, {
        expectedBytes: entry.sizeBytes,
        onProgress: function (pct, loaded, total) {
            setMessage('info', 'Downloading ' + entry.label + ' from ' + host + '… ' + pct + '% (' + formatBytes(loaded) + (total ? ' / ' + formatBytes(total) : '') + ')');
            emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded, total, phase: 'downloading' });
        },
    });
    emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded: bytes.byteLength, total: null, phase: 'finalizing' });
    var stackSha256 = await sha256Hex(bytes);
    if (entry.sha256 && stackSha256 !== entry.sha256) {
        throw new Error('sha256 mismatch for ' + entry.label + ': expected ' + shortSha(entry.sha256) + ', got ' + shortSha(stackSha256) + '; refusing to load.');
    }
    var arrayBuffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
    var parsed = parseUorLutHeader(arrayBuffer);
    var parsedVocab = parsed.shape ? Number(parsed.shape.vocab || parsed.shape.vocabulary || 0) : 0;
    var tokenizerArtifact = null;
    if (entry.tokenizerUrl) {
        tokenizerArtifact = await fetchTokenizerArtifact(entry, stackSha256, parsedVocab);
    }
    setMessage('info', 'Loading ' + entry.label + ' into the engine…');
    var fileName = (new URL(entry.url)).pathname.split('/').pop() || 'published-stack.lut';
    var result = await readArtifact(bytes, fileName, { kind: 'published', id: entry.id, label: entry.label, host: host }, {
        expectedSha256: entry.sha256 || stackSha256,
        tokenizerArtifact: tokenizerArtifact,
        sizeBytes: entry.sizeBytes || bytes.byteLength,
        format: entry.format || 'UORLUT01',
        sha256: stackSha256,
        operation: currentOperation,
    });
    if (!result.stale) setMessage('success', 'Loaded ' + entry.label + ' from ' + host + ' (' + formatBytes(bytes.byteLength) + ', ' + parsed.schema + (tokenizerArtifact ? ', tokenizer bound' : ', no tokenizer') + ').');
    return result;
}

const api = {
    STORAGE_KEY,
    EVENT_NAME,
    API_PRESETS,

    list() {
        return publicList();
    },

    active() {
        return clone(state.active);
    },

    apiConfig(id) {
        const entry = id ? findApi(id) : activeApi();
        return publicConfig(entry);
    },

    save() {
        persist();
        return publicList();
    },

    catalog() {
        return getCatalog();
    },

    bundledDemo() {
        return getBundledDemo();
    },

    loadModelCatalog() {
        return loadModelCatalog();
    },

    loadPublished(idOrEntry, operation) {
        return loadPublished(idOrEntry, operation);
    },

    setActive(next) {
        return setActive(next);
    },

    registerNativeBootstrap(input) {
        if (!input || input.ephemeral !== true) throw new Error('The native bootstrap must be ephemeral.');
        if (input.protocol !== 'openai') throw new Error('The native bootstrap must use the OpenAI protocol.');
        if (Number(input.temperature) !== 0) throw new Error('The native bootstrap must use temperature zero.');
        if (typeof input.token !== 'string' || !input.token) throw new Error('The native bootstrap token is missing.');
        const baseUrl = normalizeNativeApiUrl(input.baseUrl, document.baseURI);
        const id = 'native-loopback';
        const existing = state.api.findIndex((entry) => entry.id === id);
        if (existing >= 0) state.api.splice(existing, 1);
        const entry = this.addApi({
            id,
            preset: 'local-uor',
            label: 'UOR Chat — native loopback',
            baseUrl: baseUrl.toString(),
            model: String(input.model || 'uor-chat'),
            apiKey: input.token,
            temperature: 0,
            ephemeral: true,
        });
        this.setActive({ kind: 'api', id });
        return entry;
    },

    addApi(input) {
        const entry = apiEntry(input || {});
        if (!entry.baseUrl) throw new Error('An API base URL is required.');
        requireApiBaseUrl(entry.baseUrl);
        state.api.push(entry);
        if (!persist() && !entry.ephemeral) {
            state.api.pop();
            throw new Error(message.text);
        }
        setMessage('muted', '');
        if (mounted) mounted.render();
        emit({ kind: 'state', id: entry.id, apiConfig: publicConfig(entry) });
        return clone(entry);
    },

    updateApi(id, patch) {
        const entry = findApi(id);
        if (!entry) throw new Error('No API endpoint with that id.');
        const previous = clone(entry);
        Object.assign(entry, apiEntry(Object.assign({}, entry, patch, { id: entry.id, createdAt: entry.createdAt })));
        try {
            requireApiBaseUrl(entry.baseUrl);
        } catch (error) {
            Object.assign(entry, previous);
            throw error;
        }
        if (!persist()) {
            Object.assign(entry, previous);
            throw new Error(message.text);
        }
        setMessage('muted', '');
        if (mounted) mounted.render();
        emit({ kind: 'state', id: entry.id, apiConfig: publicConfig(entry) });
        return clone(entry);
    },

    removeApi(id) {
        const index = state.api.findIndex((item) => item.id === id);
        if (index < 0) return publicList();
        const removed = state.api[index];
        const previousActive = clone(state.active);
        const removedActive = previousActive.kind === 'api' && previousActive.id === id;
        state.api.splice(index, 1);
        if (removedActive) {
            state.active = { kind: 'wasm', id: state.wasm.id || 'bundled-demo' };
        }
        if (!persist()) {
            state.api.splice(index, 0, removed);
            state.active = previousActive;
            throw new Error(message.text);
        }
        setMessage('muted', '');
        if (mounted) mounted.render();
        emit({ kind: 'state', id: null, removedActive });
        return publicList();
    },

    async testApi(id) {
        if (!globalThis.UORStudioApi || typeof globalThis.UORStudioApi.testConnection !== 'function') {
            throw new Error('The API client module is not loaded.');
        }
        const config = publicConfig(id ? findApi(id) : activeApi());
        if (!config) throw new Error('No API endpoint selected.');
        return globalThis.UORStudioApi.testConnection(config);
    },

    async loadLocalFile(file, operation) {
        const currentOperation = beginOperation(operation);
        if (!file) throw new Error('Choose a .lut/.bin stack artifact first.');
        if (file.size > MAX_WASM_BYTES) {
            throw new Error('That model file is larger than the ' + Math.round(MAX_WASM_BYTES / (1024 * 1024)) + ' MB browser limit for this Studio.');
        }
        if (file.size === 0) throw new Error('The selected model file is empty.');
        setMessage('info', 'Reading ' + file.name + '…');
        const bytes = await file.arrayBuffer();
        emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded: bytes.byteLength, total: null, phase: 'finalizing' });
        const result = await readArtifact(bytes, file.name, { kind: 'local-file', id: 'local-file', label: file.name }, { operation: currentOperation });
        if (!result.stale) setMessage('success', 'Loaded ' + file.name + '.');
        return result;
    },

    async loadFromUrl(rawUrl, operation) {
        if (fullDesktopHost()) throw browserOnlyError();
        const currentOperation = beginOperation(operation);
        const url = resolveModelUrl(rawUrl);
        setMessage('info', 'Downloading ' + (url.pathname.split('/').pop() || 'model') + ' from ' + url.host + '…');
        const bytes = await fetchArtifactWithProgress(url.href, {
            onProgress: function (pct, loaded, total) {
                setMessage('info', 'Downloading from ' + url.host + '… ' + pct + '% (' + formatBytes(loaded) + (total ? ' / ' + formatBytes(total) : '') + ')');
                emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded, total, phase: 'downloading' });
            },
        });
        emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded: bytes.byteLength, total: null, phase: 'finalizing' });
        const fileName = url.pathname.split('/').pop() || 'downloaded-stack.lut';
        const result = await readArtifact(bytes, fileName, { kind: 'url', id: 'url', label: fileName, host: url.host }, { operation: currentOperation });
        if (!result.stale) setMessage('success', 'Loaded ' + fileName + ' (' + formatBytes(bytes.byteLength) + ').');
        return result;
    },

    loadBundled(operation) {
        return loadBundled(operation);
    },

    mount(container) {
        if (!container) throw new Error('A container element is required.');
        mounted = { container, render: () => renderAll(container) };
        container.classList.add('model-sources');
        renderAll(container);
        loadModelCatalog();
        document.addEventListener('click', onClick);
        document.addEventListener('submit', onSubmit);
        document.addEventListener('change', onChange);
        return {
            render: () => renderAll(container),
            unmount: () => {
                document.removeEventListener('click', onClick);
                document.removeEventListener('submit', onSubmit);
                document.removeEventListener('change', onChange);
                container.textContent = '';
                mounted = null;
            },
        };
    },
};

/* ------------------------------------------------------------------ rendering */

function fullDesktopHost() {
    return typeof globalThis.__TAURI__?.core?.invoke === 'function' && typeof globalThis.__TAURI__?.core?.Channel === 'function';
}

function browserOnlyError() {
    return new Error('Custom providers and remote model URLs are available in the browser build.');
}

function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
}

function button(label, action, extra) {
    const node = el('button', 'btn ' + (extra && extra.className ? extra.className : 'btn-secondary'), label);
    node.type = 'button';
    node.dataset.action = action;
    if (extra && extra.id) node.id = extra.id;
    if (extra && extra.title) node.title = extra.title;
    if (extra && extra.ariaLabel) node.setAttribute('aria-label', extra.ariaLabel);
    return node;
}

function cardHeader(kind, label, detail, active) {
    const main = el('div', 'source-card-main');
    const title = el('div', 'source-card-title');
    title.appendChild(el('span', 'source-kind', kind));
    title.appendChild(el('span', 'source-label', label));
    if (active) title.appendChild(el('span', 'badge badge-active', 'Active'));
    main.appendChild(title);
    if (detail) main.appendChild(el('div', 'source-detail', detail));
    return main;
}

function renderAll(container) {
    container.textContent = '';

    var primaryEntries = [];
    var researchEntries = [];
    for (var p = 0; p < catalog.models.length; p += 1) {
        if (catalog.models[p].primary) primaryEntries.push(catalog.models[p]);
        else researchEntries.push(catalog.models[p]);
    }

    if (primaryEntries.length) {
        var trained = el('section', 'source-group');
        trained.appendChild(el('h3', null, 'Trained M1 model'));
        trained.appendChild(el('p', 'hint', 'M1-closing 19-table trunk export; next-token prediction only, not a chat assistant. User-initiated download.'));
        for (var t = 0; t < primaryEntries.length; t += 1) trained.appendChild(publishedModelCard(primaryEntries[t]));
        container.appendChild(trained);
    }

    const nativeGuide = el('section', 'source-group native-runner-guide');
    nativeGuide.id = 'nativeRunnerGuide';
    nativeGuide.tabIndex = -1;
    const nativeTemplate = document.getElementById('nativeRunnerGuidanceTemplate');
    if (nativeTemplate) nativeGuide.appendChild(nativeTemplate.content.cloneNode(true));
    container.appendChild(nativeGuide);

    const local = el('section', 'source-group');
    local.appendChild(el('h3', null, 'Local WASM engine'));
    local.appendChild(el('p', 'hint', 'Runs in a Web Worker inside this tab. Prompts and replies stay on this device. The bundled demo is a synthetic stack, not a chat model.'));

    local.appendChild(bundledCard());
    local.appendChild(localFileCard());
    local.appendChild(urlCard());
    container.appendChild(local);

    if (researchEntries.length) {
        var research = el('section', 'source-group');
        research.appendChild(el('h3', null, 'Research artifacts (experimental)'));
        research.appendChild(el('p', 'hint', 'Earlier exports kept for inspection; not generation-capable in this Studio.'));
        for (var q = 0; q < researchEntries.length; q += 1) research.appendChild(publishedModelCard(researchEntries[q]));
        container.appendChild(research);
    }

    const remote = el('section', 'source-group');
    remote.appendChild(el('h3', null, 'OpenAI-compatible API'));
    remote.appendChild(el('p', 'hint', 'Optional. Custom API sources send prompts to their configured endpoint; native loopback stays on this device. Ordinary keys are stored only in this browser and are never committed.'));
    for (const entry of state.api) remote.appendChild(apiCard(entry));
    if (state.api.length === 0) remote.appendChild(el('p', 'empty-note', 'No API endpoints configured.'));
    const addApi = button('Add API endpoint', 'add-api', { className: 'btn-primary btn-small' });
    addApi.disabled = fullDesktopHost();
    if (addApi.disabled) addApi.title = 'Available in the browser build.';
    remote.appendChild(addApi);
    if (editing) remote.appendChild(apiForm());
    container.appendChild(remote);

    const status = el('p', 'source-status', message.text || '');
    status.dataset.role = 'status';
    status.setAttribute('role', 'status');
    status.setAttribute('aria-live', 'polite');
    status.className = 'source-status source-status-' + (message.tone || 'muted');
    container.appendChild(status);

    const note = el('p', 'source-footnote', 'Model files are UORLUT01 stack containers (.lut / .bin). Local mode is offline; only a model URL, an API endpoint or a page reload will use the network.');
    container.appendChild(note);
}

function bundledCard() {
    const demo = getBundledDemo();
    const card = el('div', 'source-card');
    const activeBundled = state.active.kind === 'wasm' && state.wasm.kind === 'bundled' && state.active.id === 'bundled-demo';
    card.appendChild(cardHeader('Demo', demo.label, 'Bundled ' + demo.url + ' · loads only after this build produces it', activeBundled));
    const actions = el('div', 'source-card-actions');
    actions.appendChild(button('Load bundled demo', 'load-bundled', { className: 'btn-secondary btn-small' }));
    card.appendChild(actions);
    return card;
}

function publishedModelCard(entry) {
    const card = el('div', 'source-card');
    const active = state.active.kind === 'wasm' && state.wasm.id === entry.id;
    const badge = entry.primary ? 'PRIMARY' : (/c131|fbs/i.test(String(entry.id || '')) ? 'RESEARCH' : 'EVIDENCE');
    card.appendChild(cardHeader(badge, entry.label, entry.note || ('Loads from ' + (entry.host || entry.url) + ' only when you click Load.'), active));
    const metadata = el('div', 'source-meta');
    [
        ['Format', entry.schema || entry.format || 'UORLUT01'],
        ['Size', entry.sizeBytes ? formatBytes(entry.sizeBytes) : 'unknown'],
        ['SHA-256', entry.sha256 ? shortSha(entry.sha256) : 'not pinned'],
        ['Tokenizer', entry.tokenizerUrl ? 'stack-bound' : 'none']
    ].forEach(function (pair) {
        const item = el('span', 'source-meta-pair');
        item.appendChild(el('b', null, pair[0]));
        item.appendChild(document.createTextNode(pair[1]));
        metadata.appendChild(item);
    });
    card.appendChild(metadata);
    card.appendChild(el('div', 'source-detail', 'Remote download starts only when you click Load.'));
    const actions = el('div', 'source-card-actions');
    const load = button('Load published model', 'load-published', { className: 'btn-secondary btn-small' });
    load.disabled = fullDesktopHost();
    if (load.disabled) load.title = 'Available in the browser build.';
    load.dataset.modelId = entry.id;
    actions.appendChild(load);
    card.appendChild(actions);
    return card;
}

function localFileCard() {
    const card = el('div', 'source-card');
    const activeLocal = state.active.kind === 'wasm' && state.wasm.kind === 'local-file';
    card.appendChild(cardHeader('Local file', activeLocal ? (state.wasm.fileName || 'Local stack file') : 'Local .lut / .bin stack file', 'Opens a UORLUT01 artifact from this device; nothing is uploaded.', activeLocal));
    const actions = el('div', 'source-card-actions');
    if (fullDesktopHost()) {
        const native = button('Choose model folder', 'native-model-folder', { className: 'btn-secondary btn-small' });
        native.id = 'nativeModelFolderButton';
        actions.appendChild(native);
        card.appendChild(actions);
        return card;
    }
    const input = el('input');
    input.type = 'file';
    input.accept = '.lut,.bin,application/octet-stream';
    input.id = 'modelFileInput';
    input.className = 'sr-only';
    input.dataset.role = 'model-file';
    const label = el('label', 'btn btn-secondary btn-small', 'Choose model file');
    label.htmlFor = 'modelFileInput';
    actions.appendChild(input);
    actions.appendChild(label);
    card.appendChild(actions);
    return card;
}

function urlCard() {
    const card = el('div', 'source-card');
    const activeUrl = state.active.kind === 'wasm' && state.wasm.kind === 'url';
    card.appendChild(cardHeader('From URL', activeUrl ? (state.wasm.fileName || 'Downloaded stack') : 'Load a stack artifact from a URL', 'A user-initiated download; the page makes no external requests on load.', activeUrl));
    const row = el('div', 'source-url-row');
    const input = el('input', 'input');
    input.type = 'url';
    input.id = 'modelUrlInput';
    input.placeholder = 'https://example.com/model.lut';
    input.autocomplete = 'off';
    input.spellcheck = false;
    input.setAttribute('aria-label', 'Model file URL');
    input.disabled = fullDesktopHost();
    row.appendChild(input);
    const load = button('Load URL', 'load-url', { className: 'btn-secondary btn-small' });
    load.disabled = fullDesktopHost();
    if (load.disabled) load.title = 'Available in the browser build.';
    row.appendChild(load);
    card.appendChild(row);
    return card;
}

function apiCard(entry) {
    const card = el('div', 'source-card');
    const active = state.active.kind === 'api' && state.active.id === entry.id;
    const detail = (entry.baseUrl || 'No base URL yet.') + ' · model: ' + (entry.model || '(not set)') + (entry.apiKey ? ' · key set' : ' · no key');
    card.appendChild(cardHeader('API', entry.label, detail, active));
    const actions = el('div', 'source-card-actions');
    if (!active) actions.appendChild(button('Use', 'api-use', { className: 'btn-primary btn-small' }));
    actions.appendChild(button('Test', 'api-test', { className: 'btn-secondary btn-small' }));
    actions.appendChild(button('Edit', 'api-edit', { className: 'btn-ghost btn-small' }));
    actions.appendChild(button('Remove', 'api-remove', { className: 'btn-ghost btn-small btn-danger', title: 'Remove this API endpoint' }));
    if (fullDesktopHost() && entry.id !== 'native-loopback') {
        for (const control of actions.children) {
            if (control.dataset.action !== 'api-remove') {
                control.disabled = true;
                control.title = 'Available in the browser build.';
            }
        }
    }
    card.appendChild(actions);
    card.dataset.apiId = entry.id;
    return card;
}

function field(labelText, input) {
    const wrap = el('label', 'field');
    wrap.appendChild(el('span', 'field-label', labelText));
    wrap.appendChild(input);
    return wrap;
}

function apiForm() {
    const form = el('form', 'api-form');
    form.dataset.role = 'api-form';
    const values = editing.values;
    const preset = el('select', 'input');
    preset.name = 'preset';
    for (const option of API_PRESETS) {
        const node = el('option', null, option.label);
        node.value = option.preset;
        if (values.preset === option.preset) node.selected = true;
        preset.appendChild(node);
    }
    form.appendChild(field('Preset', preset));

    const label = el('input', 'input');
    label.name = 'label';
    label.required = true;
    label.value = values.label || '';
    label.placeholder = 'My endpoint';
    form.appendChild(field('Name', label));

    const baseUrl = el('input', 'input');
    baseUrl.name = 'baseUrl';
    baseUrl.required = true;
    baseUrl.value = values.baseUrl || '';
    baseUrl.placeholder = 'http://127.0.0.1:8000/v1';
    baseUrl.spellcheck = false;
    baseUrl.autocomplete = 'off';
    form.appendChild(field('Base URL', baseUrl));

    const model = el('input', 'input');
    model.name = 'model';
    model.required = true;
    model.value = values.model || '';
    model.placeholder = 'model name';
    model.autocomplete = 'off';
    form.appendChild(field('Model', model));

    const apiKey = el('input', 'input');
    apiKey.name = 'apiKey';
    apiKey.type = 'password';
    apiKey.value = values.apiKey || '';
    apiKey.placeholder = 'optional; stored only in this browser';
    apiKey.autocomplete = 'off';
    form.appendChild(field('API key', apiKey));

    const actions = el('div', 'form-actions');
    const save = el('button', 'btn btn-primary btn-small', 'Save endpoint');
    save.type = 'submit';
    actions.appendChild(save);
    actions.appendChild(button('Cancel', 'api-cancel', { className: 'btn-ghost btn-small' }));
    form.appendChild(actions);
    return form;
}

function readForm(form) {
    const data = new FormData(form);
    return {
        preset: String(data.get('preset') || 'custom'),
        label: String(data.get('label') || '').trim(),
        baseUrl: String(data.get('baseUrl') || '').trim(),
        model: String(data.get('model') || '').trim(),
        apiKey: String(data.get('apiKey') || ''),
    };
}

function applyPresetToForm(form) {
    const presetName = form.elements.preset.value;
    const preset = API_PRESETS.find((item) => item.preset === presetName);
    if (!preset) return;
    if (preset.preset !== 'custom') {
        form.elements.label.value = preset.label;
        form.elements.baseUrl.value = preset.baseUrl;
        form.elements.model.value = preset.model;
    }
}

async function onClick(event) {
    const target = event.target.closest('[data-action]');
    if (!mounted || !target || !target.closest(MODEL_SCOPE_SELECTOR)) return;
    const action = target.dataset.action;
    try {
        if (action === 'native-model-folder') {
            const desktop = globalThis.UORDesktopActions;
            if (!desktop?.available) throw new Error('Native model-folder selection is unavailable.');
            const selected = await desktop.chooseModelFolder();
            if (!selected) return;
            const models = await desktop.listLocalModels(selected.path);
            const names = models.map((model) => model.name).filter(Boolean);
            setMessage('info', `${names.length} pinned model${names.length === 1 ? '' : 's'}: ${names.join(', ') || 'none found'}. This list does not switch the running sidecar weights.`);
        } else if (action === 'load-bundled') {
            await loadBundled();
        } else if (action === 'load-published') {
            target.disabled = true;
            target.setAttribute('aria-busy', 'true');
            try {
                await api.loadPublished(target.dataset.modelId);
            } finally {
                target.disabled = false;
                target.removeAttribute('aria-busy');
            }
        } else if (action === 'load-url') {
            const input = document.querySelector('#modelUrlInput');
            await api.loadFromUrl(input.value);
        } else if (action === 'add-api') {
            editing = { id: null, values: { preset: 'local-uor', label: API_PRESETS[0].label, baseUrl: API_PRESETS[0].baseUrl, model: API_PRESETS[0].model, apiKey: '' } };
            mounted.render();
        } else if (action === 'api-cancel') {
            editing = null;
            mounted.render();
        } else if (action === 'api-use') {
            const card = target.closest('[data-api-id]');
            setActive({ kind: 'api', id: card.dataset.apiId });
            setMessage('success', 'API mode active: prompts will be sent to the configured endpoint.');
        } else if (action === 'api-test') {
            const card = target.closest('[data-api-id]');
            setMessage('info', 'Testing endpoint…');
            const result = await api.testApi(card.dataset.apiId);
            setMessage(result.ok ? 'success' : 'error', result.detail || (result.ok ? 'Reachable.' : 'Unreachable.'));
        } else if (action === 'api-edit') {
            const card = target.closest('[data-api-id]');
            const entry = findApi(card.dataset.apiId);
            if (entry) {
                editing = { id: entry.id, values: { preset: entry.preset, label: entry.label, baseUrl: entry.baseUrl, model: entry.model, apiKey: entry.apiKey } };
                mounted.render();
            }
        } else if (action === 'api-remove') {
            const card = target.closest('[data-api-id]');
            api.removeApi(card.dataset.apiId);
        }
    } catch (error) {
        setMessage('error', String(error && error.message ? error.message : error));
    }
}

async function loadBundled(operation) {
    const currentOperation = beginOperation(operation);
    const demo = getBundledDemo();
    setMessage('info', 'Loading the bundled synthetic demo…');
    const url = new URL(demo.url, document.baseURI);
    const response = await fetch(url.toString(), { cache: 'no-store' });
    if (!response.ok) {
        throw new Error('The bundled demo model is not present in this build (HTTP ' + response.status + '). A wasm build must produce pkg/synthetic-stack.lut first.');
    }
    const bytes = await response.arrayBuffer();
    emitLoadProgress({ operationId: currentOperation && currentOperation.operationId, loaded: bytes.byteLength, total: null, phase: 'finalizing' });
    const fileName = url.pathname.split('/').pop() || 'synthetic-stack.lut';
    const result = await readArtifact(bytes, fileName, { kind: 'bundled', id: 'bundled-demo', label: demo.label }, { operation: currentOperation });
    if (!result.stale) setMessage('success', 'Bundled synthetic demo loaded (' + Math.round(bytes.byteLength / 1024) + ' KB).');
    return result;
}

function onSubmit(event) {
    const form = event.target.closest('[data-role="api-form"]');
    if (!form || !form.closest(MODEL_SCOPE_SELECTOR)) return;
    event.preventDefault();
    try {
        const values = readForm(form);
        if (!values.baseUrl) throw new Error('An API base URL is required.');
        if (editing && editing.id) {
            api.updateApi(editing.id, values);
            setMessage('success', 'Endpoint updated.');
        } else {
            api.addApi(values);
            setMessage('success', 'Endpoint added.');
        }
        editing = null;
        mounted.render();
    } catch (error) {
        setMessage('error', String(error && error.message ? error.message : error));
    }
}

async function onChange(event) {
    if (!event.target.closest(MODEL_SCOPE_SELECTOR)) return;
    const input = event.target.closest('[data-role="model-file"]');
    if (input && input.files && input.files[0]) {
        try {
            await api.loadLocalFile(input.files[0]);
        } catch (error) {
            setMessage('error', String(error && error.message ? error.message : error));
        }
    }
    const presetSelect = event.target.closest('select[name="preset"]');
    if (presetSelect) {
        const form = presetSelect.closest('[data-role="api-form"]');
        if (form) applyPresetToForm(form);
    }
}

/* Attach the manager to the page global for other Studio modules / tests. */
if (typeof globalThis !== 'undefined') {
    globalThis.UORModelSources = api;
}

export const ModelSources = api;

/** Mount the Model sources panel into a container element. */
export function mountModelSources(container) {
    return api.mount(container);
}

export default api;
