import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const studioUiModule = '../../assets/js/studio_ui.js';
const modelSourcesModule = '../../assets/js/model_sources.js';

describe('native setup guide rendering', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        window.localStorage.clear();
        window.matchMedia = vi.fn(() => ({ matches: false, addEventListener: vi.fn() })) as unknown as typeof window.matchMedia;
        document.body.innerHTML = '<section id="modelView"><div id="modelSourcesPanel"><div class="providers-head"></div><div id="modelSourceMount"></div></div></section><div id="toastRegion"></div><div id="statusClock"></div><div id="conversationList"></div><div id="chatMessages"></div>';
        vi.resetModules();
    });

    afterEach(() => {
        vi.clearAllTimers();
        vi.restoreAllMocks();
        vi.useRealTimers();
        document.body.innerHTML = '';
        delete (window as unknown as { __studioUi?: unknown }).__studioUi;
    });

    it('keeps one visible guide after a real model-source rerender', async () => {
        const { mountModelSources } = await import(modelSourcesModule);
        const mounted = mountModelSources(document.getElementById('modelSourceMount')!);
        await import(studioUiModule);
        document.dispatchEvent(new CustomEvent('uor-studio-ready'));
        await vi.advanceTimersByTimeAsync(400);
        expect(document.querySelectorAll('#nativeRunnerGuide').length).toBe(1);
        mounted.render();
        await vi.advanceTimersByTimeAsync(400);
        expect(document.querySelectorAll('#nativeRunnerGuide').length).toBe(1);
        mounted.unmount();
    });
});
