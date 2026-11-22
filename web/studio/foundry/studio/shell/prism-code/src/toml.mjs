/**
 * A strict reader for the TOML 1.0 subset `prism.toml` uses (issue #280).
 *
 * Supported: comments, `[table]` headers with bare, quoted and dotted keys,
 * `key = value` with dotted keys, basic and literal strings (single and
 * multi-line), decimal integers, booleans, arrays (across lines, trailing
 * comma, comments between items) and inline tables.
 *
 * Refused with `manifest.toml`, naming the line: floats, dates and times,
 * arrays of tables, hexadecimal, octal and binary integers, duplicate keys,
 * a table defined twice, and a key named `__proto__`, `constructor` or
 * `prototype`. A manifest has none of those, and refusing them is what keeps
 * this reader small enough to read. The package stays dependency-free.
 */
import { fail } from './errors.mjs';

const BARE = /^[A-Za-z0-9_-]+$/;
const FORBIDDEN_KEYS = new Set(['__proto__', 'constructor', 'prototype']);
const ESCAPES = { b: '\b', t: '\t', n: '\n', f: '\f', r: '\r', '"': '"', '\\': '\\' };

const isTable = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

class Reader {
  constructor(text) {
    this.text = text.startsWith('\uFEFF') ? text.slice(1) : text;
    this.at = 0;
    this.line = 1;
    this.root = {};
    // Tables made by a header or an inline table may not be added to again; implicit ones (a.b in [a.b.c]) may be defined once.
    this.closed = new WeakSet();
    this.defined = new WeakSet();
  }

  error(message) {
    fail('manifest.toml', `line ${this.line}: ${message}`, { line: this.line });
  }

  peek(offset = 0) {
    return this.text[this.at + offset];
  }

  startsWith(token) {
    return this.text.startsWith(token, this.at);
  }

  advance(count = 1) {
    for (let i = 0; i < count; i++) {
      if (this.text[this.at] === '\n') this.line++;
      this.at++;
    }
  }

  skipSpaces() {
    while (this.peek() === ' ' || this.peek() === '\t') this.advance();
  }

  skipComment() {
    if (this.peek() !== '#') return;
    while (this.at < this.text.length && this.peek() !== '\n') {
      const c = this.text.charCodeAt(this.at);
      if ((c < 0x20 && c !== 0x09 && c !== 0x0d) || c === 0x7f) this.error('a comment holds a control character');
      this.advance();
    }
  }

  /** Spaces, a comment, then the end of the line or of the file. */
  endOfLine() {
    this.skipSpaces();
    this.skipComment();
    if (this.at >= this.text.length) return;
    if (this.startsWith('\r\n')) this.advance(2);
    else if (this.peek() === '\n') this.advance();
    else this.error(`unexpected ${JSON.stringify(this.peek())} after the value`);
  }

  /** Whitespace, newlines and comments, as allowed inside an array. */
  skipBlank() {
    for (;;) {
      this.skipSpaces();
      this.skipComment();
      if (this.startsWith('\r\n')) this.advance(2);
      else if (this.peek() === '\n') this.advance();
      else return;
    }
  }

  parse() {
    let table = this.root;
    for (;;) {
      this.skipBlank();
      if (this.at >= this.text.length) return this.root;
      if (this.startsWith('[[')) this.error('arrays of tables ([[...]]) are not part of a manifest');
      if (this.peek() === '[') {
        this.advance();
        this.skipSpaces();
        const path = this.keyPath(']');
        this.advance();
        table = this.header(path);
        this.endOfLine();
      } else {
        this.keyValue(table);
        this.endOfLine();
      }
    }
  }

  header(path) {
    let table = this.root;
    path.forEach((key, i) => {
      if (!Object.hasOwn(table, key)) table[key] = {};
      else if (!isTable(table[key])) this.error(`${path.slice(0, i + 1).join('.')} is already a value, not a table`);
      table = table[key];
      if (this.closed.has(table) && i < path.length - 1) this.error(`${path.slice(0, i + 1).join('.')} is an inline table and cannot be extended`);
    });
    if (this.defined.has(table) || this.closed.has(table)) this.error(`table [${path.join('.')}] is defined twice`);
    this.defined.add(table);
    return table;
  }

