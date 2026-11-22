interface LegacyStudioUi {
    closePalette?(restoreFocus?: boolean): void;
}

interface LegacyStudioWindow extends Window {
    __studioUi?: LegacyStudioUi;
}

const paletteHashes = new Set(['#command-palette', '#palette']);

function paletteNode(doc: Document): HTMLElement | null {
    return doc.getElementById('commandPalette');
}

function closeLegacyPalette(view: Window): void {
    const ui = (view as LegacyStudioWindow).__studioUi;
    if (ui && typeof ui.closePalette === 'function') ui.closePalette(false);
    const palette = paletteNode(view.document);
    if (palette && !palette.hidden) palette.hidden = true;
}

function clearPaletteHash(view: Window): boolean {
    if (!paletteHashes.has(view.location.hash)) return false;
    view.history.replaceState(view.history.state, '', `${view.location.pathname}${view.location.search}`);
    return true;
}

export function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof Element)) return false;
    const tag = target.tagName;
    if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
    const editable = target.closest('[contenteditable]');
    return editable !== null && editable.getAttribute('contenteditable') !== 'false';
}

export interface LegacyShortcutGuard {
    dispose(): void;
}

export function installLegacyShortcutGuard(root: HTMLElement): LegacyShortcutGuard {
    const doc = root.ownerDocument;
    const view = doc.defaultView;
    if (!view) return { dispose() {} };

    closeLegacyPalette(view);
    clearPaletteHash(view);

    const palette = paletteNode(doc);
    const observer = palette
        ? new MutationObserver(() => { if (!palette.hidden) closeLegacyPalette(view); })
        : null;
    if (palette && observer) observer.observe(palette, { attributes: true, attributeFilter: ['hidden'] });

    const blockLegacyShortcut = (event: KeyboardEvent) => {
        if (!(event.metaKey || event.ctrlKey)) return;
        if (event.key === 'k' || event.key === 'K') {
            if (event.altKey) return;
            event.preventDefault();
            event.stopImmediatePropagation();
            closeLegacyPalette(view);
            return;
        }
        if (event.key !== ',') return;
        if (event.isComposing || event.keyCode === 229) {
            event.stopImmediatePropagation();
            return;
        }
        if (event.altKey || event.shiftKey) {
            event.stopImmediatePropagation();
            return;
        }
        if (!isEditableTarget(event.target) && !isEditableTarget(doc.activeElement)) return;
        event.preventDefault();
        event.stopImmediatePropagation();
    };
    const closeOnPaletteHash = () => { if (clearPaletteHash(view)) closeLegacyPalette(view); };
    const closeOnLegacyPublished = () => closeLegacyPalette(view);

    view.addEventListener('keydown', blockLegacyShortcut, true);
    view.addEventListener('hashchange', closeOnPaletteHash);
    view.addEventListener('uor-studio-v2-ready', closeOnLegacyPublished);

    return {
        dispose() {
            observer?.disconnect();
            view.removeEventListener('keydown', blockLegacyShortcut, true);
            view.removeEventListener('hashchange', closeOnPaletteHash);
            view.removeEventListener('uor-studio-v2-ready', closeOnLegacyPublished);
        },
    };
}
