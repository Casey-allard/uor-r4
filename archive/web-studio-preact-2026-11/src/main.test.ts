import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const legacyWindow = window as unknown as {
    __studioUi?: unknown;
    __studioFiles?: unknown;
    __studioApp?: unknown;
};

describe('foundation public entry', () => {
    beforeEach(() => {
        vi.resetModules();
        document.querySelectorAll('#studio-typed-root').forEach((node) => node.remove());
        window.history.replaceState(null, '', '');
        delete legacyWindow.__studioUi;
        delete legacyWindow.__studioFiles;
        delete legacyWindow.__studioApp;
    });

    afterEach(() => {
        vi.useRealTimers();
        vi.restoreAllMocks();
        vi.unstubAllGlobals();
        document.querySelectorAll('#studio-typed-root').forEach((node) => node.remove());
        window.history.replaceState(null, '', '');
    });

    it('exposes the foundation API object', async () => {
        const { foundation } = await import('./main.js');
        expect(foundation).toBeTypeOf('object');
    });

    it('does not mount or mutate a pre-existing preview root with an explicit legacy selection', async () => {
        window.history.replaceState(null, '', '/?legacy=1');
        const root = document.createElement('div');
        root.id = 'studio-typed-root';
        root.hidden = true;
        document.body.appendChild(root);
        await import('./main.js');
        expect(root.hidden).toBe(true);
        expect(root.dataset.active).toBeUndefined();
        expect(root.querySelector('.studio-shell')).toBeNull();
    });

    it('executes the real initialization catch with incomplete workflow methods', async () => {
        vi.useFakeTimers();
        const root = document.createElement('div');
        root.id = 'studio-typed-root';
        root.hidden = true;
        document.body.appendChild(root);
        window.history.replaceState(null, '', '/?typed=1');
        legacyWindow.__studioFiles = { tabs: () => [], activeFile: () => null };
        legacyWindow.__studioUi = { getSettings: () => ({}) };
        const pending = import('./main.js');
        await vi.advanceTimersByTimeAsync(5000);
        await pending;
        const { typedPreviewRequested } = await import('./bootstrap.js');
        expect(typedPreviewRequested(window.location.search)).toBe(true);
        expect(root.hidden).toBe(false);
        expect(root.dataset.active).toBe('true');
        expect(root.textContent).toContain('The typed Studio could not start');
        expect(root.textContent).toContain('The existing Studio workflow services did not initialize.');
        expect(getComputedStyle(root).display).not.toBe('none');
    });
});
