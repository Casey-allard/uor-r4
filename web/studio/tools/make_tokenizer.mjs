#!/usr/bin/env node
// Generate the stack-bound tokenizer artifact for the Studio worker.
//
// The artifact records the shipped UORLUT01 stack's sha256, its vocabulary
// size and the byte-to-token table for the synthetic stand-in tokenizer
// (`mode: "byte-mod"`: token id = byte mod vocab). It is written next to the
// wasm-pack output (`pkg/tokenizer.json`) so the worker can load it with a
// same-origin dynamic import and label the capability truthfully.
//
// Usage:
//   node web/studio/tools/make_tokenizer.mjs <stack.lut> [--out <tokenizer.json>]
//   node web/studio/tools/make_tokenizer.mjs <stack.lut> --check <tokenizer.json>

import { readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, resolve } from 'node:path';
import { TOKENIZER_SCHEMA, BYTE_MOD_MODE, validateTokenizerArtifact } from '../assets/js/uor_tokenizer_artifact.js';

const STACK_MAGIC = 'UORLUT01';
const STACK_SCHEMA = 'uor-r4.lut-stack/1';

export function parseStackHeader(bytes) {
    const magic = new TextDecoder('ascii').decode(bytes.subarray(0, 8));
    if (bytes.byteLength < 16 || magic !== STACK_MAGIC) {
        throw new Error(`expected a UORLUT01 stack (magic ${STACK_MAGIC}), got "${magic}"`);
    }
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const jsonLen = Number(view.getBigUint64(8, true));
    if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > bytes.byteLength) {
        throw new Error(`invalid UORLUT01 header length (${jsonLen} bytes)`);
    }
    const header = JSON.parse(new TextDecoder('utf-8').decode(bytes.subarray(16, 16 + jsonLen)));
    if (header.schema !== STACK_SCHEMA || !header.shape || !Number.isInteger(header.shape.vocab)) {
        throw new Error(`unsupported stack schema/header (${header.schema})`);
    }
    return header;
}

export function stackSha256(stackBytes) {
    return createHash('sha256').update(stackBytes).digest('hex');
}

export function buildArtifact(stackBytes) {
    const header = parseStackHeader(stackBytes);
    const vocab = header.shape.vocab;
    return {
        schema: TOKENIZER_SCHEMA,
        mode: BYTE_MOD_MODE,
        stack_sha256: stackSha256(stackBytes),
        vocab_size: vocab,
        byte_to_id: Array.from({ length: 256 }, (_, byte) => byte % vocab),
        label_note: 'synthetic stand-in: token id = prompt byte mod stack vocab; bound to this stack sha256',
    };
}

function usage() {
    console.error('usage: node web/studio/tools/make_tokenizer.mjs <stack.lut> [--out <tokenizer.json>] [--check <tokenizer.json>]');
    process.exit(2);
}

if (import.meta.url === `file://${process.argv[1]}`) {
    const args = process.argv.slice(2);
    if (args.length < 1) usage();
    const stackPath = resolve(args[0]);
    const stackBytes = readFileSync(stackPath);
    const outIdx = args.indexOf('--out');
    const checkIdx = args.indexOf('--check');
    const artifact = buildArtifact(stackBytes);
    if (checkIdx >= 0) {
        const existing = JSON.parse(readFileSync(resolve(args[checkIdx + 1]), 'utf8'));
        if (!validateTokenizerArtifact(existing, artifact.stack_sha256, artifact.vocab_size)) {
            console.error('FAIL: tokenizer artifact does not match the shipped stack (hash/vocab/table)');
            process.exit(1);
        }
        console.log(JSON.stringify({ verdict: 'TOKENIZER ARTIFACT MATCHES THE STACK', vocab_size: artifact.vocab_size, stack_sha256: artifact.stack_sha256.slice(0, 12) }));
        process.exit(0);
    }
    const outPath = outIdx >= 0 ? resolve(args[outIdx + 1]) : resolve(dirname(stackPath), 'tokenizer.json');
    writeFileSync(outPath, JSON.stringify(artifact, null, 2) + '\n');
    console.log(JSON.stringify({ verdict: 'WROTE STACK-BOUND TOKENIZER', out: outPath, vocab_size: artifact.vocab_size, stack_sha256: artifact.stack_sha256.slice(0, 12) }));
}
