import { describe, expect, it } from 'vitest';
import config from '../vite.config.js';

describe('Studio application build', () => {
    it('uses relative asset URLs and the real HTML application entry', () => {
        expect(config.base).toBe('./');
        const build = config.build!;
        expect(build.rollupOptions?.input).toBe('index.html');
        expect(build.lib).toBeUndefined();
    });
});
