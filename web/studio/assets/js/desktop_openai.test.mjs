import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createDesktopOpenAI } from './desktop_openai.js';

class TestChannel {
    constructor() {
        this.onmessage = null;
        this.queue = [];
    }
    emit(event) {
        if (this.onmessage) this.onmessage(event);
    }
}

function chatResponse({ stream = true, status = 200, chunks = [] }) {
    return async (command, args) => {
        if (command === 'host_cancel_chat') return {};
        if (command !== 'native_chat_completions') throw new Error('unexpected ' + command);
        const headers = { 'content-type': stream ? 'text/event-stream' : 'application/json' };
        for (const chunk of chunks) args.onEvent.emit({ status, headers, bodyChunk: Array.from(new TextEncoder().encode(chunk)), terminal: false });
        args.onEvent.emit({ status, headers, bodyChunk: [], terminal: true });
        return { status, headers };
    };
}

test('desktop Response shim preserves SSE usage, headers, and terminal events', async () => {
    const { fetch } = createDesktopOpenAI({ invoke: chatResponse({ chunks: ['data: {"choices":[],"usage":{"prompt_tokens":19,"completion_tokens":16,"total_tokens":35}}\r\n\r\n', 'data: [DONE]\r\n\r\n'] }), Channel: TestChannel });
    const response = await fetch('/v1/chat/completions', { method: 'POST', body: '{}' });
    assert.equal(response.status, 200);
    assert.match(response.headers.get('content-type'), /text\/event-stream/);
    const text = await response.text();
    assert.match(text, /19.*16.*35/);
    assert.match(text, /\[DONE\]/);
});

test('desktop Response shim preserves non-streaming JSON and error statuses', async () => {
    const { fetch } = createDesktopOpenAI({ invoke: chatResponse({ stream: false, status: 413, chunks: ['{"error":{"message":"context"}}'] }), Channel: TestChannel });
    const response = await fetch('/v1/chat/completions', { method: 'POST', body: '{}' });
    assert.equal(response.status, 413);
    assert.match(response.headers.get('content-type'), /application\/json/);
    assert.deepEqual(await response.json(), { error: { message: 'context' } });
});

test('desktop abort calls host cancellation and closes the reader', async () => {
    const cancelled = [];
    const { fetch } = createDesktopOpenAI({
        invoke: async (command, args) => {
            if (command === 'host_cancel_chat') { cancelled.push(args.requestId); return {}; }
            return chatResponse({ chunks: [] })(command, args);
        },
        Channel: TestChannel,
    });
    const controller = new AbortController();
    const response = await fetch('/v1/chat/completions', { method: 'POST', body: '{}', signal: controller.signal });
    controller.abort();
    await assert.rejects(response.text(), /aborted|AbortError/);
    assert.equal(cancelled.length, 1);
    assert.match(cancelled[0], /-request-1$/);
});

test('desktop request identifiers do not repeat across page transport instances', async () => {
    const cancelled = [];
    const invoke = async (command, args) => {
        if (command === 'host_cancel_chat') { cancelled.push(args.requestId); return {}; }
        return chatResponse({ chunks: [] })(command, args);
    };
    for (let index = 0; index < 2; index += 1) {
        const { fetch } = createDesktopOpenAI({ invoke, Channel: TestChannel });
        const controller = new AbortController();
        await fetch('/v1/chat/completions', { method: 'POST', body: '{}', signal: controller.signal });
        controller.abort();
    }
    assert.equal(cancelled.length, 2);
    assert.notEqual(cancelled[0], cancelled[1]);
    assert.notEqual(cancelled[0], 'request-1');
});

