/**
 * Studio API client — OpenAI-compatible chat completions for the UOR-R4 Studio.
 *
 * The browser Studio is offline-first: the default path is the local WASM engine in
 * assets/js/uor_model_worker.js. This module is only used when the user explicitly
 * selects an OpenAI-compatible API model source in the Model sources panel. Nothing
 * here runs, fetches or imports anything at page load.
 *
 * Configuration (base URL, model, API key) is owned by assets/js/model_sources.js and
 * persisted in this browser's localStorage under "uor.studio.modelSources.v1" except for the ephemeral native bootstrap, which stays in memory. API keys
 * are never bundled in the repository and never leave this browser except as the
 * Authorization header of a request the user triggered.
 *
 * Public interface (also available as window.UORStudioApi):
 *   normalizeBaseUrl(baseUrl)                  -> string (validated, trailing slash removed)
 *   configFromSources()                        -> { baseUrl, model, apiKey } | null
 *   listModels(config, options)                -> Promise<{ ok, status, models, detail }>
 *   testConnection(config)                     -> Promise<{ ok, status, models, detail }>
 *   streamChatCompletions({ config, messages, signal, onDelta, onDone, onError, options })
 *   completeChatCompletions({ config, messages, options })
 */

const DEFAULT_TIMEOUT_MS = 10000;
const STREAM_HEADER = 'text/event-stream';

function failure(message, extra = {}) {
    const error = new Error(message);
    Object.assign(error, extra);
    return error;
}

export function normalizeBaseUrl(value) {
    const raw = String(value == null ? '' : value).trim();
    if (!raw) throw failure('An API base URL is required.');
    let parsed;
    try {
        parsed = new URL(raw);
    } catch (_error) {
        throw failure('The API base URL is not a valid URL. Example: http://127.0.0.1:8000/v1');
    }
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
        throw failure('Only http:// and https:// API base URLs are supported.');
    }
    let path = parsed.pathname.replace(/\/+$/, '');
    path = path.replace(/\/chat\/completions$/i, '').replace(/\/models$/i, '');
    parsed.pathname = path || '';
    parsed.search = '';
    parsed.hash = '';
    const normalised = parsed.toString().replace(/\/$/, '');
    if (!normalised) throw failure('The API base URL is empty after normalisation.');
    return normalised;
}

function endpoint(baseUrl, path) {
    return normalizeBaseUrl(baseUrl) + path;
}

function configFromObject(config) {
    if (!config || typeof config !== 'object') return null;
    const baseUrl = config.baseUrl || config.base_url || config.url;
    if (!baseUrl) return null;
    return {
        id: config.id || null,
        label: config.label || config.name || null,
        baseUrl: String(baseUrl),
        model: String(config.model || config.modelId || config.model_id || ''),
        apiKey: typeof config.apiKey === 'string' ? config.apiKey : (typeof config.api_key === 'string' ? config.api_key : ''),
        maxTokens: Number(config.maxTokens) > 0 ? Math.min(4096, Number(config.maxTokens)) : (config.ephemeral ? undefined : 768),
        temperature: Number.isFinite(Number(config.temperature)) ? Math.max(0, Math.min(2, Number(config.temperature))) : 0.7,
    };
}

/** Active API configuration from the model-source manager (or a small standalone fallback). */
export function configFromSources() {
    try {
        const source = globalThis.UORModelSources;
        if (source && typeof source.apiConfig === 'function') {
            const config = source.apiConfig();
            if (config) return configFromObject(config);
        }
    } catch (_error) {
        /* fall through to standalone storage */
    }
    try {
        const raw = globalThis.localStorage && globalThis.localStorage.getItem('uor.studio.modelSources.v1');
        if (!raw) return null;
        const state = JSON.parse(raw);
        const active = state && state.active;
        if (!active || active.kind !== 'api') return null;
        const config = (state.api || []).find((item) => item.id === active.id);
        return configFromObject(config);
    } catch (_error) {
        return null;
    }
}

function requireConfig(config) {
    const resolved = configFromObject(config) || configFromSources();
    if (!resolved) {
        throw failure('Select an OpenAI-compatible API model source before using API mode.');
    }
    resolved.baseUrl = normalizeBaseUrl(resolved.baseUrl);
    return resolved;
}

function requestHeaders(config) {
    const headers = { 'Content-Type': 'application/json', Accept: 'application/json, text/event-stream' };
    if (config.apiKey) headers.Authorization = 'Bearer ' + config.apiKey;
    return headers;
}

function toHex(bytes) {
    let value = '';
    for (const byte of bytes) value += byte.toString(16).padStart(2, '0');
    return value;
}

