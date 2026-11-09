import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PaneDock } from './PaneDock.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); });

describe('PaneDock', () => {
    it('changes real sections and splits content with an accessible splitter', () => {
        const onSection = vi.fn(); const onCloseSplit = vi.fn();
        act(() => { render(<PaneDock activeSection="agent" onSection={onSection} onCloseSplit={onCloseSplit} secondarySection="workspace" secondaryChildren={<p>Editor body</p>}><p>Chat body</p></PaneDock>, host); });
        expect(host.querySelector('[data-pane="chat"]')?.textContent).toContain('Chat body'); expect(host.querySelector('[data-pane="editor"]')?.textContent).toContain('Editor body');
        const splitter = host.querySelector<HTMLButtonElement>('button[role="separator"]')!; expect(splitter.getAttribute('aria-valuenow')).toBe('50');
        act(() => { splitter.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true })); }); expect(splitter.getAttribute('aria-valuenow')).toBe('55');
        act(() => { host.querySelector<HTMLButtonElement>('button[data-pane-target="diff"]')!.click(); }); expect(onSection).toHaveBeenCalledWith('changes');
        act(() => { host.querySelector<HTMLButtonElement>('button[data-action="close-split"]')!.click(); }); expect(onCloseSplit).toHaveBeenCalledOnce();
    });
    it('requests a split from the current primary section', () => {
        const onSplit = vi.fn(); act(() => { render(<PaneDock activeSection="agent" onSection={vi.fn()} onSplit={onSplit}><p>Chat body</p></PaneDock>, host); });
        act(() => { host.querySelector<HTMLButtonElement>('button[data-action="split-pane"]')!.click(); }); expect(onSplit).toHaveBeenCalledOnce();
    });
});
