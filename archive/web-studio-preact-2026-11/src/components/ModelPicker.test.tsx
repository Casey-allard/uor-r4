import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { createOperationGate } from '../adapters/operations.js';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ApiProvider, LoadResult, ModelSourcesApi } from '../../assets/js/model_sources.js';
import { ModelPicker } from './ModelPicker.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

const provider: ApiProvider = { id: 'api-a', label: 'API A', preset: 'custom', baseUrl: 'https://api.example.com/v1', model: 'model-a', apiKey: 'secret-sentinel' };
const result: LoadResult = { bytes: new Uint8Array(), fileName: 'model.lut', expectedSha256: 'abc', meta: { schema: 'uor-r4.lut-stack/1' } };

function adapter(): ModelSourceAdapter {
    return {
        loadPublished: vi.fn(async () => result), loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result), loadBundled: vi.fn(async () => result),
        selectApi: vi.fn(), testApi: vi.fn(async () => ({ ok: true })), handleWorkerMessage: vi.fn(),
    };
}

function sources(): ModelSourcesApi {
    const list = () => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [provider] });
    return {
        list: vi.fn(list), active: vi.fn(() => list().active), apiConfig: vi.fn(() => provider),
        catalog: vi.fn(() => ({ models: [{ id: 'm1-a', label: 'M1 seed A', url: 'https://example.com/m1.lut' }] })),
        loadModelCatalog: vi.fn(async () => undefined),
        setActive: vi.fn(), addApi: vi.fn((input: Partial<ApiProvider>) => ({ ...provider, ...input, id: 'api-new', apiKey: input.apiKey || '' })),
        updateApi: vi.fn((id: string, patch: Partial<ApiProvider>) => ({ ...provider, ...patch, id })), removeApi: vi.fn(() => list()),
        loadPublished: vi.fn(async () => result), loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result), loadBundled: vi.fn(async () => result),
        testApi: vi.fn(async () => ({ ok: true })),
    };
}

function mount(store = new LifecycleStore(), modelAdapter = adapter(), modelSources = sources()) {
    const onError = vi.fn();
    act(() => { render(<ModelPicker store={store} adapter={modelAdapter} sources={modelSources} operations={createOperationGate()} onError={onError} />, host); });
    return { store, modelAdapter, modelSources, onError };
}