  /** A dotted key, ending at `stop` (`]` for a header, `=` for a key-value pair). */
  keyPath(stop) {
    const path = [];
    for (;;) {
      this.skipSpaces();
      path.push(this.key());
      this.skipSpaces();
      if (this.peek() === '.') {
        this.advance();
        continue;
      }
      if (this.peek() === stop) return path;
      this.error(stop === ']' ? 'a table header ends with ]' : 'a key is followed by =');
    }
  }

  key() {
    let key;
    if (this.peek() === '"') key = this.basicString();
    else if (this.peek() === "'") key = this.literalString();
    else {
      const start = this.at;
      while (this.at < this.text.length && /[A-Za-z0-9_-]/.test(this.peek())) this.advance();
      key = this.text.slice(start, this.at);
      if (!BARE.test(key)) this.error('expected a key');
    }
    if (FORBIDDEN_KEYS.has(key)) this.error(`the key ${key} is not allowed`);
    return key;
  }

  keyValue(table) {
    const path = this.keyPath('=');
    this.advance();
    this.skipSpaces();
    const value = this.value();
    let target = table;
    path.slice(0, -1).forEach((key, i) => {
      if (!Object.hasOwn(target, key)) {
        target[key] = {};
        // A table made by a dotted key belongs to this pair; a later header may not define it.
        this.defined.add(target[key]);
      } else if (!isTable(target[key]) || this.closed.has(target[key])) {
        this.error(`${path.slice(0, i + 1).join('.')} cannot be extended here`);
      }
      target = target[key];
    });
    const last = path[path.length - 1];
    if (Object.hasOwn(target, last)) this.error(`key ${path.join('.')} is defined twice`);
    target[last] = value;
  }

  value() {
    const c = this.peek();
    if (this.startsWith('"""')) return this.multilineBasic();
    if (this.startsWith("'''")) return this.multilineLiteral();
    if (c === '"') return this.basicString();
    if (c === "'") return this.literalString();
    if (c === '[') return this.array();
    if (c === '{') return this.inlineTable();
    if (this.startsWith('true') && !/[A-Za-z0-9_]/.test(this.peek(4) ?? '')) {
      this.advance(4);
      return true;
    }
    if (this.startsWith('false') && !/[A-Za-z0-9_]/.test(this.peek(5) ?? '')) {
      this.advance(5);
      return false;
    }
    return this.integer();
  }

  integer() {
    const start = this.at;
    while (this.at < this.text.length && /[0-9A-Za-z_+.:-]/.test(this.peek())) this.advance();
    const token = this.text.slice(start, this.at);
    if (token === '') this.error('expected a value');
    if (/^[+-]?(0|[1-9](_?[0-9])*)$/.test(token)) {
      const n = Number(token.replaceAll('_', ''));
      if (!Number.isSafeInteger(n)) this.error(`integer ${token} is out of range`);
      return n;
    }
    if (/^[+-]?0[xob]/.test(token)) this.error(`${token}: only decimal integers are part of a manifest`);
    if (/^\d{4}-\d{2}-\d{2}|^\d{2}:\d{2}/.test(token)) this.error(`${token}: dates and times are not part of a manifest`);
    if (/^[+-]?(\d|inf|nan)/.test(token)) this.error(`${token}: floats are not part of a manifest`);
    this.error(`${token} is not a value; strings are quoted`);
  }

  controlCheck(code, multiline) {
    if ((code < 0x20 && code !== 0x09 && !(multiline && (code === 0x0a || code === 0x0d))) || code === 0x7f) {
      this.error('a string holds an unescaped control character');
    }
  }

  escape() {
    this.advance(); // the backslash
    const c = this.peek();
    if (Object.hasOwn(ESCAPES, c)) {
      this.advance();
      return ESCAPES[c];
    }
    if (c === 'u' || c === 'U') {
      const length = c === 'u' ? 4 : 8;
      const hex = this.text.slice(this.at + 1, this.at + 1 + length);
      if (!new RegExp(`^[0-9A-Fa-f]{${length}}$`).test(hex)) this.error(`\\${c} takes ${length} hex digits`);
      const code = parseInt(hex, 16);
      if (code > 0x10ffff || (code >= 0xd800 && code <= 0xdfff)) this.error(`\\${c}${hex} is not a Unicode scalar value`);
      this.advance(1 + length);
      return String.fromCodePoint(code);
    }
    this.error(`unknown escape \\${c ?? ''}`);
  }

