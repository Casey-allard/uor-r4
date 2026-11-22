import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error Plain-JavaScript workspace controller.
import { mountMcpConnections } from '../../assets/js/mcp_client.js';

beforeEach(() => {
    vi.resetModules();
    window.localStorage.clear();
    document.body.innerHTML = '<div id="workspaceSelector"></div><div id="workspaceTree"></div><div id="workspaceFiles"></div><div id="openFiles"></div><div id="recentFiles"></div><input id="editorName"><textarea id="editorText"></textarea><div id="editorStats"></div><div id="editorSaveState"></div><div id="toastRegion"></div><div id="mcpMount"></div><button id="ctxOpenFiles"></button><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div><div id="chatMessages"></div><div id="conversationList"></div><div id="statusClock"></div><div id="modelSourceMount"></div>';
});

afterEach(() => {
    delete (window as unknown as { __studioFiles?: unknown }).__studioFiles;
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    delete (window as unknown as { UORDesktopActions?: unknown }).UORDesktopActions;
    delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    document.body.innerHTML = '';
});

describe('exact-head repair regressions', () => {
    it('saves a native tab to its original grant after a second project opens', async () => {
        const writes: Array<{ path: string; content: string }> = [];
        let root = '/project-a';
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: root })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(async (path: string) => path.endsWith('note.txt') ? 'original' : ''),
            writeFile: vi.fn(async (path: string, content: string) => { writes.push({ path, content }); }),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: { openFolder(): Promise<void>; openFile(path: string): Promise<void>; saveActive(): Promise<boolean> } }).__studioFiles;
        await files.openFolder();
        await files.openFile('note.txt');
        (document.getElementById('editorText') as HTMLTextAreaElement).value = 'from project A';
        root = '/project-b';
        await files.openFolder();
        expect(await files.saveActive()).toBe(true);
        expect(writes).toEqual([{ path: '/project-a/note.txt', content: 'from project A' }]);
    });

    it('keeps same-named native tabs bound to their own project roots', async () => {
        const writes: Array<{ path: string; content: string }> = [];
        let root = '/project-a';
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: root })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(async (path: string) => path === '/project-a/note.txt' ? 'from project A' : 'from project B'),
            writeFile: vi.fn(async (path: string, content: string) => { writes.push({ path, content }); }),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: {
            openFolder(): Promise<void>;
            openFile(path: string, rootPath?: string): Promise<void>;
            saveActive(): Promise<boolean>;
            tabs(): Array<{ path: string; rootPath: string }>;
        } }).__studioFiles;
        await files.openFolder();
        await files.openFile('note.txt');
        const editor = document.getElementById('editorText') as HTMLTextAreaElement;
        editor.value = 'project A edited';
        editor.dispatchEvent(new Event('input', { bubbles: true }));
        root = '/project-b';
        await files.openFolder();
        await files.openFile('note.txt');
        expect(editor.value).toBe('from project B');
        expect(files.tabs().map((tab) => tab.rootPath)).toEqual(['/project-a', '/project-b']);
        await files.openFile('note.txt', '/project-a');
        expect(editor.value).toBe('project A edited');
        expect(await files.saveActive()).toBe(true);
        expect(writes).toEqual([{ path: '/project-a/note.txt', content: 'project A edited' }]);
    });

    it('disconnects a native MCP session when tool listing fails', async () => {
        const session = {
            listTools: vi.fn(async () => { throw new Error('list failed'); }),
            disconnect: vi.fn(async () => undefined),
        };
        (window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke: vi.fn(), Channel: class {} } };
        mountMcpConnections({ connectRemote: async () => session });
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('list failed'));
        expect(session.disconnect).toHaveBeenCalledOnce();
    });

    it('uses real Files, Changes, Terminal and Guide navigation commands', async () => {
        window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
        await import('../../assets/js/studio_ui.js');
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.waitFor(() => expect((window as unknown as { __studioUi?: unknown }).__studioUi).toBeDefined());
        const ui = (window as unknown as { __studioUi: { openPalette(): void } }).__studioUi;
        Element.prototype.scrollIntoView = vi.fn();
        ui.openPalette();
        const commands = document.getElementById('paletteList')!.textContent || '';
        expect(commands).toContain('Go to Changes');
        expect(commands).toContain('Go to Terminal');
        expect(commands).toContain('Go to Guide');
        expect(commands).toContain('⌘6');
        (document.getElementById('ctxOpenFiles') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe('workspace');
    });
});


