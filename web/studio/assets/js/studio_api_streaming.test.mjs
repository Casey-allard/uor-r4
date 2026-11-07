import test from 'node:test';
import assert from 'node:assert/strict';
import { decomposeNativeImage, imageUrlPart, streamChatCompletions, testConnection } from './studio_api.js';

const config = { baseUrl: 'http://127.0.0.1:8765/v1', model: 'uor-chat', apiKey: 'secret', temperature: 0, ephemeral: true };
const encoder = new TextEncoder();

function responseFromChunks(chunks, init = {}) {
    const stream = new ReadableStream({
        start(controller) {
            for (const chunk of chunks) controller.enqueue(encoder.encode(chunk));
            controller.close();
        },
    });
    return new Response(stream, { status: 200, headers: { 'content-type': 'text/event-stream' }, ...init });
}

test('split CRLF SSE events preserve text, final usage, native timing, and finish reason', async () => {
    const role = JSON.stringify({ choices: [{ index: 0, delta: { role: 'assistant', content: '' }, finish_reason: null }] });
    const token = JSON.stringify({ choices: [{ index: 0, delta: { content: ' I' }, finish_reason: null }], uor: { token_id: 340 } });
    const eos = JSON.stringify({ choices: [{ index: 0, delta: { content: '' }, finish_reason: null }], uor: { token_id: 1 } });
    const finish = JSON.stringify({ choices: [{ index: 0, delta: {}, finish_reason: 'stop' }], uor: { stop: 'eos' } });
    const usage = JSON.stringify({ choices: [], usage: { prompt_tokens: 12, completion_tokens: 3, total_tokens: 15 }, uor: { completion_ids: [340, 16, 1], elapsed_ns: 5000000, first_token_ns: 2000000, tokens_per_second_milli: 600000, stop: 'eos' } });
    const wire = `data: ${role}\r\n\r\ndata: ${token}\r\n\r\ndata: ${eos}\r\n\r\ndata: ${finish}\r\n\r\ndata: ${usage}\r\n\r\ndata: [DONE]\r\n\r\n`;
    const chunks = [wire.slice(0, 17), wire.slice(17, 83), wire.slice(83, 191), wire.slice(191)];
    let request;
    let done;
    const deltas = [];
    globalThis.fetch = async (url, init) => {
        request = { url, init };
        return responseFromChunks(chunks);
    };
    const result = await streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onDelta: (text, meta) => deltas.push({ text, meta }), onDone: (value) => { done = value; } });
    const body = JSON.parse(request.init.body);
    assert.equal(body.stream, true);
    assert.deepEqual(body.stream_options, { include_usage: true });
    assert.equal(body.temperature, 0);
    assert.equal('max_tokens' in body, false);
    assert.equal(result.ok, true);
    assert.equal(result.text, ' I');
    assert.deepEqual(result.usage, { prompt_tokens: 12, completion_tokens: 3, total_tokens: 15 });
    assert.deepEqual(result.uor, { completion_ids: [340, 16, 1], elapsed_ns: 5000000, first_token_ns: 2000000, tokens_per_second_milli: 600000, stop: 'eos' });
    assert.equal(result.finishReason, 'stop');
    assert.equal(done.stopped, false);
    assert.deepEqual(done.usage, result.usage);
    assert.deepEqual(done.uor, result.uor);
    assert.ok(deltas.some((entry) => entry.meta.uor && entry.meta.uor.token_id === 340));
    assert.ok(deltas.some((entry) => entry.text === '' && entry.meta.uor && entry.meta.uor.token_id === 1));
});

