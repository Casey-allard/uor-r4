import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { SessionSummary } from '../state/lifecycle.js';
import type { SessionApi } from './workflow.js';
import { createAgentSessionController } from './agentSession.js';
import { GENERATION_FINISHED_EVENT } from './generation.js';

const first: SessionSummary = { id: 'session-a', title: 'Session A', createdAt: 1, updatedAt: 2, messageCount: 1, preview: 'first' };
const second: SessionSummary = { id: 'session-b', title: 'Session B', createdAt: 3, updatedAt: 4, messageCount: 0, preview: '' };

interface Fixture {
    api: SessionApi;
    items: SessionSummary[];
    activeId: string | null;
    nextId: number;
}

function fixture(items = [first, second], activeId: string | null = 'session-a'): Fixture {
    const state: Fixture = {
        items: items.map((item) => ({ ...item })), activeId, nextId: 0,
        api: undefined as unknown as SessionApi,
    };
    state.api = {
        getMode: vi.fn((): 'wasm' | 'api' => 'api'),
        listSessions: vi.fn(() => state.items),
        activeSessionId: vi.fn(() => state.activeId),
        selectSession: vi.fn((id: string) => { state.activeId = id; return true; }),
        newSession: vi.fn(() => {
            state.nextId += 1;
            const item = { id: `session-new-${state.nextId}`, title: 'New conversation', createdAt: 10, updatedAt: 10, messageCount: 0, preview: '' };
            state.items.unshift(item);
            state.activeId = item.id;
            return true;
        }),
        renameSession: vi.fn((id: string, title: string) => {
            const item = state.items.find((candidate) => candidate.id === id);
            if (!item) return false;
            item.title = title;
            return true;
        }),
        duplicateSession: vi.fn((id: string) => {
            const item = state.items.find((candidate) => candidate.id === id);
            if (!item) return null;
            const copy = { ...item, id: `${id}-copy`, title: `${item.title} (copy)` };
            state.items.unshift(copy);
            return copy.id;
        }),
        exportSession: vi.fn((id?: string) => id ? JSON.stringify({ session: { id } }) : null),
        deleteSession: vi.fn((id: string) => {
            const wasActive = state.activeId === id;
            state.items = state.items.filter((item) => item.id !== id);
            if (wasActive) state.api.newSession();
            return true;
        }),
        getConversationMessages: vi.fn(() => []),
        setComposerText: vi.fn(() => true),
        sendPrompt: vi.fn(() => 42),
        predict: vi.fn(() => 42),
        stopGeneration: vi.fn(),
    };
    return state;
}

const controllers: Array<ReturnType<typeof createAgentSessionController>> = [];
function control(store: LifecycleStore, api: SessionApi) {
    const controller = createAgentSessionController(store, api);
    controllers.push(controller);
    return controller;
}

beforeEach(() => { controllers.length = 0; });
afterEach(() => {
    for (const controller of controllers) controller.dispose();
    vi.restoreAllMocks();
});

