import { describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import { saveActiveFile } from './fileSave.js';
import type { WorkspaceApi } from './workflow.js';

function files(): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(), openFile: vi.fn(async () => undefined),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(), saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true),
        insertText: vi.fn(() => true), setActiveContent: vi.fn(), activeFile: vi.fn(() => ({ path: 'a.md', name: 'a.md', content: 'alpha', mode: 'directory' })), workspaceName: vi.fn(() => 'project'),
        mode: vi.fn(() => 'directory'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'directory', rootName: 'project', fileCount: 2, tabs: [], activePath: null })),
    };
}

describe('saveActiveFile', () => {
    it('does not report an old save failure against a newly selected file', async () => {
        const store = new LifecycleStore();
        const api = files();
        let finish!: (value: boolean) => void;
        api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const saving = saveActiveFile(store, api, vi.fn());
        await vi.waitFor(() => expect(api.saveActive).toHaveBeenCalledOnce());
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        finish(false);
        expect(await saving).toBe(false);
        expect(store.getState().file).toMatchObject({ path: 'b.md', phase: 'clean', error: null });
    });

    it('does not complete a newer same-path save from an older operation', async () => {
        const store = new LifecycleStore();
        const api = files();
        const completions: Array<(value: boolean) => void> = [];
        api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { completions.push(resolve); }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const first = saveActiveFile(store, api, vi.fn());
        await vi.waitFor(() => expect(completions).toHaveLength(1));
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const second = saveActiveFile(store, api, vi.fn());
        completions[0](true);
        expect(await first).toBe(true);
        expect(store.getState().file.phase).toBe('saving');
        await vi.waitFor(() => expect(completions).toHaveLength(2));
        completions[1](true);
        expect(await second).toBe(true);
        expect(store.getState().file.phase).toBe('saved');
    });

    it('does not report an older same-path failure against a newer save', async () => {
        const store = new LifecycleStore();
        const api = files();
        const onError = vi.fn();
        const completions: Array<(value: boolean) => void> = [];
        api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { completions.push(resolve); }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const first = saveActiveFile(store, api, onError);
        await vi.waitFor(() => expect(completions).toHaveLength(1));
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const second = saveActiveFile(store, api, onError);
        completions[0](false);
        expect(await first).toBe(false);
        expect(onError).not.toHaveBeenCalled();
        expect(store.getState().file.phase).toBe('saving');
        await vi.waitFor(() => expect(completions).toHaveLength(2));
        completions[1](true);
        expect(await second).toBe(true);
        expect(store.getState().file.phase).toBe('saved');
    });

    it('skips a queued save when its captured path is no longer active', async () => {
        const store = new LifecycleStore();
        const api = files();
        let finish!: (value: boolean) => void;
        api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { finish = resolve; }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const first = saveActiveFile(store, api, vi.fn());
        await vi.waitFor(() => expect(api.saveActive).toHaveBeenCalledOnce());
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        api.activeFile = vi.fn(() => ({ path: 'b.md', name: 'b.md', content: 'bravo', mode: 'directory' }));
        const second = saveActiveFile(store, api, vi.fn());
        store.dispatch({ type: 'file/opened', path: 'c.md' });
        finish(true);
        expect(await first).toBe(false);
        expect(await second).toBe(false);
        expect(api.saveActive).toHaveBeenCalledTimes(1);
        expect(store.getState().file.path).toBe('c.md');
    });

    it('refuses to save when the store and file module disagree on the active path', async () => {
        const store = new LifecycleStore();
        const api = files();
        api.activeFile = vi.fn(() => ({ path: 'b.md', name: 'b.md', content: 'bravo', mode: 'directory' }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        store.dispatch({ type: 'file/dirty' });
        expect(await saveActiveFile(store, api, vi.fn())).toBe(false);
        expect(api.saveActive).not.toHaveBeenCalled();
        expect(store.getState().file.phase).toBe('error');
        expect(store.getState().file.error).toBe('The active file changed before saving.');
    });

    it('does not complete a newer save from an older completion', async () => {
        const store = new LifecycleStore();
        const api = files();
        const completions: Array<(value: boolean) => void> = [];
        api.saveActive = vi.fn(() => new Promise<boolean>((resolve) => { completions.push(resolve); }));
        store.dispatch({ type: 'file/opened', path: 'a.md' });
        const first = saveActiveFile(store, api, vi.fn());
        await vi.waitFor(() => expect(completions).toHaveLength(1));
        store.dispatch({ type: 'file/opened', path: 'b.md' });
        api.activeFile = vi.fn(() => ({ path: 'b.md', name: 'b.md', content: 'bravo', mode: 'directory' }));
        const second = saveActiveFile(store, api, vi.fn());
        completions[0](true);
        expect(await first).toBe(false);
        expect(store.getState().file.phase).toBe('saving');
        await vi.waitFor(() => expect(completions).toHaveLength(2));
        completions[1](true);
        expect(await second).toBe(true);
        expect(store.getState().file).toMatchObject({ path: 'b.md', phase: 'saved' });
    });
});
