import { describe, expect, it } from 'vitest';
import config from '../vite.config.js';

type EntryTransform = (code: string, id: string) => { code: string };

describe('Studio application build', () => {
    it('uses relative asset URLs and the real HTML application entry', () => {
        expect(config.base).toBe('./');
        const build = config.build!;
        expect(build.rollupOptions?.input).toBe('index.html');
        expect(build.lib).toBeUndefined();
    });

    it('defaults only the built typed entry and rejects a missing preview guard', () => {
        const plugin = config.plugins?.find((candidate) => candidate && typeof candidate === 'object' && 'name' in candidate && candidate.name === 'studio-default-typed-entry') as { transform: EntryTransform; apply?: string };
        expect(plugin).toBeDefined();
        expect(plugin.apply).toBe('build');
        const source = "if (new URLSearchParams(globalThis.location?.search || '').get('typed') === '1') { import('../../src/main.ts'); }";
        const transformed = plugin.transform(source, '/fixture/web/studio/assets/js/typed_entry.js');
        expect(transformed.code).toContain("get('legacy') !== '1' && new URLSearchParams(globalThis.location?.search || '').get('typed') !== '0'");
        expect(source).toContain("get('typed') === '1'");
        expect(() => plugin.transform('export {}', '/fixture/web/studio/assets/js/typed_entry.js')).toThrow('explicit preview guard is missing');
    });
});