describe('agent/session authority', () => {
    it('synchronizes existing identity and catalog without inventing activity', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        expect(store.getState().session.activeId).toBe('session-a');
        expect(store.getState().session.items).toHaveLength(2);
        expect(store.getState().session.items[0]).not.toBe(state.items[0]);
        expect(store.getState().session.phase).toBe('active');
        expect(store.getState().session.activeGeneration).toBeNull();
        const revision = store.getState().session.revision;
        expect(controller.refresh()).toBe(true);
        expect(store.getState().session.revision).toBe(revision);
    });

    it('rejects create and resume boolean successes with missing or wrong active IDs', () => {
        const store = new LifecycleStore();
        const created = fixture([], null);
        created.api.newSession = vi.fn(() => true);
        const controller = control(store, created.api);
        expect(controller.create()).toBeNull();
        expect(store.getState().session.activeId).toBeNull();
        expect(store.getState().session.phase).toBe('new');

        const resumed = fixture();
        resumed.api.selectSession = vi.fn(() => true);
        const resumedStore = new LifecycleStore();
        const resumedController = control(resumedStore, resumed.api);
        const revision = resumedStore.getState().session.revision;
        expect(resumedController.resume('session-b')).toBeNull();
        expect(resumedStore.getState().session.activeId).toBe('session-a');
        expect(resumedStore.getState().session.revision).toBe(revision);
    });

    it('returns verified create/resume results and updates active state', () => {
        const store = new LifecycleStore();
        const state = fixture([], null);
        const controller = control(store, state.api);
        expect(controller.create()?.id).toBe('session-new-1');
        expect(store.getState().session.activeId).toBe('session-new-1');
        expect(store.getState().session.phase).toBe('active');
        expect(controller.resume('session-new-1')?.title).toBe('New conversation');
        expect(store.getState().session.activeId).toBe('session-new-1');
    });

    it('uses the same verified authority for rename, duplicate, remove, and export', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        const revision = store.getState().session.revision;
        expect(controller.rename('session-a', 'Renamed')).toBe(true);
        expect(store.getState().session.items[0].title).toBe('Renamed');
        expect(controller.duplicate('session-a')?.id).toBe('session-a-copy');
        expect(store.getState().session.activeId).toBe('session-a');
        expect(controller.export('session-a')).toEqual({ name: 'Renamed.json', content: '{"session":{"id":"session-a"}}' });
        expect(controller.remove('session-a')).toMatchObject({ id: 'session-new-1' });
        expect(store.getState().session.activeId).toBe('session-new-1');
        expect(store.getState().session.revision).toBeGreaterThan(revision);
    });

    it('creates the first session before sending and pins generation to it', () => {
        const store = new LifecycleStore();
        const state = fixture([], null);
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send('17+25')).toBe(true);
        const createOrder = (state.api.newSession as ReturnType<typeof vi.fn>).mock.invocationCallOrder[0];
        const sendOrder = (state.api.sendPrompt as ReturnType<typeof vi.fn>).mock.invocationCallOrder[0];
        expect(createOrder).toBeLessThan(sendOrder);
        expect(store.getState().session.activeGeneration?.sessionId).toBe('session-new-1');
        expect(store.getState().session.activeId).toBe('session-new-1');
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(store.getState().session.phase).toBe('complete');
        expect(store.getState().session.lastGeneration?.sessionId).toBe('session-new-1');
    });

    it('stops and records an old run instead of applying it to a changed session', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send('hello')).toBe(true);
        expect(store.getState().session.activeGeneration?.sessionId).toBe('session-a');
        state.activeId = 'session-b';
        expect(controller.refresh()).toBe(true);
        expect(state.api.stopGeneration).toHaveBeenCalledOnce();
        expect(store.getState().session.activeId).toBe('session-b');
        expect(store.getState().session.activeGeneration).toBeNull();
        expect(store.getState().session.phase).toBe('stopped');
        expect(store.getState().session.lastGeneration?.sessionId).toBe('session-a');
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(store.getState().session.phase).toBe('stopped');
        expect(store.getState().session.activeId).toBe('session-b');
    });

    it('refuses to send for an invalid prompt or failed first-session creation', () => {
        const store = new LifecycleStore();
        const state = fixture([], null);
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send(' ')).toBe(false);
        expect(state.api.newSession).not.toHaveBeenCalled();
        state.api.newSession = vi.fn(() => true);
        expect(controller.send('hello')).toBe(false);
        expect(state.api.sendPrompt).not.toHaveBeenCalled();
        expect(store.getState().session.activeGeneration).toBeNull();
    });
});


describe('agent/session verification repairs', () => {
    it('rejects a no-op create instead of returning the old active session', () => {
        const store = new LifecycleStore();
        const state = fixture();
        state.api.newSession = vi.fn(() => true);
        const controller = control(store, state.api);
        expect(controller.create()).toBeNull();
        expect(store.getState().session.activeId).toBe('session-a');
    });

    it('rejects duplicate source IDs and no-op deletion', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        state.api.duplicateSession = vi.fn(() => 'session-a');
        expect(controller.duplicate('session-a')).toBeNull();
        state.api.deleteSession = vi.fn(() => true);
        expect(controller.remove('session-a')).toBeNull();
        expect(store.getState().session.items.some((item) => item.id === 'session-a')).toBe(true);
    });

    it('verifies the legacy 80-code-unit rename boundary', () => {
        const store = new LifecycleStore();
        const state = fixture();
        state.api.renameSession = vi.fn((id: string, title: string) => {
            const item = state.items.find((candidate) => candidate.id === id);
            if (!item) return false;
            item.title = title.slice(0, 80);
            return true;
        });
        const controller = control(store, state.api);
        const title = 'x'.repeat(81);
        expect(controller.rename('session-a', title)).toBe(true);
        expect(state.api.renameSession).toHaveBeenCalledWith('session-a', 'x'.repeat(80));
        expect(store.getState().session.items[0].title).toBe('x'.repeat(80));
    });

    it('fails closed when active-session reads throw during send', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        state.api.activeSessionId = vi.fn(() => { throw new Error('read failed'); });
        expect(controller.send('hello')).toBe(false);
        expect(state.api.sendPrompt).not.toHaveBeenCalled();
        expect(store.getState().session.activeGeneration).toBeNull();
    });

    it('synchronizes identity when stop emits its terminal event synchronously', () => {
        const store = new LifecycleStore();
        const state = fixture();
        state.api.stopGeneration = vi.fn(() => {
            document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'stopped', generationId: 42 } }));
        });
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send('hello')).toBe(true);
        state.activeId = 'session-b';
        expect(controller.refresh()).toBe(true);
        expect(store.getState().session.activeId).toBe('session-b');
        expect(store.getState().session.phase).toBe('stopped');
        expect(store.getState().session.lastGeneration?.sessionId).toBe('session-a');
    });
});


