import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createDesktopActions } from '../adapters/desktopActions.js';
import { makeFiles } from './studioTestFixtures.js';
import { ChangesPanel } from './DesktopTools.js';

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
        expect(document.getElementById('gitStatusSummary')?.textContent).toContain('Git requires the local companion. Uploaded files can still be reviewed locally.');
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

describe('ChangesPanel timeouts and cleanup', () => {
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
