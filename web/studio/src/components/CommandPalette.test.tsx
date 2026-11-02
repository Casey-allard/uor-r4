import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CommandPalette } from './CommandPalette.js';

let host: HTMLDivElement;
beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = true; }) });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: vi.fn(function (this: HTMLDialogElement) { this.open = false; this.dispatchEvent(new Event('close')); }) });
});
afterEach(() => { act(() => { render(null, host); }); host.remove(); vi.restoreAllMocks(); });

function key(input: HTMLInputElement, key: string) {
    input.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
}

describe('CommandPalette', () => {
    it('filters and runs the active command with arrow keys and Enter', () => {
        const save = vi.fn();
        const open = vi.fn();
        const onClose = vi.fn();
        act(() => {
            render(<CommandPalette onClose={onClose} commands={[
                { id: 'open', group: 'Workspace', label: 'Open folder', description: 'Choose a folder', run: open },
                { id: 'save', group: 'Workspace', label: 'Save file', description: 'Write current file', shortcut: 'Ctrl+S', run: save },
            ]} />, host);
        });
        const input = host.querySelector<HTMLInputElement>('input[aria-label="Search commands"]')!;
        act(() => {
            input.value = 'save';
            input.dispatchEvent(new Event('input', { bubbles: true }));
        });
        expect(host.querySelectorAll('button[data-command-id]')).toHaveLength(1);
        act(() => { key(input, 'ArrowDown'); key(input, 'ArrowUp'); key(input, 'Enter'); });
        expect(save).toHaveBeenCalledOnce();
        expect(open).not.toHaveBeenCalled();
        expect(onClose).toHaveBeenCalledOnce();
    });

    it('never executes a disabled command', () => {
        const run = vi.fn();
        act(() => {
            render(<CommandPalette onClose={vi.fn()} commands={[{ id: 'disabled', group: 'Agent', label: 'Send', description: 'No source ready', run, disabled: true }]} />, host);
        });
        const button = host.querySelector<HTMLButtonElement>('button[data-command-id="disabled"]')!;
        expect(button.disabled).toBe(true);
        button.click();
        const input = host.querySelector<HTMLInputElement>('input')!;
        act(() => { key(input, 'Enter'); });
        expect(run).not.toHaveBeenCalled();
    });
});
