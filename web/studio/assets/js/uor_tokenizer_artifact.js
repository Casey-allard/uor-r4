// Stack-bound prompt tokenizer artifact for the Studio worker.
//
// The artifact is generated from the shipped UORLUT01 stack header by
// web/studio/tools/make_tokenizer.mjs: it records the stack's sha256, its
// vocabulary size and the byte-to-token table for the synthetic stand-in
// (mode "byte-mod"). Unlike the previous unbound mapping it is tied to the
// exact artifact loaded in the Studio, and the worker labels it with the
// stack digest and vocabulary. A real corpus tokenizer still requires a real
// trained model artifact (M4 export); this artifact's mode says which it is.

export const TOKENIZER_SCHEMA = 'uor-r4.studio-tokenizer/1';
export const BYTE_MOD_MODE = 'byte-mod';

export function sha256Hex(bytes) {
    // Works in workers and Node without importing node:crypto (web crypto).
    return crypto.subtle.digest('SHA-256', bytes).then((digest) =>
        Array.from(new Uint8Array(digest)).map((b) => b.toString(16).padStart(2, '0')).join(''));
}

export function validateTokenizerArtifact(artifact, stackSha256, vocabSize) {
    if (!artifact || artifact.schema !== TOKENIZER_SCHEMA) return false;
    if (artifact.stack_sha256 !== stackSha256) return false;
    if (artifact.vocab_size !== vocabSize) return false;
    if (artifact.mode !== BYTE_MOD_MODE) return false;
    if (!Array.isArray(artifact.byte_to_id) || artifact.byte_to_id.length !== 256) return false;
    return artifact.byte_to_id.every((id, byte) => id === byte % vocabSize);
}

export function tokenizerLabel(artifact, vocabSize) {
    if (!validateTokenizerArtifact(artifact, artifact && artifact.stack_sha256, vocabSize)) {
        return `UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize; unbound - no matching artifact)`;
    }
    return `byte-mod stand-in, bound to stack sha256 ${artifact.stack_sha256.slice(0, 12)}… (vocab ${vocabSize})`;
}

export function encodeBytesWithArtifact(bytes, vocabSize, artifact) {
    const bound = validateTokenizerArtifact(artifact, artifact && artifact.stack_sha256, vocabSize);
    const table = bound ? artifact.byte_to_id : null;
    return Array.from(bytes, (b) => (table ? table[b] : b % vocabSize));
}
