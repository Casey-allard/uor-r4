import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createDesktopActions } from '../adapters/desktopActions.js';
import { makeFiles } from './studioTestFixtures.js';
import { ChangesPanel, TerminalPanel } from './DesktopTools.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function workspace(rootPath = '') {
    const files = makeFiles();
    const summary = files.treeSummary();
    files.treeSummary = vi.fn(() => ({ ...summary, rootPath }));
    return files;
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

function input(label: string, value: string) {
    const field = host.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
    act(() => { field.value = value; field.dispatchEvent(new Event('input', { bubbles: true })); });
}

describe('ChangesPanel', () => {
    it('refreshes through the real adapter, renders inert bounded results, and copies the exact raw diff', async () => {
        const rawDiff = 'diff --git a/a.rs b/a.rs\n+<img src=x onerror=window.__gitXss=1>';
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_git_status') return { branch: 'main', is_clean: false, modified_files: ['a.rs'], untracked_files: [], truncated: false, timed_out: false };
            if (command === 'native_git_diff') return { diff: rawDiff, truncated: true, timed_out: false };
            throw new Error(`unexpected ${command}`);
        });
        const copy = vi.fn();
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} copy={copy} />, host); });
        expect(invoke).not.toHaveBeenCalled();
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('gitDiffOutput')?.textContent).toContain('<img'));
        expect(document.getElementById('gitStatusSummary')?.textContent).toContain('main · 1 changed file · truncated');
        expect(document.querySelector('#gitDiffOutput img')).toBeNull();
        expect((window as unknown as { __gitXss?: boolean }).__gitXss).toBeUndefined();
        await act(async () => { button('Copy diff').click(); await Promise.resolve(); });
        expect(copy).toHaveBeenCalledWith(rawDiff);
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_git_status', 'native_git_diff']);
        expect(invoke.mock.calls.every((call) => (call[1] as { repoPath: string }).repoPath === '/granted/project')).toBe(true);
    });

    it('shows host failures and disables actions when native Git is unavailable', async () => {
        const invoke = vi.fn(async () => { throw new Error('host failed'); });
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} copy={vi.fn()} />, host); });
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('gitStatusSummary')?.textContent).toContain('Git inspection failed: host failed'));
        expect((document.getElementById('btnCopyDiff') as HTMLButtonElement).disabled).toBe(true);
        act(() => { render(null, host); });
        const actions = createDesktopActions();
        const gitStatus = vi.spyOn(actions, 'gitStatus');
        act(() => { render(<ChangesPanel actions={actions} files={workspace()} copy={vi.fn()} />, host); });
        expect((document.getElementById('btnRefreshChanges') as HTMLButtonElement).disabled).toBe(true);
        expect(document.getElementById('gitStatusSummary')?.textContent).toContain('unavailable in this browser build');
        act(() => { (document.getElementById('btnRefreshChanges') as HTMLButtonElement).click(); });
        expect(gitStatus).not.toHaveBeenCalled();
    });

    it('ignores a stale refresh after the workspace changes and uses the latest repository', async () => {
        let resolveStatus!: () => void;
        let resolveDiff!: () => void;
        let statusCalls = 0;
        let diffCalls = 0;
        const invoke = vi.fn(async (command: string, args: { repoPath: string }) => {
            if (command === 'native_git_status') {
                statusCalls += 1;
                if (statusCalls === 1) return new Promise<unknown>((resolve) => { resolveStatus = () => resolve({ branch: 'old', modified_files: [], untracked_files: [] }); });
                return { branch: 'new', modified_files: [], untracked_files: [] };
            }
            if (command === 'native_git_diff') {
                diffCalls += 1;
                if (diffCalls === 1) return new Promise<unknown>((resolve) => { resolveDiff = () => resolve({ diff: 'old diff', truncated: false, timed_out: false }); });
                return { diff: 'new diff', truncated: false, timed_out: false };
            }
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} copy={vi.fn()} />, host); });
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { resolveStatus(); resolveDiff(); await Promise.resolve(); });
        expect(document.getElementById('gitDiffOutput')?.textContent).not.toContain('old diff');
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('gitDiffOutput')?.textContent).toContain('new diff'));
        expect(invoke.mock.calls.at(-1)?.[1].repoPath).toBe('/new/project');
    });
});