export function imageUrlPart(image) {
    const dataUrl = String(image && image.dataUrl || '');
    if (!/^data:image\/[a-z0-9.+-]+;base64,/i.test(dataUrl)) {
        throw failure('A bounded image data URL is required.');
    }
    return { type: 'image_url', image_url: { url: dataUrl } };
}

export async function decomposeNativeImage({ config, image, grid, signal } = {}) {
    const resolved = requireConfig(config);
    const width = Number(image && image.width);
    const height = Number(image && image.height);
    const rgba = image && image.rgba;
    const columns = Number(grid && grid.columns);
    const rows = Number(grid && grid.rows);
    if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || height < 1 || width > 256 || height > 256) {
        throw failure('Native image dimensions must be within 1..256.');
    }
    if (!rgba || Number(rgba.length) !== width * height * 4) {
        throw failure('Native image decomposition requires exact RGBA bytes.');
    }
    if (!Number.isInteger(columns) || !Number.isInteger(rows) || columns < 1 || rows < 1 || columns > width || rows > height) {
        throw failure('Native image grid must fit the image.');
    }
    const url = endpoint(resolved.baseUrl, '/images/decompose') +
        '?width=' + width + '&height=' + height + '&columns=' + columns + '&rows=' + rows;
    const response = await fetch(url, {
        method: 'POST',
        headers: { ...requestHeaders(resolved), 'Content-Type': 'application/octet-stream', Accept: 'application/octet-stream' },
        body: rgba,
        signal,
        cache: 'no-store',
    });
    if (!response.ok) {
        const bodyText = await response.text();
        throw describeHttpError(response, bodyText);
    }
    const header = (name) => {
        const value = response.headers.get(name);
        if (!value) throw failure('The native server returned an invalid decomposition response.');
        return value;
    };
    const schema = header('x-uor-image-schema');
    const sourceBlake3 = header('x-uor-source-blake3');
    const decompositionId = header('x-uor-decomposition-id');
    const responseWidth = Number(header('x-uor-image-width'));
    const responseHeight = Number(header('x-uor-image-height'));
    const responseColumns = Number(header('x-uor-grid-columns'));
    const responseRows = Number(header('x-uor-grid-rows'));
    const patchCount = Number(header('x-uor-patch-count'));
    if (schema !== 'uor-r4.image-decomposition/1'
        || !/^[0-9a-f]{64}$/.test(sourceBlake3)
        || decompositionId.length > 128
        || responseWidth !== width
        || responseHeight !== height
        || responseColumns !== columns
        || responseRows !== rows
        || patchCount !== columns * rows) {
        throw failure('The native server returned an invalid decomposition response.');
    }
    const canonical = new Uint8Array(await response.arrayBuffer());
    const magic = [82, 52, 73, 77, 71, 48, 48, 49];
    if (canonical.length < magic.length
        || canonical.length > 52 + 32 * 32 * 40
        || magic.some((byte, index) => canonical[index] !== byte)) {
        throw failure('The native server returned an invalid decomposition response.');
    }
    return {
        type: 'uor_r4_image',
        schema,
        canonical_hex: toHex(canonical),
        source_blake3: sourceBlake3,
        width,
        height,
        grid: { columns, rows },
        decomposition_id: decompositionId,
    };
}

function timeoutSignal(parentSignal, timeoutMs) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(failure('The API request timed out.')), timeoutMs);
    const signal = parentSignal
        ? anySignal([parentSignal, controller.signal])
        : controller.signal;
    return { signal, dispose: () => clearTimeout(timer) };
}

function anySignal(signals) {
    if (typeof AbortSignal !== 'undefined' && typeof AbortSignal.any === 'function') {
        return AbortSignal.any(signals.filter(Boolean));
    }
    const controller = new AbortController();
    for (const signal of signals.filter(Boolean)) {
        if (signal.aborted) {
            controller.abort(signal.reason);
            break;
        }
        signal.addEventListener('abort', () => controller.abort(signal.reason), { once: true });
    }
    return controller.signal;
}

function describeHttpError(response, bodyText) {
    const body = String(bodyText || '').trim();
    let detail = body.slice(0, 400);
    try {
        const parsed = JSON.parse(body);
        const message = parsed && (parsed.error?.message || parsed.error || parsed.message || parsed.detail);
        if (typeof message === 'string' && message.trim()) detail = message.trim().slice(0, 400);
    } catch (_error) {
        /* keep raw text excerpt */
    }
    return failure('API request failed with HTTP ' + response.status + (detail ? ': ' + detail : '.'), {
        status: response.status,
    });
}

