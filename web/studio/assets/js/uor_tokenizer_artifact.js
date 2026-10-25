// Stack-bound prompt tokenizer artifact (`uor-r4.studio-tokenizer/1`) for the Studio worker.
// Supports `mode: "byte-bpe"` (default: HuggingFace-compatible ByteLevel BPE `model.vocab`,
// rank-ordered `model.merges`, atomic `added_tokens`, and `pre_tokenizer` bound to
// `stack_sha256` and `vocab_size`) and legacy `mode: "byte-mod"`. Bundles
// `DEFAULT_STACK_TOKENIZER_ARTIFACT` for the `seed = 5` `mk-synthetic-stack` (`vocab_size: 40`).

export const TOKENIZER_SCHEMA = 'uor-r4.studio-tokenizer/1';
export const BYTE_BPE_MODE = 'byte-bpe';
export const BYTE_MOD_MODE = 'byte-mod';
export const DEFAULT_SYNTHETIC_STACK_SHA256 =
    'd60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df';
export const DEFAULT_SYNTHETIC_40_PIECES = Object.freeze([
    '<|bos|>', '<|eos|>', '<|unk|>', ':', 'Ġ', 'Ċ', 'U', 's', 'e', 'r',
    'A', 'i', 't', 'a', 'n', 'S', 'y', 'm', '0', '1', '2', '3', '4', '5',
    '6', '7', '8', '9', 'b', 'c', 'd', 'f', 'g', 'h', 'k', 'l', 'o', 'p',
    'q', 'v',
]);

function buildByteTables() {
    const byteEncoder = new Array(256);
    const assigned = new Array(256).fill(false);
    for (const [lo, hi] of [[0x21, 0x7e], [0xa1, 0xac], [0xae, 0xff]]) {
        for (let b = lo; b <= hi; b++) {
            byteEncoder[b] = String.fromCodePoint(b);
            assigned[b] = true;
        }
    }
    let extra = 0;
    for (let b = 0; b < 256; b++) {
        if (!assigned[b]) byteEncoder[b] = String.fromCodePoint(256 + extra++);
    }
    const byteDecoder = new Map(byteEncoder.map((ch, b) => [ch, b]));
    return { byteEncoder, byteDecoder };
}

const { byteEncoder: GPT2_BYTE_ENCODER, byteDecoder: GPT2_BYTE_DECODER } = buildByteTables();

export function createDeterministicBpePieces(vocabSize) {
    if (!Number.isInteger(vocabSize) || vocabSize < 3) {
        throw new Error(`vocabSize must be an integer >= 3, got ${vocabSize}`);
    }
    if (vocabSize === DEFAULT_SYNTHETIC_40_PIECES.length) return DEFAULT_SYNTHETIC_40_PIECES.slice();
    const pieces = ['<|bos|>', '<|eos|>', '<|unk|>'];
    const seen = new Set(pieces);
    for (const p of DEFAULT_SYNTHETIC_40_PIECES) {
        if (pieces.length < vocabSize && !seen.has(p)) { pieces.push(p); seen.add(p); }
    }
    for (let b = 0; b < 256 && pieces.length < vocabSize; b++) {
        const ch = GPT2_BYTE_ENCODER[b];
        if (!seen.has(ch)) { pieces.push(ch); seen.add(ch); }
    }
    for (let idx = 0; pieces.length < vocabSize; idx++) {
        const tok = `<|tok_${idx}|>`;
        if (!seen.has(tok)) { pieces.push(tok); seen.add(tok); }
    }
    return pieces;
}

export function createDefaultSynthetic40TokenizerArtifact(
    stackSha256 = DEFAULT_SYNTHETIC_STACK_SHA256,
    vocabSize = 40,
) {
    const pieces = createDeterministicBpePieces(vocabSize);
    const vocab = Object.fromEntries(pieces.map((p, i) => [p, i]));
    return {
        schema: TOKENIZER_SCHEMA,
        mode: BYTE_BPE_MODE,
        stack_sha256: stackSha256,
        vocab_size: vocabSize,
        unk_id: 2,
        pre_tokenizer: { type: 'ByteLevel', add_prefix_space: false },
        added_tokens: [
            { id: 0, content: '<|bos|>' },
            { id: 1, content: '<|eos|>' },
            { id: 2, content: '<|unk|>' },
        ],
        model: { type: 'BPE', vocab, merges: [] },
        label_note: 'stack-bound ByteLevel BPE tokenizer artifact',
    };
}

export const DEFAULT_STACK_TOKENIZER_ARTIFACT = Object.freeze(
    createDefaultSynthetic40TokenizerArtifact(),
);

export function sha256Hex(bytes) {
    return crypto.subtle.digest('SHA-256', bytes).then((digest) =>
        Array.from(new Uint8Array(digest)).map((b) => b.toString(16).padStart(2, '0')).join(''));
}

