import { describe, expect, it, vi } from 'vitest';
import { createLegacyWorkflowApis } from './workflow.js';
import type { SessionApi, StudioSettings, StudioUiApi, WorkspaceApi } from './workflow.js';

function files(): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(),
        openFile: vi.fn(async () => undefined), newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(),
        saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true), insertText: vi.fn(() => true),
        setActiveContent: vi.fn(), activeFile: vi.fn(() => null), workspaceName: vi.fn(() => ''),
        mode: vi.fn(() => 'none'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'none', rootName: '', fileCount: 0, tabs: [], activePath: null })),
    };
}

function sessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'wasm'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function ui(): StudioUiApi {
    return {
        setView: vi.fn(), toast: vi.fn(), getSettings: vi.fn((): StudioSettings => ({ theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false })),
        updateSettings: vi.fn(), updateStatus: vi.fn(), getAttachments: vi.fn(() => []), addAttachment: vi.fn(), removeAttachment: vi.fn(), clearAttachments: vi.fn(),
        templatePrompt: vi.fn((kind: string) => `template:${kind}`), clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(),
    };
}

describe('legacy workflow bridge', () => {
    it('forwards editor, settings, template, and clearing calls to the existing globals', () => {
        const fileApi = files();
        const sessionApi = sessions();
        const uiApi = ui();
        const root = { __studioFiles: fileApi, __studioUi: { ...sessionApi, ...uiApi } } as unknown as Window;
        const bridge = createLegacyWorkflowApis(root);
        bridge.files.setActiveContent('edited text');
        const patch: Partial<StudioSettings> = { reduceMotion: true, tabSize: 2 };
        bridge.ui.updateSettings(patch);
        expect(bridge.ui.templatePrompt('audit')).toBe('template:audit');
        bridge.ui.clearSessions();
        bridge.ui.clearEditor();
        expect(fileApi.setActiveContent).toHaveBeenCalledWith('edited text');
        expect(uiApi.updateSettings).toHaveBeenCalledWith(patch);
        expect(uiApi.clearSessions).toHaveBeenCalledOnce();
        expect(uiApi.clearEditor).toHaveBeenCalledOnce();
    });
});