describe('verified no-active deletion', () => {
    it('returns true only when target removal leaves no active session', () => {
        const store = new LifecycleStore();
        const state = fixture([first], null);
        const controller = control(store, state.api);
        expect(controller.remove(first.id)).toBe(true);
        expect(state.items).toHaveLength(0);
        expect(state.activeId).toBeNull();
        expect(store.getState().session.activeId).toBeNull();
    });
});


describe('fail-closed session operations and live catalog', () => {
    it('does not mutate duplicate, resume, or rename after a failed pre-read', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        state.api.listSessions = vi.fn(() => { throw new Error('read failed'); });
        expect(controller.duplicate('session-a')).toBeNull();
        expect(state.api.duplicateSession).not.toHaveBeenCalled();
        expect(controller.resume('session-a')).toBeNull();
        expect(state.api.selectSession).not.toHaveBeenCalled();
        expect(controller.rename('session-a', 'Changed')).toBe(false);
        expect(state.api.renameSession).not.toHaveBeenCalled();
    });

    it('refreshes catalog message metadata when generation completes', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send('17+25')).toBe(true);
        state.items[0].messageCount = 2;
        state.items[0].preview = '42';
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(store.getState().session.items[0].messageCount).toBe(2);
        expect(store.getState().session.items[0].preview).toBe('42');
    });

    it('does not stop or relabel a non-stoppable prediction during session change', () => {
        const store = new LifecycleStore();
        const state = fixture();
        state.api.getMode = vi.fn((): 'wasm' | 'api' => 'wasm');
        const controller = control(store, state.api);
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: { schema: 'uor-r4.lut-discrete-trunk/1', canGenerate: false, canPredict: true, engineSupported: true } });
        expect(controller.send('predict')).toBe(true);
        expect(store.getState().session.activeGeneration?.canStop).toBe(false);
        state.activeId = 'session-b';
        expect(controller.refresh()).toBe(false);
        expect(state.api.stopGeneration).not.toHaveBeenCalled();
        expect(store.getState().session.activeId).toBe('session-a');
        expect(store.getState().session.activeGeneration?.sessionId).toBe('session-a');
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(controller.refresh()).toBe(true);
        expect(store.getState().session.activeId).toBe('session-b');
    });
});


describe('non-stoppable remount and listener disposal', () => {
    it('preserves a non-stoppable prediction on controller remount', () => {
        const store = new LifecycleStore();
        const state = fixture();
        state.api.getMode = vi.fn((): 'wasm' | 'api' => 'wasm');
        const first = control(store, state.api);
        store.dispatch({ type: 'artifact/selected', sourceId: 'model-a' });
        store.dispatch({ type: 'artifact/ready', operation: store.getState().operation!, artifact: { schema: 'uor-r4.lut-discrete-trunk/1', canGenerate: false, canPredict: true, engineSupported: true } });
        expect(first.send('predict')).toBe(true);
        expect(store.getState().session.activeGeneration?.canStop).toBe(false);
        first.dispose();
        control(store, state.api);
        expect(state.api.stopGeneration).not.toHaveBeenCalled();
        expect(store.getState().session.phase).toBe('generating');
        expect(store.getState().session.activeGeneration?.sessionId).toBe('session-a');
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(store.getState().session.phase).toBe('complete');
        expect(store.getState().session.lastGeneration?.sessionId).toBe('session-a');
    });

    it('does not refresh catalog metadata after disposal', () => {
        const store = new LifecycleStore();
        const state = fixture();
        const controller = control(store, state.api);
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        expect(controller.send('hello')).toBe(true);
        controller.dispose();
        state.items[0].messageCount = 2;
        state.items[0].preview = '42';
        document.dispatchEvent(new CustomEvent(GENERATION_FINISHED_EVENT, { detail: { outcome: 'complete', generationId: 42 } }));
        expect(store.getState().session.items[0].messageCount).toBe(1);
        expect(store.getState().session.items[0].preview).toBe('first');
        expect(controller.refresh()).toBe(true);
        expect(store.getState().session.items[0].messageCount).toBe(2);
        expect(store.getState().session.items[0].preview).toBe('42');
    });
});