function validateBpeModel(artifact, vocabSize) {
    const model = artifact.model;
    if (!model || model.type !== 'BPE' || !model.vocab || typeof model.vocab !== 'object' || Array.isArray(model.vocab)) return false;
    if (!Array.isArray(model.merges)) return false;
    const entries = Object.entries(model.vocab);
    if (entries.length === 0 || entries.length > vocabSize) return false;
    const occupied = new Uint8Array(vocabSize);
    let count = 0;
    for (const [piece, id] of entries) {
        if (typeof piece !== 'string' || !piece.length || !Number.isInteger(id) || id < 0 || id >= vocabSize || occupied[id]) return false;
        occupied[id] = 1;
        count += 1;
    }
    if (Array.isArray(artifact.added_tokens)) {
        for (const e of artifact.added_tokens) {
            if (!e || typeof e.content !== 'string' || !e.content.length || !Number.isInteger(e.id) || e.id < 0 || e.id >= vocabSize) return false;
            if (Object.prototype.hasOwnProperty.call(model.vocab, e.content) && model.vocab[e.content] !== e.id) return false;
            if (!occupied[e.id]) { occupied[e.id] = 1; count += 1; }
        }
    }
    if (count !== vocabSize) return false;
    return model.merges.every((m) =>
        (typeof m === 'string' && m.split(' ').length === 2) ||
        (Array.isArray(m) && m.length === 2 && typeof m[0] === 'string' && typeof m[1] === 'string'));
}

export function validateTokenizerArtifact(artifact, stackSha256, vocabSize) {
    if (!artifact || artifact.schema !== TOKENIZER_SCHEMA) return false;
    if (typeof stackSha256 !== 'string' || artifact.stack_sha256 !== stackSha256) return false;
    if (!Number.isInteger(vocabSize) || vocabSize <= 0 || artifact.vocab_size !== vocabSize) return false;
    if (artifact.mode === BYTE_BPE_MODE) return validateBpeModel(artifact, vocabSize);
    if (artifact.mode === BYTE_MOD_MODE) {
        return Array.isArray(artifact.byte_to_id) && artifact.byte_to_id.length === 256 &&
            artifact.byte_to_id.every((id, byte) => id === byte % vocabSize);
    }
    return false;
}

export function tokenizerLabel(artifact, vocabSize, stackSha256 = artifact && artifact.stack_sha256) {
    if (!validateTokenizerArtifact(artifact, stackSha256, vocabSize)) {
        return 'UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize; unbound - no matching artifact)';
    }
    if (artifact.mode === BYTE_BPE_MODE) {
        return `stack-bound byte-bpe tokenizer (vocab ${vocabSize}, stack sha256 ${artifact.stack_sha256.slice(0, 12)}…)`;
    }
    return `byte-mod stand-in, bound to stack sha256 ${artifact.stack_sha256.slice(0, 12)}… (vocab ${vocabSize})`;
}

function compileBpeTables(artifact) {
    if (artifact.__compiledBpe) return artifact.__compiledBpe;
    const vocabSize = artifact.vocab_size;
    const tokenToId = new Map();
    const idToToken = new Array(vocabSize).fill('');
    for (const [piece, id] of Object.entries(artifact.model.vocab)) {
        tokenToId.set(piece, id);
        idToToken[id] = piece;
    }
    const addedTokens = [];
    const addedIds = new Set();
    for (const entry of artifact.added_tokens || []) {
        if (!tokenToId.has(entry.content)) tokenToId.set(entry.content, entry.id);
        idToToken[entry.id] = entry.content;
        addedIds.add(entry.id);
        addedTokens.push({ content: entry.content, id: entry.id });
    }
    addedTokens.sort((a, b) => b.content.length - a.content.length || (a.content < b.content ? -1 : a.content > b.content ? 1 : 0));
    const mergeRanks = new Map();
    artifact.model.merges.forEach((entry, rank) => {
        const key = Array.isArray(entry) ? `${entry[0]} ${entry[1]}` : entry;
        if (!mergeRanks.has(key)) mergeRanks.set(key, rank);
    });
    const unkId = Number.isInteger(artifact.unk_id) && artifact.unk_id >= 0 && artifact.unk_id < vocabSize
        ? artifact.unk_id
        : (tokenToId.has('<|unk|>') ? tokenToId.get('<|unk|>') : 0);
    const compiled = {
        tokenToId,
        idToToken,
        addedTokens,
        addedIds,
        mergeRanks,
        unkId,
        addPrefixSpace: Boolean(artifact.pre_tokenizer && artifact.pre_tokenizer.add_prefix_space),
    };
    try { Object.defineProperty(artifact, '__compiledBpe', { value: compiled }); } catch (_e) {}
    return compiled;
}

