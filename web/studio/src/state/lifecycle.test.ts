import { describe, expect, it } from 'vitest';
import { classifyEndpoint, LifecycleStore } from './lifecycle.js';

function operation(store: LifecycleStore) {
    const current = store.getState().operation;
    if (!current) throw new Error('expected an active operation');
    return current;
}

function selectAndStart(store: LifecycleStore, sourceId: string) {
    store.dispatch({ type: 'artifact/selected', sourceId });
    const current = operation(store);
    store.dispatch({ type: 'artifact/downloadStarted', operation: current });
    return current;
}

function readyArtifact(store: LifecycleStore, sourceId: string) {
    const current = selectAndStart(store, sourceId);
    store.dispatch({ type: 'artifact/ready', operation: current, artifact: { schema: 'test' } });
}

describe('artifact lifecycle', () => {
    it('starts unselected and cannot run', () => {
        const store = new LifecycleStore();
        expect(store.getState().artifact.phase).toBe('unselected');
        expect(store.canRun()).toBe(false);
    });

    it('selection is not readiness', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        expect(store.getState().artifact.phase).toBe('selected');
        expect(store.canRun()).toBe(false);
    });

    it('reports real bytes and percentage only for a known total', () => {
        const store = new LifecycleStore();
        const current = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/downloadProgress', operation: current, loaded: 50, total: 100 });
        expect(store.getState().progress).toEqual({ loaded: 50, total: 100, percent: 50 });
    });

    it('never invents a percentage for an unknown total', () => {
        const store = new LifecycleStore();
        const current = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/downloadProgress', operation: current, loaded: 50, total: null });
        expect(store.getState().progress).toEqual({ loaded: 50, total: null, percent: null });
    });

    it('finalization is indeterminate and still not ready', () => {
        const store = new LifecycleStore();
        const current = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/finalizing', operation: current });
        expect(store.getState().artifact.phase).toBe('finalizing');
        expect(store.getState().progress?.percent).toBeNull();
        expect(store.canRun()).toBe(false);
    });

    it('ready requires the current operation and source', () => {
        const store = new LifecycleStore();
        const current = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/finalizing', operation: current });
        store.dispatch({ type: 'artifact/ready', operation: current, artifact: { schema: 'uor-r4.lut-stack/1' } });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.getState().residentArtifact?.sourceId).toBe('model-a');
        expect(store.canRun()).toBe(true);
    });

    it('does not run a ready resident belonging to another source', () => {
        const store = new LifecycleStore();
        const current = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/ready', operation: current, artifact: { sourceId: 'model-b', schema: 'uor-r4.lut-stack/1', canGenerate: true } });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.getState().residentArtifact?.sourceId).toBe('model-b');
        expect(store.canRun()).toBe(false);
    });

    it('ignores stale progress, ready, and failure events after superseding a load', () => {
        const store = new LifecycleStore();
        const first = selectAndStart(store, 'model-a');
        const second = selectAndStart(store, 'model-b');
        store.dispatch({ type: 'artifact/downloadProgress', operation: first, loaded: 100, total: 100 });
        store.dispatch({ type: 'artifact/ready', operation: first, artifact: { schema: 'old' } });
        store.dispatch({ type: 'artifact/failed', operation: first, error: 'late error' });
        expect(store.getState().operation).toEqual(second);
        expect(store.getState().artifact.phase).toBe('downloading');
        expect(store.getState().progress).toBeNull();
        expect(store.getState().artifact.error).toBeNull();
        expect(store.canRun()).toBe(false);
    });

    it('failed replacement cannot fall back to the previous resident model', () => {
        const store = new LifecycleStore();
        const first = selectAndStart(store, 'model-a');
        store.dispatch({ type: 'artifact/ready', operation: first, artifact: { schema: 'old' } });
        expect(store.canRun()).toBe(true);
        const second = selectAndStart(store, 'model-b');
        store.dispatch({ type: 'artifact/failed', operation: second, error: 'bad model' });
        expect(store.getState().artifact.phase).toBe('error');
        expect(store.getState().residentArtifact?.sourceId).toBe('model-a');
        expect(store.canRun()).toBe(false);
    });

    it('ignores a stale failure after a newer source is ready', () => {
        const store = new LifecycleStore();
        const first = selectAndStart(store, 'model-a');
        const second = selectAndStart(store, 'model-b');
        store.dispatch({ type: 'artifact/ready', operation: second, artifact: { schema: 'new' } });
        store.dispatch({ type: 'artifact/failed', operation: first, error: 'late failure' });
        expect(store.getState().artifact.phase).toBe('ready');
        expect(store.canRun()).toBe(true);
    });
});

