import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createModelSourceAdapter, LOAD_PROGRESS_EVENT } from './modelSourceAdapter.js';
import type { ApiProvider, ModelSourceList } from '../../assets/js/model_sources.js';

function sources() {
    const result = { bytes: new Uint8Array(), fileName: 'model.lut', expectedSha256: 'abc', meta: { schema: 'uor-r4.lut-stack/1' } };
    const provider: ApiProvider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: '' };
    const list = (): ModelSourceList => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] });
    return {
        catalog: vi.fn(() => ({ models: [] })),
        loadModelCatalog: vi.fn(async () => undefined),
        list: vi.fn(list),
        active: vi.fn(() => list().active),
        apiConfig: vi.fn((id?: string) => id && id !== provider.id ? null : provider),
        addApi: vi.fn((input: Partial<ApiProvider>) => ({ ...provider, ...input, id: input.id || 'api-new' })),
        updateApi: vi.fn((id: string, patch: Partial<ApiProvider>) => ({ ...provider, ...patch, id })),
        removeApi: vi.fn(() => list()),
        loadPublished: vi.fn(async () => result),
        loadFromUrl: vi.fn(async (url: string) => ({ ...result, fileName: url.split('/').pop() || 'downloaded-stack.lut' })),
        loadLocalFile: vi.fn(async () => result),
        loadBundled: vi.fn(async () => result),
        setActive: vi.fn(),
        testApi: vi.fn(async () => ({ ok: true, detail: 'ok', models: [] })),
    };
}

function worker() {
    return { postMessage: vi.fn() };
}

