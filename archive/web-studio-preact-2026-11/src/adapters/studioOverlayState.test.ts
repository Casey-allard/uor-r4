import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const studioUiModule = '../../assets/js/studio_ui.js';
const addDocumentListener = document.addEventListener;
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
    window.matchMedia = vi.fn((query: string) => query === '(max-width: 900px)' ? { matches: true, addEventListener: vi.fn() } : { matches: false, addEventListener: vi.fn() }) as unknown as typeof window.matchMedia;
    vi.spyOn(Element.prototype, 'getClientRects').mockImplementation(() => [new DOMRect()] as unknown as DOMRectList);
    Element.prototype.scrollIntoView = vi.fn();
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><header><button id="btnMobileNav" aria-expanded="false">Menu</button></header><aside id="navSidebar"><button class="nav-item" id="navAgent" data-section="agent">Agent</button><button class="nav-item" id="navWorkspace" data-section="workspace">Workspace</button><button class="nav-item" id="navModels" data-section="models">Connections</button></aside><button id="btnNavScrim" hidden></button><main id="mainColumn"><input id="chatInput"><section id="modelView" data-view="model"></section></main></div><div id="settingsDialog" class="modal-backdrop" hidden><button id="btnCloseSettings">Close</button></div><div id="helpDialog" class="modal-backdrop" hidden><button id="btnCloseHelp">Close help</button></div><div id="commandPalette" hidden><input id="paletteInput"><div id="paletteList"></div></div><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div><div id="modelSourceMount"></div>';
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
});