function splitOnAddedTokens(text, addedTokens) {
    const pieces = [];
    let rest = text;
    while (rest.length > 0) {
        let bestPos = -1, bestLen = 0, bestId = 0;
        for (const { content, id } of addedTokens) {
            const pos = rest.indexOf(content);
            if (pos !== -1 && (bestPos === -1 || pos < bestPos)) {
                bestPos = pos; bestLen = content.length; bestId = id;
                if (pos === 0) break;
            }
        }
        if (bestPos !== -1) {
            if (bestPos > 0) pieces.push({ special: false, text: rest.slice(0, bestPos) });
            pieces.push({ special: true, id: bestId });
            rest = rest.slice(bestPos + bestLen);
        } else {
            pieces.push({ special: false, text: rest });
            break;
        }
    }
    return pieces;
}

function bpeSegmentBytes(rawBytes, compiled, outIds) {
    if (!rawBytes.length) return;
    let symbols = Array.from(rawBytes, (b) => GPT2_BYTE_ENCODER[b]);
    while (symbols.length > 1) {
        let bestRank = Infinity, bestIdx = -1;
        for (let i = 0; i < symbols.length - 1; i++) {
            const rank = compiled.mergeRanks.get(`${symbols[i]} ${symbols[i + 1]}`);
            if (rank !== undefined && rank < bestRank) { bestRank = rank; bestIdx = i; }
        }
        if (bestIdx === -1) break;
        const left = symbols[bestIdx], right = symbols[bestIdx + 1], merged = `${left}${right}`;
        const nextSymbols = [];
        for (let i = 0; i < symbols.length;) {
            if (i + 1 < symbols.length && symbols[i] === left && symbols[i + 1] === right) {
                nextSymbols.push(merged); i += 2;
            } else {
                nextSymbols.push(symbols[i]); i += 1;
            }
        }
        symbols = nextSymbols;
    }
    for (const sym of symbols) {
        if (compiled.tokenToId.has(sym)) {
            outIds.push(compiled.tokenToId.get(sym));
        } else {
            for (const ch of sym) {
                outIds.push(compiled.tokenToId.has(ch) ? compiled.tokenToId.get(ch) : compiled.unkId);
            }
        }
    }
}

export function encodeTextWithArtifact(text, vocabSize, artifact, stackSha256 = artifact && artifact.stack_sha256) {
    const str = String(text ?? '');
    if (!validateTokenizerArtifact(artifact, stackSha256, vocabSize)) {
        return encodeBytesWithArtifact(new TextEncoder().encode(str), vocabSize, null);
    }
    if (artifact.mode === BYTE_MOD_MODE) {
        return encodeBytesWithArtifact(new TextEncoder().encode(str), vocabSize, artifact, stackSha256);
    }
    const compiled = compileBpeTables(artifact);
    const encoder = new TextEncoder();
    const outIds = [];
    for (const piece of splitOnAddedTokens(str, compiled.addedTokens)) {
        if (piece.special) outIds.push(piece.id);
        else {
            const seg = compiled.addPrefixSpace && !piece.text.startsWith(' ') ? ` ${piece.text}` : piece.text;
            bpeSegmentBytes(encoder.encode(seg), compiled, outIds);
        }
    }
    return outIds;
}

export function encodeBytesWithArtifact(bytes, vocabSize, artifact, stackSha256 = artifact && artifact.stack_sha256) {
    const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes || []);
    if (!validateTokenizerArtifact(artifact, stackSha256, vocabSize)) {
        return Array.from(u8, (b) => b % vocabSize);
    }
    if (artifact.mode === BYTE_MOD_MODE) return Array.from(u8, (b) => artifact.byte_to_id[b]);
    return encodeTextWithArtifact(new TextDecoder('utf-8').decode(u8), vocabSize, artifact, stackSha256);
}

export function decodeTokenIdsWithArtifact(ids, artifact, stackSha256 = artifact && artifact.stack_sha256, vocabSize = artifact && artifact.vocab_size) {
    if (!validateTokenizerArtifact(artifact, stackSha256, vocabSize) || artifact.mode !== BYTE_BPE_MODE) {
        return Array.from(ids || [], (id, idx) => (idx === 0 ? `${id >>> 0}` : ` ${id >>> 0}`)).join('');
    }
    const compiled = compileBpeTables(artifact);
    const bytes = [];
    const encoder = new TextEncoder();
    for (const rawId of ids || []) {
        const id = rawId >>> 0;
        const token = compiled.idToToken[id];
        if (!token) continue;
        if (compiled.addedIds.has(id)) {
            for (const b of encoder.encode(token)) bytes.push(b);
            continue;
        }
        for (const ch of token) {
            if (GPT2_BYTE_DECODER.has(ch)) bytes.push(GPT2_BYTE_DECODER.get(ch));
            else for (const b of encoder.encode(ch)) bytes.push(b);
        }
    }
    return new TextDecoder('utf-8').decode(new Uint8Array(bytes));
}

export function decodeTokenIdWithArtifact(id, artifact, stackSha256 = artifact && artifact.stack_sha256, vocabSize = artifact && artifact.vocab_size) {
    return decodeTokenIdsWithArtifact([id], artifact, stackSha256, vocabSize);
}
