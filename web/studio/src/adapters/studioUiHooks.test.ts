import { afterAll, beforeAll, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi } from './workflow.js';

let ui: StudioUiApi;
const legacyWindow = window as unknown as { __studioUi: StudioUiApi };
const counts: number[] = [];
const listSessions = vi.fn(() => [{ id: 'one' }, { id: 'two' }]);
const deleteSession = vi.fn((id: string) => id === 'one');

beforeAll(async () => {
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    legacyWindow.__studioUi = { getMode: () => 'api', listSessions, deleteSession } as unknown as StudioUiApi;
    document.body.innerHTML = '<div id="app"></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div>';
    document.addEventListener('uor-attachments-changed', (event) => {
        counts.push((event as CustomEvent<{ count: number }>).detail.count);
    });
    await import('../../assets/js/studio_ui.js');
    await vi.waitFor(() => expect(legacyWindow.__studioUi.addAttachment).toBeTypeOf('function'));
    counts.length = 0;
    ui = legacyWindow.__studioUi;
});

afterAll(() => {
    vi.clearAllTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
});

describe('actual studio UI attachment hooks', () => {
    it('emits attachment changes on add, remove, and clear', () => {
        ui.addAttachment({ name: 'a.md', content: 'a', source: 'editor' });
        ui.removeAttachment('a.md');
        ui.addAttachment({ name: 'b.md', content: 'b', source: 'editor' });
        ui.clearAttachments();
        expect(counts).toEqual([1, 0, 1, 0]);
        expect(ui.getAttachments()).toEqual([]);
    });
});

describe('actual studio UI settings hooks', () => {
    it('clamps finite numeric values and applies the editor font size', () => {
        const changed = vi.fn();
        window.addEventListener('uor-settings-changed', changed);
        ui.updateSettings({ fontSize: 0, tabSize: 0 });
        expect(ui.getSettings()).toMatchObject({ fontSize: 10, tabSize: 1 });
        expect(document.documentElement.style.getPropertyValue('--editor-font-size')).toBe('10px');
        ui.updateSettings({ fontSize: 99, tabSize: 99, wordWrap: true, reduceMotion: true });
        expect(ui.getSettings()).toMatchObject({ fontSize: 24, tabSize: 8, wordWrap: true, reduceMotion: true });
        expect(document.documentElement.style.getPropertyValue('--editor-font-size')).toBe('24px');
        expect(document.documentElement.getAttribute('data-reduce-motion')).toBe('1');
        expect(JSON.parse(window.localStorage.getItem('uor.studio.settings.v1') || '{}')).toMatchObject({ fontSize: 24, tabSize: 8, wordWrap: true, reduceMotion: true });
        expect(changed).toHaveBeenCalledTimes(2);
        expect(changed.mock.calls[1][0].detail).toMatchObject({ fontSize: 24, tabSize: 8, wordWrap: true, reduceMotion: true });
        window.removeEventListener('uor-settings-changed', changed);
    });
});

describe('actual studio UI session clearing', () => {
    it('returns the real per-session deletion result', () => {
        expect(ui.clearSessions()).toEqual({ cleared: 1, total: 2 });
        expect(deleteSession).toHaveBeenCalledWith('one', false);
        expect(deleteSession).toHaveBeenCalledWith('two', false);
    });
});