  basicString() {
    this.advance();
    let out = '';
    for (;;) {
      if (this.at >= this.text.length || this.peek() === '\n') this.error('a string is not closed on its line');
      const c = this.peek();
      if (c === '"') {
        this.advance();
        return out;
      }
      if (c === '\\') {
        out += this.escape();
        continue;
      }
      this.controlCheck(c.charCodeAt(0), false);
      out += c;
      this.advance();
    }
  }

  literalString() {
    this.advance();
    const start = this.at;
    while (this.peek() !== "'") {
      if (this.at >= this.text.length || this.peek() === '\n') this.error('a string is not closed on its line');
      this.controlCheck(this.text.charCodeAt(this.at), false);
      this.advance();
    }
    const out = this.text.slice(start, this.at);
    this.advance();
    return out;
  }

  /** A newline right after the opening delimiter is trimmed, as TOML says. */
  trimFirstNewline() {
    if (this.startsWith('\r\n')) this.advance(2);
    else if (this.peek() === '\n') this.advance();
  }

  multilineBasic() {
    this.advance(3);
    this.trimFirstNewline();
    let out = '';
    for (;;) {
      if (this.at >= this.text.length) this.error('a multi-line string is not closed');
      if (this.startsWith('"""')) {
        // Up to two quotes may sit right before the closing delimiter.
        let extra = 0;
        while (this.peek(3 + extra) === '"' && extra < 2) extra++;
        out += '"'.repeat(extra);
        this.advance(3 + extra);
        return out;
      }
      const c = this.peek();
      if (c === '\\') {
        // A line-ending backslash trims the newline and the whitespace after it.
        let k = 1;
        while (this.peek(k) === ' ' || this.peek(k) === '\t') k++;
        if (this.peek(k) === '\n' || (this.peek(k) === '\r' && this.peek(k + 1) === '\n')) {
          this.advance(k);
          while ([' ', '\t', '\n', '\r'].includes(this.peek())) this.advance();
          continue;
        }
        out += this.escape();
        continue;
      }
      if (c === '\r' && this.peek(1) === '\n') {
        out += '\n';
        this.advance(2);
        continue;
      }
      this.controlCheck(c.charCodeAt(0), true);
      out += c;
      this.advance();
    }
  }

  multilineLiteral() {
    this.advance(3);
    this.trimFirstNewline();
    let out = '';
    for (;;) {
      if (this.at >= this.text.length) this.error('a multi-line string is not closed');
      if (this.startsWith("'''")) {
        let extra = 0;
        while (this.peek(3 + extra) === "'" && extra < 2) extra++;
        out += "'".repeat(extra);
        this.advance(3 + extra);
        return out;
      }
      const c = this.peek();
      if (c === '\r' && this.peek(1) === '\n') {
        out += '\n';
        this.advance(2);
        continue;
      }
      this.controlCheck(c.charCodeAt(0), true);
      out += c;
      this.advance();
    }
  }

  array() {
    this.advance();
    const out = [];
    for (;;) {
      this.skipBlank();
      if (this.peek() === ']') {
        this.advance();
        return out;
      }
      out.push(this.value());
      this.skipBlank();
      if (this.peek() === ',') {
        this.advance();
        continue;
      }
      if (this.peek() === ']') {
        this.advance();
        return out;
      }
      this.error('array items are separated by commas');
    }
  }

  inlineTable() {
    this.advance();
    const table = {};
    this.skipSpaces();
    if (this.peek() === '}') {
      this.advance();
      this.closed.add(table);
      return table;
    }
    for (;;) {
      this.skipSpaces();
      if (this.peek() === '\n' || this.peek() === '\r') this.error('an inline table stays on one line');
      this.keyValue(table);
      this.skipSpaces();
      if (this.peek() === ',') {
        this.advance();
        continue;
      }
      if (this.peek() === '}') {
        this.advance();
        this.closed.add(table);
        return table;
      }
      this.error('inline table members are separated by commas and the table ends with }');
    }
  }
}

/**
 * Parse TOML text (or UTF-8 bytes) into plain objects, arrays, strings,
 * integers and booleans. Throws `PrismCodeError` `manifest.toml` naming the
 * line of the first problem.
 */
export function parseToml(source) {
  let text = source;
  if (source instanceof Uint8Array) {
    try {
      text = new TextDecoder('utf-8', { fatal: true }).decode(source);
    } catch {
      fail('manifest.toml', 'the file is not UTF-8');
    }
  }
  if (typeof text !== 'string') fail('manifest.toml', 'TOML is text or UTF-8 bytes');
  return new Reader(text).parse();
}
