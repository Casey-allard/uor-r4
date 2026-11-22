import preact from '@preact/preset-vite';
import { defineConfig } from 'vitest/config';

const studioDefaultTypedEntry = {
    name: 'studio-default-typed-entry',
    apply: 'build',
    transform(code: string, id: string) {
        if (!id.endsWith('/assets/js/typed_entry.js')) return null;
        const guard = "get('typed') === '1'";
        if (!code.includes(guard)) throw new Error('explicit preview guard is missing');
        return { code: code.replace(guard, "get('legacy') !== '1' && new URLSearchParams(globalThis.location?.search || '').get('typed') !== '0'"), map: null };
    },
};

export default defineConfig({
    base: './',
    plugins: [preact(), studioDefaultTypedEntry],
    test: {
        environment: 'jsdom',
        include: ['src/**/*.test.{ts,tsx}'],
    },
    build: {
        target: 'es2022',
        outDir: 'dist',
        emptyOutDir: true,
        rollupOptions: {
            input: 'index.html',
        },
    },
});
