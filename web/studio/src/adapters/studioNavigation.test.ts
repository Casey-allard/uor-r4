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
    vi.spyOn(Element.prototype, 'getClientRects').mockImplementation(() => [new DOMRect()] as unknown as DOMRectList);
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><header><button id="btnMobileNav" aria-expanded="false">Menu</button></header><div class="shell-body"><aside id="navSidebar"><button class="nav-item" id="navAgent" data-section="agent">Agent</button><button class="nav-item" id="navWorkspace" data-section="workspace">Workspace</button><button class="nav-item" id="navModels" data-section="models">Connections</button></aside><button id="btnNavScrim" aria-label="Close navigation" hidden></button><main id="mainColumn"><button id="viewBtnChat">Chat</button><button id="viewBtnModel">Connections</button><button id="viewBtnEditor">Workspace</button><section id="modelView" data-view="model" aria-label="Connections"></section></main></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div><textarea id="editorText"></textarea><div id="settingsDialog" class="modal-backdrop" hidden><button id="btnCloseSettings">Close</button></div><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div></div>';
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
        expect(document.getElementById('navModels')!.textContent).toBe('Connections');
        expect(document.getElementById('modelView')!.getAttribute('aria-label')).toBe('Connections');
        expect(document.getElementById('modelView')!.getAttribute('data-view')).toBe('model');
        menu.click();
        expect(app.getAttribute('data-nav')).toBe('expanded');
        expect(menu.getAttribute('aria-expanded')).toBe('true');
        expect(scrim.hidden).toBe(false);
        expect(main.inert).toBe(true);
        expect(document.activeElement?.id).toBe('navAgent');
        scrim.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        expect(document.activeElement?.id).toBe('btnMobileNav');
        menu.click();
        expect(document.activeElement?.id).toBe('navAgent');
        document.getElementById('navWorkspace')!.click();
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        expect(document.activeElement?.id).toBe('btnMobileNav');
        expect(JSON.parse(window.localStorage.getItem('uor.studio.ui.v2') || '{}').nav).toBe('collapsed');
        menu.click();
        expect(scrim.hidden).toBe(false);
        expect(document.activeElement?.id).toBe('navAgent');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(app.getAttribute('data-nav')).toBe('collapsed');
        expect(menu.getAttribute('aria-expanded')).toBe('false');
        expect(scrim.hidden).toBe(true);
        expect(main.inert).toBe(false);
        expect(document.activeElement?.id).toBe('btnMobileNav');
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
        document.getElementById('viewBtnModel')!.focus();
        synchronize!();
        expect(scrim.hidden).toBe(false);
        expect(main.inert).toBe(true);
        expect(document.activeElement?.id).toBe('navAgent');
    });

    it('keeps the wired model id on the actual Connections navigation control', async () => {
        // @ts-expect-error Node's filesystem API is available in the Vitest runtime.
        const { readFileSync } = await import('node:fs');
        const html = readFileSync('index.html', 'utf8');
        expect(html).toMatch(/id="navModels"[^>]*data-section="models"|data-section="models"[^>]*id="navModels"/);
        expect(html).not.toContain('id="navConnections"');
    });

    it('starts closed on mobile after a stored desktop expansion', async () => {
        window.localStorage.setItem('uor.studio.ui.v2', JSON.stringify({ nav: 'expanded' }));
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
        expect(document.getElementById('btnMobileNav')!.getAttribute('aria-expanded')).toBe('false');
        expect(document.getElementById('btnNavScrim')!.hidden).toBe(true);
        expect(document.getElementById('mainColumn')!.inert).toBe(false);
        expect(document.activeElement?.id).not.toBe('navAgent');
    });

    it('contains Tab inside the mobile drawer and restores Menu on Escape', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const menu = document.getElementById('btnMobileNav')!;
        const first = document.getElementById('navAgent')!;
        const last = document.getElementById('navModels')!;
        menu.click();
        last.focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        expect(document.activeElement?.id).toBe('navAgent');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }));
        expect(document.activeElement?.id).toBe('navModels');
        menu.focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement?.id).toBe('btnMobileNav');
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
        expect(first).not.toBe(document.activeElement);
    });

    it('closes the drawer and restores Menu on a Connections destination', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const menu = document.getElementById('btnMobileNav')!;
        menu.click();
        document.getElementById('navModels')!.click();
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
        expect(document.activeElement?.id).toBe('btnMobileNav');
        expect(document.getElementById('navModels')!.getAttribute('aria-current')).toBe('page');
    });

    it('does not steal focus from an open modal during a live breakpoint change', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void } }).__studioUi;
        ui.openSettings();
        const registration = mobileMedia.addEventListener.mock.calls.find(([name]) => name === 'change');
        const synchronize = registration?.[1] as (() => void) | undefined;
        mobileMedia.matches = false;
        synchronize!();
        mobileMedia.matches = true;
        synchronize!();
        expect(document.activeElement?.id).toBe('btnCloseSettings');
        expect(document.activeElement?.id).not.toBe('navAgent');
    });

    it('synchronizes the top inspector toggle with the actual inspector state', async () => {
        const toggle = document.createElement('button');
        toggle.id = 'btnToggleInspectorTop';
        toggle.setAttribute('aria-expanded', 'true');
        document.body.appendChild(toggle);
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        expect(toggle.getAttribute('aria-expanded')).toBe('false');
        toggle.click();
        expect(toggle.getAttribute('aria-expanded')).toBe('true');
        toggle.click();
        expect(toggle.getAttribute('aria-expanded')).toBe('false');
    });
});


describe('deep-linked initial section history', () => {
    function addHistoryButtons() {
        document.body.insertAdjacentHTML('beforeend', '<button id="btnSectionBack">Back</button><button id="btnSectionForward">Forward</button><button id="navChanges" data-section="changes">Changes</button><button id="navTerminal" data-section="terminal">Terminal</button>');
    }

    it.each(['changes', 'terminal'])('opens #view=%s as the initial section without Chat history', async (section) => {
        addHistoryButtons();
        window.location.hash = `#view=${section}`;
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(120);
        expect(document.body.getAttribute('data-section')).toBe(section);
        expect((document.getElementById('btnSectionBack') as HTMLButtonElement).disabled).toBe(true);
        document.getElementById('navAgent')!.click();
        (document.getElementById('btnSectionBack') as HTMLButtonElement).click();
        expect(document.body.getAttribute('data-section')).toBe(section);
    });

    it('does not prepend Chat to an initial Files deep link', async () => {
        addHistoryButtons();
        window.location.hash = '#view=files';
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(120);
        expect(document.body.getAttribute('data-section')).toBe('workspace');
        expect((document.getElementById('btnSectionBack') as HTMLButtonElement).disabled).toBe(true);
    });
});
