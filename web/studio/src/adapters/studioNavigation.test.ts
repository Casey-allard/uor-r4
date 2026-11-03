import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { StudioUiApi } from './workflow.js';

const studioUiModule = '../../assets/js/studio_ui.js';
const addDocumentListener = document.addEventListener;
let mobileMedia: { matches: boolean; addEventListener: ReturnType<typeof vi.fn> };
let keydownListeners: Array<{ listener: EventListenerOrEventListenerObject; options?: boolean | AddEventListenerOptions }>;

beforeEach(() => {
    keydownListeners = [];
    vi.spyOn(document, 'addEventListener').mockImplementation((type: string, listener: EventListenerOrEventListenerObject, options?: boolean | AddEventListenerOptions) => {
        if (type === 'keydown') keydownListeners.push({ listener, options });
        addDocumentListener.call(document, type, listener, options);
    });
    vi.useFakeTimers();
    window.localStorage.clear();
    window.location.hash = '';
    mobileMedia = { matches: true, addEventListener: vi.fn() };
    window.matchMedia = vi.fn((query: string) => query === '(max-width: 900px)' ? mobileMedia : { matches: false, addEventListener: vi.fn() }) as unknown as typeof window.matchMedia;
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><header><button id="btnMobileNav" aria-expanded="false">Menu</button></header><div class="shell-body"><aside id="navSidebar"><button id="navAgent" data-section="agent">Agent</button><button id="navWorkspace" data-section="workspace">Workspace</button></aside><button id="btnNavScrim" aria-label="Close navigation" hidden></button><main id="mainColumn"><button id="viewBtnChat">Chat</button><button id="viewBtnModel">Models</button><button id="viewBtnEditor">Workspace</button></main></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div><textarea id="editorText"></textarea><div id="settingsDialog" class="modal-backdrop" hidden></div><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div></div>';
    vi.resetModules();
});

afterEach(() => {
    keydownListeners.forEach(({ listener, options }) => document.removeEventListener('keydown', listener, options));
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    document.documentElement.classList.remove('nav-open');
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: StudioUiApi }).__studioUi;
    delete (Element.prototype as Partial<Element>).scrollIntoView;
});

describe('actual mobile navigation dismissal', () => {
    it('coordinates scrim, ARIA, section, and Escape dismissal', async () => {
        await import(studioUiModule);
        if (document.readyState === 'loading') document.dispatchEvent(new Event('DOMContentLoaded'));
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const app = document.getElementById('app')!;
        const menu = document.getElementById('btnMobileNav')!;
        const scrim = document.getElementById('btnNavScrim')!;
        const main = document.getElementById('mainColumn')!;
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        menu.click();
        expect(app.getAttribute('data-nav')).toBe('expanded');
        expect(menu.getAttribute('aria-expanded')).toBe('true');
        expect(scrim.hidden).toBe(false);
        expect(main.inert).toBe(true);
        scrim.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        menu.click();
        document.getElementById('navWorkspace')!.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        expect(JSON.parse(window.localStorage.getItem('uor.studio.ui.v2') || '{}').nav).toBe('collapsed');
        menu.click();
        expect(scrim.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(menu.getAttribute('aria-expanded')).toBe('false');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
    });

    it('closes an open modal or palette before the mobile navigation drawer', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const app = document.getElementById('app')!;
        const menu = document.getElementById('btnMobileNav')!;
        const ui = (window as unknown as { __studioUi: StudioUiApi & { openSettings(): void; openPalette(): void } }).__studioUi;
        menu.click();
        Element.prototype.scrollIntoView = vi.fn();
        ui.openSettings();
        const settings = document.getElementById('settingsDialog')!;
        expect(settings.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(settings.hidden).toBe(true);
        expect(app.getAttribute('data-nav')).toBe('expanded');
        ui.openPalette();
        const palette = document.getElementById('commandPalette')!;
        expect(palette.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(palette.hidden).toBe(true);
        expect(app.getAttribute('data-nav')).toBe('expanded');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(app.getAttribute('data-nav')).toBe('collapsed');
    });

    it('resynchronizes scrim and inert state across the mobile breakpoint', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const menu = document.getElementById('btnMobileNav')!;
        const scrim = document.getElementById('btnNavScrim')!;
        const main = document.getElementById('mainColumn')!;
        const registration = mobileMedia.addEventListener.mock.calls.find(([name]) => name === 'change');
        const synchronize = registration?.[1] as (() => void) | undefined;
        expect(synchronize).toBeTypeOf('function');
        menu.click();
        mobileMedia.matches = false;
        synchronize!();
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        mobileMedia.matches = true;
        synchronize!();
        expect(scrim.hidden).toBe(false);
        expect(main.inert).toBe(true);
    });
});
