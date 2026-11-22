import { describe, expect, it } from 'vitest';
import source from '../../index.html?raw';

function bootSource(): string {
    const from = source.indexOf('function boot()');
    const to = source.indexOf('// Interface hooks', from);
    if (from < 0 || to < 0) throw new Error('Missing boot source');
    return source.slice(from, to);
}

describe('conversation-first connectivity guidance', () => {
    it('loads only the bundled demo automatically when no model is loaded', () => {
        const boot = bootSource();
        expect(boot).toContain("active.kind === 'wasm' && active.id === 'bundled-demo' && !app.caps");
        expect(boot).toContain("ModelSources.loadBundled({ operationId: 1, sourceId: 'bundled-demo' })");
        expect(boot).not.toContain('ModelSources.loadPublished()');
    });

    it('distinguishes local runner, secure hosted tunnel, and the Pages API limit', () => {
        expect(source).toContain('GitHub Pages cannot host an API');
        expect(source).toContain('runner-served local Studio');
        expect(source).toContain('CORS alone does not make an HTTP loopback safe');
        expect(source).not.toContain('secure HTTPS tunnel');
        expect(source).toContain('native-runner.sh');
        expect(source).toContain('Try the offline demo');
    });
});
