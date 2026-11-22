import type { LifecycleStore } from '../state/lifecycle.js';
import type { WorkspaceApi } from './workflow.js';

let saveQueue: Promise<void> = Promise.resolve();

export function saveActiveFile(store: LifecycleStore, files: WorkspaceApi, onError: (message: string, operation: string) => void, onSuccess?: (operation: string) => void): Promise<boolean> {
    const path = store.getState().file.path;
    if (!path || !store.canSave()) return Promise.resolve(false);
    store.dispatch({ type: 'file/saveStarted', path });
    const saveId = store.getState().file.saveId;
    if (store.getState().file.phase !== 'saving' || saveId === null) return Promise.resolve(false);

    let started = false;
    const operation = saveQueue.then(async () => {
        if (!started && (store.getState().file.path !== path || store.getState().file.saveId !== saveId)) return false;
        started = true;
        const failActivePath = () => {
            if (store.getState().file.path === path && store.getState().file.saveId === saveId) {
                store.dispatch({ type: 'file/saveFailed', path, saveId, error: 'The active file changed before saving.' });
                onError('The active file changed before saving.', `file:save:${path}`);
            }
            return false;
        };
        try {
            if (files.activeFile()?.path !== path) return failActivePath();
            const saved = await files.saveActive();
            if (files.activeFile()?.path !== path) return failActivePath();
            if (store.getState().file.path !== path || store.getState().file.saveId !== saveId) return saved;
            if (saved) {
                store.dispatch({ type: 'file/saveFinished', path, saveId });
                onSuccess?.(`file:save:${path}`);
            } else {
                store.dispatch({ type: 'file/saveFailed', path, saveId, error: 'Save did not complete.' });
                onError('Save did not complete.', `file:save:${path}`);
            }
            return saved;
        } catch (error) {
            if (store.getState().file.path === path && store.getState().file.saveId === saveId) {
                const message = error instanceof Error ? error.message : String(error);
                store.dispatch({ type: 'file/saveFailed', path, saveId, error: message });
                onError(message, `file:save:${path}`);
            }
            return false;
        }
    });
    saveQueue = operation.then(() => undefined, () => undefined);
    return operation;
}

export function saveActiveFileCopy(store: LifecycleStore, files: WorkspaceApi, onError: (message: string, operation: string) => void, onSuccess: (operation: string) => void): boolean {
    const path = store.getState().file.path;
    const operation = `file:save-as:${path}`;
    if (!path || !store.canSave() || files.activeFile()?.path !== path) {
        onError('The active file changed before Save as.', operation);
        return false;
    }
    try {
        if (!files.saveAsCopy()) {
            onError('Save as did not complete.', operation);
            return false;
        }
        onSuccess(operation);
        return true;
    } catch (error) {
        onError(error instanceof Error ? error.message : String(error), operation);
        return false;
    }
}
