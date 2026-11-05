import { afterEach, describe, expect, it, vi } from 'vitest';
// @ts-expect-error The plain-JavaScript desktop adapter has no declaration file.
import { createDesktopActions, mountDesktopActions } from '../../assets/js/desktop_actions.js';

const workspaceModule = '../../assets/js/studio_files.js';

function controls() {
    document.body.innerHTML = `
        <button id="btnRefreshChanges" disabled></button>
        <p id="gitStatusSummary"></p>
        <button id="btnCopyDiff" disabled></button>
        <pre id="gitDiffOutput"></pre>
        <p id="terminalConsentState"></p>
        <input id="terminalCwd">
        <input id="terminalCommand">
        <button id="btnTerminalEnable" disabled></button>
        <button id="btnRunTerminal" disabled></button>
        <pre id="terminalOutput"></pre>`;
}

afterEach(() => {
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { UORDesktopActions?: unknown }).UORDesktopActions;
    delete (window as unknown as { __studioFiles?: unknown }).__studioFiles;
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    vi.restoreAllMocks();
});

describe('desktop Git and terminal actions', () => {
    it('waits for explicit actions and renders bounded host results as inert text', async () => {
        controls();
        const calls: Array<{ command: string; args: Record<string, unknown> }> = [];
        const invoke = vi.fn(async (command: string, args: Record<string, unknown> = {}) => {
            calls.push({ command, args });
            if (command === 'native_git_status') return { branch: 'main', is_clean: false, modified_files: ['a.rs'], untracked_files: [], truncated: false, timed_out: false };
            if (command === 'native_git_diff') return { diff: 'diff --git a/a.rs b/a.rs\n+<img src=x onerror=window.__gitXss=1>', truncated: true, timed_out: false };
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: true };
            if (command === 'native_run_terminal_command') return '<img src=x onerror=window.__terminalXss=1>';
            throw new Error('unexpected command: ' + command);
        });
        const actions = createDesktopActions({ invoke });
        mountDesktopActions({ actions });
        expect(calls).toHaveLength(0);

        (document.getElementById('terminalCwd') as HTMLInputElement).value = '/granted/project';
        (document.getElementById('btnRefreshChanges') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('gitDiffOutput')?.textContent).toContain('<img'));
        expect(document.getElementById('gitStatusSummary')?.textContent).toContain('main');
        expect(document.getElementById('gitStatusSummary')?.textContent).toContain('truncated');
        expect(document.querySelector('#gitDiffOutput img')).toBeNull();
        expect((window as unknown as { __gitXss?: boolean }).__gitXss).toBeUndefined();

        (document.getElementById('btnTerminalEnable') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('enabled'));
        (document.getElementById('terminalCwd') as HTMLInputElement).value = '/granted/project';
        (document.getElementById('terminalCommand') as HTMLInputElement).value = 'pwd';
        (document.getElementById('btnRunTerminal') as HTMLButtonElement).click();
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.textContent).toContain('<img'));
        expect(document.querySelector('#terminalOutput img')).toBeNull();
        expect((window as unknown as { __terminalXss?: boolean }).__terminalXss).toBeUndefined();
        expect(calls.map((call) => call.command)).toEqual([
            'native_git_status',
            'native_git_diff',
            'native_enable_terminal',
            'native_confirm_terminal_command',
            'native_run_terminal_command',
        ]);
        expect(calls.at(-1)?.args).toMatchObject({ cwd: '/granted/project', command: 'pwd', confirmedCommand: 'pwd', userConfirmed: true });
    });

    it('chooses a native model folder and lists pinned models without the browser file input', async () => {
        document.body.innerHTML = '<div id="modelSourceMount"></div><input id="modelFileInput"><label for="modelFileInput">Choose model file</label>';
        const invoke = vi.fn(async (command: string) => {
            if (command === 'native_choose_model_folder') return { path: '/granted/models' };
            if (command === 'native_list_local_models') return [{ name: 'synthetic-stack.lut', status_label: 'synthetic stand-in' }];
            throw new Error('unexpected command: ' + command);
        });
        mountDesktopActions({ actions: createDesktopActions({ invoke }) });
        const button = document.getElementById('nativeModelFolderButton') as HTMLButtonElement | null;
        expect(button).not.toBeNull();
        expect(document.querySelector('label[for="modelFileInput"]')).toBeNull();
        button?.click();
        await vi.waitFor(() => expect(document.getElementById('nativeModelFolderStatus')?.textContent).toContain('synthetic-stack.lut'));
        expect(document.getElementById('nativeModelFolderStatus')?.textContent).toContain('does not switch the running sidecar');
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_choose_model_folder', 'native_list_local_models']);
    });

    it('disables saved custom providers and remote model downloads on every Tauri render', async () => {
        vi.resetModules();
        document.body.innerHTML = '<div id="modelSourceMount"></div>';
        (window as unknown as { __TAURI__?: unknown }).__TAURI__ = { core: { invoke: vi.fn(), Channel: class {} } };
        globalThis.fetch = vi.fn(async () => new Response(JSON.stringify({ schema: 'uor-r4.studio-models/1', models: [{ id: 'm1', label: 'M1', kind: 'published', url: './models/m1.lut', sha256: 'a'.repeat(64) }] }), { status: 200 }));
        const { ModelSources } = await import('../../assets/js/model_sources.js');
        ModelSources.addApi({ id: 'custom', label: 'Custom', baseUrl: 'https://api.example.com/v1', model: 'custom', ephemeral: true });
        await ModelSources.loadModelCatalog();
        ModelSources.mount!(document.getElementById('modelSourceMount')!);
        for (const action of ['api-use', 'api-test', 'api-edit', 'add-api', 'load-url', 'load-published']) {
            const button = document.querySelector(`[data-action="${action}"]`) as HTMLButtonElement | null;
            expect(button, action).not.toBeNull();
            expect(button?.disabled, action).toBe(true);
        }
        delete (window as unknown as { __TAURI__?: unknown }).__TAURI__;
    });

    it('keeps an honest disabled state when the host is unavailable', () => {
        controls();
        const actions = createDesktopActions();
        expect(actions.available).toBe(false);
        mountDesktopActions({ actions });
        expect((document.getElementById('btnRefreshChanges') as HTMLButtonElement).disabled).toBe(true);
        expect((document.getElementById('btnRunTerminal') as HTMLButtonElement).disabled).toBe(true);
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('browser build');
    });
});