function input(label: string): HTMLInputElement {
    return host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('ModelPicker', () => {
    it('subscribes to selected-not-ready and known byte progress', () => {
        const { store } = mount();
        act(() => { store.dispatch({ type: 'artifact/selected', sourceId: 'url' }); });
        expect(host.textContent).toContain('Selected model; not ready');
        act(() => { store.dispatch({ type: 'artifact/downloadProgress', operation: store.getState().operation!, loaded: 25, total: 100 }); });
        const progress = host.querySelector<HTMLProgressElement>('progress[aria-label="Model download progress"]')!;
        expect(progress.max).toBe(100);
        expect(progress.value).toBe(25);
        expect(host.textContent).toContain('25 / 100 bytes (25%)');
    });

    it('keeps unknown totals and finalization indeterminate', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'url' });
        act(() => { store.dispatch({ type: 'artifact/downloadProgress', operation: store.getState().operation!, loaded: 50, total: null }); });
        const progress = host.querySelector<HTMLProgressElement>('progress')!;
        expect(progress.hasAttribute('value')).toBe(false);
        expect(host.textContent).toContain('50 bytes downloaded; total unknown');
        act(() => { store.dispatch({ type: 'artifact/finalizing', operation: store.getState().operation! }); });
        expect(host.textContent).toContain('Finalizing model; progress is indeterminate');
        expect(host.textContent).not.toContain('100%');
    });

    it('shows resident verification and next-token capability without calling it chat', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'url' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', fileName: 'm1.lut', sha256: 'abc123', verified: true,
                engineSupported: true, canGenerate: false, canPredict: true, source: 'url',
            } });
        });
        expect(host.textContent).toContain('m1.lut');
        expect(host.textContent).toContain('SHA-256 abc123 (verified)');
        expect(host.textContent).toContain('Engine supported');
        expect(host.textContent).toContain('Next-token prediction only');
        expect(host.textContent).not.toContain('Conversational chat');
    });

    it('does not present an engine-unsupported artifact as prediction-capable', () => {
        const { store } = mount();
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        act(() => {
            store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: {
                schema: 'uor-r4.lut-discrete-trunk/1', engineSupported: false, canGenerate: false, canPredict: true,
            } });
        });
        expect(host.textContent).toContain('Engine unsupported');
        expect(host.textContent).toContain('Generation unavailable');
        expect(host.textContent).not.toContain('Next-token prediction only');
    });

    it('forwards bundled, published, URL, and local-file loads', () => {
        const { modelAdapter, modelSources } = mount();
        expect(modelSources.loadModelCatalog).toHaveBeenCalledOnce();
        button('Load bundled demo').click();
        button('Load M1 seed A').click();
        act(() => {
            input('Model URL').value = 'https://example.com/model.lut';
            input('Model URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="load-url"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        const file = new File([new Uint8Array()], 'local.lut');
        Object.defineProperty(input('Model file'), 'files', { configurable: true, value: [file] });
        input('Model file').dispatchEvent(new Event('change', { bubbles: true }));
        expect(modelAdapter.loadBundled).toHaveBeenCalledOnce();
        expect(modelAdapter.loadPublished).toHaveBeenCalledWith('m1-a');
        expect(modelAdapter.loadFromUrl).toHaveBeenCalledWith('https://example.com/model.lut');
        expect(modelAdapter.loadLocalFile).toHaveBeenCalledWith(file);
    });

    it('tests an unselected provider without changing selection', async () => {
        const { store, modelAdapter, modelSources } = mount();
        act(() => { button('Test API A').click(); });
        await act(async () => { await Promise.resolve(); });
        expect(modelSources.testApi).toHaveBeenCalledWith('api-a');
        expect(modelAdapter.selectApi).not.toHaveBeenCalled();
        expect(store.getState().selectedSource).toBeNull();
        expect(host.textContent).toContain('Test: passed');
    });

    it('manages providers through native dialogs without rendering an existing secret', async () => {
        const { store, modelAdapter, modelSources } = mount();
        const apiConfigChanged = vi.fn();
        document.addEventListener('uor-api-config-changed', apiConfigChanged);
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        expect(host.innerHTML).not.toContain('secret-sentinel');
        act(() => { button('Use API A').click(); });
        expect(modelAdapter.selectApi).toHaveBeenCalledWith('api-a', provider.baseUrl);
        await act(async () => {
            button('Test API A').click();
            await Promise.resolve();
        });
        expect(modelAdapter.selectApi).toHaveBeenCalledTimes(1);
        expect(modelAdapter.testApi).not.toHaveBeenCalled();
        expect(modelSources.testApi).toHaveBeenCalledWith('api-a');
        expect(store.getState().selectedSource?.id).toBe('api-a');
        expect(host.textContent).toContain('Test: passed');
        expect(host.textContent).toContain('Connection test: passed');
        act(() => { button('Edit API A').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        expect(host.innerHTML).not.toContain('secret-sentinel');
        act(() => {
            input('Provider base URL').value = 'https://new.example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => {
            input('API key').value = 'new-secret';
            input('API key').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(modelSources.updateApi).toHaveBeenCalledWith('api-a', expect.objectContaining({ baseUrl: 'https://new.example.com/v1', apiKey: 'new-secret' }));
        expect(store.getState().selectedSource?.endpoint).toBe('https://new.example.com/v1');
        expect(apiConfigChanged).toHaveBeenCalledWith(expect.objectContaining({ detail: { id: 'api-a' } }));
        expect(host.textContent).toContain('Test: untested');
        act(() => { button('Edit API A').click(); });
        expect(input('API key').value).toBe('');
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(modelSources.removeApi).toHaveBeenCalledWith('api-a');
        expect(store.getState().selectedSource?.kind).toBe('artifact');
        expect(store.canRun()).toBe(false);
        document.removeEventListener('uor-api-config-changed', apiConfigChanged);
    });

    it('keeps the provider draft and shows an inline error when persistence fails', () => {
        const { store, modelSources, onError } = mount();
        modelSources.updateApi = vi.fn(() => { throw new Error('Could not persist model sources: browser storage is unavailable or full.'); });
        act(() => { button('Edit API A').click(); });
        act(() => {
            input('API key').value = 'new-secret';
            input('API key').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(host.querySelector('dialog')).not.toBeNull();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Could not persist model sources');
        expect(input('API key').value).toBe('new-secret');
        expect(onError).toHaveBeenCalledWith('Could not persist model sources: browser storage is unavailable or full.', 'provider:save:api-a');
        expect(store.getState().selectedSource).toBeNull();
    });

    it('ignores a provider test result from before an edit', async () => {
        const { modelSources } = mount();
        let finish!: (result: { ok: boolean }) => void;
        modelSources.testApi = vi.fn((): Promise<{ ok: boolean }> => new Promise((resolve) => { finish = resolve; }));
        act(() => { button('Test API A').click(); });
        act(() => { button('Edit API A').click(); });
        act(() => {
            input('Provider base URL').value = 'https://new.example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await act(async () => { finish({ ok: true }); await Promise.resolve(); });
        expect(host.textContent).toContain('Test: untested');
        expect(host.textContent).not.toContain('Test: passed');
    });

    it('ignores a provider test rejection from before an edit', async () => {
        const { modelSources, onError } = mount();
        let reject!: (error: Error) => void;
        modelSources.testApi = vi.fn((): Promise<{ ok: boolean }> => new Promise((_resolve, rejectPromise) => { reject = rejectPromise; }));
        act(() => { button('Test API A').click(); });
        act(() => { button('Edit API A').click(); });
        act(() => {
            input('Provider base URL').value = 'https://new.example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        await act(async () => { reject(new Error('old endpoint failed')); await Promise.resolve(); });
        expect(onError).not.toHaveBeenCalled();
        expect(host.textContent).toContain('Test: untested');
    });

    it('shows provider selection persistence failures', () => {
        const { store, modelAdapter, onError } = mount();
        modelAdapter.selectApi = vi.fn(() => { throw new Error('Could not persist model sources: browser storage is unavailable or full.'); });
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('Could not persist model sources');
        expect(onError).toHaveBeenCalled();
        expect(store.getState().selectedSource).toBeNull();
    });

    it('keeps removal open and selected when provider deletion cannot persist', () => {
        const { store, modelAdapter, modelSources, onError } = mount();
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        modelSources.removeApi = vi.fn(() => { throw new Error('Could not persist model sources: browser storage is unavailable or full.'); });
        act(() => { button('Use API A').click(); });
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Could not persist model sources');
        expect(onError).toHaveBeenCalled();
        expect(store.getState().selectedSource?.id).toBe('api-a');
    });

    it('does not write selection again after removing the active provider', () => {
        const { store, modelAdapter, modelSources } = mount();
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        modelSources.removeApi = vi.fn(() => {
            modelSources.active = vi.fn(() => ({ kind: 'wasm', id: 'bundled-demo' }));
            return { active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [] };
        });
        act(() => { button('Use API A').click(); });
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(modelSources.setActive).not.toHaveBeenCalled();
        expect(store.getState().selectedSource?.kind).toBe('artifact');
    });

    it('adds a provider through the native dialog and clears the draft', () => {
        const { modelSources } = mount();
        act(() => { button('Add API provider').click(); });
        act(() => {
            input('Provider name').value = 'API B';
            input('Provider name').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => {
            input('Provider base URL').value = 'https://b.example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => {
            input('Provider model').value = 'model-b';
            input('Provider model').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => {
            input('API key').value = 'new-secret';
            input('API key').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(modelSources.addApi).toHaveBeenCalledWith({ label: 'API B', preset: 'custom', baseUrl: 'https://b.example.com/v1', model: 'model-b', apiKey: 'new-secret' });
        expect(host.querySelector('dialog')).toBeNull();
        act(() => { button('Add API provider').click(); });
        expect(input('API key').value).toBe('');
    });

    it('clears an unsaved API key when the provider dialog closes', () => {
        mount();
        act(() => { button('Add API provider').click(); });
        act(() => {
            input('API key').value = 'unsaved-secret';
            input('API key').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector<HTMLDialogElement>('dialog')!.close(); });
        expect(host.querySelector('dialog')).toBeNull();
        act(() => { button('Add API provider').click(); });
        expect(input('API key').value).toBe('');
    });

    it('rejects a non-HTTP provider selection before writing or emitting config', () => {
        const modelAdapter = adapter();
        const modelSources = sources();
        const configChanged = vi.fn();
        document.addEventListener('uor-api-config-changed', configChanged);
        const invalid = { ...provider, baseUrl: 'ftp://example.com/v1' };
        modelSources.list = vi.fn(() => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [invalid] }));
        const { store, onError } = mount(new LifecycleStore(), modelAdapter, modelSources);
        act(() => { button('Use API A').click(); });
        expect(modelAdapter.selectApi).not.toHaveBeenCalled();
        expect(configChanged).not.toHaveBeenCalled();
        expect(store.getState().selectedSource).toBeNull();
        expect(onError).toHaveBeenCalledWith('Only HTTP(S) API endpoints are supported.', expect.any(String));
        document.removeEventListener('uor-api-config-changed', configChanged);
    });

    it('rejects a non-HTTP provider edit before writing or emitting config', () => {
        const { modelSources, onError } = mount();
        const configChanged = vi.fn();
        document.addEventListener('uor-api-config-changed', configChanged);
        act(() => { button('Edit API A').click(); });
        act(() => {
            input('Provider base URL').value = 'ftp://example.com/v1';
            input('Provider base URL').dispatchEvent(new Event('input', { bubbles: true }));
        });
        act(() => { host.querySelector('form[data-action="save-provider"]')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })); });
        expect(modelSources.updateApi).not.toHaveBeenCalled();
        expect(configChanged).not.toHaveBeenCalled();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Only HTTP(S) API endpoints are supported.');
        expect(onError).toHaveBeenCalledWith('Only HTTP(S) API endpoints are supported.', expect.any(String));
        document.removeEventListener('uor-api-config-changed', configChanged);
    });

    it('keeps a selection error after an unrelated successful connection test', async () => {
        const { modelAdapter } = mount();
        modelAdapter.selectApi = vi.fn(() => { throw new Error('selection failed'); });
        act(() => { button('Use API A').click(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('selection failed');
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('[role="alert"]')?.textContent).toContain('selection failed');
    });

    it('loads the local fallback after removing the selected provider', () => {
        const { store, modelAdapter, modelSources } = mount();
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        modelSources.active = vi.fn(() => ({ kind: 'wasm', id: 'bundled-demo' }));
        act(() => { button('Use API A').click(); });
        act(() => { button('Remove API A').click(); });
        act(() => { button('Confirm removal').click(); });
        expect(modelAdapter.loadBundled).toHaveBeenCalledOnce();
    });

    it('does not render a provider-test error inside an edit dialog', async () => {
        const { modelSources } = mount();
        modelSources.testApi = vi.fn().mockRejectedValue(new Error('test failed'));
        act(() => { button('Edit API A').click(); });
        await act(async () => { button('Test API A').click(); await Promise.resolve(); });
        expect(host.querySelector('dialog [role="alert"]')).toBeNull();
    });

    it('ignores a stale artifact-load rejection after unmount and remount', async () => {
        const first = mount();
        let rejectFirst!: (error: Error) => void;
        let finishSecond!: (result: LoadResult) => void;
        first.modelAdapter.loadBundled = vi.fn((): Promise<LoadResult> => new Promise((_resolve, reject) => { rejectFirst = reject; }));
        act(() => { button('Load bundled demo').click(); });
        act(() => { render(null, host); });
        const second = mount();
        second.modelAdapter.loadBundled = vi.fn((): Promise<LoadResult> => new Promise((resolve) => { finishSecond = resolve; }));
        act(() => { button('Load bundled demo').click(); });
        await act(async () => { finishSecond(result); await Promise.resolve(); });
        await act(async () => { rejectFirst(new Error('stale failure')); await Promise.resolve(); });
        expect(first.onError).not.toHaveBeenCalled();
        expect(second.onError).not.toHaveBeenCalled();
    });

    it('blocks provider removal during generation', () => {
        const { store, modelAdapter } = mount();
        modelAdapter.selectApi = vi.fn((id: string, endpoint: string) => store.dispatch({ type: 'api/selected', sourceId: id, endpoint }));
        act(() => { button('Use API A').click(); });
        act(() => { store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } }); });
        expect(button('Remove API A').disabled).toBe(true);
    });
});