test('a non-stream JSON fallback returns exact usage and native timing', async () => {
    const payload = {
        choices: [{ index: 0, message: { role: 'assistant', content: 'I.' }, finish_reason: 'stop' }],
        usage: { prompt_tokens: 12, completion_tokens: 3, total_tokens: 15 },
        uor: { completion_ids: [340, 16, 1], elapsed_ns: 5000000, first_token_ns: 2000000, tokens_per_second_milli: 600000, stop: 'eos' },
    };
    globalThis.fetch = async () => new Response(JSON.stringify(payload), { status: 200, headers: { 'content-type': 'application/json' } });
    let done;
    const result = await streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onDone: (value) => { done = value; } });
    assert.equal(result.text, 'I.');
    assert.deepEqual(result.usage, payload.usage);
    assert.deepEqual(result.uor, payload.uor);
    assert.equal(result.finishReason, 'stop');
    assert.deepEqual(done.usage, payload.usage);
});

test('text and native image content parts survive request normalization unchanged', async () => {
    const payload = { choices: [{ index: 0, message: { role: 'assistant', content: 'I.' }, finish_reason: 'stop' }] };
    const messages = [{ role: 'user', content: [
        { type: 'text', text: 'Hi' },
        { type: 'uor_r4_image', schema: 'uor-r4.image-decomposition/1', canonical_hex: '5234494d4730303166697874757265', source_blake3: 'a'.repeat(64), width: 1, height: 1, grid: { columns: 1, rows: 1 }, decomposition_id: 'r4img-test' },
    ] }];
    let request;
    globalThis.fetch = async (url, init) => {
        request = init;
        return new Response(JSON.stringify(payload), { status: 200, headers: { 'content-type': 'application/json' } });
    };
    const result = await streamChatCompletions({ config, messages });
    assert.equal(result.text, 'I.');
    assert.deepEqual(JSON.parse(request.body).messages, messages);
});

test('custom image_url parts and provider errors remain verbatim', async () => {
    const messages = [{ role: 'user', content: [
        { type: 'text', text: 'Hi' },
        { type: 'image_url', image_url: { url: 'data:image/png;base64,preview' } },
    ] }];
    let request;
    globalThis.fetch = async (url, init) => {
        request = init;
        return new Response(JSON.stringify({ error: { message: 'images are not supported by this provider' } }), { status: 501, headers: { 'content-type': 'application/json' } });
    };
    await assert.rejects(streamChatCompletions({ config, messages }), /501.*images are not supported by this provider/);
    assert.deepEqual(JSON.parse(request.body).messages, messages);
});

test('mid-stream abort returns the partial text without raising a second error', async () => {
    const stream = new ReadableStream({
        pull(controller) {
            if (!controller.enqueued) {
                controller.enqueued = true;
                controller.enqueue(encoder.encode(`data: ${JSON.stringify({ choices: [{ delta: { content: 'I' }, finish_reason: null }] })}\r\n\r\n`));
                return;
            }
            return Promise.reject(new DOMException('aborted', 'AbortError'));
        },
    });
    globalThis.fetch = async () => new Response(stream, { status: 200, headers: { 'content-type': 'text/event-stream' } });
    let done;
    let errors = 0;
    const result = await streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onDone: (value) => { done = value; }, onError: () => { errors += 1; } });
    assert.equal(result.stopped, true);
    assert.equal(result.text, 'I');
    assert.equal(done.stopped, true);
    assert.equal(done.text, 'I');
    assert.equal(errors, 0);
});

test('OpenAI error envelopes retain status and message', async () => {
    globalThis.fetch = async () => new Response(JSON.stringify({ error: { code: 'not_implemented', message: 'this OpenAI endpoint family is not implemented', type: 'server_error' } }), { status: 501, headers: { 'content-type': 'application/json' } });
    let reported;
    await assert.rejects(streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onError: (error) => { reported = error; } }), (error) => {
        assert.equal(error.status, 501);
        assert.match(error.message, /501.*this OpenAI endpoint family is not implemented/);
        return true;
    });
    assert.equal(reported.status, 501);
});

test('malformed SSE data and in-band OpenAI errors fail instead of completing successfully', async () => {
    let errors = 0;
    globalThis.fetch = async () => responseFromChunks(['data: {not-json}\r\n\r\n']);
    await assert.rejects(streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onError: () => { errors += 1; } }), /malformed SSE JSON/);
    globalThis.fetch = async () => responseFromChunks([`data: ${JSON.stringify({ error: { code: 'busy', message: 'one generation is already in flight', type: 'rate_limit_error' } })}\r\n\r\n`]);
    await assert.rejects(streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }], onError: () => { errors += 1; } }), /error event.*one generation is already in flight/);
    assert.equal(errors, 2);
});