describe('native folder mapping in the shared editor', () => {
    it('opens, reads and saves full paths through the granted host folder', async () => {
        vi.resetModules();
        document.body.innerHTML = '<div id="fileTree"></div><div id="fileTreeMeta"></div><div id="editorTabs"></div><input id="editorFileName"><textarea id="editorText"></textarea>';
        const calls: Array<{ command: string; args: unknown[] }> = [];
        const record = (command: string, ...args: unknown[]) => { calls.push({ command, args }); };
        (window as unknown as { UORDesktopActions?: unknown }).UORDesktopActions = {
            available: true,
            chooseProjectFolder: async () => { record('chooseProjectFolder'); return { path: '/granted/project' }; },
            listDirectory: async (path: string, depth: number) => { record('listDirectory', path, depth); return ['src/lib.rs']; },
            readFile: async (path: string) => { record('readFile', path); return 'pub fn answer() {}\n'; },
            writeFile: async (path: string, content: string) => { record('writeFile', path, content); return { written: true }; },
        };

        await import(workspaceModule);
        const files = (window as unknown as { __studioFiles: {
            openFolder(): Promise<void>;
            openFile(path: string): Promise<void>;
            saveActive(): Promise<boolean>;
            treeSummary(): { rootPath: string };
        } }).__studioFiles;
        await files.openFolder();
        expect(files.treeSummary().rootPath).toBe('/granted/project');
        await files.openFile('src/lib.rs');
        expect(document.getElementById('editorText')).toHaveProperty('value', 'pub fn answer() {}\n');
        expect(await files.saveActive()).toBe(true);
        expect(calls.map((call) => call.command)).toEqual(['chooseProjectFolder', 'listDirectory', 'readFile', 'writeFile']);
        expect(calls.at(-1)?.args).toEqual(['/granted/project/src/lib.rs', 'pub fn answer() {}\n']);
    });
});
