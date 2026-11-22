import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { WorkspaceApi } from './workflow.js';

const TABS_KEY = 'uor.studio.tabs.v2';
const studioFilesModule = '../../assets/js/studio_files.js';

async function loadFiles(): Promise<WorkspaceApi> {
    vi.resetModules();
    document.body.innerHTML = '<div id="toastRegion"></div><textarea id="editorText"></textarea><div id="fileTree"></div><div id="editorTabs"></div><ul id="recentFiles"></ul><span id="topWorkspaceName"></span><span id="statusWorkspaceLabel"></span><span id="workspaceMeta"></span><span id="ctxWorkspaceLabel"></span><span id="ctxWorkspaceMeta"></span>';
    await import(studioFilesModule);
    if (document.readyState === 'loading') document.dispatchEvent(new Event('DOMContentLoaded'));
    return (window as unknown as { __studioFiles: WorkspaceApi }).__studioFiles;
}

beforeEach(() => {
    vi.useFakeTimers();
    window.location.hash = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
});

afterEach(async () => {
    await vi.runAllTimersAsync();
    vi.clearAllTimers();
    (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles?.closeAll();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
});

describe('actual demo workspace behavior', () => {
    it('refuses to replace dirty work and preserves the active buffer', async () => {
        const files = await loadFiles();
        files.newFile();
        files.setActiveContent('unsaved work');
        const attachments = [{ name: 'real-notes.md', content: 'keep this' }];
        expect(files.openDemo()).toBe(false);
        expect(files.activeFile()?.content).toBe('unsaved work');
        expect(attachments).toEqual([{ name: 'real-notes.md', content: 'keep this' }]);
        expect(files.mode()).not.toBe('demo');
    });

    it('loads a clean demo without network or fabricated dirty edits', async () => {
        vi.useFakeTimers();
        const network = vi.fn();
        vi.stubGlobal('fetch', network);
        const files = await loadFiles();
        expect(files.openDemo()).toBe(true);
        await vi.advanceTimersByTimeAsync(1800);
        expect(files.mode()).toBe('demo');
        expect(files.workspaceName()).toContain('demo');
        expect(files.tabs()).toHaveLength(2);
        expect(files.tabs().every((tab) => !tab.dirty)).toBe(true);
        expect(document.getElementById('topWorkspaceName')?.textContent).toContain('demo');
        expect(network).not.toHaveBeenCalled();
    });

    it('restores demo tabs with their labels and synthetic tree', async () => {
        window.localStorage.setItem(TABS_KEY, JSON.stringify({
            activePath: 'web/studio/index.html',
            tabs: [{ path: 'web/studio/index.html', content: '<p>demo</p>', dirty: false, mode: 'demo' }],
        }));
        const files = await loadFiles();
        expect(files.mode()).toBe('demo');
        expect(files.workspaceName()).toContain('demo');
        expect(files.activeFile()?.path).toBe('web/studio/index.html');
        expect(files.activeFile()?.content).toBe('<p>demo</p>');
        expect(document.getElementById('topWorkspaceName')?.textContent).toContain('demo');
        expect(document.getElementById('statusWorkspaceLabel')?.textContent).toContain('demo');
        expect(document.getElementById('fileTree')?.textContent).not.toContain('No folder open');
        expect(document.getElementById('fileTree')?.textContent).toContain('index.css');
    });

    it('keeps new and inserted demo tabs in demo mode after reload', async () => {
        const files = await loadFiles();
        expect(files.openDemo()).toBe(true);
        await vi.advanceTimersByTimeAsync(1800);
        files.newFile();
        expect(files.activeFile()?.mode).toBe('demo');
        files.insertText('agent context', 'agent-snippet.md');
        expect(files.activeFile()?.mode).toBe('demo');
        delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
        const restored = await loadFiles();
        expect(restored.mode()).toBe('demo');
        expect(restored.tabs().every((tab) => tab.mode === 'demo')).toBe(true);
        expect(restored.activeFile()?.content).toBe('agent context');
        expect(document.getElementById('fileTree')?.textContent).toContain('agent-snippet.md');
        expect(document.getElementById('fileTree')?.textContent).not.toContain('No folder open');
    });

    it('restores an empty new demo tab and its tree entry', async () => {
        const files = await loadFiles();
        expect(files.openDemo()).toBe(true);
        await vi.advanceTimersByTimeAsync(1800);
        files.newFile();
        delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
        const restored = await loadFiles();
        expect(restored.mode()).toBe('demo');
        expect(restored.activeFile()).toMatchObject({ path: 'untitled.md', content: '', mode: 'demo' });
        expect(restored.treeSummary().fileCount).toBe(7);
        expect(document.getElementById('fileTree')?.textContent).toContain('untitled.md');
    });
});