describe('API source state', () => {
    it('selects without an artifact readiness or reachability claim', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(store.getState().api.testStatus).toBe('untested');
        expect(store.getState().artifact.phase).toBe('unselected');
        expect(store.canRun()).toBe(true);
    });

    it('resets connection test state when an artifact is selected', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'api/testStarted', sourceId: 'api-a' });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        expect(store.getState().api.testStatus).toBe('untested');
    });

    it('tracks an independent connection test', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'api/testStarted', sourceId: 'api-a' });
        expect(store.getState().api.testStatus).toBe('testing');
        store.dispatch({ type: 'api/testFinished', sourceId: 'api-a', ok: true });
        expect(store.getState().api.testStatus).toBe('passed');
        store.dispatch({ type: 'api/testFinished', sourceId: 'api-a', ok: false });
        expect(store.getState().api.testStatus).toBe('failed');
    });
});

describe('file lifecycle', () => {
    it('disables save without an open file', () => {
        const store = new LifecycleStore();
        expect(store.getState().file.phase).toBe('no-folder');
        expect(store.canSave()).toBe(false);
        store.dispatch({ type: 'file/saveStarted', path: 'notes.md' });
        expect(store.getState().file.phase).toBe('no-folder');
    });

    it('moves through clean, dirty, saving, and saved', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'notes.md' });
        expect(store.getState().file.phase).toBe('clean');
        store.dispatch({ type: 'file/dirty' });
        expect(store.getState().file.phase).toBe('dirty');
        store.dispatch({ type: 'file/saveStarted', path: 'notes.md' });
        expect(store.getState().file.phase).toBe('saving');
        expect(store.canSave()).toBe(false);
        store.dispatch({ type: 'file/saveFinished', path: 'notes.md' });
        expect(store.getState().file.phase).toBe('saved');
        expect(store.canSave()).toBe(true);
    });

    it('keeps a save error retryable', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'notes.md' });
        store.dispatch({ type: 'file/dirty' });
        store.dispatch({ type: 'file/saveStarted', path: 'notes.md' });
        store.dispatch({ type: 'file/saveFailed', path: 'notes.md', error: 'disk full' });
        expect(store.getState().file.phase).toBe('error');
        expect(store.getState().file.error).toBe('disk full');
        expect(store.canSave()).toBe(true);
    });

    it('does not apply an old save result to a newly selected file', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/saveStarted', path: 'a.md' });
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        store.dispatch({ type: 'file/saveFailed', path: 'a.md', error: 'old failure' });
        expect(store.getState().file).toMatchObject({ path: 'b.md', phase: 'clean', error: null });
    });
});

describe('generation privacy', () => {
    it('pins source and privacy while the picker changes', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a', endpoint: 'https://api.example.com/v1' } });
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-b' });
        expect(store.getState().session.activeGeneration?.source.id).toBe('api-a');
        expect(store.getState().session.activeGeneration?.privacy).toBe('remoteAPI');
        store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 });
        expect(store.getState().session.lastGeneration?.source.id).toBe('api-a');
        expect(store.getState().session.lastGeneration?.privacy).toBe('remoteAPI');
    });

    it('retains stopped and error outcomes with their source snapshot', () => {
        const store = new LifecycleStore();
        readyArtifact(store, 'model-a');
        store.dispatch({ type: 'generation/started', source: { kind: 'artifact', id: 'model-a' } });
        store.dispatch({ type: 'generation/finished', outcome: 'stopped', generationId: 1 });
        expect(store.getState().session.phase).toBe('stopped');
        expect(store.getState().session.lastGeneration?.privacy).toBe('localWASM');
        store.dispatch({ type: 'api/selected', sourceId: 'api-b', endpoint: 'http://127.0.0.1:8000' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-b', endpoint: 'http://127.0.0.1:8000' } });
        store.dispatch({ type: 'generation/finished', outcome: 'error', generationId: 2 });
        expect(store.getState().session.phase).toBe('error');
        expect(store.getState().session.lastGeneration?.privacy).toBe('loopbackAPI');
    });
});

