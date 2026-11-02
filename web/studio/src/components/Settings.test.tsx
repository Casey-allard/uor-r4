import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Settings } from './Settings.js';
import { makeUi } from './studioTestFixtures.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); });

function button(label: string): HTMLButtonElement {
    const found = Array.from(host.querySelectorAll('button')).find((node) => node.textContent === label);
    if (!found) throw new Error(`Missing button ${label}`);
    return found;
}

describe('Settings', () => {
    it('updates appearance and editor preferences through the existing writer', () => {
        const ui = makeUi();
        const onSettingsChange = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onSettingsChange={onSettingsChange} />, host); });
        const theme = host.querySelector<HTMLSelectElement>('select[aria-label="Theme"]')!;
        act(() => { theme.value = 'light'; theme.dispatchEvent(new Event('change', { bubbles: true })); });
        const reduce = host.querySelector<HTMLInputElement>('input[aria-label="Reduce motion"]')!;
        act(() => { reduce.checked = true; reduce.dispatchEvent(new Event('change', { bubbles: true })); });
        const font = host.querySelector<HTMLInputElement>('input[aria-label="Editor font size"]')!;
        act(() => { font.value = '16'; font.dispatchEvent(new Event('input', { bubbles: true })); font.dispatchEvent(new Event('blur', { bubbles: true })); });
        const tab = host.querySelector<HTMLInputElement>('input[aria-label="Tab size"]')!;
        act(() => { tab.value = '2'; tab.dispatchEvent(new Event('input', { bubbles: true })); tab.dispatchEvent(new Event('blur', { bubbles: true })); });
        const wrap = host.querySelector<HTMLInputElement>('input[aria-label="Word wrap"]')!;
        act(() => { wrap.checked = true; wrap.dispatchEvent(new Event('change', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenCalledWith({ theme: 'light' });
        expect(ui.updateSettings).toHaveBeenCalledWith({ reduceMotion: true });
        expect(ui.updateSettings).toHaveBeenCalledWith({ fontSize: 16 });
        expect(ui.updateSettings).toHaveBeenCalledWith({ tabSize: 2 });
        expect(ui.updateSettings).toHaveBeenCalledWith({ wordWrap: true });
        expect(onSettingsChange).toHaveBeenLastCalledWith({ theme: 'light', reduceMotion: true, fontSize: 16, tabSize: 2, wordWrap: true });
    });

    it('reports partial session clearing through the typed error channel', () => {
        const ui = makeUi();
        ui.clearSessions = vi.fn(() => ({ cleared: 1, total: 2 }));
        const onError = vi.fn();
        const onChange = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onError={onError} onChange={onChange} />, host); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(onError).toHaveBeenCalledWith('Cleared 1 of 2 sessions; stop the current reply before deleting the rest.', 'settings:sessions');
        expect(onChange).toHaveBeenCalledWith('sessions');
    });

    it('does not activate a session after the clear operation errors', () => {
        const ui = makeUi();
        ui.clearSessions = vi.fn(() => ({ cleared: 0, total: 1, error: 'Session store not ready.' }));
        const onChange = vi.fn();
        const onError = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onError={onError} onChange={onChange} />, host); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(onError).toHaveBeenCalledWith('Session store not ready.', 'settings:sessions');
        expect(onChange).not.toHaveBeenCalled();
        expect(host.querySelector('dialog [role="alert"]')?.textContent).toContain('Session store not ready.');
    });

    it('does not change session state when a clear deletes nothing, including an empty store', () => {
        const ui = makeUi();
        ui.clearSessions = vi.fn(() => ({ cleared: 0, total: 2 }));
        const onChange = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onChange={onChange} />, host); });
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(onChange).not.toHaveBeenCalled();
        ui.clearSessions = vi.fn(() => ({ cleared: 0, total: 0 }));
        act(() => { button('Clear sessions').click(); });
        act(() => { button('Confirm clear sessions').click(); });
        expect(onChange).not.toHaveBeenCalled();
    });

    it('requires confirmation before clearing sessions or editor tabs', () => {
        const ui = makeUi();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} />, host); });
        act(() => { button('Clear sessions').click(); });
        expect(ui.clearSessions).not.toHaveBeenCalled();
        act(() => { button('Confirm clear sessions').click(); });
        expect(ui.clearSessions).toHaveBeenCalledOnce();
        act(() => { button('Clear editor tabs').click(); });
        expect(ui.clearEditor).not.toHaveBeenCalled();
        act(() => { button('Confirm clear editor tabs').click(); });
        expect(ui.clearEditor).toHaveBeenCalledOnce();
    });

    it('clamps numeric editor preferences on blur', () => {
        const ui = makeUi();
        const onSettingsChange = vi.fn();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} onSettingsChange={onSettingsChange} />, host); });
        const font = host.querySelector<HTMLInputElement>('input[aria-label="Editor font size"]')!;
        const tab = host.querySelector<HTMLInputElement>('input[aria-label="Tab size"]')!;
        act(() => { font.value = '0'; font.dispatchEvent(new Event('input', { bubbles: true })); font.dispatchEvent(new Event('blur', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenLastCalledWith({ fontSize: 10 });
        expect(font.value).toBe('10');
        act(() => { font.value = '99'; font.dispatchEvent(new Event('input', { bubbles: true })); font.dispatchEvent(new Event('blur', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenLastCalledWith({ fontSize: 24 });
        act(() => { tab.value = '0'; tab.dispatchEvent(new Event('input', { bubbles: true })); tab.dispatchEvent(new Event('blur', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenLastCalledWith({ tabSize: 1 });
        expect(tab.value).toBe('1');
        act(() => { tab.value = '99'; tab.dispatchEvent(new Event('input', { bubbles: true })); tab.dispatchEvent(new Event('blur', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenLastCalledWith({ tabSize: 8 });
        expect(onSettingsChange).toHaveBeenLastCalledWith({ theme: 'dark', reduceMotion: false, fontSize: 24, tabSize: 8, wordWrap: false });
    });

    it('keeps the numeric draft intact during sequential typing', () => {
        const ui = makeUi();
        act(() => { render(<Settings ui={ui} onClose={vi.fn()} />, host); });
        const font = host.querySelector<HTMLInputElement>('input[aria-label="Editor font size"]')!;
        act(() => { font.value = '1'; font.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(font.value).toBe('1');
        expect(ui.updateSettings).not.toHaveBeenCalled();
        act(() => { font.value = '16'; font.dispatchEvent(new Event('input', { bubbles: true })); });
        expect(font.value).toBe('16');
        expect(ui.updateSettings).not.toHaveBeenCalled();
        act(() => { font.dispatchEvent(new Event('blur', { bubbles: true })); });
        expect(ui.updateSettings).toHaveBeenCalledWith({ fontSize: 16 });
        expect(font.value).toBe('16');
    });
});