test('a successful non-JSON response fails loudly instead of inventing a reply', async () => {
    globalThis.fetch = async () => new Response('plain text', { status: 200, headers: { 'content-type': 'text/plain' } });
    await assert.rejects(streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }] }), /non-JSON chat completion/);
});

test('native image decomposition returns the exact structured content part', async () => {
    let request;
    globalThis.fetch = async (url, init) => {
        request = { url, init };
        return new Response(new TextEncoder().encode('R4IMG001fixture'), {
            status: 200,
            headers: {
                'content-type': 'application/octet-stream',
                'x-uor-image-schema': 'uor-r4.image-decomposition/1',
                'x-uor-source-blake3': 'ab'.repeat(32),
                'x-uor-image-width': '1',
                'x-uor-image-height': '1',
                'x-uor-grid-columns': '1',
                'x-uor-grid-rows': '1',
                'x-uor-patch-count': '1',
                'x-uor-decomposition-id': 'r4img-test',
            },
        });
    };
    const part = await decomposeNativeImage({
        config,
        image: { width: 1, height: 1, rgba: new Uint8Array([255, 0, 0, 255]) },
        grid: { columns: 1, rows: 1 },
    });
    const url = new URL(request.url);
    assert.equal(url.pathname, '/v1/images/decompose');
    assert.deepEqual(Object.fromEntries(url.searchParams), { width: '1', height: '1', columns: '1', rows: '1' });
    assert.equal(request.init.headers['Content-Type'], 'application/octet-stream');
    assert.deepEqual(part, {
        type: 'uor_r4_image',
        schema: 'uor-r4.image-decomposition/1',
        canonical_hex: '5234494d4730303166697874757265',
        source_blake3: 'ab'.repeat(32),
        width: 1,
        height: 1,
        grid: { columns: 1, rows: 1 },
        decomposition_id: 'r4img-test',
    });
});

test('structured content survives chat normalization and custom images use image_url', async () => {
    globalThis.fetch = async (_url, init) => {
        assert.deepEqual(JSON.parse(init.body).messages[0].content, [
            { type: 'text', text: 'What is this?' },
            { type: 'image_url', image_url: { url: 'data:image/png;base64,AA==' } },
        ]);
        return new Response(JSON.stringify({ choices: [{ message: { content: 'ok' }, finish_reason: 'stop' }], usage: { prompt_tokens: 4, completion_tokens: 1, total_tokens: 5 } }), { status: 200, headers: { 'content-type': 'application/json' } });
    };
    const messages = [{
        role: 'user',
        content: [
            { type: 'text', text: 'What is this?' },
            imageUrlPart({ dataUrl: 'data:image/png;base64,AA==' }),
        ],
    }];
    const result = await streamChatCompletions({ config, messages });
    assert.equal(result.ok, true);
});

test('an image-only promised native part is sent without an empty text part', async () => {
    const nativePart = {
        type: 'uor_r4_image',
        schema: 'uor-r4.image-decomposition/1',
        canonical_hex: '5234',
        source_blake3: 'ab'.repeat(32),
        width: 1,
        height: 1,
        grid: { columns: 1, rows: 1 },
    };
    globalThis.fetch = async (_url, init) => {
        assert.deepEqual(JSON.parse(init.body).messages, [{ role: 'user', content: [nativePart] }]);
        return new Response(JSON.stringify({ choices: [{ message: { content: 'ok' }, finish_reason: 'stop' }], usage: { prompt_tokens: 2, completion_tokens: 1, total_tokens: 3 } }), { status: 200, headers: { 'content-type': 'application/json' } });
    };
    const result = await streamChatCompletions({
        config,
        messages: [{ role: 'user', content: Promise.resolve([nativePart]) }],
    });
    assert.equal(result.ok, true);
});

