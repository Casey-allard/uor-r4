import type { LifecycleStore } from '../state/lifecycle.js';

export interface StudioFile {
    path: string;
    name: string;
    content: string;
    mode: string;
    rootPath?: string;
}

export interface WorkspaceTab {
    path: string;
    dirty: boolean;
    mode: string;
    rootPath?: string;
}

export interface WorkspaceSummary {
    mode: string;
    rootName: string;
    fileCount: number;
    tabs: Array<Pick<WorkspaceTab, 'path' | 'dirty'>>;
    activePath: string | null;
    activeRootPath?: string;
}

export interface WorkspaceApi {
    openFolder(): Promise<void>;
    importFiles(reason?: string): void;
    openDemo(): void;
    openFile(path: string, rootPath?: string): Promise<void>;
    newFile(): void;
    closeTab(path: string, rootPath?: string): void;
    closeAll(): void;
    saveActive(): Promise<boolean>;
    saveAsCopy(): boolean;
    insertText(text: string, name?: string): boolean;
    setActiveContent(content: string): void;
    activeFile(): StudioFile | null;
    workspaceName(): string;
    mode(): string;
    tabs(): WorkspaceTab[];
    treeSummary(): WorkspaceSummary;
}

export interface SessionSummary {
    id: string;
    title: string;
    createdAt: number;
    updatedAt: number;
    messageCount: number;
    preview: string;
}

export interface ChatMessage {
    role: string;
    content: string;
}

export interface SessionApi {
    getMode(): 'wasm' | 'api';
    listSessions(): SessionSummary[];
    activeSessionId(): string | null;
    selectSession(id: string): boolean;
    newSession(): boolean;
    renameSession(id: string, title: string): boolean;
    duplicateSession(id: string): string | null;
    exportSession(id?: string): string | null;
    deleteSession(id: string, createReplacement?: boolean): boolean;
    getConversationMessages(id?: string): ChatMessage[];
    setComposerText(text: string): boolean;
    sendPrompt(prompt: string): number | false;
    predict(prompt: string): number | false;
    stopGeneration(): void;
}

export interface Attachment {
    name: string;
    content: string;
    source?: string;
    kind?: string;
    size?: number;
    previewUrl?: string;
    imageUrl?: string;
    rgba?: Uint8Array | null;
    width?: number;
    height?: number;
    grid?: number;
    reattachRequired?: boolean;
    decomposition?: {
        schema: string;
        canonical_hex: string;
        source_blake3: string;
        width: number;
        height: number;
        grid: { columns: number; rows: number };
        decomposition_id: string;
    } | null;
}

export interface StudioSettings {
    theme: 'dark' | 'light' | 'system';
    reduceMotion: boolean;
    fontSize: number;
    tabSize: number;
    wordWrap: boolean;
}

export interface StudioUiApi {
    setView(view: string): void;
    toast(text: string, tone?: string): void;
    getSettings(): StudioSettings;
    updateSettings(patch: Partial<StudioSettings>): void;
    updateStatus(): void;
    getAttachments(): Attachment[];
    getImageRecallCount?(): number;
    addAttachment(entry: Attachment): void;
    removeAttachment(name: string): void;
    clearAttachments(): void;
    templatePrompt(kind: string): string;
    clearSessions(): { cleared: number; total: number; error?: string };
    clearEditor(): void;
}

export interface LegacyWorkflowApis {
    files: WorkspaceApi;
    sessions: SessionApi;
    ui: StudioUiApi;
}

export function createLegacyWorkflowApis(root: Window = window): LegacyWorkflowApis {
    const legacy = root as unknown as {
        __studioFiles: WorkspaceApi;
        __studioUi: SessionApi & StudioUiApi;
    };
    return { files: legacy.__studioFiles, sessions: legacy.__studioUi, ui: legacy.__studioUi };
}

export function syncWorkspaceSummary(store: LifecycleStore, summary: WorkspaceSummary): void {
    if (summary.mode !== 'none' && store.getState().file.phase === 'no-folder') {
        store.dispatch({ type: 'folder/opened', path: summary.rootName });
    }
    if (summary.activePath) {
        if (store.getState().file.path !== summary.activePath) {
            store.dispatch({ type: 'file/opened', path: summary.activePath });
        }
        const activeTab = summary.tabs.find((tab) => tab.path === summary.activePath);
        if (activeTab?.dirty && !['dirty', 'saving', 'error'].includes(store.getState().file.phase)) {
            store.dispatch({ type: 'file/dirty' });
        }
    } else if (store.getState().file.path !== null) {
        store.dispatch({ type: 'file/closed' });
    }
}
