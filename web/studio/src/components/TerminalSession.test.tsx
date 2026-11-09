import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from 'vitest';
import { TerminalPanel } from './DesktopTools.js';
import { createDesktopActions } from '../adapters/desktopActions.js';
import { makeFiles } from './studioTestFixtures.js';

function workspace(rootPath = '') {
    const files = makeFiles();
    const summary = files.treeSummary();
    files.treeSummary = vi.fn(() => ({ ...summary, rootPath }));
    return files;
}

const { terminals } = vi.hoisted(() => ({ terminals: [] as Record<string, Mock>[] }));
vi.mock('@xterm/xterm', () => ({
    Terminal: class {
        options = {};
        open = vi.fn();
        write = vi.fn();
        onData = vi.fn();
        onResize = vi.fn();
        loadAddon = vi.fn();
        focus = vi.fn();
        dispose = vi.fn();
        resize = vi.fn();
        constructor() { terminals.push(this as unknown as Record<string, Mock>); }
    },
}));
vi.mock('@xterm/addon-fit', () => ({ FitAddon: class { fit = vi.fn(); } }));

let host: HTMLElement;
const button = (label: string) => Array.from(host.querySelectorAll('button')).find((element) => element.textContent === label) as HTMLButtonElement;
const hex = (text: string) => Array.from(new TextEncoder().encode(text), (byte) => byte.toString(16).padStart(2, '0')).join('');

function nativeMock(overrides: Record<string, (args: Record<string, unknown>) => unknown> = {}) {
    return vi.fn(async (command: string, args: Record<string, unknown> = {}) => {
        if (overrides[command]) return overrides[command](args);
        if (command === 'native_enable_terminal') return { enabled: true };
        if (command === 'native_open_terminal') return { sessionId: 'session-a', cursor: 0, active: true, events: [] };
        if (command === 'native_read_terminal') return { sessionId: 'session-a', cursor: 1, active: true, events: [{ type: 'output', sessionId: 'session-a', sequence: 1, dataHex: hex('<img src=x onerror=window.__terminalXss=1>') }] };
        if (command === 'native_terminal_input' || command === 'native_resize_terminal' || command === 'native_close_terminal') return { active: true };
        throw new Error(`unexpected ${command}`);
    });
}

async function openTerminal() {
    await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
    await vi.waitFor(() => expect(terminals.length).toBe(1));
}

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    terminals.length = 0;
    delete (window as unknown as { __terminalXss?: boolean }).__terminalXss;
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
});