describe('TerminalPanel', () => {
    it('enables explicitly, approves every command, and renders inert output', async () => {
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: true };
            if (command === 'native_run_terminal_command') return '<img src=x onerror=window.__terminalXss=1>';
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        expect(button('Run command').disabled).toBe(true);
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalConsentState')?.textContent).toContain('enabled'));
        input('Terminal command', 'pwd');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.textContent).toContain('<img'));
        expect(document.querySelector('#terminalOutput img')).toBeNull();
        expect((window as unknown as { __terminalXss?: boolean }).__terminalXss).toBeUndefined();
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_enable_terminal', 'native_confirm_terminal_command', 'native_run_terminal_command']);
        expect(invoke.mock.calls.at(-1)?.[1]).toMatchObject({ cwd: '/granted/project', command: 'pwd', confirmedCommand: 'pwd', userConfirmed: true });
    });

    it('reports cancelled approval without running the command', async () => {
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: false };
            if (command === 'native_run_terminal_command') throw new Error('must not run');
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Terminal command', 'pwd');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.textContent).toContain('cancelled'));
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_enable_terminal', 'native_confirm_terminal_command']);
    });

    it('disables guarded actions when the native host is unavailable', () => {
        act(() => { render(<TerminalPanel actions={createDesktopActions()} files={workspace()} />, host); });
        expect(button('Enable terminal').disabled).toBe(true);
        expect(button('Run command').disabled).toBe(true);
        expect(document.getElementById('terminalConsentState')?.textContent).toContain('browser build');
    });

    it('ignores a stale command result after the workspace changes and runs only the latest path', async () => {
        let resolveOutput!: () => void;
        let runCalls = 0;
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: true };
            if (command === 'native_run_terminal_command') {
                runCalls += 1;
                if (runCalls === 1) return new Promise<unknown>((resolve) => { resolveOutput = () => resolve('old output'); });
                return 'new output';
            }
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Terminal command', 'pwd');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '/new/project' } })); });
        await act(async () => { resolveOutput(); await Promise.resolve(); });
        expect(document.getElementById('terminalOutput')?.textContent).not.toContain('old output');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.textContent).toContain('new output'));
        expect(invoke.mock.calls.at(-1)?.[1]).toMatchObject({ cwd: '/new/project', command: 'pwd' });
    });

    it('prevents Enable during Run and restores both actions after completion', async () => {
        let resolveRun!: () => void;
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: true };
            if (command === 'native_run_terminal_command') return new Promise<unknown>((resolve) => { resolveRun = () => resolve('done'); });
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Terminal command', 'pwd');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        expect(button('Enable terminal').disabled).toBe(true);
        await act(async () => { resolveRun(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('terminalOutput')?.textContent).toContain('done'));
        expect(button('Enable terminal').disabled).toBe(false);
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        expect(button('Run command').disabled).toBe(false);
    });

    it('prevents Run during Enable and clears the enabling state on completion', async () => {
        let resolveEnable!: () => void;
        let enableCalls = 0;
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') {
                enableCalls += 1;
                if (enableCalls === 2) return new Promise<unknown>((resolve) => { resolveEnable = () => resolve({ enabled: true }); });
                return { enabled: true };
            }
            if (command === 'native_confirm_terminal_command') return { approved: true };
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Terminal command', 'pwd');
        expect(button('Run command').disabled).toBe(true);
        await act(async () => { resolveEnable(); await Promise.resolve(); });
        expect(button('Run command').disabled).toBe(false);
    });

    it('invalidates a stale refresh when the workspace rootPath is cleared', async () => {
        let resolveStatus!: () => void;
        let resolveDiff!: () => void;
        const invoke = vi.fn(async (command: string) => {
            if (command === 'native_git_status') return new Promise<unknown>((resolve) => { resolveStatus = () => resolve({ branch: 'old', modified_files: [], untracked_files: [] }); });
            if (command === 'native_git_diff') return new Promise<unknown>((resolve) => { resolveDiff = () => resolve({ diff: 'old diff', timed_out: false }); });
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} copy={vi.fn()} />, host); });
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '' } })); });
        await act(async () => { resolveStatus(); resolveDiff(); await Promise.resolve(); });
        expect(document.getElementById('gitDiffOutput')?.textContent).not.toContain('old diff');
        expect((document.getElementById('terminalCwd') as HTMLInputElement).value).toBe('');
        expect(button('Refresh changes').disabled).toBe(true);
    });

    it('invalidates a stale command result when the workspace rootPath is cleared', async () => {
        let resolveOutput!: () => void;
        const invoke = vi.fn(async (command: string, _args: Record<string, unknown> = {}) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            if (command === 'native_confirm_terminal_command') return { approved: true };
            if (command === 'native_run_terminal_command') return new Promise<unknown>((resolve) => { resolveOutput = () => resolve('old output'); });
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Terminal command', 'pwd');
        await act(async () => { button('Run command').click(); await Promise.resolve(); });
        act(() => { document.dispatchEvent(new CustomEvent('uor-files-changed', { detail: { rootPath: '' } })); });
        await act(async () => { resolveOutput(); await Promise.resolve(); });
        expect(document.getElementById('terminalOutput')?.textContent).not.toContain('old output');
        expect((document.getElementById('terminalCwd') as HTMLInputElement).value).toBe('');
        expect(button('Run command').disabled).toBe(true);
    });

    it('explains empty working directory and command states without running', async () => {
        const invoke = vi.fn(async (command: string) => {
            if (command === 'native_enable_terminal') return { enabled: true };
            throw new Error(`unexpected ${command}`);
        });
        act(() => { render(<TerminalPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} />, host); });
        await act(async () => { button('Enable terminal').click(); await Promise.resolve(); });
        input('Working directory', '');
        input('Terminal command', 'pwd');
        expect(document.getElementById('terminalOutput')?.textContent).toContain('Choose a project folder and enter an allowed command.');
        expect(button('Run command').disabled).toBe(true);
        input('Working directory', '/granted/project');
        input('Terminal command', '');
        expect(document.getElementById('terminalOutput')?.textContent).toContain('Enter an allowed command.');
        expect(button('Run command').disabled).toBe(true);
        expect(invoke.mock.calls.map((call) => call[0])).toEqual(['native_enable_terminal']);
    });

    it('reports timeout states without changing the raw diff', async () => {
        const rawDiff = 'diff --git a/a.rs b/a.rs\n+change';
        const invoke = vi.fn(async (command: string) => {
            if (command === 'native_git_status') return { branch: 'main', modified_files: [], untracked_files: [], timed_out: true };
            if (command === 'native_git_diff') return { diff: rawDiff, truncated: false, timed_out: true };
            throw new Error(`unexpected ${command}`);
        });
        const copy = vi.fn();
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/granted/project')} copy={copy} />, host); });
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        await vi.waitFor(() => expect(document.getElementById('gitStatusSummary')?.textContent).toContain('timed out'));
        expect(document.getElementById('gitDiffOutput')?.textContent).toContain('[diff timed out]');
        await act(async () => { button('Copy diff').click(); await Promise.resolve(); });
        expect(copy).toHaveBeenCalledWith(rawDiff);
    });

    it('ignores completion after unmount', async () => {
        let resolveStatus!: () => void;
        let resolveDiff!: () => void;
        const invoke = vi.fn(async (command: string) => {
            if (command === 'native_git_status') return new Promise<unknown>((resolve) => { resolveStatus = () => resolve({ branch: 'old', modified_files: [], untracked_files: [] }); });
            if (command === 'native_git_diff') return new Promise<unknown>((resolve) => { resolveDiff = () => resolve({ diff: 'old diff', timed_out: false }); });
            throw new Error(`unexpected ${command}`);
        });
        const errors = vi.spyOn(console, 'error').mockImplementation(() => {});
        act(() => { render(<ChangesPanel actions={createDesktopActions({ invoke })} files={workspace('/old/project')} copy={vi.fn()} />, host); });
        await act(async () => { button('Refresh changes').click(); await Promise.resolve(); });
        act(() => { render(null, host); });
        await act(async () => { resolveStatus(); resolveDiff(); await Promise.resolve(); });
        expect(errors).not.toHaveBeenCalled();
    });
});
