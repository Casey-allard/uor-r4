import test from 'node:test';
import assert from 'node:assert/strict';

let counter = 0;
async function fresh(failStorage = false) {
    const values = new Map();
    const listeners = new Map();
    globalThis.localStorage = {
        getItem: (key) => values.get(key) ?? null,
        setItem: (key, value) => { if (failStorage) throw new Error('storage denied'); values.set(key, value); },
        removeItem: (key) => values.delete(key),
    };
    globalThis.document = {
        baseURI: 'http://127.0.0.1:8765/',
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
