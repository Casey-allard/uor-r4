import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useScopedShortcuts } from './useScopedShortcuts.js';
import type { ScopedShortcutHandlers } from './useScopedShortcuts.js';

let host: HTMLDivElement;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
});

afterEach(() => {
    act(() => { render(null, host); });
    host.remove();
    vi.restoreAllMocks();
});

function Harness(props: { handlers: ScopedShortcutHandlers }) {
    useScopedShortcuts(props.handlers);
    return <div>
        <input aria-label="Text field" />
        <textarea aria-label="Text area" />
        <select aria-label="Picker"><option>One</option></select>
        <div aria-label="Editable region" contenteditable="true" />
    </div>;
}

function mount(handlers: ScopedShortcutHandlers) {
    act(() => { render(<Harness handlers={handlers} />, host); });
}

function press(key: string, init: KeyboardEventInit & { isComposing?: boolean; keyCode?: number } = {}, target: EventTarget = document.body) {
    const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true, ...init });
    Object.defineProperties(event, {
        isComposing: { value: Boolean(init.isComposing) },
        keyCode: { value: init.keyCode ?? 0 },
    });
    act(() => { target.dispatchEvent(event); });
    return event;
}

function dispatchTo(element: HTMLElement, key: string, init: KeyboardEventInit = {}) {
    element.focus();
    return press(key, init, element);
}

describe('useScopedShortcuts', () => {
    it('opens the palette for Meta/Ctrl+Shift+P outside editable fields', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        const meta = press('P', { metaKey: true, shiftKey: true });
        const ctrl = press('P', { ctrlKey: true, shiftKey: true });

        expect(meta.defaultPrevented).toBe(true);
        expect(ctrl.defaultPrevented).toBe(true);
        expect(onPalette).toHaveBeenCalledTimes(2);
        expect(onSettings).not.toHaveBeenCalled();
    });

    it('opens settings for Meta/Ctrl+Comma outside editable fields', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        const meta = press(',', { metaKey: true });
        const ctrl = press(',', { ctrlKey: true });

        expect(meta.defaultPrevented).toBe(true);
        expect(ctrl.defaultPrevented).toBe(true);
        expect(onSettings).toHaveBeenCalledTimes(2);
        expect(onPalette).not.toHaveBeenCalled();
    });

    it('ignores unrelated chords including legacy Cmd/Ctrl+K and shifted comma', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        press('P', { metaKey: true });
        press('k', { metaKey: true });
        press('K', { ctrlKey: true });
        press(',', { metaKey: true, shiftKey: true });
        press('P', { metaKey: true, shiftKey: true, altKey: true });

        expect(onPalette).not.toHaveBeenCalled();
        expect(onSettings).not.toHaveBeenCalled();
    });

    it('ignores input, textarea, select and contenteditable targets', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        dispatchTo(host.querySelector<HTMLInputElement>('input')!, 'P', { metaKey: true, shiftKey: true });
        dispatchTo(host.querySelector<HTMLTextAreaElement>('textarea')!, 'P', { metaKey: true, shiftKey: true });
        dispatchTo(host.querySelector<HTMLSelectElement>('select')!, ',', { metaKey: true });
        dispatchTo(host.querySelector<HTMLElement>('[contenteditable="true"]')!, ',', { metaKey: true });

        expect(onPalette).not.toHaveBeenCalled();
        expect(onSettings).not.toHaveBeenCalled();
    });

    it('ignores IME composition events and keyCode 229', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        press('P', { metaKey: true, shiftKey: true, isComposing: true });
        press('P', { metaKey: true, shiftKey: true, keyCode: 229 });
        press(',', { metaKey: true, isComposing: true });
        press(',', { metaKey: true, keyCode: 229 });

        expect(onPalette).not.toHaveBeenCalled();
        expect(onSettings).not.toHaveBeenCalled();
    });

    it('attaches one listener family, follows handler changes, and cleans up exactly', () => {
        const add = vi.spyOn(window, 'addEventListener');
        const remove = vi.spyOn(window, 'removeEventListener');
        const first = vi.fn();
        const second = vi.fn();
        mount({ onPalette: first, onSettings: first });
        const keydownAdds = add.mock.calls.filter(([type]) => type === 'keydown');
        expect(keydownAdds).toHaveLength(1);
        expect(keydownAdds[0]![2]).toBe(true);
        const listener = keydownAdds[0]![1];

        press('P', { metaKey: true, shiftKey: true });
        expect(first).toHaveBeenCalledOnce();
        expect(second).not.toHaveBeenCalled();

        mount({ onPalette: second, onSettings: second });
        expect(add.mock.calls.filter(([type]) => type === 'keydown')).toHaveLength(1);
        press('P', { metaKey: true, shiftKey: true });
        expect(second).toHaveBeenCalledOnce();
        expect(first).toHaveBeenCalledOnce();

        act(() => { render(null, host); });
        expect(remove).toHaveBeenCalledWith('keydown', listener, true);
        const callsAfterUnmount = second.mock.calls.length;
        press('P', { metaKey: true, shiftKey: true });
        expect(second).toHaveBeenCalledTimes(callsAfterUnmount);
    });

    it('does not capture Tab or Escape', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        mount({ onPalette, onSettings });

        const tab = press('Tab');
        const escape = press('Escape');

        expect(tab.defaultPrevented).toBe(false);
        expect(escape.defaultPrevented).toBe(false);
        expect(onPalette).not.toHaveBeenCalled();
        expect(onSettings).not.toHaveBeenCalled();
    });

    it('stops a handled shortcut before document-level legacy listeners', () => {
        const onPalette = vi.fn();
        const onSettings = vi.fn();
        const legacy = vi.fn();
        document.addEventListener('keydown', legacy);
        try {
            mount({ onPalette, onSettings });
            press('P', { metaKey: true, shiftKey: true });
            press(',', { ctrlKey: true });
            expect(onPalette).toHaveBeenCalledOnce();
            expect(onSettings).toHaveBeenCalledOnce();
            expect(legacy).not.toHaveBeenCalled();
            press('P', { metaKey: true });
            expect(legacy).toHaveBeenCalledOnce();
        } finally {
            document.removeEventListener('keydown', legacy);
        }
    });
});
