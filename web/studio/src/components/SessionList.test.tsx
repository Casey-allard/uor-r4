import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { SessionApi, SessionSummary } from '../adapters/workflow.js';
import { SessionList } from './SessionList.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

const summary: SessionSummary = { id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 2, preview: 'Latest message' };

function sessions(items: SessionSummary[] = [summary]): SessionApi {
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'), listSessions: vi.fn(() => items), activeSessionId: vi.fn(() => items[0]?.id ?? null),
        selectSession: vi.fn(() => true), newSession: vi.fn(() => true), renameSession: vi.fn(() => true),
        duplicateSession: vi.fn(() => 'session-copy'), exportSession: vi.fn(() => '{"schema":"uor-r4.studio-session/1"}'),
        deleteSession: vi.fn(() => true), getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function mount(api = sessions()) {
    const store = new LifecycleStore();
    const download = vi.fn();
    const onError = vi.fn();
    act(() => { render(<SessionList store={store} sessions={api} download={download} onError={onError} />, host); });
    return { store, api, download, onError };
}

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('SessionList', () => {
    it('shows an honest empty state', () => {
        mount(sessions([]));
        expect(host.textContent).toContain('No sessions yet');
        expect(button('New session').disabled).toBe(false);
    });

    it('reports rejected new/select actions instead of activating them', () => {
        const { store, api, onError } = mount();
        api.newSession = vi.fn(() => false);
        api.selectSession = vi.fn(() => false);
        act(() => { button('New session').click(); });
        expect(onError).toHaveBeenCalledWith('Could not create a session.', 'session:new');
        act(() => { button('Select Session A').click(); });
        expect(onError).toHaveBeenCalledWith('Could not select that session.', 'session:select:session-a');
        expect(store.getState().session.phase).toBe('new');
    });

    it('reports rejected rename and duplicate operations', () => {
        const { api, onError } = mount();
        api.renameSession = vi.fn(() => false);
        api.duplicateSession = vi.fn(() => null);
        act(() => { button('Duplicate Session A').click(); });
        expect(onError).toHaveBeenCalledWith('Could not duplicate that session.', 'session:duplicate:session-a');
        act(() => { button('Rename Session A').click(); });
        act(() => { button('Save title').click(); });
        expect(onError).toHaveBeenCalledWith('Could not rename that session.', 'session:rename:session-a');
        expect(host.querySelector('dialog')).not.toBeNull();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Could not rename that session.');
    });

    it('reports rejected export and delete operations', () => {
        const { api, onError } = mount();
        api.exportSession = vi.fn(() => null);
        api.deleteSession = vi.fn(() => false);
        act(() => { button('Export Session A').click(); });
        expect(onError).toHaveBeenCalledWith('Session export is unavailable.', 'session:export:session-a');
        act(() => { button('Delete Session A').click(); });
        act(() => { button('Confirm delete').click(); });
        expect(onError).toHaveBeenCalledWith('Could not delete that session. Stop the current reply first.', 'session:delete:session-a');
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Could not delete that session.');
    });

    it('disables session mutations while a reply is generating', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        act(() => { store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } }); });
        expect(button('New session').disabled).toBe(true);
        expect(button('Select Session A').disabled).toBe(true);
        expect(button('Rename Session A').disabled).toBe(true);
        expect(button('Duplicate Session A').disabled).toBe(true);
        expect(button('Delete Session A').disabled).toBe(true);
    });

    it('supports new, select, rename, duplicate, export, and confirmed delete', () => {
        const { store, api, download } = mount();
        act(() => { button('New session').click(); });
        expect(api.newSession).toHaveBeenCalledOnce();
        expect(store.getState().session.phase).toBe('active');
        act(() => { button('Select Session A').click(); });
        expect(api.selectSession).toHaveBeenCalledWith('session-a');
        act(() => { button('Rename Session A').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Session title"]')!;
        act(() => { input.value = 'Renamed session'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { button('Save title').click(); });
        expect(api.renameSession).toHaveBeenCalledWith('session-a', 'Renamed session');
        act(() => { button('Duplicate Session A').click(); });
        expect(api.duplicateSession).toHaveBeenCalledWith('session-a');
        act(() => { button('Export Session A').click(); });
        expect(download).toHaveBeenCalledWith('Session A.json', '{"schema":"uor-r4.studio-session/1"}');
        act(() => { button('Delete Session A').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        act(() => { button('Confirm delete').click(); });
        expect(api.deleteSession).toHaveBeenCalledWith('session-a');
    });
});
