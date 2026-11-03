import { vi } from 'vitest';
import type { ModelSourceAdapter } from '../adapters/modelSourceAdapter.js';
import type { ModelSourcesApi } from '../../assets/js/model_sources.js';
import type { SessionApi, StudioSettings, StudioUiApi, WorkspaceApi } from '../adapters/workflow.js';

export const testSettings: StudioSettings = { theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false };

export function makeModelAdapter(): ModelSourceAdapter {
    const result = { bytes: new Uint8Array(), fileName: 'model.lut', meta: { schema: 'uor-r4.lut-stack/1' } };
    return {
        loadPublished: vi.fn(async () => result), loadFromUrl: vi.fn(async () => result),
        loadLocalFile: vi.fn(async () => result), loadBundled: vi.fn(async () => result),
        selectApi: vi.fn(), testApi: vi.fn(async () => ({ ok: true })), handleWorkerMessage: vi.fn(),
    };
}

export function makeSources(): ModelSourcesApi {
    const list = () => ({ active: { kind: 'wasm', id: 'bundled-demo' }, wasm: {}, api: [] });
    return {
        catalog: vi.fn(() => ({ models: [] })), loadModelCatalog: vi.fn(async () => undefined),
        list: vi.fn(list), active: vi.fn(() => list().active), apiConfig: vi.fn(() => null),
        setActive: vi.fn(), addApi: vi.fn(), updateApi: vi.fn(), removeApi: vi.fn(() => list()),
        loadPublished: vi.fn(), loadFromUrl: vi.fn(), loadLocalFile: vi.fn(), loadBundled: vi.fn(), testApi: vi.fn(),
    };
}

export function makeFiles(): WorkspaceApi {
    return {
        openFolder: vi.fn(async () => undefined), importFiles: vi.fn(), openDemo: vi.fn(), openFile: vi.fn(async () => undefined),
        newFile: vi.fn(), closeTab: vi.fn(), closeAll: vi.fn(), saveActive: vi.fn(async () => true), saveAsCopy: vi.fn(() => true),
        insertText: vi.fn(() => true), setActiveContent: vi.fn(), activeFile: vi.fn(() => null), workspaceName: vi.fn(() => ''),
        mode: vi.fn(() => 'none'), tabs: vi.fn(() => []),
        treeSummary: vi.fn(() => ({ mode: 'none', rootName: '', fileCount: 0, tabs: [], activePath: null })),
    };
}

export function makeSessions(): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => []), activeSessionId: vi.fn(() => null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => null), exportSession: vi.fn(() => null), deleteSession: vi.fn(() => true),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true), sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

export function makeUi(): StudioUiApi {
    return {
        setView: vi.fn(), toast: vi.fn(), getSettings: vi.fn(() => ({ ...testSettings })), updateSettings: vi.fn(), updateStatus: vi.fn(),
        getAttachments: vi.fn(() => []), addAttachment: vi.fn(), removeAttachment: vi.fn(), clearAttachments: vi.fn(),
        templatePrompt: vi.fn((kind: string) => `Prompt for ${kind}`), clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(),
    };
}
