import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'preact';
import { LifecycleStore } from './state/lifecycle.js';
import {
    createStudioAgentOwner,
    mountTypedStudio,
    studioTestApiEnabled,
    typedPreviewRequested,
} from './bootstrap.js';
import { AgentClient } from './agent/client.js';
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
    __TAURI__?: { core: { invoke(command: string, payload?: unknown): Promise<unknown> } };
};

describe('typed Studio bootstrap', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.unstubAllEnvs();
        vi.unstubAllGlobals();
        vi.stubGlobal('Worker', FakeWorker);
        activateStudioMigration(false);
        window.history.replaceState({}, '', '/');
        window.localStorage.clear();
        delete legacyWindow.__studioApp;
        delete legacyWindow.__TAURI__;
        legacyWindow.__studioFiles = { tabs: () => [], activeFile: () => null };
        legacyWindow.__studioUi = {
            getSettings: () => ({}),
            getAttachments: () => [],
            listSessions: () => [],
            activeSessionId: () => null,
            getConversationMessages: () => [],
        };
    });

    it('defaults to typed mounting while preserving explicit legacy selections', () => {
        expect(typedPreviewRequested('?typed=1')).toBe(true);
        expect(typedPreviewRequested('?typed=0')).toBe(false);
        expect(typedPreviewRequested('?legacy=1')).toBe(false);
        expect(typedPreviewRequested('?other=1')).toBe(true);
        expect(typedPreviewRequested('')).toBe(true);
    });

    it('requires both the build flag and the studioTest query', () => {
        expect(studioTestApiEnabled('1', '?studioTest=1')).toBe(true);
        expect(studioTestApiEnabled('1', '?typed=1')).toBe(false);
        expect(studioTestApiEnabled(undefined, '?studioTest=1')).toBe(false);
        expect(studioTestApiEnabled('0', '?studioTest=1')).toBe(false);
        expect(studioTestApiEnabled('1', '?studioTest=0')).toBe(false);
    });

    it('installs the test API only when both gates are present', async () => {
        vi.stubEnv('VITE_STUDIO_TEST', '1');
        window.history.replaceState({}, '', '/?typed=1&studioTest=1');
        try {
            const root = document.createElement('div');
            const mounted = await mountTypedStudio(root);
            expect(window.__studioTest).toBeDefined();
            expect(window.__studioTest!.clientForTest()).toBe(mounted.agentClient);
            mounted.dispose();
            delete window.__studioTest;

            window.history.replaceState({}, '', '/?typed=1');
            const noQueryRoot = document.createElement('div');
            const noQuery = await mountTypedStudio(noQueryRoot);
            expect(window.__studioTest).toBeUndefined();
            noQuery.dispose();
        } finally {
            delete window.__studioTest;
            vi.unstubAllEnvs();
            window.history.replaceState({}, '', '/');
        }
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
        expect(['migrated', 'current']).toContain(studioMigrationStatus().state);
        mounted.dispose();
        expect(root.hidden).toBe(true);
    });

    it('preserves an already migrated store instead of reactivating it', async () => {
        activateStudioMigration(true);
        expect(studioMigrationStatus().state).toBe('migrated');
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        expect(studioMigrationStatus().state).toBe('migrated');
        mounted.dispose();
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

    it('disposes the installed legacy shortcut guard when workflow initialization rejects', async () => {
        const palette = document.createElement('div');
        palette.id = 'commandPalette';
        palette.hidden = true;
        document.body.appendChild(palette);
        const remove = vi.spyOn(window, 'removeEventListener');
        const root = document.createElement('div');
        legacyWindow.__studioUi = { getSettings: () => ({}), getAttachments: () => [] };
        try {
            await expect(mountTypedStudio(root, { timeoutMs: 10 })).rejects.toThrow('did not initialize');
            expect(remove).toHaveBeenCalledWith('keydown', expect.any(Function), true);

            const blocked = new KeyboardEvent('keydown', { key: 'k', metaKey: true, bubbles: true, cancelable: true });
            document.body.dispatchEvent(blocked);
            expect(blocked.defaultPrevented, 'the legacy Cmd/Ctrl+K blocker must be removed').toBe(false);

            palette.hidden = false;
            window.dispatchEvent(new CustomEvent('uor-studio-v2-ready'));
            await new Promise((resolve) => setTimeout(resolve, 0));
            expect(palette.hidden, 'the readiness listener must be removed').toBe(false);

            palette.hidden = true;
            await new Promise((resolve) => setTimeout(resolve, 0));
            palette.hidden = false;
            await new Promise((resolve) => setTimeout(resolve, 0));
            expect(palette.hidden, 'the palette observer must be disconnected').toBe(false);
        } finally {
            palette.remove();
            remove.mockRestore();
        }
    });

    it('suppresses legacy Cmd/Ctrl+comma for editable, IME and modified chords', async () => {
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        const editable = document.createElement('div');
        editable.setAttribute('contenteditable', 'true');
        const child = document.createElement('span');
        editable.appendChild(child);
        document.body.appendChild(editable);
        const input = document.createElement('input');
        document.body.appendChild(input);
        const outside = document.createElement('button');
        document.body.appendChild(outside);
        const press = (target: EventTarget, init: KeyboardEventInit & { isComposing?: boolean; keyCode?: number }) => {
            const event = new KeyboardEvent('keydown', { key: ',', bubbles: true, cancelable: true, ...init });
            Object.defineProperties(event, {
                isComposing: { value: Boolean(init.isComposing) },
                keyCode: { value: init.keyCode ?? 0 },
            });
            target.dispatchEvent(event);
            return event;
        };
        const legacy = vi.fn();
        document.addEventListener('keydown', legacy);
        try {
            expect(press(child, { metaKey: true }).defaultPrevented, 'contenteditable descendant').toBe(true);
            expect(press(input, { ctrlKey: true }).defaultPrevented, 'input').toBe(true);
            expect(legacy).not.toHaveBeenCalled();

            expect(press(outside, { metaKey: true }).defaultPrevented, 'outside editable').toBe(false);
            expect(legacy).toHaveBeenCalledTimes(1);
            const baseline = legacy.mock.calls.length;

            for (const [context, target, init] of [
                ['IME composition in editable', child, { metaKey: true, isComposing: true }],
                ['IME keyCode 229 in editable', child, { metaKey: true, keyCode: 229 }],
                ['Alt+Meta comma in editable', child, { metaKey: true, altKey: true }],
                ['Alt+Ctrl comma outside', outside, { ctrlKey: true, altKey: true }],
                ['Shift+Meta comma in editable', child, { metaKey: true, shiftKey: true }],
                ['Shift+Meta comma outside', outside, { metaKey: true, shiftKey: true }],
            ] as const) {
                const event = press(target, init);
                expect(event.defaultPrevented, context).toBe(false);
                expect(legacy.mock.calls.length, context).toBe(baseline);
            }

            expect(press(child, { metaKey: true, shiftKey: true, key: 'P' } as KeyboardEventInit).defaultPrevented, 'scoped palette chord').toBe(false);
        } finally {
            document.removeEventListener('keydown', legacy);
            editable.remove();
            input.remove();
            outside.remove();
            mounted.dispose();
        }
    });

    it('refuses Preact rendering when migration fails', async () => {
        window.localStorage.setItem('uor.studio.settings.v1', '{bad-json');
        expect(activateStudioMigration(true).state).toBe('error');
        expect(studioMigrationStatus().state).toBe('error');
        const root = document.createElement('div');

        await expect(mountTypedStudio(root)).rejects.toThrow('migration');
        expect(render).not.toHaveBeenCalled();
    });

    it('applies an explicit theme query through the existing settings API', async () => {
        const updateSettings = vi.fn();
        legacyWindow.__studioUi = { ...legacyWindow.__studioUi, updateSettings };
        window.history.replaceState({}, '', '/?typed=1&theme=light');
        try {
            const root = document.createElement('div');
            const mounted = await mountTypedStudio(root);
            expect(updateSettings).toHaveBeenCalledWith({ theme: 'light' });
            mounted.dispose();
        } finally {
            window.history.replaceState({}, '', '/');
        }
    });

    it('mounts exactly one browser runtime owner and passes its client to StudioApp', async () => {
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        const vnode = vi.mocked(render).mock.calls[0]![0] as unknown as {
            props: { agentClient?: AgentClient | null };
        };
        expect(mounted.agentClient).toBeInstanceOf(AgentClient);
        expect(vnode.props.agentClient).toBe(mounted.agentClient);
        expect(mounted.agentStore).toBe(mounted.agentClient.store);
        expect(mounted.store).toBeInstanceOf(LifecycleStore);
        mounted.dispose();
    });

    it('composes the production Tauri host client as the single runtime owner', async () => {
        const commands: string[] = [];
        const invoke = vi.fn(async (command: string) => {
            commands.push(command);
            if (command === 'agent_host_capabilities') {
                return {
                    tree: true,
                    changes: true,
                    readArtifact: true,
                    terminal: true,
                    git: true,
                    mcp: true,
                    approvals: true,
                };
            }
            throw new Error(`unexpected host command ${command}`);
        });
        legacyWindow.__TAURI__ = { core: { invoke } };
        vi.stubGlobal('indexedDB', {} as IDBFactory);
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        try {
            expect(root.dataset.host).toBe('tauri');
            expect(await mounted.agentClient.hostCapabilities()).toEqual({
                tree: true,
                changes: true,
                readArtifact: true,
                terminal: true,
                git: true,
                mcp: true,
                approvals: true,
            });
            expect(commands).toEqual(['agent_host_capabilities']);
            expect(mounted.agentRuntime).not.toBeNull();
            expect(mounted.agentRuntime!.hostClient).not.toBeNull();
            expect(mounted.agentClient.store).toBe(mounted.agentStore);
            expect(mounted.agentRuntime!.store).toBe(mounted.agentStore);
            expect(mounted.agentTaskStore!.store).toBe(mounted.agentStore);
        } finally {
            mounted.dispose();
            delete legacyWindow.__TAURI__;
        }
    });

    it('keeps the static host explicit and host_unavailable without a Tauri bridge', async () => {
        vi.stubGlobal('indexedDB', {} as IDBFactory);
        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        try {
            expect(root.dataset.host).toBe('static');
            expect(await mounted.agentClient.hostCapabilities()).toEqual({
                tree: false,
                changes: false,
                readArtifact: false,
                terminal: false,
                git: false,
                mcp: false,
                approvals: false,
            });
            expect(mounted.agentRuntime).not.toBeNull();
            expect(mounted.agentRuntime!.hostClient).not.toBeNull();
            expect(mounted.agentClient.store).toBe(mounted.agentStore);
            expect(mounted.agentRuntime!.store).toBe(mounted.agentStore);
            expect(mounted.agentTaskStore!.store).toBe(mounted.agentStore);
        } finally {
            mounted.dispose();
        }
    });

    it('reports an unavailable runtime without removing the legacy shell', async () => {
        const owner = createStudioAgentOwner();
        expect(owner.client).toBeInstanceOf(AgentClient);
        expect(owner.runtime).toBeNull();
        expect(owner.taskStore).toBeNull();
        await expect(owner.client.listTasks({})).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });

        const root = document.createElement('div');
        const mounted = await mountTypedStudio(root);
        const vnode = vi.mocked(render).mock.calls[0]![0] as unknown as {
            props: { store: LifecycleStore; files: unknown; agentClient: AgentClient };
        };
        expect(vnode.props.store).toBeInstanceOf(LifecycleStore);
        expect(vnode.props.files).toBe(legacyWindow.__studioFiles);
        expect(vnode.props.agentClient).toBe(mounted.agentClient);
        await expect(mounted.agentClient.listTasks({})).rejects.toMatchObject({
            code: 'runtime_unavailable',
        });
        mounted.dispose();
    });
});
