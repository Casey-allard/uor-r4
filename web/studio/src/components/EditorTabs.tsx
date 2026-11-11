import { useEffect, useId, useRef, useState } from 'preact/hooks';
import type { WorkspaceTab } from '../adapters/workflow.js';
import { Icon } from './ui/Icon.js';

export interface EditorTabsProps {
    tabs: Array<Pick<WorkspaceTab, 'path' | 'dirty'>>;
    activePath: string | null;
    onSelect(path: string): void;
    onClose(path: string): void;
    onFocusFallback(): void;
}

export function EditorTabs({ tabs, activePath, onSelect, onClose, onFocusFallback }: EditorTabsProps) {
    const idPrefix = useId();
    const dialogTitleId = useId();
    const dialogBodyId = useId();
    const dialogRef = useRef<HTMLDialogElement>(null);
    const tabRefs = useRef(new Map<string, HTMLButtonElement>());
    const pendingRef = useRef<string | null>(null);
    const [pendingClose, setPendingClose] = useState<string | null>(null);
    const tabsRef = useRef(tabs);
    tabsRef.current = tabs;

    useEffect(() => {
        const dialog = dialogRef.current;
        if (pendingClose === null || !dialog) return;
        if (typeof dialog.showModal === 'function' && !dialog.open) dialog.showModal();
        return () => {
            if (dialog.open) dialog.close();
        };
    }, [pendingClose]);

    function requestClose(path: string) {
        const tab = tabsRef.current.find((candidate) => candidate.path === path);
        if (!tab) return;
        if (!tab.dirty) {
            closeAndFocus(path);
            return;
        }
        pendingRef.current = path;
        setPendingClose(path);
    }

    function closeAndFocus(path: string) {
        const current = tabsRef.current;
        const index = current.findIndex((candidate) => candidate.path === path);
        const remaining = current.filter((candidate) => candidate.path !== path);
        const next = remaining[Math.min(Math.max(index, 0), remaining.length - 1)];
        onClose(path);
        const target = next ? tabRefs.current.get(next.path) : null;
        if (target) target.focus();
        else onFocusFallback();
    }

    function onDialogClose() {
        const dialog = dialogRef.current;
        const value = dialog?.returnValue ?? '';
        if (dialog) dialog.returnValue = '';
        const path = pendingRef.current;
        pendingRef.current = null;
        setPendingClose(null);
        if (value === 'discard' && path !== null) closeAndFocus(path);
    }

    function onKeyDown(event: KeyboardEvent & { readonly currentTarget: HTMLUListElement }) {
        if (tabs.length === 0) return;
        const index = Math.max(0, tabs.findIndex((tab) => tab.path === activePath));
        let next: number;
        if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
        else if (event.key === 'ArrowLeft') next = (index - 1 + tabs.length) % tabs.length;
        else if (event.key === 'Home') next = 0;
        else if (event.key === 'End') next = tabs.length - 1;
        else return;
        event.preventDefault();
        const tab = tabs[next]!;
        onSelect(tab.path);
        tabRefs.current.get(tab.path)?.focus();
    }

    return (
        <>
            {tabs.length > 0 && (
                <ul class="editor-tabs" role="tablist" aria-label="Open editor tabs" onKeyDown={onKeyDown}>
                    {tabs.map((tab, index) => {
                        const selected = tab.path === activePath;
                        return (
                            <li role="presentation" key={tab.path}>
                                <button
                                    type="button"
                                    role="tab"
                                    id={`${idPrefix}-tab-${index}`}
                                    aria-selected={selected}
                                    aria-current={selected ? 'page' : undefined}
                                    tabIndex={selected || (activePath === null && index === 0) ? 0 : -1}
                                    ref={(element) => {
                                        if (element) tabRefs.current.set(tab.path, element);
                                        else tabRefs.current.delete(tab.path);
                                    }}
                                    onClick={() => onSelect(tab.path)}
                                >
                                    {`Open ${tab.path}`}
                                    {tab.dirty && <span class="tab-dirty">Unsaved</span>}
                                </button>
                                <button type="button" aria-label={`Close ${tab.path}`} onClick={() => requestClose(tab.path)}>
                                    <Icon name="close" />
                                    <span class="visually-hidden">{`Close ${tab.path}`}</span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
            )}
            {pendingClose !== null && (
                <dialog ref={dialogRef} aria-labelledby={dialogTitleId} aria-describedby={dialogBodyId} onClose={onDialogClose}>
                    <h2 id={dialogTitleId}>Discard changes to {pendingClose}?</h2>
                    <p id={dialogBodyId}>Your unsaved changes will be lost.</p>
                    <form method="dialog">
                        <button type="submit" value="cancel" autofocus>Keep editing</button>
                        <button type="submit" value="discard">Discard changes</button>
                    </form>
                </dialog>
            )}
        </>
    );
}