function setLocation(url) {
    Object.defineProperty(globalThis, 'location', { value: new URL(url), configurable: true, writable: true });
}

test('a missing native runner gets an actionable connection error', async () => {
    setLocation('http://127.0.0.1:8765');
    globalThis.fetch = async () => { throw new TypeError('Failed to fetch'); };
    const result = await testConnection(config);
    assert.equal(result.ok, false);
    assert.match(result.detail, /Native runner is unreachable.*native-runner\.sh.*runner-served local Studio/);
});

test('HTTPS Pages refuse insecure HTTP loopback with native setup guidance', async () => {
    setLocation('https://uor-foundation.github.io/uor-r4/studio/');
    globalThis.fetch = async () => { throw new TypeError('Failed to fetch'); };
    await assert.rejects(
        streamChatCompletions({ config, messages: [{ role: 'user', content: 'Hi' }] }),
        /HTTPS Pages cannot call insecure HTTP loopback.*runner-served local Studio/,
    );
    setLocation('http://127.0.0.1:8765');
});

test('authentication failures keep the HTTP status and provider detail', async () => {
    globalThis.fetch = async () => new Response(JSON.stringify({ error: { message: 'bad key' } }), { status: 401, headers: { 'content-type': 'application/json' } });
    const result = await testConnection(config);
    assert.equal(result.ok, false);
    assert.match(result.detail, /API authentication failed.*401.*bad key/);
});

test('unsupported endpoint families keep the HTTP status and provider detail', async () => {
    globalThis.fetch = async () => new Response(JSON.stringify({ error: { message: 'not implemented' } }), { status: 501, headers: { 'content-type': 'application/json' } });
    const result = await testConnection(config);
    assert.equal(result.ok, false);
    assert.match(result.detail, /API endpoint is unsupported.*501.*not implemented/);
});

test('a registered desktop transport handles API requests without direct browser fetch', async () => {
    assert.equal(typeof globalThis.UORStudioApiTransport?.register, 'function');
    let directCalls = 0;
    let desktopCalls = 0;
    globalThis.fetch = async () => { directCalls += 1; throw new Error('direct fetch must not run'); };
    globalThis.UORStudioApiTransport.register(async (url) => {
        desktopCalls += 1;
        assert.equal(new URL(url).pathname, '/v1/models');
        return new Response(JSON.stringify({ data: [{ id: 'uor-chat' }] }), { status: 200, headers: { 'content-type': 'application/json' } });
    });
    try {
        const result = await testConnection(config);
        assert.equal(result.ok, true);
        assert.deepEqual(result.models, ['uor-chat']);
        assert.equal(desktopCalls, 1);
        assert.equal(directCalls, 0);
    } finally {
        globalThis.UORStudioApiTransport.register(null);
    }
});

test('exact tool exchanges stay visible but are not sent back to the model', async () => {
    const payload = { choices: [{ index: 0, message: { role: 'assistant', content: 'synthetic continuation' }, finish_reason: 'length' }] };
    let request;
    globalThis.fetch = async (_url, init) => {
        request = init;
        return new Response(JSON.stringify(payload), { status: 200, headers: { 'content-type': 'application/json' } });
    };
    const messages = [
        { role: 'user', content: 'What is 17 + 25? Answer with the number first.' },
        { role: 'assistant', content: '42\n\nR4 geometric exact arithmetic — tool, not model generation\n- p0: 7 + 5 + 0 -> 2, carry 1', tool: 'm6-exact-arithmetic' },
        { role: 'user', content: 'Hi' },
    ];
    const result = await streamChatCompletions({ config, messages });
    assert.equal(result.ok, true);
    assert.deepEqual(JSON.parse(request.body).messages, [messages[2]]);
});