/** GET {base}/models. Returns the model ids when the endpoint exposes them. */
export async function listModels(config, options = {}) {
    const resolved = requireConfig(config);
    const timeoutMs = Number(options.timeoutMs) > 0 ? Number(options.timeoutMs) : DEFAULT_TIMEOUT_MS;
    const { signal, dispose } = timeoutSignal(options.signal, timeoutMs);
    try {
        const response = await fetch(endpoint(resolved.baseUrl, '/models'), {
            method: 'GET',
            headers: requestHeaders(resolved),
            signal,
            cache: 'no-store',
        });
        const bodyText = await response.text();
        if (!response.ok) throw describeHttpError(response, bodyText);
        let models = [];
        try {
            const parsed = JSON.parse(bodyText);
            const data = parsed && Array.isArray(parsed.data) ? parsed.data : [];
            models = data.map((item) => item && (item.id || item.model || item.name)).filter(Boolean).map(String);
        } catch (_error) {
            /* expose the endpoint as reachable even when /models is not JSON */
        }
        return { ok: true, status: response.status, models, detail: 'Endpoint reachable.' };
    } catch (error) {
        if (error && error.name === 'AbortError') throw failure('The API connection test timed out.');
        throw error;
    } finally {
        dispose();
    }
}

/** Test an API configuration without sending a prompt: GET /models. */
export async function testConnection(config) {
    const resolved = requireConfig(config);
    try {
        const result = await listModels(resolved);
        return {
            ok: true,
            status: result.status,
            models: result.models,
            detail: result.models.length
                ? 'Reachable. Models: ' + result.models.slice(0, 6).join(', ') + (result.models.length > 6 ? '…' : '')
                : 'Reachable (the endpoint did not list models).',
        };
    } catch (error) {
        return { ok: false, status: error && error.status ? error.status : 0, models: [], detail: String(error && error.message ? error.message : error) };
    }
}

async function normaliseMessages(messages) {
    if (!Array.isArray(messages) || messages.length === 0) throw failure('At least one chat message is required.');
    const normalized = [];
    for (const message of messages) {
        if (!message) continue;
        const content = await message.content;
        if (!((typeof content === 'string' && content.length > 0) || (Array.isArray(content) && content.length > 0))) continue;
        normalized.push({ role: ['system', 'user', 'assistant'].includes(message.role) ? message.role : 'user', content });
    }
    return normalized;
}

/** Parse one SSE event block ("data: {...}") into an OpenAI streaming chunk. */
function parseSseBlock(block) {
    const lines = block.split(/\r?\n/);
    const dataLines = lines
        .filter((line) => line.startsWith('data:'))
        .map((line) => line.slice(5).trim());
    if (dataLines.length === 0) return null;
    const payload = dataLines.join('\n');
    if (payload === '[DONE]') return { done: true };
    try {
        return { chunk: JSON.parse(payload) };
    } catch (_error) {
        return { error: failure('The API returned malformed SSE JSON.') };
    }
}

/**
 * Stream an OpenAI-compatible chat completion.
 * Calls onDelta(text, meta) for each content fragment and resolves when the stream ends.
 */