describe('overlay and view-state coordination', () => {
    it('keeps overlays exclusive, traps palette Tab, and closes the top overlay first', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openPalette(): void } }).__studioUi;
        const menu = document.getElementById('btnMobileNav')!;
        const settings = document.getElementById('settingsDialog')!;
        const palette = document.getElementById('commandPalette')!;
        menu.click();
        ui.openSettings();
        expect(settings.hidden).toBe(false);
        ui.openPalette();
        expect(settings.hidden).toBe(true);
        expect(palette.hidden).toBe(false);
        ui.openSettings();
        expect(palette.hidden).toBe(true);
        expect(settings.hidden).toBe(false);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(settings.hidden).toBe(true);
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('expanded');
        ui.openPalette();
        const input = document.getElementById('paletteInput') as HTMLInputElement;
        const commands = document.querySelectorAll<HTMLButtonElement>('#paletteList button');
        const lastCommand = commands[commands.length - 1];
        lastCommand.focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(input);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(lastCommand);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(palette.hidden).toBe(true);
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('expanded');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
        expect(document.activeElement).toBe(menu);
    });

    it('synchronizes nav current state for programmatic view changes', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'model' } }));
        expect(document.getElementById('navModels')!.getAttribute('aria-current')).toBe('page');
        expect(document.getElementById('navAgent')!.getAttribute('aria-current')).toBe('false');
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'workspace' } }));
        expect(document.getElementById('navWorkspace')!.getAttribute('aria-current')).toBe('page');
        expect(document.getElementById('navModels')!.getAttribute('aria-current')).toBe('false');
        document.dispatchEvent(new CustomEvent('uor-view-changed', { detail: { view: 'chat' } }));
        expect(document.getElementById('navAgent')!.getAttribute('aria-current')).toBe('page');
    });

    it('keeps real labels available and prevents boot autofocus', async () => {
        // @ts-expect-error Node's filesystem API is available in the Vitest runtime.
        const { readFileSync } = await import('node:fs');
        const html = readFileSync('index.html', 'utf8');
        expect(html).toMatch(/class="sr-only" for="chatInput"/);
        expect(html).toMatch(/class="sr-only" for="paletteInput"/);
        expect(html).toContain("setView('chat', { focus: false })");
        expect(html).not.toContain('under Model sources');
        expect(html).not.toContain("when Claude's API endpoint is available");
        const readme = readFileSync('README.md', 'utf8');
        expect(readme).not.toContain('**Models** is four labeled sections');
    });

    it('closes other modals and blocks background navigation shortcuts', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openHelp(): void } }).__studioUi;
        const settings = document.getElementById('settingsDialog')!;
        const help = document.getElementById('helpDialog')!;
        ui.openSettings();
        ui.openHelp();
        expect(settings.hidden).toBe(true);
        expect(help.hidden).toBe(false);
        ui.openSettings();
        expect(help.hidden).toBe(true);
        expect(settings.hidden).toBe(false);
        await vi.advanceTimersByTimeAsync(20);
        expect(document.activeElement?.id).toBe('btnCloseSettings');
        document.body.dataset.view = 'model';
        document.dispatchEvent(new KeyboardEvent('keydown', { key: '1', metaKey: true, bubbles: true, cancelable: true }));
        expect(document.body.dataset.view).toBe('model');
        expect(document.activeElement?.id).not.toBe('chatInput');
        ui.openHelp();
        expect(settings.hidden).toBe(true);
        expect(help.hidden).toBe(false);
        await vi.advanceTimersByTimeAsync(20);
        expect(document.activeElement?.id).toBe('btnCloseHelp');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(help.hidden).toBe(true);
        expect(document.querySelectorAll('.modal-backdrop:not([hidden])')).toHaveLength(0);
    });

    it('restores the opener after switching overlays and closes transient menus', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openPalette(): void; openHelp(): void } }).__studioUi;
        const opener = document.createElement('button');
        opener.id = 'btnSettings';
        document.body.appendChild(opener);
        opener.focus();
        const sessionTrigger = document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!;
        sessionTrigger.click();
        expect(document.querySelector('.session-popover')).not.toBeNull();
        const templates = document.createElement('div');
        templates.id = 'templateMenu';
        document.body.appendChild(templates);
        ui.openSettings();
        ui.openPalette();
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(templates.hidden).toBe(true);
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(sessionTrigger);
        opener.focus();
        ui.openSettings();
        ui.openHelp();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(opener);
    });

    it('preserves return focus when reopening the same modal', async () => {
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const ui = (window as unknown as { __studioUi: { openSettings(): void; openHelp(): void } }).__studioUi;
        const opener = document.createElement('button');
        opener.id = 'btnSettings';
        document.body.appendChild(opener);
        opener.focus();
        ui.openSettings();
        ui.openSettings();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(opener);
        opener.focus();
        ui.openHelp();
        ui.openHelp();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(opener);
    });

    it('restores the session trigger and removes delete confirmation on Escape', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const trigger = document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!;
        const deleteItem = () => Array.from(document.querySelectorAll<HTMLButtonElement>('.session-popover-item')).find((button) => button.textContent === 'Delete')!;
        const cancel = () => document.querySelector<HTMLButtonElement>('.modal-backdrop .ghost-btn')!;
        trigger.click();
        deleteItem().click();
        expect(document.getElementById('confirmDeleteTitle')).not.toBeNull();
        expect(document.activeElement).toBe(cancel());
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.getElementById('confirmDeleteTitle')).toBeNull();
        expect(document.activeElement).toBe(trigger);
        trigger.click();
        deleteItem().click();
        cancel().click();
        expect(document.getElementById('confirmDeleteTitle')).toBeNull();
        expect(document.activeElement).toBe(trigger);
    });

    it('makes session and template menus exclusive and keyboard-dismissible', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        const build = document.createElement('button');
        build.id = 'btnBuildTask';
        document.body.appendChild(build);
        const templates = document.createElement('div');
        templates.id = 'templateMenu';
        templates.hidden = true;
        document.body.appendChild(templates);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const trigger = document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!;
        trigger.click();
        build.click();
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(templates.hidden).toBe(false);
        expect(build.getAttribute('aria-expanded')).toBe('true');
        trigger.click();
        expect(templates.hidden).toBe(true);
        expect(build.getAttribute('aria-expanded')).toBe('false');
        expect(document.querySelector('.session-popover')).not.toBeNull();
        expect(trigger.getAttribute('aria-expanded')).toBe('true');
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(document.activeElement).toBe(trigger);
        expect(trigger.getAttribute('aria-expanded')).toBe('false');
    });

    it('keeps Tab inside a session menu when the mobile drawer is expanded', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        document.getElementById('btnMobileNav')!.click();
        document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!.click();
        const items = Array.from(document.querySelectorAll<HTMLButtonElement>('.session-popover-item'));
        expect(document.activeElement).toBe(items[0]);
        items[items.length - 1].focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(items[0]);
        items[0].focus();
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, bubbles: true, cancelable: true }));
        expect(document.activeElement).toBe(items[items.length - 1]);
    });

    it('preserves the clicked composer focus after dismissing a session menu outside', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const trigger = document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!;
        trigger.click();
        await vi.advanceTimersByTimeAsync(0);
        const composer = document.getElementById('chatInput')!;
        composer.focus();
        composer.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(document.activeElement).toBe(composer);
        expect(document.activeElement).not.toBe(trigger);
    });

    it('preserves the clicked composer focus after dismissing the template menu outside', async () => {
        const build = document.createElement('button');
        build.id = 'btnBuildTask';
        document.body.appendChild(build);
        const templates = document.createElement('div');
        templates.id = 'templateMenu';
        templates.hidden = true;
        templates.innerHTML = '<button type="button">Template</button>';
        document.body.appendChild(templates);
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        build.click();
        await vi.advanceTimersByTimeAsync(0);
        const composer = document.getElementById('chatInput')!;
        composer.focus();
        composer.dispatchEvent(new MouseEvent('click', { bubbles: true }));
        expect(templates.hidden).toBe(true);
        expect(document.activeElement).toBe(composer);
        expect(document.activeElement).not.toBe(build);
    });

    it('opens only the rename input when F2 is pressed on a session', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        const main = document.querySelector<HTMLButtonElement>('.session-main')!;
        main.focus();
        main.dispatchEvent(new KeyboardEvent('keydown', { key: 'F2', bubbles: true, cancelable: true }));
        expect(document.querySelector('.session-popover')).toBeNull();
        expect(document.activeElement).toBe(document.querySelector('.conv-rename'));
        expect(document.querySelectorAll('.conv-rename')).toHaveLength(1);
    });

    it('blocks navigation shortcuts while session or template menus own focus', async () => {
        const rows = document.createElement('div');
        rows.id = 'sessionRowsV2';
        document.body.appendChild(rows);
        const build = document.createElement('button');
        build.id = 'btnBuildTask';
        document.body.appendChild(build);
        const templates = document.createElement('div');
        templates.id = 'templateMenu';
        templates.hidden = true;
        templates.innerHTML = '<button type="button">Template</button>';
        document.body.appendChild(templates);
        (window as unknown as { __studioUi: { listSessions(): unknown[] } }).__studioUi = {
            listSessions: () => [{ id: 's1', title: 'Session', createdAt: Date.now(), updatedAt: Date.now(), messageCount: 1, preview: 'preview' }],
        };
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(0);
        document.body.dataset.section = 'agent';
        document.querySelector<HTMLButtonElement>('.session-row .conv-menu')!.click();
        const sessionFirst = document.querySelector<HTMLButtonElement>('.session-popover-item')!;
        for (const key of ['2', 'b']) {
            document.dispatchEvent(new KeyboardEvent('keydown', { key, metaKey: true, bubbles: true, cancelable: true }));
            expect(document.body.dataset.section).toBe('agent');
            expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
            expect(document.activeElement).toBe(sessionFirst);
        }
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
        build.click();
        const templateFirst = templates.querySelector('button')!;
        for (const key of ['2', 'b']) {
            document.dispatchEvent(new KeyboardEvent('keydown', { key, metaKey: true, bubbles: true, cancelable: true }));
            expect(document.body.dataset.section).toBe('agent');
            expect(document.getElementById('app')!.getAttribute('data-nav')).toBe('collapsed');
            expect(document.activeElement).toBe(templateFirst);
        }
    });
});
