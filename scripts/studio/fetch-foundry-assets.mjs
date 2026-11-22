#!/usr/bin/env node
// Fetch and SHA-256-verify the locked upstream Foundry binary assets listed in
// web/studio/foundry-assets.lock.json into web/studio/foundry/.
//
// Usage:
//   node scripts/studio/fetch-foundry-assets.mjs [--check] [--seed-dir <dir>]
//
// Blobs are cached by SHA-256 under local/foundry-assets-cache/<sha256> inside the
// primary repo checkout so repeat builds and worktrees avoid redundant downloads.

import { createHash } from 'node:crypto';
import {
    copyFileSync,
    existsSync,
    mkdirSync,
    readFileSync,
    renameSync,
    statSync,
    writeFileSync,
} from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '../..');
const lockPath = resolve(repoRoot, 'web/studio/foundry-assets.lock.json');
const foundryRoot = resolve(repoRoot, 'web/studio/foundry');

// Share the local content-addressed cache across worktrees via the repo common root.
function defaultCacheDir() {
    if (process.env.UOR_FOUNDRY_ASSETS_CACHE) {
        return resolve(process.env.UOR_FOUNDRY_ASSETS_CACHE);
    }
    const worktreesMarker = `${resolve(repoRoot)}`;
    const idx = worktreesMarker.indexOf('/.worktrees/');
    const mainRoot = idx >= 0 ? worktreesMarker.slice(0, idx) : repoRoot;
    return resolve(mainRoot, 'local/foundry-assets-cache');
}

function parseArgs(argv) {
    let checkOnly = false;
    let seedDir = process.env.UOR_FOUNDRY_SEED_DIR || null;
    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === '--check') {
            checkOnly = true;
        } else if (arg === '--seed-dir') {
            i += 1;
            if (!argv[i]) throw new Error('--seed-dir requires a directory path');
            seedDir = resolve(argv[i]);
        } else {
            throw new Error(`Unknown option: ${arg}`);
        }
    }
    return { checkOnly, seedDir };
}

function sha256Hex(buf) {
    return createHash('sha256').update(buf).digest('hex');
}

function verifyFile(filePath, expectedSha, expectedBytes) {
    if (!existsSync(filePath)) return false;
    const st = statSync(filePath);
    if (!st.isFile() || st.size !== expectedBytes) return false;
    const data = readFileSync(filePath);
    return sha256Hex(data) === expectedSha;
}

async function downloadVerified(url, expectedSha, expectedBytes) {
    const resp = await fetch(url, { redirect: 'follow' });
    if (!resp.ok) {
        throw new Error(`Failed to fetch ${url}: HTTP ${resp.status} ${resp.statusText}`);
    }
    const buf = Buffer.from(await resp.arrayBuffer());
    if (buf.byteLength !== expectedBytes) {
        throw new Error(`Size mismatch for ${url}: got ${buf.byteLength} bytes, expected ${expectedBytes}`);
    }
    const gotSha = sha256Hex(buf);
    if (gotSha !== expectedSha) {
        throw new Error(`SHA-256 mismatch for ${url}: got ${gotSha}, expected ${expectedSha}`);
    }
    return buf;
}

async function main() {
    const { checkOnly, seedDir } = parseArgs(process.argv.slice(2));
    const lock = JSON.parse(readFileSync(lockPath, 'utf8'));
    if (lock.schema !== 'uor-r4.foundry-assets-lock/1' || !Array.isArray(lock.assets)) {
        throw new Error(`Invalid foundry-assets lockfile schema in ${lockPath}`);
    }

    const cacheDir = defaultCacheDir();
    if (!checkOnly) {
        mkdirSync(cacheDir, { recursive: true });
    }

    let checked = 0;
    let copiedFromDestOrCache = 0;
    let fetchedFromRemote = 0;

    for (const asset of lock.assets) {
        const { path: relPath, sha256, bytes, url } = asset;
        if (!relPath || !/^[0-9a-f]{64}$/.test(sha256) || !Number.isInteger(bytes) || bytes < 0 || !url) {
            throw new Error(`Malformed asset record in ${lockPath}: ${JSON.stringify(asset)}`);
        }
        const destPath = resolve(foundryRoot, relPath);
        if (!destPath.startsWith(foundryRoot + '/')) {
            throw new Error(`Path traversal refused: ${relPath}`);
        }

        if (checkOnly) {
            if (!verifyFile(destPath, sha256, bytes)) {
                throw new Error(`Locked asset missing or corrupt: ${destPath} (expected ${sha256})`);
            }
            checked += 1;
            continue;
        }

        if (verifyFile(destPath, sha256, bytes)) {
            const cacheBlob = resolve(cacheDir, sha256);
            if (!verifyFile(cacheBlob, sha256, bytes)) {
                const tmp = `${cacheBlob}.tmp-${process.pid}`;
                copyFileSync(destPath, tmp);
                renameSync(tmp, cacheBlob);
            }
            copiedFromDestOrCache += 1;
            continue;
        }

        const cacheBlob = resolve(cacheDir, sha256);
        mkdirSync(dirname(destPath), { recursive: true });
        if (verifyFile(cacheBlob, sha256, bytes)) {
            copyFileSync(cacheBlob, destPath);
            copiedFromDestOrCache += 1;
            continue;
        }

        if (seedDir) {
            const seedCandidate = resolve(seedDir, relPath);
            if (verifyFile(seedCandidate, sha256, bytes)) {
                const tmp = `${cacheBlob}.tmp-${process.pid}`;
                copyFileSync(seedCandidate, tmp);
                renameSync(tmp, cacheBlob);
                copyFileSync(cacheBlob, destPath);
                copiedFromDestOrCache += 1;
                continue;
            }
        }

        const buf = await downloadVerified(url, sha256, bytes);
        const tmp = `${cacheBlob}.tmp-${process.pid}`;
        writeFileSync(tmp, buf);
        renameSync(tmp, cacheBlob);
        copyFileSync(cacheBlob, destPath);
        fetchedFromRemote += 1;
    }

    if (checkOnly) {
        console.log(JSON.stringify({ schema: lock.schema, mode: 'check', verified: checked }));
    } else {
        console.log(JSON.stringify({
            schema: lock.schema,
            mode: 'fetch',
            total: lock.assets.length,
            cached_or_present: copiedFromDestOrCache,
            fetched: fetchedFromRemote,
        }));
    }
}

main().catch((err) => {
    console.error('fetch-foundry-assets FAILED:', err.message || err);
    process.exit(1);
});