describe('model source adapter', () => {
    let store: LifecycleStore;
    beforeEach(() => {
        store = new LifecycleStore();
        document.dispatchEvent(new CustomEvent('uor-test-reset'));
    });

    it('forwards URL loads and waits for worker readiness', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        await adapter.loadFromUrl('https://example.com/model.lut');
        expect(api.loadFromUrl).toHaveBeenCalledWith('https://example.com/model.lut', { operationId: 1, sourceId: 'url' });
        expect(store.getState().artifact.phase).toBe('finalizing');
        expect(store.canRun()).toBe(false);
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'uor-r4.lut-stack/1' } });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.canRun()).toBe(true);
    });

    it('maps structured byte progress and finalization without inventing percentages', async () => {
        const adapter = createModelSourceAdapter(store, sources(), worker());
        const load = adapter.loadFromUrl('https://example.com/model.lut');
        const operation = store.getState().operation!;
        document.dispatchEvent(new CustomEvent(LOAD_PROGRESS_EVENT, {
            detail: { operationId: operation.operationId, loaded: 25, total: 100, phase: 'downloading' },
        }));
        expect(store.getState().progress).toEqual({ loaded: 25, total: 100, percent: 25 });
        document.dispatchEvent(new CustomEvent(LOAD_PROGRESS_EVENT, {
            detail: { operationId: operation.operationId, loaded: 50, total: null, phase: 'downloading' },
        }));
        expect(store.getState().progress?.percent).toBeNull();
        await load;
        expect(store.getState().artifact.phase).toBe('finalizing');
    });

    it('failed replacement leaves run disabled rather than falling back', async () => {
        const adapter = createModelSourceAdapter(store, sources(), worker());
        await adapter.loadFromUrl('https://example.com/a.lut');
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'old' } });
        expect(store.canRun()).toBe(true);
        await adapter.loadFromUrl('https://example.com/b.lut');
        adapter.handleWorkerMessage({ action: 'operation_error', id: 2, error: 'bad model' });
        expect(store.getState().artifact.phase).toBe('error');
        expect(store.getState().residentArtifact?.sourceId).toBe('url');
        expect(store.canRun()).toBe(false);
    });

    it('forwards published, local-file, and bundled operations', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        await adapter.loadPublished('published-a');
        expect(api.loadPublished).toHaveBeenCalledWith('published-a', { operationId: 1, sourceId: 'published-a' });
        await adapter.loadLocalFile(new File([new Uint8Array()], 'local.lut'));
        expect(api.loadLocalFile).toHaveBeenCalledWith(expect.any(File), { operationId: 2, sourceId: 'local-file' });
        await adapter.loadBundled();
        expect(api.loadBundled).toHaveBeenCalledWith({ operationId: 3, sourceId: 'bundled-demo' });
    });

    it('uses the existing API test path without an artifact-ready claim', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://api.example.com/v1');
        expect(store.getState().api.testStatus).toBe('untested');
        expect(store.getState().artifact.phase).toBe('unselected');
        await adapter.testApi('api-a');
        expect(api.testApi).toHaveBeenCalledWith('api-a');
        expect(api.setActive).toHaveBeenCalledTimes(1);
        expect(api.setActive).toHaveBeenCalledWith({ kind: 'api', id: 'api-a' });
        expect(store.getState().api.testStatus).toBe('passed');
    });

    it('uses the selected API id when omitted', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://api.example.com/v1');
        await adapter.testApi();
        expect(api.testApi).toHaveBeenCalledWith('api-a');
        expect(store.getState().api.testStatus).toBe('passed');
    });

    it('forwards the real expected hash to the worker', async () => {
        const transport = worker();
        const adapter = createModelSourceAdapter(store, sources(), transport);
        await adapter.loadFromUrl('https://example.com/model.lut');
        expect(transport.postMessage).toHaveBeenCalledWith({
            action: 'load_model',
            id: 1,
            payload: expect.objectContaining({ expectedSha256: 'abc' }),
        });
    });

    it('does not mark a newer operation ready from an older worker result', async () => {
        const transport = worker();
        const adapter = createModelSourceAdapter(store, sources(), transport);
        await adapter.loadFromUrl('https://example.com/a.lut');
        await adapter.loadFromUrl('https://example.com/b.lut');
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'old', file_name: 'a.lut', artifact_sha256: 'abc' } });
        expect(store.canRun()).toBe(false);
        await vi.waitFor(() => expect(transport.postMessage).toHaveBeenCalledTimes(2));
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'new', file_name: 'b.lut', artifact_sha256: 'abc', sha256_verified: true, engine_supported: true, can_generate: false, can_predict: true, source: 'url' } });
        expect(store.getState().residentArtifact).toMatchObject({
            sourceId: 'url',
            schema: 'new',
            fileName: 'b.lut',
            sha256: 'abc',
            verified: true,
            engineSupported: true,
            canGenerate: false,
            canPredict: true,
            source: 'url',
        });
        expect(store.canRun()).toBe(true);
    });

    it('waits for an older terminal result before sending a same-artifact replacement', async () => {
        const transport = worker();
        const adapter = createModelSourceAdapter(store, sources(), transport);
        await adapter.loadFromUrl('https://example.com/model.lut');
        await adapter.loadFromUrl('https://example.com/model.lut');
        expect(transport.postMessage).toHaveBeenCalledTimes(1);
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { file_name: 'model.lut', artifact_sha256: 'abc' } });
        expect(store.canRun()).toBe(false);
        await vi.waitFor(() => expect(transport.postMessage).toHaveBeenCalledTimes(2));
        adapter.handleWorkerMessage({ action: 'operation_error', id: 2, error: 'replacement failed' });
        expect(store.getState().artifact.phase).toBe('error');
        expect(store.canRun()).toBe(false);
    });

    it('ignores non-load worker errors after readiness', async () => {
        const adapter = createModelSourceAdapter(store, sources(), worker());
        await adapter.loadFromUrl('https://example.com/model.lut');
        adapter.handleWorkerMessage({ action: 'model_loaded', capabilities: { schema: 'new', file_name: 'model.lut' } });
        expect(store.canRun()).toBe(true);
        adapter.handleWorkerMessage({ action: 'operation_error', id: 999, error: 'unrelated error' });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.canRun()).toBe(true);
    });

    it('records rejected loads as errors instead of leaving them pending', async () => {
        const api = sources();
        api.loadFromUrl.mockRejectedValueOnce(new Error('network down'));
        const adapter = createModelSourceAdapter(store, api, worker());
        await expect(adapter.loadFromUrl('https://example.com/model.lut')).rejects.toThrow('network down');
        expect(store.getState().artifact.phase).toBe('error');
        expect(store.getState().artifact.error).toBe('network down');
    });

    it('does not attribute an API test to a source selected later', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://api.example.com/v1');
        const pending = adapter.testApi('api-a');
        adapter.selectApi('api-b', 'https://other.example.com/v1');
        await pending;
        expect(store.getState().api.testStatus).toBe('untested');
    });

    it('does not attribute an API test to a changed endpoint with the same id', async () => {
        const api = sources();
        let finish!: () => void;
        api.testApi.mockReturnValueOnce(new Promise((resolve) => {
            finish = () => resolve({ ok: true, detail: 'ok', models: [] });
        }));
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://old.example.com/v1');
        const pending = adapter.testApi('api-a');
        adapter.selectApi('api-a', 'https://new.example.com/v1');
        finish();
        await pending;
        expect(store.getState().api.testStatus).toBe('untested');
    });

    it('rejects a non-HTTP API selection before changing active state', () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        expect(() => adapter.selectApi('api-a', 'ftp://example.com/v1')).toThrow('Only HTTP(S) API endpoints are supported.');
        expect(api.setActive).not.toHaveBeenCalled();
        expect(store.getState().selectedSource).toBeNull();
    });

    it('refuses to test an endpoint that is not selected', async () => {
        const api = sources();
        const adapter = createModelSourceAdapter(store, api, worker());
        adapter.selectApi('api-a', 'https://api.example.com/v1');
        await expect(adapter.testApi('api-b')).rejects.toThrow('non-selected API');
        expect(api.testApi).not.toHaveBeenCalled();
    });
});
