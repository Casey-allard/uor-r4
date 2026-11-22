import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { AgentTranscript } from './AgentTranscript.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); });

describe('AgentTranscript', () => {
    it('renders role-specific messages and preserves message actions', () => {
        const copy = vi.fn(); const insert = vi.fn(); const download = vi.fn(); const retry = vi.fn();
        act(() => { render(<AgentTranscript messages={[{ role: 'user', content: 'Inspect this' }, { role: 'assistant', content: '**Done**' }]} canRun copy={copy} insert={insert} download={download} retry={retry} />, host); });
        expect(host.querySelectorAll('[data-role="user"]')).toHaveLength(1); expect(host.querySelectorAll('[data-role="assistant"]')).toHaveLength(1); expect(host.querySelector('strong')?.textContent).toBe('Done');
        const buttons = Array.from(host.querySelectorAll('button'));
        act(() => { buttons.find((button) => button.textContent === 'Copy')!.click(); }); expect(copy).toHaveBeenCalledWith('Inspect this');
        act(() => { buttons.find((button) => button.textContent === 'Insert into editor')!.click(); }); expect(insert).toHaveBeenCalledWith('Inspect this');
        act(() => { buttons.find((button) => button.textContent === 'Save as file')!.click(); }); expect(download).toHaveBeenCalledWith('Inspect this');
        act(() => { buttons.find((button) => button.textContent === 'Retry')!.click(); }); expect(retry).toHaveBeenCalledWith('Inspect this');
    });

    it('exposes a polite live region with the actual busy state', () => {
        const handlers = { copy: vi.fn(), insert: vi.fn(), download: vi.fn(), retry: vi.fn() };
        act(() => { render(<AgentTranscript messages={[]} canRun={false} busy={false} {...handlers} />, host); });
        expect(host.querySelector('.agent-transcript')?.getAttribute('aria-live')).toBe('polite');
        expect(host.querySelector('.agent-transcript')?.getAttribute('aria-busy')).toBe('false');
        act(() => { render(<AgentTranscript messages={[]} canRun={false} busy={true} {...handlers} />, host); });
        expect(host.querySelector('.agent-transcript')?.getAttribute('aria-busy')).toBe('true');
    });
});