describe('connection and file repair regressions', () => {
    it('keeps the working MCP session and tools when a replacement fails', async () => {
        const previous = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: 'working' }]),
            callTool: vi.fn(async () => ({ content: [{ type: 'text', text: 'still connected' }] })),
            disconnect: vi.fn(async () => undefined),
        };
        mountMcpConnections({ connectRemote: async (config: { url: string }) => {
            if (config.url.includes('bad')) throw new Error('replacement failed');
            return previous;
        } });
        const url = document.getElementById('mcpUrlInput') as HTMLInputElement;
        url.value = 'http://good.invalid/mcp';
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        url.value = 'http://bad.invalid/mcp';
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('mcpConnectionStatus')?.textContent).toContain('replacement failed'));
        expect(previous.disconnect).not.toHaveBeenCalled();
        (document.querySelector('.mcp-tool-button') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('mcpResultOutput')?.textContent).toContain('still connected'));
        expect(previous.callTool).toHaveBeenCalledOnce();
    });

    it('disables both native connect controls throughout one in-flight connection', async () => {
        const session = {
            listTools: vi.fn(async () => [{ name: 'read_file', description: 'remote' }]),
            disconnect: vi.fn(async () => undefined),
        };
        let resolveConnection!: (value: typeof session) => void;
        const pending = new Promise<typeof session>((resolve) => { resolveConnection = resolve; });
        const invoke = vi.fn(async () => null);
        (window as unknown as { __TAURI__: unknown }).__TAURI__ = { core: { invoke, Channel: class {} } };
        mountMcpConnections({ connectRemote: () => pending });
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        const stdio = document.getElementById('mcpStdioButton') as HTMLButtonElement;
        const remote = document.getElementById('mcpConnectButton') as HTMLButtonElement;
        const disabledDuringFlight = stdio.disabled && remote.disabled;
        stdio.click();
        resolveConnection(session);
        await vi.waitFor(() => expect(document.querySelector('.mcp-tool-button')).not.toBeNull());
        expect(disabledDuringFlight).toBe(true);
        expect(invoke).not.toHaveBeenCalled();
        expect(stdio.disabled).toBe(false);
        expect(remote.disabled).toBe(false);
    });

    it('disconnects a replacement that resolves after pagehide', async () => {
        const stale = {
            listTools: vi.fn(async () => []),
            disconnect: vi.fn(async () => undefined),
        };
        let resolveConnection!: (value: typeof stale) => void;
        const pending = new Promise<typeof stale>((resolve) => { resolveConnection = resolve; });
        mountMcpConnections({ connectRemote: () => pending });
        (document.getElementById('mcpConnectButton') as HTMLButtonElement).click();
        window.dispatchEvent(new Event('pagehide'));
        resolveConnection(stale);
        await vi.waitFor(() => expect(stale.disconnect).toHaveBeenCalledOnce());
        expect(stale.listTools).not.toHaveBeenCalled();
    });

    it('deduplicates concurrent opens by path and granted root', async () => {
        let release!: (value: string) => void;
        const content = new Promise<string>((resolve) => { release = resolve; });
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: '/project-a' })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(() => content),
            writeFile: vi.fn(async () => undefined),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: {
            openFolder(): Promise<void>;
            openFile(path: string): Promise<void>;
            tabs(): Array<{ path: string; rootPath: string }>;
        } }).__studioFiles;
        await files.openFolder();
        const first = files.openFile('note.txt');
        const second = files.openFile('note.txt');
        release('one shared buffer');
        await Promise.all([first, second]);
        expect(files.tabs().map((tab) => ({ path: tab.path, rootPath: tab.rootPath }))).toEqual([
            { path: 'note.txt', rootPath: '/project-a' },
        ]);
    });

    it('persists recent file identity and labels its granted root', async () => {
        let root = '/project-a';
        (window as unknown as { UORDesktopActions: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: vi.fn(async () => ({ path: root })),
            listDirectory: vi.fn(async () => ['note.txt']),
            readFile: vi.fn(async (path: string) => path),
            writeFile: vi.fn(async () => undefined),
        };
        // @ts-expect-error The plain-JavaScript workspace controller has no declaration file.
        await import('../../assets/js/studio_files.js');
        const files = (window as unknown as { __studioFiles: {
            openFolder(): Promise<void>;
            openFile(path: string): Promise<void>;
        } }).__studioFiles;
        await files.openFolder();
        await files.openFile('note.txt');
        root = '/project-b';
        await files.openFolder();
        await files.openFile('note.txt');
        const recent = JSON.parse(window.localStorage.getItem('uor.studio.recent.v2') || '[]');
        expect(recent).toEqual([
            { path: 'note.txt', rootPath: '/project-b' },
            { path: 'note.txt', rootPath: '/project-a' },
        ]);
        const labels = Array.from(document.querySelectorAll('.recent-file')).map((node) => node.textContent);
        expect(labels).toEqual([
            'note.txt — /project-b',
            'note.txt — /project-a',
        ]);
    });
});