async function inlineStudioFunction(name) {
    const { readFileSync } = await import('node:fs');
    const { default: vm } = await import('node:vm');
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    const start = html.indexOf(`function ${name}(`);
    assert.notEqual(start, -1, `missing inline function ${name}`);
    const open = html.indexOf('{', start);
    let depth = 0;
    for (let index = open; index < html.length; index += 1) {
        if (html[index] === '{') depth += 1;
        if (html[index] === '}') {
            depth -= 1;
            if (depth === 0) {
                const source = html.slice(start, index + 1);
                if (name === 'onWorkerMessage') {
                    const clear = await inlineStudioFunction('clearWorkerModel');
                    return { source: `${clear.source}\n${source}`, vm };
                }
                return { source, vm };
            }
        }
    }
    throw new Error(`unterminated inline function ${name}`);
}

test('apiMessages preserves exact-tool metadata for the next request', async () => {
    const { source, vm } = await inlineStudioFunction('apiMessages');
    const apiMessages = vm.runInNewContext(`${source};apiMessages`, {
        window: {},
        app: { apiConfig: null, abortController: null },
        decomposeNativeImage: null,
        imageUrlPart: null,
    });
    const messages = apiMessages({
        messages: [
            { role: 'user', content: 'What is 17 + 25? Answer with the number first.' },
            { role: 'assistant', content: '42\n\nR4 geometric exact arithmetic — tool, not model generation', tool: 'm6-exact-arithmetic' },
            { role: 'user', content: 'Hi' },
        ],
    });
    assert.equal(messages[1].tool, 'm6-exact-arithmetic');
});

test('loadConversations preserves exact-tool metadata across restore', async () => {
    const { source, vm } = await inlineStudioFunction('loadConversations');
    const stored = [{
        id: 'saved-chat',
        title: 'Saved chat',
        messages: [
            { role: 'user', content: 'What is 17 + 25? Answer with the number first.' },
            { role: 'assistant', content: '42\n\nR4 geometric exact arithmetic — tool, not model generation', tool: 'm6-exact-arithmetic' },
            { role: 'user', content: 'Hi' },
        ],
    }];
    const context = {
        CONVERSATIONS_KEY: 'conversations',
        app: { conversations: [], activeConversationId: null },
        readJson(key, fallback) {
            return key === 'conversations' ? stored : fallback;
        },
    };
    vm.runInNewContext(`${source};loadConversations`, context)();
    assert.equal(context.app.conversations[0].messages[1].tool, 'm6-exact-arithmetic');
});

function liveWorkerContext() {
    const context = {
        app: {
            caps: null,
            workerState: 'loading',
            expectedSha256: '',
            requestedFileName: 'model.lut',
            loadStartedAt: null,
        },
        setEngineStatus() {},
        describeCapabilities() { return ''; },
        document: { documentElement: { dataset: {} } },
        fillPredictorSha() {},
        fillPredictorModelInfo() {},
        updateCapabilitiesFooter() {},
        updateComposerState() {},
        renderMessages() {},
        restoreActiveConversation() {},
        isDiscreteCapabilities() { return false; },
        setPredictorMode() {},
    };
    context.setModelStatus = (detail) => { context.modelStatus = detail; };
    context.toast = (detail) => { context.toastMessage = detail; };
    return context;
}

test('live model_loaded rejects mismatched identity explicitly', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    context.app.currentLoadId = 1;
    context.app.workerState = 'loading';
    context.app.expectedSha256 = 'a'.repeat(64);
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', id: 1, capabilities: { file_name: 'other.lut', artifact_sha256: 'a'.repeat(64) } },
    });
    assert.equal(context.app.workerState, 'error');
    assert.equal(context.app.caps, null);
    assert.match(context.toastMessage, /other\.lut/);
});

test('live model_loaded rejects missing filenames and missing or invalid hashes', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    for (const [capabilities, expected] of [
        [{ artifact_sha256: 'a'.repeat(64) }, /<missing filename>/],
        [{ file_name: 'model.lut' }, /<missing hash>/],
        [{ file_name: 'model.lut', artifact_sha256: 'not-a-sha256' }, /not-a-sha256/],
    ]) {
        const context = liveWorkerContext();
        context.app.currentLoadId = 1;
        context.app.workerState = 'loading';
        context.app.expectedSha256 = 'a'.repeat(64);
        vm.runInNewContext(`${source};onWorkerMessage`, context)({
            data: { action: 'model_loaded', id: 1, capabilities },
        });
        assert.equal(context.app.workerState, 'error');
        assert.equal(context.app.caps, null);
        assert.match(context.toastMessage, expected);
    }
});

