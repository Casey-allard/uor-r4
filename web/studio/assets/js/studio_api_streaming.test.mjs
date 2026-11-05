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
