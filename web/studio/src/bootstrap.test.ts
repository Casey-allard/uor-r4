import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'preact';
import { LifecycleStore } from './state/lifecycle.js';
import { mountTypedStudio, typedPreviewRequested } from './bootstrap.js';
import { StudioApp } from './components/StudioApp.js';
import { activateStudioMigration, studioMigrationStatus } from './persistence/migration.js';

vi.mock('preact', async (importOriginal) => {
    const actual = await importOriginal<typeof import('preact')>();
    return { ...actual, render: vi.fn() };
});

vi.mock('../assets/js/model_sources.js', () => ({ ModelSources: {}, LOAD_PROGRESS_EVENT: 'uor-model-load-progress' }));

class FakeWorker {
    postMessage = vi.fn();
    addEventListener = vi.fn();
    removeEventListener = vi.fn();
    terminate = vi.fn();
}

const legacyWindow = window as unknown as {
    __studioApp?: { worker: FakeWorker };
    __studioFiles: Record<string, unknown>;
    __studioUi: Record<string, unknown>;
};

describe('typed Studio bootstrap', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('Worker', FakeWorker);
        activateStudioMigration(false);
        window.localStorage.clear();
        delete legacyWindow.__studioApp;
        legacyWindow.__studioFiles = { tabs: () => [], activeFile: () => null };
        legacyWindow.__studioUi = {
            getSettings: () => ({}),
            getAttachments: () => [],
            listSessions: () => [],
            activeSessionId: () => null,
            getConversationMessages: () => [],
        };
    });

    it('mounts only for the explicit typed=1 preview', () => {
        expect(typedPreviewRequested('?typed=1')).toBe(true);
        expect(typedPreviewRequested('?typed=0')).toBe(false);
        expect(typedPreviewRequested('?other=1')).toBe(false);
        expect(typedPreviewRequested('')).toBe(false);
    });

    it('passes the real StudioApp with lifecycle, model worker, and workflow APIs to Preact', async () => {
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        expect(render).toHaveBeenCalledOnce();
        expect(render).toHaveBeenCalledWith(expect.objectContaining({ type: StudioApp }), root);
        expect(root.hidden).toBe(false);
        expect(mounted.store).toBeInstanceOf(LifecycleStore);
        expect(mounted.worker).toBeInstanceOf(FakeWorker);
        expect(mounted.worker.addEventListener).toHaveBeenCalledWith('message', expect.any(Function));
        expect(mounted.files).toBe(legacyWindow.__studioFiles);
        expect(mounted.sessions).toBe(legacyWindow.__studioUi);
        expect(mounted.ui).toBe(legacyWindow.__studioUi);
        mounted.dispose();
        expect(root.hidden).toBe(true);
    });

    it('reuses an existing model worker instead of starting another engine', async () => {
        const root = document.createElement('div');
        const existing = new FakeWorker();
        legacyWindow.__studioApp = { worker: existing };
        const mounted = await mountTypedStudio(root);
        expect(mounted.worker).toBe(existing);
        expect(mounted.worker.addEventListener).toHaveBeenCalledWith('message', expect.any(Function));
        mounted.dispose();
        expect(existing.terminate).not.toHaveBeenCalled();
        expect(root.hidden).toBe(true);
    });

    it('refuses to mount when session methods are missing', async () => {
        const root = document.createElement('div');
        legacyWindow.__studioUi = { getSettings: () => ({}), getAttachments: () => [] };
        await expect(mountTypedStudio(root, { timeoutMs: 10 })).rejects.toThrow('did not initialize');
        expect(render).not.toHaveBeenCalled();
        expect(root.hidden).toBe(false);
    });

    it('refuses Preact rendering when migration fails', async () => {
        window.localStorage.setItem('uor.studio.settings.v1', '{bad-json');
        expect(activateStudioMigration(true).state).toBe('error');
        expect(studioMigrationStatus().state).toBe('error');
        const root = document.createElement('div');

        await expect(mountTypedStudio(root)).rejects.toThrow('migration');
        expect(render).not.toHaveBeenCalled();
    });
});
