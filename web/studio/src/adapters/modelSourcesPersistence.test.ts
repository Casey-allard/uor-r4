import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ModelSourcesApi } from '../../assets/js/model_sources.js';

const modulePath = '../../assets/js/model_sources.js';

describe('actual model-source persistence status', () => {
    afterEach(() => {
        vi.restoreAllMocks();
        document.body.innerHTML = '';
        window.localStorage.clear();
        delete (window as unknown as { UORModelSources?: ModelSourcesApi }).UORModelSources;
    });

    it('clears a previous storage error after a successful write', async () => {
        vi.resetModules();
        document.body.innerHTML = '<div id="mount"></div><div id="modelsVerifyRow"></div>';
        const { ModelSources } = await import(modulePath) as { ModelSources: ModelSourcesApi & { mount(container: HTMLElement): { unmount(): void } } };
        const mounted = ModelSources.mount(document.getElementById('mount')!);
        const provider = ModelSources.list().api[0];
        const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('storage full'); });
        expect(() => ModelSources.updateApi(provider.id, { model: 'changed' })).toThrow('Could not persist model sources');
        expect(document.getElementById('modelsVerifyRow')?.textContent).toContain('Could not persist model sources');
        setItem.mockRestore();
        ModelSources.updateApi(provider.id, { model: 'changed' });
        expect(document.getElementById('modelsVerifyRow')?.textContent).toBe('');
        mounted.unmount();
    });
});
