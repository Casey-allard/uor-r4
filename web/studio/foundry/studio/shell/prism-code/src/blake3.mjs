/**
 * BLAKE3, unkeyed hash mode, 32-byte output. Written from the BLAKE3
 * specification (Section 2) and the reference implementation; checked against
 * the official test vectors in `test/blake3.test.mjs`.
 *
 * Hologram kappas, blob labels and the `.holo` footer are BLAKE3, which Node's
 * `crypto` does not provide, so the package carries its own.
 */

const IV = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);
const MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];

const BLOCK_LEN = 64;
const CHUNK_LEN = 1024;
const CHUNK_START = 1;
const CHUNK_END = 2;
const PARENT = 4;
const ROOT = 8;

function rotr(x, n) {
  return (x >>> n) | (x << (32 - n));
}

function g(s, a, b, c, d, mx, my) {
  s[a] = (s[a] + s[b] + mx) >>> 0;
  s[d] = rotr(s[d] ^ s[a], 16);
  s[c] = (s[c] + s[d]) >>> 0;
  s[b] = rotr(s[b] ^ s[c], 12);
  s[a] = (s[a] + s[b] + my) >>> 0;
  s[d] = rotr(s[d] ^ s[a], 8);
  s[c] = (s[c] + s[d]) >>> 0;
  s[b] = rotr(s[b] ^ s[c], 7);
}

function round(s, m) {
  g(s, 0, 4, 8, 12, m[0], m[1]);
  g(s, 1, 5, 9, 13, m[2], m[3]);
  g(s, 2, 6, 10, 14, m[4], m[5]);
  g(s, 3, 7, 11, 15, m[6], m[7]);
  g(s, 0, 5, 10, 15, m[8], m[9]);
  g(s, 1, 6, 11, 12, m[10], m[11]);
  g(s, 2, 7, 8, 13, m[12], m[13]);
  g(s, 3, 4, 9, 14, m[14], m[15]);
}

function permute(m) {
  const out = new Uint32Array(16);
  for (let i = 0; i < 16; i++) out[i] = m[MSG_PERMUTATION[i]];
  return out;
}

/** The compression function: returns the 16-word output state. */
function compress(cv, blockWords, counter, blockLen, flags) {
  const s = new Uint32Array(16);
  s.set(cv.subarray(0, 8), 0);
  s.set(IV.subarray(0, 4), 8);
  s[12] = Number(BigInt.asUintN(32, BigInt(counter)));
  s[13] = Number(BigInt(counter) >> 32n);
  s[14] = blockLen;
  s[15] = flags;
  let m = Uint32Array.from(blockWords);
  for (let i = 0; i < 7; i++) {
    round(s, m);
    if (i < 6) m = permute(m);
  }
  for (let i = 0; i < 8; i++) {
    s[i] ^= s[i + 8];
    s[i + 8] ^= cv[i];
  }
  return s;
}

function wordsFromBlock(block, len) {
  const words = new Uint32Array(16);
  for (let i = 0; i < 16; i++) {
    const o = i * 4;
    words[i] =
      (o < len ? block[o] : 0) |
      ((o + 1 < len ? block[o + 1] : 0) << 8) |
      ((o + 2 < len ? block[o + 2] : 0) << 16) |
      ((o + 3 < len ? block[o + 3] : 0) << 24);
  }
  return words;
}

class Output {
  constructor(inputCv, blockWords, counter, blockLen, flags) {
    this.inputCv = inputCv;
    this.blockWords = blockWords;
    this.counter = counter;
    this.blockLen = blockLen;
    this.flags = flags;
  }

  chainingValue() {
    return compress(this.inputCv, this.blockWords, this.counter, this.blockLen, this.flags).slice(0, 8);
  }

  rootBytes(length) {
    const out = new Uint8Array(length);
    let written = 0;
    let outputCounter = 0;
    while (written < length) {
      const words = compress(this.inputCv, this.blockWords, outputCounter, this.blockLen, this.flags | ROOT);
      for (let i = 0; i < 16 && written < length; i++) {
        const w = words[i];
        for (let b = 0; b < 4 && written < length; b++) out[written++] = (w >>> (8 * b)) & 0xff;
      }
      outputCounter++;
    }
    return out;
  }
}

class ChunkState {
  constructor(key, chunkCounter) {
    this.cv = Uint32Array.from(key);
    this.chunkCounter = chunkCounter;
    this.block = new Uint8Array(BLOCK_LEN);
    this.blockLen = 0;
    this.blocksCompressed = 0;
  }

  length() {
    return BLOCK_LEN * this.blocksCompressed + this.blockLen;
  }

  startFlag() {
    return this.blocksCompressed === 0 ? CHUNK_START : 0;
  }

  update(input) {
    let offset = 0;
    while (offset < input.length) {
      if (this.blockLen === BLOCK_LEN) {
        const words = wordsFromBlock(this.block, BLOCK_LEN);
        this.cv = compress(this.cv, words, this.chunkCounter, BLOCK_LEN, this.startFlag()).slice(0, 8);
        this.blocksCompressed++;
        this.block.fill(0);
        this.blockLen = 0;
      }
      const take = Math.min(BLOCK_LEN - this.blockLen, input.length - offset);
      this.block.set(input.subarray(offset, offset + take), this.blockLen);
      this.blockLen += take;
      offset += take;
    }
  }

  output() {
    const words = wordsFromBlock(this.block, this.blockLen);
    return new Output(this.cv, words, this.chunkCounter, this.blockLen, this.startFlag() | CHUNK_END);
  }
}

function parentOutput(left, right, key) {
  const words = new Uint32Array(16);
  words.set(left, 0);
  words.set(right, 8);
  return new Output(key, words, 0, BLOCK_LEN, PARENT);
}

export class Blake3 {
  constructor() {
    this.key = IV;
    this.chunk = new ChunkState(this.key, 0);
    this.stack = [];
  }

  pushChunkCv(cv, totalChunks) {
    let newCv = cv;
    let n = totalChunks;
    while ((n & 1) === 0) {
      newCv = parentOutput(this.stack.pop(), newCv, this.key).chainingValue();
      n = Math.floor(n / 2);
    }
    this.stack.push(newCv);
  }

  update(input) {
    const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
    let offset = 0;
    while (offset < bytes.length) {
      if (this.chunk.length() === CHUNK_LEN) {
        const cv = this.chunk.output().chainingValue();
        const totalChunks = this.chunk.chunkCounter + 1;
        this.pushChunkCv(cv, totalChunks);
        this.chunk = new ChunkState(this.key, totalChunks);
      }
      const take = Math.min(CHUNK_LEN - this.chunk.length(), bytes.length - offset);
      this.chunk.update(bytes.subarray(offset, offset + take));
      offset += take;
    }
    return this;
  }

  digest(length = 32) {
    let output = this.chunk.output();
    for (let i = this.stack.length - 1; i >= 0; i--) {
      output = parentOutput(this.stack[i], output.chainingValue(), this.key);
    }
    return output.rootBytes(length);
  }
}

/** BLAKE3 of `bytes`, 32 bytes. */
export function blake3(bytes, length = 32) {
  return new Blake3().update(bytes).digest(length);
}

export function toHex(bytes) {
  let s = '';
  for (const b of bytes) s += b.toString(16).padStart(2, '0');
  return s;
}

/** BLAKE3 of `bytes` as lowercase hex. */
export function blake3Hex(bytes) {
  return toHex(blake3(bytes));
}

/** A Hologram kappa label: `blake3:` followed by 64 hex characters (71 bytes). */
export function kappa(bytes) {
  return `blake3:${blake3Hex(bytes)}`;
}
