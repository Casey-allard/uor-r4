import { useEffect, useRef } from 'preact/hooks';
import { isEditableTarget } from '../agent/legacyShortcutGuard.js';

export interface ScopedShortcutHandlers {
    onPalette(): void;
    onSettings(): void;
}

export function useScopedShortcuts(handlers: ScopedShortcutHandlers): void {
    const handlersRef = useRef(handlers);
    handlersRef.current = handlers;

    useEffect(() => {
        const onKeyDown = (event: KeyboardEvent) => {
            if (!(event.metaKey || event.ctrlKey) || event.altKey) return;
            if (event.isComposing || event.keyCode === 229) return;
            if (isEditableTarget(event.target) || isEditableTarget(document.activeElement)) return;
            const palette = event.shiftKey && (event.key === 'p' || event.key === 'P');
            const settings = !event.shiftKey && event.key === ',';
            if (!palette && !settings) return;
            event.preventDefault();
            event.stopPropagation();
            if (palette) handlersRef.current.onPalette();
            else handlersRef.current.onSettings();
        };
        window.addEventListener('keydown', onKeyDown, true);
        return () => {
            if (typeof window !== 'undefined') {
                window.removeEventListener('keydown', onKeyDown, true);
            }
        };
    }, []);
}
