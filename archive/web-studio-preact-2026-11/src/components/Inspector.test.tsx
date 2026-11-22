import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LifecycleStore } from '../state/lifecycle.js';
import type { StudioSettings, StudioUiApi } from '../adapters/workflow.js';
import { Inspector } from './Inspector.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); });

const attachments = [{ name: 'notes.md', content: 'not secret', source: 'editor' }];

function ui(): StudioUiApi {
    return {
        setView: vi.fn(), toast: vi.fn(),
        getSettings: vi.fn((): StudioSettings => ({ theme: 'dark', reduceMotion: false, fontSize: 13, tabSize: 4, wordWrap: false })),
        updateSettings: vi.fn(), updateStatus: vi.fn(), getAttachments: vi.fn(() => attachments),
        addAttachment: vi.fn(), removeAttachment: vi.fn(), clearAttachments: vi.fn(), templatePrompt: vi.fn(() => ''),
        clearSessions: vi.fn(() => ({ cleared: 0, total: 0 })), clearEditor: vi.fn(),
    };
}

describe('Inspector', () => {
    it('labels a previous generation as last rather than current', () => {
        const store = new LifecycleStore();
        store.dispatch({ type: 'api/selected', sourceId: 'api-a', endpoint: 'https://api.example.com/v1' });
        store.dispatch({ type: 'generation/started', source: { kind: 'api', id: 'api-a' } });
        store.dispatch({ type: 'generation/finished', outcome: 'complete', generationId: 1 });
        act(() => { render(<Inspector store={store} ui={ui()} open onClose={vi.fn()} />, host); });
        expect(host.textContent).toContain('Last generation');
    });
    it('renders details only when opened', () => {
        const store = new LifecycleStore();
        act(() => { render(<Inspector store={store} ui={ui()} open={false} onClose={vi.fn()} />, host); });
        expect(host.textContent).toBe('');
        act(() => { render(<Inspector store={store} ui={ui()} open onClose={vi.fn()} />, host); });
        expect(host.querySelector('#studio-inspector')).not.toBeNull();
        expect(host.textContent).toContain('notes.md');
        expect(host.textContent).toContain('No source selected');
        attachments.splice(0);
        attachments.push({ name: 'other.md', content: 'new context', source: 'workspace' });
        act(() => { document.dispatchEvent(new CustomEvent('uor-attachments-changed')); });
        expect(host.textContent).toContain('other.md');
        expect(host.textContent).not.toContain('notes.md');
    });
});
