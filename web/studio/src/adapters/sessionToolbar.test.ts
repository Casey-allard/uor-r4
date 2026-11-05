import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

beforeEach(async () => {
    vi.resetModules();
    window.localStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();
    document.body.innerHTML = `
        <div id="app" data-nav="collapsed"></div>
        <button id="btnSessionNav" type="button">Sidebar</button>
        <button id="btnSectionBack" type="button">Back</button>
        <button id="btnSectionForward" type="button">Forward</button>
        <select id="sessionSwitcher" aria-label="Switch session"></select>
        <button id="btnSessionChanges" type="button">Changes</button>
        <button id="btnSessionTerminal" type="button">Terminal</button>
        <button id="btnSessionFiles" type="button">Files</button>
        <button id="btnSessionNew" type="button">New chat</button>
        <button id="btnSessionOptions" type="button">View options</button>
        <div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div>
        <div id="sessionRowsV2"></div><div id="conversationList"></div>
        <div id="chatMessages"></div><div id="modelSourceMount"></div><div id="statusClock"></div>`;
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    (window as unknown as { __studioUi: unknown }).__studioUi = {
        listSessions: () => [
            { id: 'a', title: 'Alpha', createdAt: 1, updatedAt: 1, messageCount: 1, preview: 'first' },
            { id: 'b', title: 'Beta', createdAt: 2, updatedAt: 2, messageCount: 2, preview: 'second' },
        ],
        activeSessionId: () => 'a',
        selectSession: vi.fn(),
        newSession: vi.fn(),
    };
    await import('../../assets/js/studio_ui.js');
    document.dispatchEvent(new CustomEvent('uor-studio-ready'));
    await vi.waitFor(() => expect(document.getElementById('sessionSwitcher')?.querySelectorAll('option').length).toBe(2));
});

afterEach(() => {
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    document.body.innerHTML = '';
    vi.restoreAllMocks();
});

describe('Claude-style session toolbar', () => {
    it('opens real sections and moves backward and forward through their history', () => {
        const back = document.getElementById('btnSectionBack') as HTMLButtonElement;
        const forward = document.getElementById('btnSectionForward') as HTMLButtonElement;
        expect(back.disabled).toBe(true);
        expect(forward.disabled).toBe(true);
        (document.getElementById('btnSessionChanges') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe('changes');
        expect(back.disabled).toBe(false);
        back.click();
        expect(document.body.getAttribute('data-section')).toBe('agent');
        forward.click();
        expect(document.body.getAttribute('data-section')).toBe('changes');
        (document.getElementById('btnSessionTerminal') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe('terminal');
        back.click();
        expect(document.body.getAttribute('data-section')).toBe('changes');
        forward.click();
        expect(document.body.getAttribute('data-section')).toBe('terminal');
        (document.getElementById('btnSessionFiles') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe('workspace');
        back.click();
        expect(document.body.getAttribute('data-section')).toBe('terminal');
    });

    it('switches sessions and opens navigation and view options', () => {
        const sessions = document.getElementById('sessionSwitcher') as HTMLSelectElement;
        expect(sessions.value).toBe('a');
        sessions.value = 'b';
        sessions.dispatchEvent(new Event('change', { bubbles: true }));
        const ui = (window as unknown as { __studioUi: { selectSession: ReturnType<typeof vi.fn>; newSession: ReturnType<typeof vi.fn> } }).__studioUi;
        expect(ui.selectSession).toHaveBeenCalledWith('b');
        (document.getElementById('btnSessionNav') as HTMLButtonElement).click();
        expect(document.getElementById('app')?.getAttribute('data-nav')).toBe('expanded');
        (document.getElementById('btnSessionOptions') as HTMLButtonElement).click();
        expect(document.getElementById('commandPalette')?.hasAttribute('hidden')).toBe(false);
        (document.getElementById('btnSessionNew') as HTMLButtonElement).click();
        expect(ui.newSession).toHaveBeenCalledOnce();
        expect(document.body.getAttribute('data-section')).toBe('agent');
    });
});
