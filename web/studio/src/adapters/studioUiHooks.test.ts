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

describe('actual studio UI attachment preservation', () => {
    it('preserves long content and replaces/removes duplicates without silent truncation', () => {
        const long = 'x'.repeat(120001);
        ui.addAttachment({ name: 'long.txt', content: long, source: 'paste' });
        expect(ui.getAttachments()).toHaveLength(1);
        expect(ui.getAttachments()[0].content).toBe(long);
        ui.addAttachment({ name: 'long.txt', content: 'replacement', source: 'paste' });
        expect(ui.getAttachments()).toHaveLength(1);
        expect(ui.getAttachments()[0].content).toBe('replacement');
        ui.removeAttachment('long.txt');
        expect(ui.getAttachments()).toEqual([]);
    });
});

describe('actual studio UI native source labels', () => {
    it('keeps native loopback privacy and attachment notes local', () => {
        document.body.insertAdjacentHTML('beforeend', '<span id="privacyStatus">Native loopback</span><span id="chatSourceBadge">Native uor-chat</span><span id="privacyStatusText"></span><span id="composerModeNote"></span><span id="composerContextNote"></span><span id="ctxPrivacy"></span><span id="engineStatusText"></span><span id="envEngine"></span><span id="envModel">none</span><span id="envTokenizer">awaiting model</span><span id="ctxEngine"></span><span id="ctxTokenizer">awaiting model</span><span id="ctxModelLabel">No model loaded</span><span id="ctxModelMeta">Load the synthetic demo or the M1 trunk in Connections.</span><span id="modelStatusText"></span>');
        ui.updateStatus();
        expect(document.querySelector('#ctxPrivacy')!.textContent).not.toContain('leaves device');
        expect(document.querySelector('#engineStatusText')!.textContent).not.toContain('local WASM');
        expect(document.querySelector('#envEngine')!.textContent).toBe('native uor-chat');
        expect(document.querySelector('#envModel')!.textContent).toBe('native uor-chat');
        expect(document.querySelector('#envTokenizer')!.textContent).toBe('native artifact tokenizer');
        expect(document.querySelector('#ctxEngine')!.textContent).toBe('native uor-chat');
        expect(document.querySelector('#ctxTokenizer')!.textContent).toBe('native artifact tokenizer');
        expect(document.querySelector('#ctxModelLabel')!.textContent).toBe('Native uor-chat');
        expect(document.querySelector('#ctxModelMeta')!.textContent).toBe('Native loopback chat artifact · prompts stay on this device');
        expect(document.querySelector('#modelStatusText')!.textContent).toBe('native uor-chat');
        expect(document.querySelector('#privacyStatusText')!.textContent).toBe('Native loopback');
        expect(document.querySelector('#composerModeNote')!.textContent).toBe('Native uor-chat — prompts stay on this device');
        ui.addAttachment({ name: 'note.txt', content: 'text', source: 'file' });
        expect(document.querySelector('#composerContextNote')!.textContent).toContain('stays on this device');
        expect(document.querySelector('#composerContextNote')!.textContent).not.toContain('leaves device');
        ui.clearAttachments();
        document.querySelector('#privacyStatus')!.remove();
        document.querySelector('#chatSourceBadge')!.remove();
        document.querySelector('#privacyStatusText')!.remove();
        document.querySelector('#composerModeNote')!.remove();
        document.querySelector('#composerContextNote')!.remove();
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
