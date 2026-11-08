import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { writeStudioCatalog } from '../../../scripts/studio/catalog.mjs';

const artifacts = new URL('../pkg/', import.meta.url);
const destination = new URL('../dist/pkg/', import.meta.url);

if (!existsSync(artifacts)) {
    throw new Error('Generated Studio artifacts are missing. Build the WASM engine and seed-5 demo before staging.');
}

mkdirSync(destination, { recursive: true });
for (const name of ['uor_r4_integer.js', 'uor_r4_integer_bg.wasm', 'synthetic-stack.lut', 'tokenizer.json']) {
    copyFileSync(new URL(`../pkg/${name}`, import.meta.url), new URL(`../dist/pkg/${name}`, import.meta.url));
}

const migrationRuntime = new URL('../assets/js/lifecycle_persistence.js', import.meta.url);
const migrationDestination = new URL('../dist/assets/js/lifecycle_persistence.js', import.meta.url);
mkdirSync(new URL('../dist/assets/js/', import.meta.url), { recursive: true });
copyFileSync(migrationRuntime, migrationDestination);

writeStudioCatalog({
    dest: fileURLToPath(new URL('../dist/', import.meta.url)),
    publicUrl: '',
    publicLabel: 'Public model',
    trainedManifestPath: '',
    receiptGate: '',
});

console.log('Staged generated Studio artifacts in dist/pkg.');