describe('interactive PTY terminal', () => {
    it('streams raw bytes and sends direct input without a Run command or another approval', async () => {
        const invoke = nativeMock();
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        expect(button('Run command')).toBeUndefined();
        await openTerminal();
        await vi.waitFor(() => expect(terminals[0].write.mock.calls.length).toBeGreaterThan(0));
        const raw = terminals[0].write.mock.calls[0][0] as Uint8Array;
        expect(new TextDecoder().decode(raw)).toContain('<img');
        expect(host.querySelector('img')).toBeNull();
        expect((window as unknown as { __terminalXss?: boolean }).__terminalXss).toBeUndefined();
        terminals[0].onData.mock.calls[0][0]('printf ready\\n');
        expect(invoke).toHaveBeenCalledWith('native_terminal_input', { sessionId: 'session-a', data: 'printf ready\\n' });
        expect(invoke).not.toHaveBeenCalledWith('native_run_terminal_command', expect.anything());
        expect(invoke).not.toHaveBeenCalledWith('native_confirm_terminal_command', expect.anything());
        terminals[0].onResize.mock.calls[0][0]({ cols: 100, rows: 40 });
        expect(invoke).toHaveBeenCalledWith('native_resize_terminal', { sessionId: 'session-a', cols: 100, rows: 40 });
    });

    it('reports a cancelled shell opening without creating a terminal', async () => {
        const invoke = nativeMock({ native_open_terminal: () => { throw new Error('TerminalConsentRequired'); } });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toMatch(/cancelled|approval/i));
        expect(terminals).toHaveLength(0);
        expect(invoke).not.toHaveBeenCalledWith('native_run_terminal_command', expect.anything());
    });

    it('disables opening when native PTY support is unavailable', () => {
        act(() => { render(<TerminalPanel actions={createDesktopActions()} files={workspace()} />, host); });
        expect(button('Open terminal').disabled).toBe(true);
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('browser build');
    });

    it('closes the PTY and rejects later input on close and unmount', async () => {
        const invoke = nativeMock();
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        act(() => { button('Close shell').click(); });
        expect(invoke).toHaveBeenCalledWith('native_close_terminal', { sessionId: 'session-a' });
        const writes = invoke.mock.calls.filter(([name]) => name === 'native_terminal_input').length;
        terminals[0].onData.mock.calls[0][0]('exit\n');
        expect(invoke.mock.calls.filter(([name]) => name === 'native_terminal_input')).toHaveLength(writes);
        act(() => { render(null, host); });
        expect(invoke).toHaveBeenCalledWith('native_close_terminal', { sessionId: 'session-a' });
    });

    it('closes the old session on workspace change and uses the new root for the next shell', async () => {
        const invoke = nativeMock({ native_open_terminal: (args) => ({ sessionId: String(args.cwd), cursor: 0, active: true, events: [] }) });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await openTerminal();
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        expect(invoke).toHaveBeenCalledWith('native_close_terminal', { sessionId: '/old/project' });
        await vi.waitFor(() => expect(button('Open terminal').disabled).toBe(false));
        act(() => { button('Open terminal').click(); });
        await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('native_open_terminal', expect.objectContaining({ cwd: '/new/project' })));
    });

    it('closes a shell whose opening completes after the workspace changes', async () => {
        let resolveOpen!: (value: unknown) => void;
        const invoke = nativeMock({ native_open_terminal: () => new Promise((resolve) => { resolveOpen = resolve; }) });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { resolveOpen({ sessionId: 'stale-shell', cursor: 0, active: true, events: [] }); await Promise.resolve(); });
        expect(invoke).toHaveBeenCalledWith('native_close_terminal', { sessionId: 'stale-shell' });
        expect(terminals).toHaveLength(0);
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined();
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('Workspace changed');
    });

    it('drops a stale read after workspace change', async () => {
        let resolveRead!: (value: unknown) => void;
        const invoke = nativeMock({ native_read_terminal: () => new Promise((resolve) => { resolveRead = resolve; }) });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await openTerminal();
        await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('native_read_terminal', { sessionId: 'session-a', after: 0 }));
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { resolveRead({ sessionId: 'session-a', cursor: 1, active: true, events: [{ type: 'output', sessionId: 'session-a', sequence: 1, dataHex: hex('stale') }] }); await Promise.resolve(); });
        expect(terminals[0].write).not.toHaveBeenCalled();
    });

    it('drops a stale read after the workspace root is cleared', async () => {
        let resolveRead!: (value: unknown) => void;
        const invoke = nativeMock({ native_read_terminal: () => new Promise((resolve) => { resolveRead = resolve; }) });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await openTerminal();
        await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('native_read_terminal', { sessionId: 'session-a', after: 0 }));
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '' } })); });
        await act(async () => { resolveRead({ sessionId: 'session-a', cursor: 1, active: true, events: [{ type: 'output', sessionId: 'session-a', sequence: 1, dataHex: hex('stale') }] }); await Promise.resolve(); });
        expect(terminals[0].write).not.toHaveBeenCalled();
        expect((document.getElementById('terminalCwd') as HTMLInputElement).value).toBe('');
        await vi.waitFor(() => expect(button('Open terminal').disabled).toBe(false));
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('Choose a project folder');
        expect(invoke.mock.calls.filter(([name]) => name === 'native_open_terminal')).toHaveLength(1);
    });

    it('opens the first shell after empty initial cleanup settles during enablement', async () => {
        let resolveEnable!: (value: unknown) => void;
        const invoke = nativeMock({ native_enable_terminal: () => new Promise((resolve) => { resolveEnable = resolve; }) });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        await act(async () => { resolveEnable({ enabled: true }); await Promise.resolve(); });
        expect(invoke).toHaveBeenCalledWith('native_open_terminal', expect.objectContaining({ cwd: '/granted/project' }));
        await vi.waitFor(() => expect(terminals.length).toBe(1));
    });

    it('closes a shell whose opening completes after the panel unmounts', async () => {
        let resolveOpen!: (value: unknown) => void;
        const close = vi.fn(async () => ({ active: false }));
        const invoke = nativeMock({
            native_open_terminal: () => new Promise((resolve) => { resolveOpen = resolve; }),
            native_close_terminal: close,
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        act(() => { render(null, host); });
        await act(async () => { resolveOpen({ sessionId: 'unmounted-shell', cursor: 0, active: true, events: [] }); await Promise.resolve(); });
        expect(close).toHaveBeenCalledWith({ sessionId: 'unmounted-shell' });
        expect(terminals).toHaveLength(0);
    });

    it('retains the session and visible error when close fails, then permits retry', async () => {
        let failures = 0;
        const close = vi.fn(async () => {
            failures += 1;
            if (failures === 1) throw new Error('reap failed');
            return { active: false };
        });
        const invoke = nativeMock({ native_close_terminal: close });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        act(() => { button('Close shell').click(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('reap failed'));
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBe('session-a');
        expect(button('Close shell').disabled).toBe(false);
        act(() => { button('Close shell').click(); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(2));
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined());
    });

    it('closes an opened shell when unmount happens before the session effect', async () => {
        let resolveOpen!: (value: unknown) => void;
        const close = vi.fn(async () => ({ active: false }));
        const invoke = nativeMock({
            native_open_terminal: () => new Promise((resolve) => { resolveOpen = resolve; }),
            native_close_terminal: close,
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        await act(async () => {
            resolveOpen({ sessionId: 'pre-effect-shell', cursor: 0, active: true, events: [] });
            await Promise.resolve();
            await Promise.resolve();
            render(null, host);
        });
        await vi.waitFor(() => expect(close).toHaveBeenCalledWith({ sessionId: 'pre-effect-shell' }));
        expect(terminals).toHaveLength(0);
    });

    it('retains a rejected stale close for retry without mounting it', async () => {
        let resolveOpen!: (value: unknown) => void;
        let attempts = 0;
        const close = vi.fn(async () => {
            attempts += 1;
            if (attempts === 1) throw new Error('stale reap failed');
            return { active: false };
        });
        const invoke = nativeMock({
            native_open_terminal: () => new Promise((resolve) => { resolveOpen = resolve; }),
            native_close_terminal: close,
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { resolveOpen({ sessionId: 'stale-close-failure', cursor: 0, active: true, events: [] }); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('stale reap failed'));
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined();
        expect(terminals).toHaveLength(0);
        expect(invoke).not.toHaveBeenCalledWith('native_read_terminal', expect.anything());
        act(() => { button('Retry close').click(); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(2));
        expect(button('Open terminal')).toBeDefined();
        expect(terminals).toHaveLength(0);
    });

    it('does not restore an old session when its close fails after a workspace change', async () => {
        let rejectClose!: (error: Error) => void;
        let attempts = 0;
        const close = vi.fn(() => {
            attempts += 1;
            if (attempts === 1) return new Promise((_resolve, reject) => { rejectClose = reject; });
            return Promise.resolve({ active: false });
        });
        const invoke = nativeMock({ native_close_terminal: close });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await openTerminal();
        act(() => { button('Close shell').click(); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(1));
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { rejectClose(new Error('reap after workspace change')); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('reap after workspace change'));
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined();
        expect(document.getElementById('terminalConsentState')?.textContent).not.toContain('Interactive PTY ·');
        expect(terminals).toHaveLength(1);
        act(() => { button('Retry close').click(); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(2));
        expect(button('Open terminal')).toBeDefined();
        expect(terminals).toHaveLength(1);
    });

    it('never restores an old workspace when close and retry both reject', async () => {
        let attempts = 0;
        const close = vi.fn(async () => {
            attempts += 1;
            if (attempts < 3) throw new Error(`workspace close failure ${attempts}`);
            return { active: false };
        });
        const invoke = nativeMock({ native_close_terminal: close });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await openTerminal();
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('workspace close failure 1'));
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined();
        expect(button('Retry close')).toBeDefined();
        act(() => { button('Retry close').click(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('workspace close failure 2'));
        expect(document.getElementById('terminalOutput')?.dataset.sessionId).toBeUndefined();
        expect(button('Retry close')).toBeDefined();
        act(() => { button('Retry close').click(); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(3));
        expect(button('Open terminal')).toBeDefined();
        expect(terminals).toHaveLength(1);
    });

    it('retries a rejected close once after unmount', async () => {
        let attempts = 0;
        const close = vi.fn(async () => {
            attempts += 1;
            if (attempts === 1) throw new Error('first unmount close failed');
            return { active: false };
        });
        const invoke = nativeMock({ native_close_terminal: close });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        act(() => { render(null, host); });
        await vi.waitFor(() => expect(close).toHaveBeenCalledTimes(2));
        expect(close).toHaveBeenNthCalledWith(1, { sessionId: 'session-a' });
        expect(close).toHaveBeenNthCalledWith(2, { sessionId: 'session-a' });
        expect(terminals).toHaveLength(1);
    });

    it('drops stale read input and resize completions after a new shell opens', async () => {
        let sequence = 0;
        let rejectRead!: (error: Error) => void;
        let rejectInput!: (error: Error) => void;
        let rejectResize!: (error: Error) => void;
        const invoke = nativeMock({
            native_open_terminal: () => { sequence += 1; return { sessionId: `session-${sequence}`, cursor: 0, active: true, events: [] }; },
            native_read_terminal: (args) => args.sessionId === 'session-1'
                ? new Promise((_resolve, reject) => { rejectRead = reject; })
                : { sessionId: 'session-2', cursor: 1, active: true, events: [] },
            native_terminal_input: () => new Promise((_resolve, reject) => { rejectInput = reject; }),
            native_resize_terminal: () => new Promise((_resolve, reject) => { rejectResize = reject; }),
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('native_read_terminal', { sessionId: 'session-1', after: 0 }));
        terminals[0].onData.mock.calls[0][0]('printf stale\n');
        terminals[0].onResize.mock.calls[0][0]({ cols: 100, rows: 40 });
        act(() => { button('Close shell').click(); });
        await vi.waitFor(() => expect(button('Open terminal').disabled).toBe(false));
        act(() => { button('Open terminal').click(); });
        await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('native_read_terminal', { sessionId: 'session-2', after: 0 }));
        await act(async () => {
            rejectRead(new Error('old read failed'));
            rejectInput(new Error('old input failed'));
            rejectResize(new Error('old resize failed'));
            await Promise.resolve();
        });
        const status = document.getElementById('terminalConsentState')?.textContent || '';
        expect(status).not.toContain('old read failed');
        expect(status).not.toContain('old input failed');
        expect(status).not.toContain('old resize failed');
    });

    it('resets the truncation notice for a new shell', async () => {
        let sequence = 0;
        const notices: string[] = [];
        const observer = new MutationObserver(() => {
            const text = document.getElementById('terminalConsentState')?.textContent || '';
            if (text && notices.at(-1) !== text) notices.push(text);
        });
        const truncationNotices = () => notices.filter((text) => text.includes('capped at 1 MiB'));
        observer.observe(host, { childList: true, characterData: true, subtree: true });
        const invoke = nativeMock({
            native_open_terminal: () => { sequence += 1; return { sessionId: `session-${sequence}`, cursor: 0, active: true, events: [] }; },
            native_read_terminal: (args) => ({ sessionId: String(args.sessionId), cursor: 1, active: true, truncated: true, events: [] }),
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        await vi.waitFor(() => expect(truncationNotices()).toHaveLength(1));
        act(() => { button('Close shell').click(); });
        await vi.waitFor(() => expect(button('Open terminal').disabled).toBe(false));
        act(() => { button('Open terminal').click(); });
        await vi.waitFor(() => expect(sequence).toBe(2));
        await vi.waitFor(() => expect(truncationNotices()).toHaveLength(2));
        observer.disconnect();
    });

    it('renders the first output only after xterm has mounted', async () => {
        let reads = 0;
        const invoke = nativeMock({
            native_read_terminal: () => {
                reads += 1;
                return reads === 1
                    ? { sessionId: 'session-a', cursor: 1, active: true, events: [{ type: 'output', sessionId: 'session-a', sequence: 1, dataHex: hex('first-output') }] }
                    : { sessionId: 'session-a', cursor: 1, active: true, events: [] };
            },
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        await vi.waitFor(() => expect(terminals[0].write.mock.calls.length).toBeGreaterThan(0));
        const first = terminals[0].write.mock.calls[0][0] as Uint8Array;
        expect(new TextDecoder().decode(first)).toBe('first-output');
    });

    it('does not open a shell without a granted working directory', async () => {
        const invoke = nativeMock();
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace()} />, host); });
        await act(async () => { button('Open terminal').click(); await Promise.resolve(); });
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('Choose a project folder');
        expect(invoke).not.toHaveBeenCalledWith('native_open_terminal', expect.anything());
        expect(terminals).toHaveLength(0);
    });

    it('reports open and input failures without inventing terminal output', async () => {
        const invoke = nativeMock({ native_terminal_input: () => { throw new Error('session closed'); } });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await openTerminal();
        terminals[0].onData.mock.calls[0][0]('exit\n');
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('session closed'));
        expect(invoke).not.toHaveBeenCalledWith('native_run_terminal_command', expect.anything());
    });
});
