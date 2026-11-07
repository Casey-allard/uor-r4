import { isDeclaredDesktopOrigin } from './desktop_origin.js';

const DEFAULT_MAX_QUEUE_BYTES = 256 * 1024;

class DesktopEventQueue {
    constructor(maxBytes) {
        this.maxBytes = maxBytes;
        this.bytes = 0;
        this.chunks = [];
        this.waiters = [];
        this.done = false;
        this.error = null;
    }

    push(event) {
        if (this.done || this.error) return;
        const body = event.bodyChunk || event.body_chunk || [];
        if (event.error) {
            this.close(new Error(String(event.error)));
            return;
        }
        if (this.bytes + body.length > this.maxBytes) {
            this.close(new Error('Desktop event queue overflow'));
            if (typeof this.onOverflow === 'function') this.onOverflow();
            return;
        }
        if (body.length) {
            this.chunks.push(Uint8Array.from(body));
            this.bytes += body.length;
        }
        if (event.terminal) this.done = true;
        this.wake();
    }

    close(error) {
        if (this.error) return;
        this.done = true;
        this.error = error || new Error('Desktop channel closed');
        this.wake();
    }

    wake() {
        const waiters = this.waiters.splice(0);
        for (const { resolve, reject } of waiters) {
            if (this.error) reject(this.error);
            else resolve();
        }
    }

    async read() {
        if (this.chunks.length) {
            const chunk = this.chunks.shift();
            this.bytes -= chunk.length;
            return chunk;
        }
        if (this.error) throw this.error;
        if (this.done) return null;
        await new Promise((resolve, reject) => this.waiters.push({ resolve, reject }));
        return this.read();
    }
}

function directResponse(result) {
    return new Response(Uint8Array.from(result.body || []), {
        status: result.status,
        headers: result.headers || {},
    });
}

function jsonResponse(value, status = 200) {
    return new Response(JSON.stringify(value), {
        status,
        headers: { 'content-type': 'application/json' },
    });
}

function eventResponse(queue, initial) {
    let firstPull = true;
    const stream = new ReadableStream({
        async pull(controller) {
            if (firstPull) {
                firstPull = false;
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
            const chunk = await queue.read();
            if (chunk === null) controller.close();
            else controller.enqueue(chunk);
        },
        cancel() {
            queue.done = true;
        },
    });
    return { response: new Response(stream, { status: initial.status, headers: initial.headers || {} }), queue };
}

export function createDesktopOpenAI({ invoke, Channel, maxQueueBytes = DEFAULT_MAX_QUEUE_BYTES } = {}) {
    if (typeof invoke !== 'function' || typeof Channel !== 'function') {
        throw new Error('The declared desktop OpenAI transport is unavailable.');
    }
    let serial = 0;
    const pageId = globalThis.crypto?.randomUUID?.() || 'page-' + Date.now().toString(36);
    const nativeLocation = globalThis.location || 'http://desktop.local/';
    async function desktopFetch(url, init = {}) {
        const parsed = new URL(String(url), nativeLocation);
        if (!isDeclaredDesktopOrigin(parsed, nativeLocation)) {
            throw new Error('Custom OpenAI providers and remote model URLs are available in the browser build.');
        }
        if (parsed.pathname === '/v1/models') {
            return directResponse(await invoke('native_models', {}));
        }
        if (parsed.pathname === '/v1/images/decompose') {
            const rgba = init.body instanceof Uint8Array ? init.body : new Uint8Array(init.body || []);
            return directResponse(await invoke('native_decompose_image', {
                width: Number(parsed.searchParams.get('width')),
                height: Number(parsed.searchParams.get('height')),
                columns: Number(parsed.searchParams.get('columns')),
                rows: Number(parsed.searchParams.get('rows')),
                rgba: Array.from(rgba),
            }));
        }
        if (parsed.pathname !== '/v1/chat/completions') {
            throw new Error('Unsupported desktop OpenAI endpoint: ' + parsed.pathname);
        }
        let exact = null;
        try {
            exact = await invoke('native_exact_arithmetic', { request: String(init.body || '{}') });
        } catch (_error) {
            exact = null;
        }
        if (exact?.handled) {
            if (exact.error) {
                return jsonResponse({ error: { message: String(exact.error), type: 'invalid_request_error' } }, 400);
            }
            return jsonResponse({
                id: pageId + '-exact-' + (++serial),
                object: 'chat.completion',
                created: Math.floor(Date.now() / 1000),
                model: 'uor-chat',
                choices: [{ index: 0, message: { role: 'assistant', content: exact.text }, finish_reason: 'stop' }],
                usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
                uor: {
                    tool: 'm6-exact-arithmetic',
                    completion_tokens: 0,
                    completion_ids: [],
                    tokens_per_second_milli: null,
                    stop: 'tool',
                },
            });
        }
        const channel = new Channel();
        const queue = new DesktopEventQueue(maxQueueBytes);
        channel.onmessage = (event) => queue.push(event);
        const requestId = pageId + '-request-' + (++serial);
        queue.onOverflow = () => {
            Promise.resolve(invoke('host_cancel_chat', { requestId })).catch(() => {});
        };
        const initial = await invoke('native_chat_completions', {
            requestId,
            request: String(init.body || '{}'),
            onEvent: channel,
        });
        const { response } = eventResponse(queue, initial);
        if (init.signal) {
            const abort = () => {
                queue.close(new DOMException('The request was aborted', 'AbortError'));
                Promise.resolve(invoke('host_cancel_chat', { requestId })).catch(() => {});
            };
            if (init.signal.aborted) abort();
            else init.signal.addEventListener('abort', abort, { once: true });
        }
        return response;
    }
    return {
        fetch: desktopFetch,
        bootstrap: Object.freeze({
            baseUrl: '/v1',
            model: 'uor-chat',
            token: 'desktop-transport',
            protocol: 'openai',
            temperature: 0,
            ephemeral: true,
        }),
    };
}

export function registerDesktopOpenAI(options) {
    const adapter = createDesktopOpenAI(options);
    if (typeof globalThis.UORStudioApiTransport?.register === 'function') {
        globalThis.UORStudioApiTransport.register(adapter.fetch);
    }
    globalThis.__UOR_NATIVE_BOOTSTRAP__ = adapter.bootstrap;
    return adapter;
}