export async function streamChatCompletions({ config, messages, signal, onDelta, onDone, onError, options = {} } = {}) {
    let errorReported = false;
    const reportError = (error) => {
        if (errorReported) return;
        errorReported = true;
        if (typeof onError === 'function') onError(error);
    };
    let resolved;
    try {
        resolved = requireConfig(config);
    } catch (error) {
        reportError(error);
        throw error;
    }
    const body = {
        model: resolved.model || 'default',
        messages: await normaliseMessages(messages),
        stream: true,
        stream_options: { include_usage: true },
        temperature: Number.isFinite(Number(options.temperature)) ? Number(options.temperature) : resolved.temperature,
        max_tokens: Number(options.maxTokens) > 0 ? Math.min(4096, Number(options.maxTokens)) : resolved.maxTokens,
    };
    let text = '';
    let usage;
    let uor;
    let finishReason = null;
    const complete = (stopped) => {
        const result = { ok: !stopped, stopped, text, usage, uor, finishReason };
        if (typeof onDone === 'function') onDone(result);
        return result;
    };
    const emit = (delta, meta = {}) => {
        const hasToken = meta.uor && Number.isFinite(Number(meta.uor.token_id));
        if (!delta && !hasToken) return;
        text += delta;
        if (typeof onDelta === 'function') onDelta(delta, { ...meta, text });
    };
    const applyChunk = (chunk) => {
        if (chunk && chunk.error) {
            const detail = chunk.error;
            const message = typeof detail === 'string' ? detail : (detail.message || detail.code || 'unknown API error');
            const error = failure('The API returned an error event: ' + message, {
                status: Number.isFinite(Number(detail.status)) ? Number(detail.status) : 500,
                code: detail.code,
                param: detail.param,
                errorType: detail.type,
            });
            reportError(error);
            throw error;
        }
        if (chunk && chunk.usage && typeof chunk.usage === 'object') usage = chunk.usage;
        if (chunk && chunk.uor && typeof chunk.uor === 'object' && !Array.isArray(chunk.uor.token_id)) uor = chunk.uor;
        const choice = chunk && chunk.choices && chunk.choices[0];
        if (choice) {
            finishReason = choice.finish_reason || finishReason;
            const delta = choice.delta && typeof choice.delta.content === 'string'
                ? choice.delta.content
                : (choice.message && typeof choice.message.content === 'string'
                    ? choice.message.content
                    : (typeof choice.text === 'string' ? choice.text : ''));
            emit(delta, { finishReason, model: chunk.model || resolved.model, uor: chunk.uor });
            return;
        }
        if (chunk && chunk.uor && typeof chunk.uor.stop === 'string') finishReason = finishReason || chunk.uor.stop;
    };
    let response;
    try {
        response = await fetch(endpoint(resolved.baseUrl, '/chat/completions'), {
            method: 'POST',
            headers: requestHeaders(resolved),
            signal,
            body: JSON.stringify(body),
        });
    } catch (error) {
        if (error && error.name === 'AbortError') return complete(true);
        reportError(error);
        throw error;
    }
    if (!response.ok) {
        let bodyText = '';
        try {
            bodyText = await response.text();
        } catch (_error) {
            /* ignore */
        }
        const error = describeHttpError(response, bodyText);
        reportError(error);
        throw error;
    }

    const contentType = String(response.headers.get('content-type') || '').toLowerCase();
    if (!contentType.includes(STREAM_HEADER) || !response.body) {
        const bodyText = await response.text();
        let parsed;
        try {
            parsed = JSON.parse(bodyText);
        } catch (_error) {
            const error = failure('The API returned a non-JSON chat completion.');
            reportError(error);
            throw error;
        }
        applyChunk(parsed);
        return complete(false);
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    const nextBoundary = () => {
        const match = /\r?\n\r?\n/.exec(buffer);
        return match ? { index: match.index, length: match[0].length } : null;
    };
    try {
        for (;;) {
            const { value, done } = await reader.read();
            if (done) break;
            buffer += decoder.decode(value, { stream: true });
            let boundary = nextBoundary();
            while (boundary) {
                const block = buffer.slice(0, boundary.index);
                buffer = buffer.slice(boundary.index + boundary.length);
                const parsed = parseSseBlock(block);
                if (parsed && parsed.done) return complete(false);
                if (parsed && parsed.error) {
                    reportError(parsed.error);
                    throw parsed.error;
                }
                if (parsed && parsed.chunk) applyChunk(parsed.chunk);
                boundary = nextBoundary();
            }
        }
        buffer += decoder.decode();
        if (buffer.trim()) {
            const parsed = parseSseBlock(buffer.trim());
            if (parsed && parsed.error) {
                reportError(parsed.error);
                throw parsed.error;
            }
            if (parsed && parsed.chunk) applyChunk(parsed.chunk);
        }
        return complete(false);
    } catch (error) {
        if (error && error.name === 'AbortError') return complete(true);
        reportError(error);
        throw error;
    } finally {
        try {
            reader.releaseLock();
        } catch (_error) {
            /* ignore */
        }
    }
}

/** Non-streaming completion (used by diagnostics; chat uses the streaming path). */
export async function completeChatCompletions({ config, messages, options = {} } = {}) {
    const resolved = requireConfig(config);
    const response = await fetch(endpoint(resolved.baseUrl, '/chat/completions'), {
        method: 'POST',
        headers: requestHeaders(resolved),
        body: JSON.stringify({
            model: resolved.model || 'default',
            messages: await normaliseMessages(messages),
            stream: false,
            temperature: Number.isFinite(Number(options.temperature)) ? Number(options.temperature) : resolved.temperature,
            max_tokens: Number(options.maxTokens) > 0 ? Math.min(4096, Number(options.maxTokens)) : resolved.maxTokens,
        }),
    });
    const bodyText = await response.text();
    if (!response.ok) throw describeHttpError(response, bodyText);
    const parsed = JSON.parse(bodyText);
    const content = parsed?.choices?.[0]?.message?.content ?? parsed?.choices?.[0]?.text ?? '';
    return { ok: true, text: String(content || ''), raw: parsed };
}

const api = {
    normalizeBaseUrl,
    configFromSources,
    listModels,
    testConnection,
    decomposeNativeImage,
    imageUrlPart,
    streamChatCompletions,
    completeChatCompletions,
};

if (typeof globalThis !== 'undefined') {
    globalThis.UORStudioApi = api;
}

export default api;
