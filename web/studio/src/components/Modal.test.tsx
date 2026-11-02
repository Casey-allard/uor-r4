import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Modal } from './Modal.js';

let host: HTMLDivElement;
let showModal: ReturnType<typeof vi.fn>;
let close: ReturnType<typeof vi.fn>;

beforeEach(() => {
    host = document.createElement('div');
    document.body.appendChild(host);
    showModal = vi.fn(function (this: HTMLDialogElement) { this.open = true; });
    close = vi.fn(function (this: HTMLDialogElement) {
        this.open = false;
        this.dispatchEvent(new Event('close'));
    });
    Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value: showModal });
    Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value: close });
});

afterEach(() => {
    render(null, host);
    host.remove();
    vi.restoreAllMocks();
});

describe('Modal', () => {
    it('uses a named native modal and forwards its native close event', () => {
        const onClose = vi.fn();
        act(() => {
            render(<Modal title="Studio settings" onClose={onClose}><button type="button" autofocus>Done</button></Modal>, host);
        });
        const dialog = host.querySelector('dialog')!;
        const titleId = dialog.getAttribute('aria-labelledby')!;
        expect(showModal).toHaveBeenCalledTimes(1);
        expect(document.getElementById(titleId)?.textContent).toBe('Studio settings');
        expect(dialog.hasAttribute('aria-modal')).toBe(false);
        expect(dialog.getAttribute('role')).toBeNull();
        dialog.close();
        expect(close).toHaveBeenCalledTimes(1);
        expect(onClose).toHaveBeenCalledTimes(1);
    });
});