test('API selection invalidates a posted worker load before its completion arrives', async () => {
    const changed = await inlineStudioFunction('onModelSourceChanged');
    const worker = await inlineStudioFunction('onWorkerMessage');
    const vm = changed.vm;
    const context = {
        app: { currentLoadId: 1, readyLoadId: null, workerState: 'loading', mode: 'wasm', apiConfig: null, caps: null, generating: false },
        document: { documentElement: { dataset: {} } },
        setModelStatus(detail) { context.modelStatus = detail; },
        updatePrivacyStatus() {},
        updateComposerState() {},
        updateCapabilitiesFooter() {},
        renderMessages() {},
        setView() {},
        isNativeLoopback() { return false; },
        toast(detail) { context.toastMessage = detail; },
        setEngineStatus() {},
        describeCapabilities() { return 'synthetic'; },
        setPredictorMode() {},
        fillPredictorSha() {},
        fillPredictorModelInfo() {},
        ModelSources: {},
    };
    vm.runInNewContext(`${worker.source}\n${changed.source};onModelSourceChanged`, context)({
        detail: { kind: 'api', apiConfig: { baseUrl: 'https://api.example.test/v1', model: 'api-model' } },
    });
    const apiStatus = context.modelStatus;
    context.toastMessage = undefined;
    vm.runInNewContext(`${worker.source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', id: 1, capabilities: { file_name: 'model.lut', artifact_sha256: 'a'.repeat(64), synthetic: true, engine_supported: true, can_generate: true } },
    });
    assert.equal(context.app.currentLoadId, 2);
    assert.equal(context.app.readyLoadId, null);
    assert.equal(context.app.caps, null);
    assert.equal(context.modelStatus, apiStatus);
    assert.equal(context.toastMessage, undefined);
});

test('live model_loaded rejects a missing current load ID', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    context.app.currentLoadId = 1;
    context.app.workerState = 'loading';
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', capabilities: { file_name: 'model.lut', artifact_sha256: 'a'.repeat(64) } },
    });
    assert.equal(context.app.workerState, 'error');
    assert.equal(context.app.caps, null);
    assert.match(context.toastMessage, /without the current load operation ID/);
});


test('live model_loaded clears previous capabilities on current identity failure', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    let composerUpdates = 0;
    context.updateComposerState = () => { composerUpdates += 1; };
    context.app.caps = { synthetic: true, engine_supported: true, can_generate: true };
    context.app.modelMeta = { fileName: 'old.lut' };
    context.app.currentLoadId = 3;
    context.app.readyLoadId = null;
    context.app.workerState = 'loading';
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', id: 3, capabilities: { file_name: 'other.lut', artifact_sha256: 'a'.repeat(64) } },
    });
    assert.equal(context.app.workerState, 'error');
    assert.equal(context.app.caps, null);
    assert.equal(context.app.modelMeta, null);
    assert.equal(context.document.documentElement.dataset.modelSha256, '');
    assert.ok(composerUpdates > 0);
    const readiness = await inlineStudioFunction('chatReadiness');
    assert.equal(vm.runInNewContext(`${readiness.source};chatReadiness`, {
        app: context.app,
        normalizeBaseUrl() { return 'http://127.0.0.1'; },
        isDiscreteCapabilities() { return false; },
    })(), false);
});

test('live model_loaded ignores a stale numeric load response', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    context.app.currentLoadId = 3;
    context.app.readyLoadId = null;
    context.app.workerState = 'loading';
    context.app.expectedSha256 = 'a'.repeat(64);
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', id: 2, capabilities: { file_name: 'model.lut', artifact_sha256: 'a'.repeat(64) } },
    });
    assert.equal(context.app.workerState, 'loading');
    assert.equal(context.app.caps, null);
    assert.equal(context.toastMessage, undefined);
});

test('exact-tool local completion omits model rates, stop reasons, allocation deltas, and checkpoints', async () => {
    const { source, vm } = await inlineStudioFunction('finishLocalGeneration');
    const conversation = { id: 'chat-tool', messages: [] };
    const app = {
        chatRequests: { gen_tool: 7 },
        activeChatRequestId: 'gen_tool',
        activeGenerationId: 7,
        chatGenerationId: 7,
        generating: true,
        assistantBuffer: '',
        conversations: [conversation],
        activeConversationId: conversation.id,
        caps: { turn_memory: { present: true } },
    };
    const appended = [];
    let checkpointCalls = 0;
    const context = {
        app,
        chatRequestMatches(id, generationId) { return id === 'gen_tool' && generationId === 7; },
        activeConversation() { return conversation; },
        document: { querySelector() { return null; }, dispatchEvent() {} },
        CustomEvent: class { constructor(type, init) { this.type = type; this.detail = init.detail; } },
        appendMessageElement(role, text, meta) { appended.push({ role, text, meta }); },
        touchConversation() {},
        updateComposerState() {},
        setMetrics(text) { context.footerMetrics = text; },
        requestCheckpoint() { checkpointCalls += 1; },
        renderConversationList() {},
        scrollChat() {},
        clearChatRequest(id) { delete app.chatRequests[id]; },
        formatTps(value) { return String(value); },
    };
    vm.runInNewContext(`${source};finishLocalGeneration`, context)({
        id: 'gen_tool',
        fullText: '42\n\nR4 geometric exact arithmetic — tool, not model generation',
        tokenCount: 0,
        tps: null,
        stoppedBy: 'tool',
        tool: 'm6-exact-arithmetic',
        allocDelta: 0,
    });
    assert.equal(appended.length, 1);
    assert.equal(appended[0].meta, '0 model tokens · tool');
    assert.equal(context.footerMetrics, '0 model tokens · tool');
    assert.doesNotMatch(appended[0].meta, /tok\/s|length|stop|alloc/);
    assert.equal(checkpointCalls, 0, 'an exact answer must not request an unsupported turn-memory checkpoint');
});

test('exact-tool API completion omits computed model metrics in every render phase', async () => {
    const { source, vm } = await inlineStudioFunction('sendViaApi');
    const { createStreamMetrics } = await import('./stream_metrics.js');
    const conversation = { id: 'api-tool', messages: [] };
    const app = {
        apiConfig: config,
        apiMetrics: null,
        assistantBuffer: '',
        apiCompletionIds: [],
        apiStartedNs: null,
        apiFirstTokenNs: null,
        generating: false,
    };
    const context = {
        app,
        AbortController: globalThis.AbortController,
        beginGeneration() {
            app.activeChatRequestId = 11;
            app.chatGenerationId = 12;
            app.activeGenerationId = 12;
            app.generating = true;
            app.assistantBuffer = '';
            app.apiMetrics = createStreamMetrics();
            app.apiStartedNs = 0;
            app.apiFirstTokenNs = null;
        },
        streamChatCompletions(options) {
            options.onDelta('42', { uor: { tool: 'm6-exact-arithmetic' } });
            options.onDone({
                usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
                uor: { tool: 'm6-exact-arithmetic', stop: 'tool' },
                finishReason: 'stop',
                stopped: false,
            });
            return Promise.resolve({ ok: true });
        },
        configFromSources() { return config; },
        chatRequestMatches() { return true; },
        window: { performance: { now() { return 1; } } },
        document: { querySelector() { return null; }, dispatchEvent() {} },
        CustomEvent: class { constructor(type, init) { this.type = type; this.detail = init.detail; } },
        touchConversation() {},
        updatePrivacyStatus() {},
        updateComposerState() {},
        setMetrics(text) { (context.metricLabels ||= []).push(text); },
        renderConversationList() {},
        scrollChat() {},
        clearChatRequest() {},
        finishApiWithError(error) { throw error; },
    };
    await vm.runInNewContext(`async ${source};sendViaApi`, context)('17+25', conversation, [{ role: 'user', content: '17+25' }]);
    assert.equal(conversation.messages.length, 1);
    assert.equal(conversation.messages[0].meta, '0 model tokens · tool');
    assert.doesNotMatch(conversation.messages[0].meta, /tok\/s|length|stop|alloc/);
    assert.deepEqual(context.metricLabels, ['0 model tokens · tool', '0 model tokens · tool']);
});

test('exact-tool local streaming uses the tool label', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    context.app.streamingBodyEl = null;
    context.app.assistantBuffer = '';
    context.scrollChat = () => {};
    context.formatTps = () => 'n/a';
    context.setMetrics = (text) => { context.footerMetrics = text; };
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: {
            action: 'stream_token',
            fullText: '42\n\nR4 geometric exact arithmetic — tool, not model generation',
            tokenCount: 0,
            tps: null,
            tool: 'm6-exact-arithmetic',
        },
    });
    assert.equal(context.footerMetrics, '0 model tokens · tool');
});


test('live model_loaded rejects a valid but wrong expected digest', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    const actual = 'b'.repeat(64);
    context.app.currentLoadId = 1;
    context.app.workerState = 'loading';
    context.app.expectedSha256 = 'a'.repeat(64);
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'model_loaded', id: 1, capabilities: { file_name: 'model.lut', artifact_sha256: actual } },
    });
    assert.equal(context.app.workerState, 'error');
    assert.equal(context.app.caps, null);
    assert.match(context.toastMessage, new RegExp(actual));
    assert.match(context.toastMessage, new RegExp('a'.repeat(64)));
});


test('current load operation_error preserves both full digests', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    const expected = 'a'.repeat(64);
    const actual = 'b'.repeat(64);
    context.app.currentLoadId = 7;
    context.app.predictionRequests = {};
    context.app.workerState = 'loading';
    context.app.expectedSha256 = expected;
    const error = `Artifact sha256 ${actual} does not match the expected ${expected}; refusing to load.`;
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'operation_error', operation: 'load_model', id: 7, error },
    });
    assert.equal(context.app.workerState, 'error');
    assert.equal(context.app.caps, null);
    assert.match(context.modelStatus, new RegExp(expected));
    assert.match(context.modelStatus, new RegExp(actual));
});


test('live model_loaded rejects no-ID in ready and pre-load states', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    for (const state of ['ready', 'pre-load']) {
        const context = liveWorkerContext();
        if (state === 'ready') {
            context.app.currentLoadId = 1;
            context.app.readyLoadId = 1;
            context.app.workerState = 'ready';
        } else {
            context.app.currentLoadId = null;
            context.app.readyLoadId = null;
            context.app.workerState = 'starting';
        }
        vm.runInNewContext(`${source};onWorkerMessage`, context)({
            data: { action: 'model_loaded', capabilities: { file_name: 'model.lut', artifact_sha256: 'a'.repeat(64) } },
        });
        assert.equal(context.app.workerState, 'error', state);
        assert.equal(context.app.caps, null, state);
        assert.match(context.toastMessage, /without the current load operation ID/, state);
    }
});

test('stale load errors are not recorded in current diagnostics', async () => {
    const { source, vm } = await inlineStudioFunction('onWorkerMessage');
    const context = liveWorkerContext();
    context.app.currentLoadId = 3;
    context.app.readyLoadId = 3;
    context.app.workerState = 'ready';
    context.app.caps = { synthetic: true, engine_supported: true, can_generate: true };
    vm.runInNewContext(`${source};onWorkerMessage`, context)({
        data: { action: 'operation_error', operation: 'load_model', id: 2, error: 'stale failure' },
    });
    assert.equal(context.app.lastWorkerError, undefined);
    assert.equal(context.document.documentElement.dataset.lastWorkerError, undefined);
    assert.equal(context.toastMessage, undefined);
    assert.equal(context.app.workerState, 'ready');
});