describe('endpoint privacy classification', () => {
    it('classifies local execution and loopback hosts', () => {
        expect(classifyEndpoint()).toBe('localWASM');
        expect(classifyEndpoint('http://localhost:8000')).toBe('loopbackAPI');
        expect(classifyEndpoint('http://127.0.0.1:8000')).toBe('loopbackAPI');
        expect(classifyEndpoint('http://[::1]:8000')).toBe('loopbackAPI');
        expect(classifyEndpoint('http://127.2.3.4:8000')).toBe('loopbackAPI');
    });

    it('classifies private IPv4 and IPv6 networks', () => {
        for (const endpoint of [
            'http://10.0.0.2',
            'http://192.168.1.20',
            'http://172.16.4.5',
            'http://172.31.255.255',
            'http://169.254.1.2',
            'http://[fd00::1]',
            'http://[fe80::1]',
        ]) {
            expect(classifyEndpoint(endpoint)).toBe('privateNetworkAPI');
        }
    });

    it('classifies public endpoints and rejects invalid input', () => {
        expect(classifyEndpoint('https://api.example.com')).toBe('remoteAPI');
        expect(classifyEndpoint('http://172.32.0.1')).toBe('remoteAPI');
        expect(() => classifyEndpoint('not a url')).toThrow(TypeError);
        expect(() => classifyEndpoint('ftp://example.com/v1')).toThrow(TypeError);
    });

    it('does not treat private-looking public hostnames as local networks', () => {
        for (const endpoint of ['https://fda.gov', 'https://127.example.com', 'https://10.example.com', 'https://fe80.example.com']) {
            expect(classifyEndpoint(endpoint)).toBe('remoteAPI');
        }
    });

    it('covers the full fe80::/10 link-local range', () => {
        expect(classifyEndpoint('http://[fe90::1]')).toBe('privateNetworkAPI');
        expect(classifyEndpoint('http://[febf::1]')).toBe('privateNetworkAPI');
        expect(classifyEndpoint('http://[fec0::1]')).toBe('remoteAPI');
    });
});

describe('generation source authority', () => {
    it('uses the selected endpoint rather than a caller-supplied endpoint', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a', endpoint: 'http://127.0.0.1' } });
        expect(store.getState().session.activeGeneration?.privacy).toBe('remoteAPI');
        expect(store.getState().session.activeGeneration?.source.endpoint).toBe('https://api.example.com/v1');
    });

    it('uses the selected endpoint when the action omits it', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        expect(store.getState().session.activeGeneration?.privacy).toBe('remoteAPI');
    });

    it('fails closed when an API selection has no endpoint', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: undefined as unknown as string });
        expect(() => store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } })).not.toThrow();
        expect(store.getState().session.phase).toBe('error');
        expect(store.getState().session.activeGeneration).toBeNull();
        expect(store.canRun()).toBe(false);
    });

    it('reports an invalid selected endpoint without throwing', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'not a url' });
        expect(() => store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } })).not.toThrow();
        expect(store.getState().session.phase).toBe('error');
        expect(store.canRun()).toBe(false);
    });
});

describe('lifecycle subscriptions', () => {
    it('notifies external transitions and stops after unsubscribe', () => {
        const store = new LifecycleStore();
        let calls = 0;
        const unsubscribe = store.subscribe(() => { calls += 1; });
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(calls).toBe(1);
        store.dispatch({ type: 'api/testStarted', sourceId: 'other' });
        expect(calls).toBe(1);
        unsubscribe();
        store.dispatch({ type: 'api/selected', sourceId: 'api-b', endpoint: 'https://other.example.com/v1' });
        expect(calls).toBe(1);
    });
});

describe('folder, file-close, and session activation states', () => {
    it('supports no-open-file after opening a folder or closing a file', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'folder/opened', path: 'project' });
        expect(store.getState().file.phase).toBe('no-open-file');
        store.dispatch({ type: 'file/opened', path: 'project/a.md' });
        store.dispatch({ type: 'file/closed' });
        expect(store.getState().file.phase).toBe('no-open-file');
        expect(store.canSave()).toBe(false);
    });

    it('supports new to active to generating', () => {
        const store = new LifecycleStore();
        readyArtifact(store, 'model-a');
        store.dispatch({ type: 'session/activated' });
        expect(store.getState().session.phase).toBe('active');
        store.dispatch({ type: 'generation/started', source: { kind: 'artifact', id: 'model-a' } });
        expect(store.getState().session.phase).toBe('generating');
    });
});
