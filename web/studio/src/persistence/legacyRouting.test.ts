import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';
import './migration.js';

interface PersistenceStatus { state: string }
interface PersistenceGlobal { activate(typed: boolean): PersistenceStatus }
const legacyWindow = window as unknown as {
    UORStudioPersistence: PersistenceGlobal;
    __studioUi: StudioUiApi;
    __studioFiles: WorkspaceApi;
};

function persistence(): PersistenceGlobal {
    return legacyWindow.UORStudioPersistence;
}

function uiDom() {
    document.body.innerHTML = '<div id="app"></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div><textarea id="editorText"></textarea>';
}

beforeEach(() => {
    vi.resetModules();
    window.localStorage.clear();
    persistence().activate(false);
    legacyWindow.__studioUi = { getMode: () => 'api', listSessions: () => [], deleteSession: () => false } as unknown as StudioUiApi;
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
});

afterEach(() => {
    vi.clearAllTimers();
    persistence().activate(false);
    window.localStorage.clear();
    document.body.innerHTML = '';
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
    delete (window as unknown as { __studioFiles?: WorkspaceApi }).__studioFiles;
});

describe('actual legacy storage routing after migration', () => {
    it('restores UI state and writes settings, theme, and attachments only to destinations', async () => {
        const settings = '{"theme":"dark","fontSize":16,"futureSetting":true}';
        const attachments = '[{"name":"seed.txt","content":"seed"}]';
        window.localStorage.setItem('uor.studio.settings.v1', settings);
        window.localStorage.setItem('uor.studio.attachments.v1', attachments);
        window.localStorage.setItem('uor.studio.theme', 'dark');
        expect(persistence().activate(true).state).toBe('migrated');
        uiDom();

        await import('../../assets/js/studio_ui.js');
        await vi.waitFor(() => expect(legacyWindow.__studioUi.addAttachment).toBeTypeOf('function'));
        const ui = legacyWindow.__studioUi;
        expect(ui.getSettings()).toMatchObject({ fontSize: 16, futureSetting: true });
        expect(ui.getAttachments()).toMatchObject([{ name: 'seed.txt', content: 'seed' }]);

        ui.updateSettings({ fontSize: 20 });
        ui.addAttachment({ name: 'new.txt', content: 'new' });

        expect(window.localStorage.getItem('uor.studio.settings.v1')).toBe(settings);
        expect(window.localStorage.getItem('uor.studio.attachments.v1')).toBe(attachments);
        expect(window.localStorage.getItem('uor.studio.theme')).toBe('dark');
        expect(JSON.parse(window.localStorage.getItem('uor.studio.lifecycle.settings.v2') || '{}')).toMatchObject({ fontSize: 20, futureSetting: true });
        expect(JSON.parse(window.localStorage.getItem('uor.studio.lifecycle.attachments.v2') || '[]')).toHaveLength(2);
        expect(window.localStorage.getItem('uor.studio.lifecycle.theme.v1')).toBe('dark');

        vi.resetModules();
        // @ts-expect-error A query suffix creates a fresh real UI initialization against divergent destinations.
        await import('../../assets/js/studio_ui.js?divergent-read');
        await vi.waitFor(() => expect(legacyWindow.__studioUi.getSettings().fontSize).toBe(20));
        expect(legacyWindow.__studioUi.getAttachments()).toMatchObject([{ name: 'seed.txt', content: 'seed' }, { name: 'new.txt', content: 'new' }]);
        expect(window.localStorage.getItem('uor.studio.settings.v1')).toBe(settings);
        expect(window.localStorage.getItem('uor.studio.attachments.v1')).toBe(attachments);
    });

    it('retains direct legacy writes when typed activation is off', async () => {
        const settings = '{"theme":"dark","fontSize":16}';
        window.localStorage.setItem('uor.studio.settings.v1', settings);
        expect(persistence().activate(false).state).toBe('inactive');
        uiDom();

        await import('../../assets/js/studio_ui.js');
        await vi.waitFor(() => expect(legacyWindow.__studioUi.updateSettings).toBeTypeOf('function'));
        legacyWindow.__studioUi.updateSettings({ fontSize: 22 });

        expect(JSON.parse(window.localStorage.getItem('uor.studio.settings.v1') || '{}')).toMatchObject({ fontSize: 22 });
        expect(window.localStorage.getItem('uor.studio.lifecycle.settings.v2')).toBeNull();
        expect(window.localStorage.getItem('uor.studio.lifecycle.migration.v1')).toBeNull();
    });

    it('restores tabs and routes later editor persistence away from rollback keys', async () => {
        const tabs = JSON.stringify({ activePath: 'draft.md', activeRootPath: '', tabs: [{ path: 'draft.md', content: 'seed draft', dirty: true, mode: 'draft', rootPath: '' }] });
        window.localStorage.setItem('uor.studio.tabs.v2', tabs);
        expect(persistence().activate(true).state).toBe('migrated');
        document.body.innerHTML = '<div id="fileTree"></div><div id="fileTreeMeta"></div><textarea id="editorText"></textarea>';
        document.dispatchEvent(new Event('DOMContentLoaded'));

        // @ts-expect-error The existing plain-JavaScript workspace module has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = legacyWindow.__studioFiles;
        expect(files.activeFile()).toMatchObject({ path: 'draft.md', content: 'seed draft' });
        files.setActiveContent('edited draft');

        expect(window.localStorage.getItem('uor.studio.tabs.v2')).toBe(tabs);
        expect(JSON.parse(window.localStorage.getItem('uor.studio.lifecycle.tabs.v3') || '{}')).toMatchObject({ activePath: 'draft.md' });
        expect(JSON.parse(window.localStorage.getItem('uor.studio.lifecycle.tabs.v3') || '{}').tabs[0].content).toBe('edited draft');

        vi.resetModules();
        document.body.innerHTML = '<div id="fileTree"></div><div id="fileTreeMeta"></div><textarea id="editorText"></textarea>';
        document.dispatchEvent(new Event('DOMContentLoaded'));
        // @ts-expect-error A query suffix creates a fresh real workspace initialization against divergent destinations.
        await import('../../assets/js/studio_files.js?divergent-read');
        expect(legacyWindow.__studioFiles.activeFile()).toMatchObject({ path: 'draft.md', content: 'edited draft' });
        expect(window.localStorage.getItem('uor.studio.tabs.v2')).toBe(tabs);
    });
});
