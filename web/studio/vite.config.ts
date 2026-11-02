import preact from '@preact/preset-vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
    plugins: [preact()],
    test: {
        environment: 'jsdom',
        include: ['src/**/*.test.{ts,tsx}'],
    },
    build: {
        target: 'es2022',
        outDir: 'dist',
        emptyOutDir: true,
        lib: {
            entry: 'src/main.ts',
            formats: ['es'],
            fileName: 'studio-foundation',
        },
    },
});
