import test from 'node:test';
import assert from 'node:assert/strict';
import { streamChatCompletions } from './studio_api.js';

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
