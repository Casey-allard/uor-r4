import { cpSync, copyFileSync, existsSync, mkdirSync } from 'node:fs';

const artifacts = new URL('../pkg/', import.meta.url);
const destination = new URL('../dist/pkg/', import.meta.url);

if (!existsSync(artifacts)) {
    throw new Error('Generated Studio artifacts are missing. Build the WASM engine and seed-5 demo before staging.');
}

mkdirSync(destination, { recursive: true });
cpSync(artifacts, destination, { recursive: true });

const catalog = new URL('../models.json', import.meta.url);
if (existsSync(catalog)) copyFileSync(catalog, new URL('../dist/models.json', import.meta.url));

console.log('Staged generated Studio artifacts in dist/pkg.');