test('desktop queue overflow aborts instead of growing without bound', async () => {
    const cancelled = [];
    const { fetch } = createDesktopOpenAI({
        invoke: async (command, args) => {
            if (command === 'host_cancel_chat') { cancelled.push(args.requestId); return {}; }
            args.onEvent.emit({ status: 200, headers: { 'content-type': 'text/event-stream' }, bodyChunk: new Array(2049).fill(120), terminal: false });
            return { status: 200, headers: { 'content-type': 'text/event-stream' } };
        },
        Channel: TestChannel,
        maxQueueBytes: 1024,
    });
    const response = await fetch('/v1/chat/completions', { method: 'POST', body: '{}' });
    await assert.rejects(response.text(), /queue overflow/);
    assert.equal(cancelled.length, 1);
    assert.match(cancelled[0], /-request-1$/);
});

test('desktop queue releases capped bytes as the reader consumes chunks', async () => {
    let channel;
    const { fetch } = createDesktopOpenAI({
        invoke: async (command, args) => {
            if (command === 'host_cancel_chat') return {};
            channel = args.onEvent;
            return { status: 200, headers: { 'content-type': 'text/event-stream' } };
        },
        Channel: TestChannel,
        maxQueueBytes: 1024,
    });
    const response = await fetch('/v1/chat/completions', { method: 'POST', body: '{}' });
    const reader = response.body.getReader();
    const event = (bodyChunk, terminal = false) => ({ status: 200, headers: {}, bodyChunk, terminal });
    channel.emit(event(new Array(1024).fill(120)));
    assert.equal((await reader.read()).value.length, 1024);
    channel.emit(event(new Array(1024).fill(121)));
    assert.equal((await reader.read()).value.length, 1024);
    channel.emit(event([], true));
    assert.equal((await reader.read()).done, true);
});

test('desktop transport refuses custom provider origins instead of routing them to uor-chat', async () => {
    globalThis.location = new URL('https://tauri.localhost/');
    const calls = [];
    const { fetch } = createDesktopOpenAI({ invoke: async (...args) => { calls.push(args); throw new Error('must not invoke'); }, Channel: TestChannel });
    await assert.rejects(fetch('https://api.example.com/v1/chat/completions', { method: 'POST', body: '{}' }), /browser build/);
    assert.deepEqual(calls, []);
    delete globalThis.location;
});

test('desktop transport accepts the matching HTTP Tauri asset origin', async () => {
    globalThis.location = new URL('tauri://localhost/');
    const { fetch } = createDesktopOpenAI({
        invoke: async (command) => {
            assert.equal(command, 'native_models');
            return { status: 200, headers: { 'content-type': 'application/json' }, body: Array.from(new TextEncoder().encode('{"data":[{"id":"uor-chat"}]}')) };
        },
        Channel: TestChannel,
    });
    const response = await fetch('http://tauri.localhost/v1/models');
    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), { data: [{ id: 'uor-chat' }] });
    delete globalThis.location;
});

test('desktop transport refuses non-default ports and userinfo on the native hostname', async () => {
    globalThis.location = new URL('tauri://localhost/');
    const calls = [];
    const { fetch } = createDesktopOpenAI({ invoke: async (...args) => { calls.push(args); throw new Error('must not invoke'); }, Channel: TestChannel });
    for (const url of [
        'http://tauri.localhost:8000/v1/chat/completions',
        'https://tauri.localhost:444/v1/chat/completions',
        'https://user@tauri.localhost/v1/chat/completions',
        'tauri://evil/v1/chat/completions',
    ]) await assert.rejects(fetch(url, { method: 'POST', body: '{}' }), /browser build|same-origin/);
    assert.deepEqual(calls, []);
    delete globalThis.location;
});

test('the Tauri boot registers the desktop transport before native model bootstrap', () => {
    const html = readFileSync(new URL('../../index.html', import.meta.url), 'utf8');
    assert.match(html, /import \{ registerDesktopOpenAI \} from '\.\/assets\/js\/desktop_openai\.js'/);
    const registration = html.indexOf('registerDesktopOpenAI({');
    const bootstrap = html.indexOf('ModelSources.registerNativeBootstrap', registration);
    assert.ok(registration > 0, 'desktop registration is missing');
    assert.ok(bootstrap > registration, 'native model bootstrap ran before desktop registration');
    assert.match(html, /window\.__TAURI__[\s\S]*?\.core\.Channel/);
});
