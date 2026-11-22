import { useSyncExternalStore } from 'preact/compat';
import type { LifecycleState, LifecycleStore } from './lifecycle.js';

export function useLifecycle(store: LifecycleStore): LifecycleState {
    return useSyncExternalStore(
        (listener) => store.subscribe(listener),
        () => store.getState(),
    );
}
