import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const studioUiModule = '../../assets/js/studio_ui.js';

beforeEach(() => {
    vi.useFakeTimers();
    window.localStorage.clear();
    window.location.hash = '';
    window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
    document.body.innerHTML = '<div id="app" data-nav="collapsed"><button id="btnMobileNav">Menu</button><aside><button id="navAgent" data-section="agent">Agent</button><button id="navModels" data-section="models">Models</button></aside><main id="mainColumn"><button id="viewBtnModel">Models</button><section id="modelView"><div id="modelSourcesPanel"><div class="providers-head"></div><div id="modelSourceMount"><section class="source-group" id="localGroup"><h3>Local WASM engine</h3><article class="source-card" id="bundledCard"><button data-action="load-bundled">Load bundled demo</button></article><article class="source-card" id="localFileCard"><input type="file"></article><article class="source-card" id="urlCard"><button data-action="load-url">Load URL</button></article></section><section class="source-group" id="trainedGroup"><h3>Trained M1 model</h3><article class="source-card">M1</article></section><section class="source-group" id="researchGroup"><h3>Research artifacts (experimental)</h3><article class="source-card">FBS</article></section><section class="source-group" id="apiGroup"><h3>OpenAI-compatible API</h3><article class="source-card">Custom endpoint<button id="addApiButton" data-action="add-api" type="button">Add provider</button></article></section></div></div></section></main><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div></div>';
    vi.resetModules();
});

afterEach(() => {
    vi.clearAllTimers();
    vi.restoreAllMocks();
    vi.useRealTimers();
    document.body.innerHTML = '';
    window.localStorage.clear();
    delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    delete (Element.prototype as Partial<Element>).scrollIntoView;
});

describe('simplified Studio connections', () => {
    it('moves existing sources into UOR Chat, custom providers, and diagnostics without replacing behavior', async () => {
        const bundled = document.querySelector<HTMLButtonElement>('[data-action="load-bundled"]')!;
        const localFile = document.getElementById('localFileCard')!;
        const urlCard = document.getElementById('urlCard')!;
        const trained = document.getElementById('trainedGroup')!;
        const research = document.getElementById('researchGroup')!;
        const addApi = document.getElementById('addApiButton')!;
        const clicked = vi.fn();
        bundled.addEventListener('click', clicked);
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(400);

        const local = document.querySelector<HTMLElement>('.models-section[data-kind="local"]')!;
        const api = document.querySelector<HTMLElement>('.models-section[data-kind="api"]')!;
        const diagnostics = document.querySelector<HTMLDetailsElement>('#artifactDiagnostics')!;
        expect(local.textContent).toContain('UOR Chat — experimental');
        expect(api.textContent).toContain('Custom API providers');
        expect(api.contains(addApi)).toBe(true);
        expect(addApi.tagName).toBe('BUTTON');
        expect(diagnostics.open).toBe(false);
        expect(diagnostics.contains(trained)).toBe(true);
        expect(diagnostics.contains(research)).toBe(true);
        expect(diagnostics.contains(localFile)).toBe(true);
        expect(diagnostics.contains(urlCard)).toBe(true);
        expect(document.getElementById('mcpServers')).toBeNull();
        expect(bundled.dataset.action).toBe('load-bundled');
        expect(bundled.textContent).toBe('Connect UOR Chat');
        expect(local.contains(bundled)).toBe(true);
        bundled.click();
        expect(clicked).toHaveBeenCalledOnce();
    });

    it('opens artifact diagnostics before focusing the URL input', async () => {
        const input = document.createElement('input');
        input.id = 'modelUrlInput';
        document.getElementById('urlCard')!.appendChild(input);
        const palette = document.createElement('div');
        palette.id = 'commandPalette';
        palette.innerHTML = '<input id="paletteInput"><div id="paletteList"></div>';
        document.body.appendChild(palette);
        Element.prototype.scrollIntoView = vi.fn();
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(400);
        const diagnostics = document.querySelector<HTMLDetailsElement>('#artifactDiagnostics')!;
        expect(diagnostics.open).toBe(false);
        const ui = (window as unknown as { __studioUi: { promptUrl(): boolean; openPalette(): void } }).__studioUi;
        expect(ui.promptUrl()).toBe(true);
        expect(diagnostics.open).toBe(true);
        expect(document.activeElement).toBe(input);
        diagnostics.open = false;
        ui.openPalette();
        const paletteInput = document.getElementById('paletteInput') as HTMLInputElement;
        paletteInput.value = 'Load model from URL';
        paletteInput.dispatchEvent(new Event('input', { bubbles: true }));
        expect(paletteInput.getAttribute('aria-activedescendant')).toBe('palette-item-0');
        expect(document.getElementById('palette-item-0')!.textContent).toContain('Load model from URL');
        document.getElementById('palette-item-0')!.click();
        await vi.advanceTimersByTimeAsync(20);
        expect(diagnostics.open).toBe(true);
        expect(document.activeElement).toBe(input);
    });
});
