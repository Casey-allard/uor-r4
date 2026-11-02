import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi, WorkspaceApi } from './workflow.js';

const studioFilesModule = '../../assets/js/studio_files.js';
const studioUiModule = '../../assets/js/studio_ui.js';

beforeEach(() => {
    vi.useFakeTimers();
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    window.localStorage.clear();
    window.location.hash = '';
    document.body.innerHTML = '<div id="app"></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div><textarea id="editorText"></textarea><div id="fileTree"></div><div id="editorTabs"></div><ul id="recentFiles"></ul><span id="topWorkspaceName"></span><span id="statusWorkspaceLabel"></span>';
});

afterEach(() => {
    vi.clearAllTimers();
    (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles?.closeAll();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
});

describe('actual clearEditor writer', () => {
    it('closes real tabs and clears the legacy editor', async () => {
        vi.resetModules();
        await import(studioFilesModule);
        await import(studioUiModule);
        if (document.readyState === 'loading') document.dispatchEvent(new Event('DOMContentLoaded'));
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        const files = (window as unknown as { __studioFiles: WorkspaceApi }).__studioFiles;
        files.openDemo();
        await vi.advanceTimersByTimeAsync(1800);
        const ui = (window as unknown as { __studioUi: StudioUiApi }).__studioUi;
        expect(files.tabs()).toHaveLength(2);
        ui.clearEditor();
        expect(files.tabs()).toEqual([]);
        expect(files.activeFile()).toBeNull();
        expect(document.getElementById('editorText')).toHaveProperty('value', '');
    });
});
