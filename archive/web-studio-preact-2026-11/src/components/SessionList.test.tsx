import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { SessionApi, SessionSummary } from '../adapters/workflow.js';
import { createAgentSessionController } from '../adapters/agentSession.js';
import type { AgentSessionController } from '../adapters/agentSession.js';
import { SessionList } from './SessionList.js';

let host: HTMLDivElement;
const controllers: AgentSessionController[] = [];
beforeEach(() => {
    controllers.length = 0;
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => {
    render(null, host);
    for (const controller of controllers) controller.dispose();
    host.remove();
    vi.restoreAllMocks();
});

const summary: SessionSummary = { id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 2, preview: 'Latest message' };

function sessions(items: SessionSummary[] = [summary]): SessionApi {
    items = items.map((item) => ({ ...item }));
    let activeId = items[0]?.id ?? null;
    let nextId = 0;
    const create = () => {
        nextId += 1;
        items.unshift({ id: `session-new-${nextId}`, title: 'New conversation', createdAt: 3, updatedAt: 3, messageCount: 0, preview: '' });
        activeId = items[0].id;
        return true;
    };
    return {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'),
        listSessions: vi.fn(() => items),
        activeSessionId: vi.fn(() => activeId),
        selectSession: vi.fn((id: string) => {
            if (!items.some((item) => item.id === id)) return false;
            activeId = id;
            return true;
        }),
        newSession: vi.fn(create),
        renameSession: vi.fn((id: string, title: string) => {
            const item = items.find((candidate) => candidate.id === id);
            if (!item) return false;
            item.title = title;
            return true;
        }),
        duplicateSession: vi.fn((id: string) => {
            const item = items.find((candidate) => candidate.id === id);
            if (!item) return null;
            const copy = { ...item, id: `${id}-copy`, title: `${item.title} (copy)` };
            items.unshift(copy);
            return copy.id;
        }),
        exportSession: vi.fn(() => '{"schema":"uor-r4.studio-session/1"}'),
        deleteSession: vi.fn((id: string) => {
            const index = items.findIndex((item) => item.id === id);
            if (index < 0) return false;
            const wasActive = activeId === id;
            items.splice(index, 1);
            if (wasActive) create();
            return true;
        }),
        getConversationMessages: vi.fn(() => []), setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => 1), predict: vi.fn(() => 1), stopGeneration: vi.fn(),
    };
}

function mount(api = sessions()) {
    const store = new LifecycleStore();
    const agent = createAgentSessionController(store, api);
    controllers.push(agent);
    const download = vi.fn();
    const onError = vi.fn();
    act(() => { render(<SessionList store={store} agent={agent} download={download} onError={onError} />, host); });
    return { store, agent, api, download, onError };
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

    it('reports rejected new/select actions without changing verified state', () => {
        const { store, api, onError } = mount();
        const before = store.getState().session;
        api.newSession = vi.fn(() => false);
        api.selectSession = vi.fn(() => false);
        act(() => { button('New session').click(); });
        expect(onError).toHaveBeenCalledWith('Could not create a session.', 'session:new');
        act(() => { button('Select Session A').click(); });
        expect(onError).toHaveBeenCalledWith('Could not select that session.', 'session:select:session-a');
        expect(store.getState().session).toEqual(before);
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
        expect(onError).toHaveBeenCalledWith('Could not delete that session or verify its result.', 'session:delete:session-a');
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Could not delete that session or verify its result.');
    });

    it('disables session mutations while a reply is generating', () => {
        const { store } = mount();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        act(() => { store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' }, sessionId: 'session-a' }); });
        expect(button('New session').disabled).toBe(true);
        expect(button('Select Session A').disabled).toBe(true);
        expect(button('Rename Session A').disabled).toBe(true);
        expect(button('Duplicate Session A').disabled).toBe(true);
        expect(button('Delete Session A').disabled).toBe(true);
    });

    it('marks the verified active session without adding interactive row roles', () => {
        const { store } = mount();
        const active = host.querySelector('li[aria-current="true"]');
        expect(active?.querySelector('strong')?.textContent).toBe('Session A');
        expect(active?.getAttribute('role')).toBeNull();
        act(() => { button('New session').click(); });
        expect(host.querySelector('li[aria-current="true"] strong')?.textContent).toBe('New conversation');
        expect(store.getState().session.activeId).toBe('session-new-1');
    });

    it('supports new, select, rename, duplicate, export, and confirmed delete', () => {
        const { store, api, download } = mount();
        act(() => { button('New session').click(); });
        expect(api.newSession).toHaveBeenCalledOnce();
        expect(store.getState().session.phase).toBe('active');
        act(() => { button('Select Session A').click(); });
        expect(api.selectSession).toHaveBeenCalledWith('session-a');
        expect(store.getState().session.activeId).toBe('session-a');
        act(() => { button('Rename Session A').click(); });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Session title"]')!;
        expect(input.maxLength).toBe(80);
        act(() => { input.value = 'Renamed session'; input.dispatchEvent(new Event('input', { bubbles: true })); });
        act(() => { button('Save title').click(); });
        expect(api.renameSession).toHaveBeenCalledWith('session-a', 'Renamed session');
        act(() => { button('Duplicate Renamed session').click(); });
        expect(api.duplicateSession).toHaveBeenCalledWith('session-a');
        act(() => { button('Export Renamed session').click(); });
        expect(download).toHaveBeenCalledWith('Renamed session.json', '{"schema":"uor-r4.studio-session/1"}');
        act(() => { button('Delete Renamed session').click(); });
        expect(host.querySelector('dialog')).not.toBeNull();
        act(() => { button('Confirm delete').click(); });
        expect(api.deleteSession).toHaveBeenCalledWith('session-a');
        expect(host.querySelector('dialog')).toBeNull();
        expect(host.querySelector('[role="alert"]')).toBeNull();
        expect(store.getState().session.activeId).toBe('session-new-2');
        const replacement = host.querySelector('li[data-session-id="session-new-2"][aria-current="true"]');
        expect(replacement).not.toBeNull();
        expect(replacement?.querySelector('strong')?.textContent).toBe('New conversation');
        expect(host.querySelector('li[data-session-id="session-new-1"]')?.getAttribute('aria-current')).toBeNull();
    });
});


describe('replacement-row mutation evidence', () => {
    it('rejects the wrong active-row marker instead of accepting title-only evidence', () => {
        const { store } = mount();
        act(() => { button('New session').click(); });
        act(() => { button('Select Session A').click(); });
        act(() => { button('Delete Session A').click(); });
        act(() => { button('Confirm delete').click(); });
        expect(store.getState().session.activeId).toBe('session-new-2');
        const oldRow = host.querySelector('li[data-session-id="session-new-1"]')!;
        oldRow.setAttribute('aria-current', 'true');
        const replacement = host.querySelector('li[data-session-id="session-new-2"]')!;
        replacement.setAttribute('aria-current', 'false');
        expect(host.querySelector('li[data-session-id="session-new-2"][aria-current="true"]')).toBeNull();
        expect(oldRow.querySelector('strong')?.textContent).toBe('New conversation');
    });
});


describe('generation-specific delete refusal', () => {
    it('requires stopping the current reply when generation starts after the modal opens', () => {
        const { store, api, onError } = mount();
        act(() => { button('Delete Session A').click(); });
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        act(() => { store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' }, sessionId: 'session-a' }); });
        act(() => { button('Confirm delete').click(); });
        expect(onError).toHaveBeenCalledWith('Could not delete that session. Stop the current reply first.', 'session:delete:session-a');
        expect(api.deleteSession).not.toHaveBeenCalled();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Stop the current reply first.');
    });
});
