import test from 'node:test';
import assert from 'node:assert/strict';
import { isDeclaredDesktopOrigin } from './desktop_origin.js';

test('the native asset origin rejects userinfo, aliases, and non-default ports', () => {
    for (const location of ['tauri://localhost/', 'http://tauri.localhost/', 'https://tauri.localhost/']) {
        for (const candidate of [
            'tauri://evil/v1',
            'tauri://user@localhost/v1',
            'http://tauri.localhost:8000/v1',
            'https://tauri.localhost:444/v1',
            'https://user@tauri.localhost/v1',
            'https://example.com/v1',
        ]) assert.equal(isDeclaredDesktopOrigin(candidate, location), false, `${location} accepted ${candidate}`);
    }
    assert.equal(isDeclaredDesktopOrigin('tauri://localhost/v1', 'tauri://localhost/'), true);
    assert.equal(isDeclaredDesktopOrigin('http://tauri.localhost/v1', 'tauri://localhost/'), true);
    assert.equal(isDeclaredDesktopOrigin('https://tauri.localhost/v1', 'tauri://localhost/'), true);
    assert.equal(isDeclaredDesktopOrigin('http://tauri.localhost:80/v1', 'tauri://localhost/'), true);
    assert.equal(isDeclaredDesktopOrigin('https://tauri.localhost:443/v1', 'tauri://localhost/'), true);
});

let counter = 0;
async function fresh(failStorage = false, baseURI = 'http://127.0.0.1:8765/') {
    const values = new Map();
    const listeners = new Map();
    globalThis.localStorage = {
        getItem: (key) => values.get(key) ?? null,
        setItem: (key, value) => { if (failStorage) throw new Error('storage denied'); values.set(key, value); },
        removeItem: (key) => values.delete(key),
    };
    globalThis.document = {
        baseURI,
        addEventListener(type, listener) {
            if (!listeners.has(type)) listeners.set(type, []);
            listeners.get(type).push(listener);
        },
        dispatchEvent() { return true; },
    };
    globalThis.CustomEvent = class { constructor(type, init = {}) { this.type = type; this.detail = init.detail; } };
    counter += 1;
    const module = await import(`./model_sources.js?native=${counter}`);
    return { api: module.ModelSources, values };
}

test('native bootstrap selects an in-memory temperature-zero source without persisting its token or id', async () => {
    const { api, values } = await fresh();
    const entry = api.registerNativeBootstrap({ baseUrl: '/v1', model: 'uor-chat', token: 'launch-secret', protocol: 'openai', temperature: 0, ephemeral: true });
    assert.equal(entry.baseUrl, 'http://127.0.0.1:8765/v1');
    assert.equal(entry.temperature, 0);
    assert.equal(entry.ephemeral, true);
    assert.deepEqual(api.active(), { kind: 'api', id: entry.id });
    assert.equal(api.apiConfig().apiKey, 'launch-secret');
    const stored = values.get('uor.studio.modelSources.v1') || '';
    assert.equal(stored.includes('launch-secret'), false);
    assert.equal(stored.includes(entry.id), false);
    assert.equal(stored.includes('native-loopback'), false);
});

test('native bootstrap rejects cross-origin and non-ephemeral credentials', async () => {
    const { api } = await fresh();
    assert.throws(() => api.registerNativeBootstrap({ baseUrl: 'https://example.com/v1', model: 'uor-chat', token: 'secret', protocol: 'openai', temperature: 0, ephemeral: true }), /same-origin/);
    assert.throws(() => api.registerNativeBootstrap({ baseUrl: '/v1', model: 'uor-chat', token: 'secret', protocol: 'openai', temperature: 0, ephemeral: false }), /ephemeral/);
});

test('native bootstrap works when persistent storage is unavailable', async () => {
    const { api } = await fresh(true);
    const entry = api.registerNativeBootstrap({ baseUrl: '/v1', model: 'uor-chat', token: 'secret', protocol: 'openai', temperature: 0, ephemeral: true });
    assert.equal(entry.ephemeral, true);
    assert.deepEqual(api.active(), { kind: 'api', id: 'native-loopback' });
});

test('native bootstrap normalizes the declared Tauri asset origin for HTTP-compatible IPC', async () => {
    const { api, values } = await fresh(false, 'tauri://localhost/');
    const entry = api.registerNativeBootstrap({ baseUrl: '/v1', model: 'uor-chat', token: 'memory-only', protocol: 'openai', temperature: 0, ephemeral: true });
    assert.equal(entry.baseUrl, 'http://tauri.localhost/v1');
    assert.deepEqual(api.active(), { kind: 'api', id: 'native-loopback' });
    assert.equal(values.get('uor.studio.modelSources.v1')?.includes('memory-only'), false);
    assert.throws(() => api.registerNativeBootstrap({ baseUrl: 'tauri://other-host/v1', model: 'uor-chat', token: 'memory-only', protocol: 'openai', temperature: 0, ephemeral: true }), /same-origin/);
});
