import type { LifecycleStore, SessionSummary } from '../state/lifecycle.js';
import { createGenerationAdapter, GENERATION_FINISHED_EVENT } from './generation.js';
import type { SessionApi } from './workflow.js';

interface SessionSnapshot {
    activeId: string | null;
    items: SessionSummary[];
}

export interface AgentSessionController {
    refresh(): boolean;
    create(): SessionSummary | null;
    resume(id: string): SessionSummary | null;
    rename(id: string, title: string): boolean;
    duplicate(id: string): SessionSummary | null;
    remove(id: string): SessionSummary | true | null;
    export(id: string): { name: string; content: string } | null;
    send(prompt: string): boolean;
    stop(): void;
    dispose(): void;
}

export function createAgentSessionController(store: LifecycleStore, sessions: SessionApi): AgentSessionController {
    const generation = createGenerationAdapter(store, sessions);
    let interrupting = false;

    function read(): SessionSnapshot | null {
        try {
            const items = sessions.listSessions().map((item) => ({ ...item }));
            const activeId = sessions.activeSessionId();
            if (activeId !== null && !items.some((item) => item.id === activeId)) return null;
            return { activeId, items };
        } catch {
            return null;
        }
    }

    function refresh(): boolean {
        const snapshot = read();
        if (!snapshot) return false;
        const active = store.getState().session.activeGeneration;
        if (active && active.sessionId !== snapshot.activeId) {
            if (active.canStop === false) return false;
            interrupting = true;
            try {
                sessions.stopGeneration();
                store.dispatch({ type: 'session/interrupted', activeId: snapshot.activeId, items: snapshot.items, generationId: active.generationId });
            } catch {
                interrupting = false;
                return false;
            }
            interrupting = false;
            return store.getState().session.activeGeneration === null && store.getState().session.activeId === snapshot.activeId;
        }
        store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
        return store.getState().session.activeId === snapshot.activeId;
    }

    function verifiedActive(expectedId?: string): SessionSummary | null {
        const snapshot = read();
        if (!snapshot?.activeId || (expectedId !== undefined && snapshot.activeId !== expectedId)) return null;
        const active = snapshot.items.find((item) => item.id === snapshot.activeId);
        if (!active) return null;
        store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
        return store.getState().session.activeId === active.id ? active : null;
    }

    function create(): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            const before = read();
            if (!before || !sessions.newSession()) return null;
            const after = read();
            if (!after?.activeId || after.activeId === before.activeId || before.items.some((item) => item.id === after.activeId)) return null;
            const created = after.items.find((item) => item.id === after.activeId);
            if (!created) return null;
            store.dispatch({ type: 'session/synchronized', activeId: after.activeId, items: after.items });
            return store.getState().session.activeId === created.id ? created : null;
        } catch {
            return null;
        }
    }

    function resume(id: string): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            const before = read();
            if (!before?.items.some((item) => item.id === id)) return null;
            return sessions.selectSession(id) ? verifiedActive(id) : null;
        } catch {
            return null;
        }
    }

    function rename(id: string, title: string): boolean {
        if (store.getState().session.phase === 'generating') return false;
        try {
            const before = read();
            if (!before?.items.some((item) => item.id === id)) return false;
            const normalized = title.slice(0, 80);
            if (!sessions.renameSession(id, normalized)) return false;
            const snapshot = read();
            if (!snapshot || snapshot.items.find((item) => item.id === id)?.title !== normalized) return false;
            store.dispatch({ type: 'session/synchronized', activeId: snapshot.activeId, items: snapshot.items });
            return true;
        } catch {
            return false;
        }
    }

    function duplicate(id: string): SessionSummary | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            const before = read();
            if (!before?.items.some((item) => item.id === id)) return null;
            const duplicateId = sessions.duplicateSession(id);
            if (!duplicateId || duplicateId === id || before.items.some((item) => item.id === duplicateId)) return null;
            const after = read();
            const duplicate = after?.items.find((item) => item.id === duplicateId);
            if (!after || !duplicate || after.activeId !== before.activeId) return null;
            store.dispatch({ type: 'session/synchronized', activeId: after.activeId, items: after.items });
            return duplicate;
        } catch {
            return null;
        }
    }

    function remove(id: string): SessionSummary | true | null {
        if (store.getState().session.phase === 'generating') return null;
        try {
            const before = read();
            if (!before?.items.some((item) => item.id === id) || !sessions.deleteSession(id)) return null;
            const after = read();
            if (!after || after.activeId === id || after.items.some((item) => item.id === id)) return null;
            if (before.activeId !== id && after.activeId !== before.activeId) return null;
            if (after.activeId === null) {
                if (before.activeId !== null && after.items.length > 0) return null;
                store.dispatch({ type: 'session/synchronized', activeId: null, items: after.items });
                return store.getState().session.activeId === null ? true : null;
            }
            const active = after.items.find((item) => item.id === after.activeId);
            if (!active) return null;
            store.dispatch({ type: 'session/synchronized', activeId: after.activeId, items: after.items });
            return store.getState().session.activeId === active.id ? active : null;
        } catch {
            return null;
        }
    }

    function exportSession(id: string): { name: string; content: string } | null {
        try {
            const item = read()?.items.find((candidate) => candidate.id === id);
            if (!item) return null;
            const content = sessions.exportSession(id);
            return content ? { name: `${item.title}.json`, content } : null;
        } catch {
            return null;
        }
    }

    const completed = () => { if (!interrupting) refresh(); };
    document.addEventListener(GENERATION_FINISHED_EVENT, completed);
    refresh();

    return {
        refresh,
        create,
        resume,
        rename,
        duplicate,
        remove,
        export: exportSession,
        send(prompt) {
            try {
                if (!prompt.trim() || !store.canRun()) return false;
                const snapshot = read();
                if (!snapshot) return false;
                if (snapshot.activeId === null) {
                    if (!create()) return false;
                } else if (!refresh()) {
                    return false;
                }
                const sent = generation.send(prompt);
                refresh();
                return sent;
            } catch {
                return false;
            }
        },
        stop() {
            generation.stop();
            refresh();
        },
        dispose() {
            document.removeEventListener(GENERATION_FINISHED_EVENT, completed);
            generation.dispose();
        },
    };
}
