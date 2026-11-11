#!/usr/bin/env node
// Exports the Rust-owned Studio agent contract schema and generates the
// TypeScript contract and canonicalizer from it.
//
// Every path is resolved from this module's URL, never from the caller's
// working directory. The schema bytes, including the trailing newline, are the
// digest input; the generated files are a pure function of those bytes.

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const repoRoot = resolve(dirname(scriptPath), '..', '..');
const manifestPath = join(repoRoot, 'Cargo.toml');
const schemaPath = join(repoRoot, 'crates', 'uor-r4-studio-agent', 'contract.schema.json');
const generatedDir = join(repoRoot, 'web', 'studio', 'src', 'agent', 'generated');

function fail(message) {
    console.error('generate-agent-contract: ' + message);
    process.exit(1);
}

function assert(condition, message) {
    if (!condition) {
        fail(message);
    }
}

function literal(value) {
    return JSON.stringify(value);
}

function constArray(values) {
    return '[\n' + values.map((value) => '    ' + literal(value) + ',').join('\n') + '\n] as const';
}

// A schema node is a property schema from the exported document.
function schemaType(node, indent) {
    if (node === null || typeof node !== 'object') {
        return 'unknown';
    }
    if (Array.isArray(node.enum)) {
        return node.enum.map((value) => literal(value)).join(' | ');
    }
    if (Object.prototype.hasOwnProperty.call(node, 'const')) {
        return literal(node.const);
    }
    const type = node.type;
    if (Array.isArray(type)) {
        return type.map((entry) => schemaType({ type: entry }, indent)).join(' | ');
    }
    switch (type) {
        case 'string':
            return 'string';
        case 'integer':
            return 'number';
        case 'boolean':
            return 'boolean';
        case 'null':
            return 'null';
        case 'array':
            return 'Array<' + schemaType(node.items, indent) + '>';
        case 'object':
            return objectType(node, indent);
        default:
            return 'unknown';
    }
}

function propertyKey(key) {
    return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key) ? key : literal(key);
}

function objectType(node, indent) {
    const properties = node.properties ?? {};
    const required = new Set(node.required ?? []);
    const keys = Object.keys(properties).sort();
    if (keys.length === 0) {
        return 'Record<string, unknown>';
    }
    const pad = ' '.repeat(indent + 4);
    const closing = ' '.repeat(indent);
    const lines = keys.map((key) => {
        const optional = required.has(key) ? '' : '?';
        return pad + propertyKey(key) + optional + ': ' + schemaType(properties[key], indent + 4) + ';';
    });
    return '{\n' + lines.join('\n') + '\n' + closing + '}';
}

function typeName(eventType) {
    return (
        eventType
            .split(/[._-]+/)
            .filter(Boolean)
            .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
            .join('') + 'Data'
    );
}

const exportRun = spawnSync(
    'cargo',
    [
        'run',
        '--manifest-path',
        manifestPath,
        '-p',
        'uor-r4-studio-agent',
        '--bin',
        'export-contract',
        '--',
        '--write',
        schemaPath,
    ],
    { cwd: repoRoot, stdio: 'inherit' },
);
if (exportRun.error) {
    fail('cannot run cargo: ' + exportRun.error.message);
}
if (exportRun.status !== 0) {
    fail('export-contract exited with status ' + String(exportRun.status));
}

const schemaBytes = readFileSync(schemaPath);
const schemaText = schemaBytes.toString('utf8');
const contractDigest = createHash('sha256').update(schemaBytes).digest('hex');

let schema;
try {
    schema = JSON.parse(schemaText);
} catch (error) {
    fail('contract.schema.json is not valid JSON: ' + error.message);
}

assert(schemaText.endsWith('\n'), 'contract.schema.json must end with a trailing newline');
assert(schema.$id === 'uor-r4.studio.agent-event/1', 'unexpected contract $id');
assert(
    schema.$schema === 'https://json-schema.org/draft/2020-12/schema',
    'contract must be a JSON Schema 2020-12 document',
);
assert(schema.schema_id === schema.$id, 'contract schema_id must equal $id');
assert(schema.contract_version === 1, 'unexpected contract_version');
assert(typeof schema.error_schema === 'string', 'contract must name its error schema');
assert(typeof schema.canonical_json_schema === 'string', 'contract must name canonical JSON');
assert(schema.envelope && Array.isArray(schema.envelope.required), 'contract envelope is missing');
assert(
    schema.event_types && typeof schema.event_types === 'object',
    'contract event_types is missing',
);
assert(schema.limits && typeof schema.limits === 'object', 'contract limits are missing');
assert(Array.isArray(schema.error_codes) && schema.error_codes.length > 0, 'error codes are missing');
assert(Array.isArray(schema.run_states) && schema.run_states.length > 0, 'run states are missing');

const eventTypes = Object.keys(schema.event_types).sort();
const terminalEventTypes = eventTypes.filter(
    (eventType) => schema.event_types[eventType].terminal === true,
);

const header =
    '// Generated by scripts/studio/generate-agent-contract.mjs from\n' +
    '// crates/uor-r4-studio-agent/contract.schema.json (sha256:' +
    contractDigest +
    ').\n' +
    '// Do not edit by hand; run the contract:agent script from web/studio.\n';

const eventInterfaces = eventTypes
    .map((eventType) => 'export interface ' + typeName(eventType) + ' ' + objectType(schema.event_types[eventType].data, 0) + '\n')
    .join('\n');

const dataMapEntries = eventTypes
    .map((eventType) => '    ' + literal(eventType) + ': ' + typeName(eventType) + ';')
    .join('\n');

const contractSource =
    header +
    '\n' +
    'export const AGENT_EVENT_SCHEMA = ' + literal(schema.$id) + ' as const;\n' +
    'export const AGENT_ERROR_SCHEMA = ' + literal(schema.error_schema) + ' as const;\n' +
    'export const CANONICAL_JSON_SCHEMA = ' + literal(schema.canonical_json_schema) + ' as const;\n' +
    'export const AGENT_CONTRACT_VERSION = ' + literal(schema.contract_version) + ' as const;\n' +
    'export const CURSOR_PREFIX = ' + literal(schema.cursor_prefix) + ' as const;\n' +
    'export const AGENT_CONTRACT_DIGEST = ' + literal(contractDigest) + ' as const;\n' +
    '\n' +
    'export const MAX_SAFE_INTEGER = 9007199254740991;\n' +
    'export const MAX_EVENT_BYTES = ' + literal(schema.limits.max_event_bytes) + ';\n' +
    'export const MAX_EVENTS_PER_RUN = ' + literal(schema.limits.max_events_per_run) + ';\n' +
    'export const APPROVAL_TTL_MS = ' + literal(schema.limits.approval_ttl_ms) + ';\n' +
    '\n' +
    'export const AGENT_EVENT_TYPES = ' + constArray(eventTypes) + ';\n' +
    'export type KnownAgentEventType = (typeof AGENT_EVENT_TYPES)[number];\n' +
    '\n' +
    'export const TERMINAL_AGENT_EVENT_TYPES = ' + constArray(terminalEventTypes) + ';\n' +
    'export type TerminalAgentEventType = (typeof TERMINAL_AGENT_EVENT_TYPES)[number];\n' +
    '\n' +
    'export const AGENT_ERROR_CODES = ' + constArray(schema.error_codes) + ';\n' +
    'export type AgentErrorCode = (typeof AGENT_ERROR_CODES)[number];\n' +
    '\n' +
    'export const AGENT_RUN_STATES = ' + constArray(schema.run_states) + ';\n' +
    'export type RunState = (typeof AGENT_RUN_STATES)[number];\n' +
    '\n' +
    eventInterfaces +
    '\n' +
    'export interface AgentEventDataMap {\n' +
    dataMapEntries +
    '\n' +
    '}\n' +
    '\n' +
    'export interface AgentEventEnvelope<TType extends string, TData> {\n' +
    '    schema: typeof AGENT_EVENT_SCHEMA;\n' +
    '    event_id: string;\n' +
    '    project_id: string;\n' +
    '    task_id: string;\n' +
    '    run_id: string;\n' +
    '    generation: number;\n' +
    '    sequence: number;\n' +
    '    recorded_at_ms: number;\n' +
    '    type: TType;\n' +
    '    data: TData;\n' +
    '}\n' +
    '\n' +
    'export type AgentEvent = {\n' +
    '    [K in KnownAgentEventType]: AgentEventEnvelope<K, AgentEventDataMap[K]>;\n' +
    '}[KnownAgentEventType];\n' +
    '\n' +
    'export type UnknownAgentEvent = AgentEventEnvelope<string, Record<string, unknown>>;\n' +
    '\n' +
    '// Templated types: contract.schema.json does not carry complete object\n' +
    '// definitions for these three shapes. They mirror the Rust structs in\n' +
    '// crates/uor-r4-studio-agent/src/error.rs and src/contract.rs; the\n' +
    '// consistency assertions in agentContract.test.ts keep the fields that the\n' +
    '// schema does describe tied to it.\n' +
    'export interface AgentError {\n' +
    '    schema: typeof AGENT_ERROR_SCHEMA;\n' +
    '    code: AgentErrorCode;\n' +
    '    message: string;\n' +
    '    retryable: boolean;\n' +
    '    task_id: string | null;\n' +
    '    run_id: string | null;\n' +
    '}\n' +
    '\n' +
    'export interface AgentCursor {\n' +
    '    project_id: string;\n' +
    '    task_id: string;\n' +
    '    run_id: string;\n' +
    '    generation: number;\n' +
    '    sequence: number;\n' +
    '}\n' +
    '\n' +
    'export interface ApprovalRequest {\n' +
    '    approval_id: string;\n' +
    '    task_id: string;\n' +
    '    run_id: string;\n' +
    '    generation: number;\n' +
    '    tool_call_id: string;\n' +
    '    tool: string;\n' +
    '    scope: string;\n' +
    '    request_digest: string;\n' +
    '    requested_at_ms: number;\n' +
    '    expires_at_ms: number;\n' +
    '}\n';

const canonicalSource =
    header +
    String.raw`
import type { AgentEvent } from './agentContract.js';

export const CANONICAL_JSON_ERROR_CODES = ['invalid_event', 'malformed_event'] as const;
export type CanonicalJsonErrorCode = (typeof CANONICAL_JSON_ERROR_CODES)[number];

export class CanonicalJsonError extends Error {
    readonly code: CanonicalJsonErrorCode;

    constructor(code: CanonicalJsonErrorCode, message: string) {
        super(code + ': ' + message);
        this.name = 'CanonicalJsonError';
        this.code = code;
    }
}

const MAX_SAFE_INTEGER = 9007199254740991;
const MAX_NESTING_DEPTH = 127;
const CANONICAL_KEY_PATTERN = /^[a-z0-9_.-]*$/;

function invalidEvent(message: string): never {
    throw new CanonicalJsonError('invalid_event', message);
}

function malformedEvent(message: string): never {
    throw new CanonicalJsonError('malformed_event', message);
}

export function canonicalJson(value: unknown): string {
    const parts: string[] = [];
    writeCanonical(value, parts);
    return parts.join('');
}

export function parseCanonicalJson(text: string): string {
    if (typeof text !== 'string') {
        return malformedEvent('canonical JSON input must be a string');
    }
    const parser = new StrictJsonParser(text);
    return canonicalJson(parser.parseDocument());
}

export async function digestEvent(event: AgentEvent): Promise<string> {
    const canonical = canonicalJson(event);
    const digest = await sha256Hex(canonical);
    return 'sha256:' + digest;
}

async function sha256Hex(text: string): Promise<string> {
    const subtle = globalThis.crypto?.subtle;
    if (subtle === undefined) {
        throw new Error('Web Crypto SHA-256 is not available in this environment');
    }
    const bytes = new TextEncoder().encode(text);
    const digest = new Uint8Array(await subtle.digest('SHA-256', bytes));
    let hex = '';
    for (const byte of digest) {
        hex += byte.toString(16).padStart(2, '0');
    }
    return hex;
}

function writeCanonical(value: unknown, parts: string[]): void {
    if (value === null) {
        parts.push('null');
        return;
    }
    switch (typeof value) {
        case 'boolean':
            parts.push(value ? 'true' : 'false');
            return;
        case 'number':
            parts.push(canonicalNumber(value));
            return;
        case 'string':
            parts.push(escapeString(value));
            return;
        case 'object':
            writeObjectOrArray(value, parts);
            return;
        default:
            return invalidEvent('only JSON values are canonical');
    }
}

function writeObjectOrArray(value: object, parts: string[]): void {
    if (Array.isArray(value)) {
        parts.push('[');
        for (let index = 0; index < value.length; index += 1) {
            if (index > 0) {
                parts.push(',');
            }
            writeCanonical(value[index], parts);
        }
        parts.push(']');
        return;
    }
    const prototype = Object.getPrototypeOf(value);
    if (prototype !== Object.prototype && prototype !== null) {
        return invalidEvent('only plain JSON objects are canonical');
    }
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    for (const key of keys) {
        if (!CANONICAL_KEY_PATTERN.test(key)) {
            return invalidEvent(
                'object key ' + JSON.stringify(key) + ' is not in the canonical ASCII key set',
            );
        }
    }
    parts.push('{');
    keys.forEach((key, index) => {
        if (index > 0) {
            parts.push(',');
        }
        parts.push(escapeString(key), ':');
        writeCanonical(record[key], parts);
    });
    parts.push('}');
}

// JavaScript has one number type, so a value-level 1.0 is already 1; the raw
// parser rejects lexical floats and exponents before they reach this function.
function canonicalNumber(value: number): string {
    if (Object.is(value, -0)) {
        return invalidEvent('negative zero is not a canonical number');
    }
    if (!Number.isSafeInteger(value)) {
        if (Number.isInteger(value) && value > MAX_SAFE_INTEGER) {
            return invalidEvent(
                'integer ' + String(value) + ' exceeds the canonical safe integer range',
            );
        }
        return invalidEvent('only non-negative safe integers are canonical');
    }
    if (value < 0) {
        return invalidEvent('only non-negative safe integers are canonical');
    }
    return String(value);
}

function escapeString(text: string): string {
    const parts: string[] = ['"'];
    for (let index = 0; index < text.length; index += 1) {
        const code = text.charCodeAt(index);
        if (code === 0x22) {
            parts.push('\\"');
        } else if (code === 0x5c) {
            parts.push('\\\\');
        } else if (code <= 0x1f) {
            parts.push('\\u00' + code.toString(16).padStart(2, '0'));
        } else if (code >= 0xd800 && code <= 0xdbff) {
            const next = text.charCodeAt(index + 1);
            if (!(next >= 0xdc00 && next <= 0xdfff)) {
                return invalidEvent('string contains an unpaired high surrogate');
            }
            parts.push(text[index], text[index + 1]);
            index += 1;
        } else if (code >= 0xdc00 && code <= 0xdfff) {
            return invalidEvent('string contains an unpaired low surrogate');
        } else {
            parts.push(text[index]);
        }
    }
    parts.push('"');
    return parts.join('');
}

function isDigit(character: string | undefined): boolean {
    return character !== undefined && character >= '0' && character <= '9';
}

class StrictJsonParser {
    private index = 0;
    private depth = 0;

    constructor(private readonly text: string) {}

    parseDocument(): unknown {
        this.skipWhitespace();
        const value = this.parseValue();
        this.skipWhitespace();
        if (this.index !== this.text.length) {
            return malformedEvent('trailing input after the JSON value');
        }
        return value;
    }

    private parseValue(): unknown {
        const character = this.peek();
        if (character === '{') {
            return this.parseObject();
        }
        if (character === '[') {
            return this.parseArray();
        }
        if (character === '"') {
            return this.parseString();
        }
        if (character === '-' || isDigit(character)) {
            return this.parseNumber();
        }
        if (character === 't') {
            this.expectWord('true');
            return true;
        }
        if (character === 'f') {
            this.expectWord('false');
            return false;
        }
        if (character === 'n') {
            this.expectWord('null');
            return null;
        }
        return malformedEvent(
            character === undefined
                ? 'unexpected end of JSON input'
                : 'unexpected character ' + JSON.stringify(character),
        );
    }

    private parseObject(): Record<string, unknown> {
        this.depth += 1;
        if (this.depth > MAX_NESTING_DEPTH) {
            return malformedEvent('recursion limit exceeded');
        }
        const result = this.parseObjectMembers();
        this.depth -= 1;
        return result;
    }

    private parseObjectMembers(): Record<string, unknown> {
        this.index += 1;
        this.skipWhitespace();
        const result: Record<string, unknown> = Object.create(null);
        const seen = new Set<string>();
        if (this.peek() === '}') {
            this.index += 1;
            return result;
        }
        while (true) {
            this.skipWhitespace();
            if (this.peek() !== '"') {
                return malformedEvent('JSON object keys must be strings');
            }
            const key = this.parseString();
            if (seen.has(key)) {
                return invalidEvent('duplicate object key ' + JSON.stringify(key));
            }
            seen.add(key);
            this.skipWhitespace();
            if (this.peek() !== ':') {
                return malformedEvent('expected a colon after a JSON object key');
            }
            this.index += 1;
            this.skipWhitespace();
            result[key] = this.parseValue();
            this.skipWhitespace();
            const character = this.peek();
            if (character === ',') {
                this.index += 1;
                continue;
            }
            if (character === '}') {
                this.index += 1;
                return result;
            }
            return malformedEvent('expected a comma or closing brace in a JSON object');
        }
    }

    private parseArray(): unknown[] {
        this.depth += 1;
        if (this.depth > MAX_NESTING_DEPTH) {
            return malformedEvent('recursion limit exceeded');
        }
        const items = this.parseArrayItems();
        this.depth -= 1;
        return items;
    }

    private parseArrayItems(): unknown[] {
        this.index += 1;
        this.skipWhitespace();
        const items: unknown[] = [];
        if (this.peek() === ']') {
            this.index += 1;
            return items;
        }
        while (true) {
            this.skipWhitespace();
            items.push(this.parseValue());
            this.skipWhitespace();
            const character = this.peek();
            if (character === ',') {
                this.index += 1;
                continue;
            }
            if (character === ']') {
                this.index += 1;
                return items;
            }
            return malformedEvent('expected a comma or closing bracket in a JSON array');
        }
    }

    private parseString(): string {
        if (this.peek() !== '"') {
            return malformedEvent('expected a JSON string');
        }
        this.index += 1;
        let result = '';
        while (this.index < this.text.length) {
            const code = this.text.charCodeAt(this.index);
            if (code === 0x22) {
                this.index += 1;
                return result;
            }
            if (code === 0x5c) {
                this.index += 1;
                result += this.parseEscape();
                continue;
            }
            if (code <= 0x1f) {
                return malformedEvent('unescaped control character in a JSON string');
            }
            if (code >= 0xd800 && code <= 0xdbff) {
                const next = this.text.charCodeAt(this.index + 1);
                if (!(next >= 0xdc00 && next <= 0xdfff)) {
                    return malformedEvent('unpaired high surrogate in a JSON string');
                }
                result += this.text.slice(this.index, this.index + 2);
                this.index += 2;
                continue;
            }
            if (code >= 0xdc00 && code <= 0xdfff) {
                return malformedEvent('unpaired low surrogate in a JSON string');
            }
            result += this.text[this.index];
            this.index += 1;
        }
        return malformedEvent('unterminated JSON string');
    }

    private parseEscape(): string {
        const character = this.text[this.index];
        this.index += 1;
        switch (character) {
            case '"':
                return '"';
            case '\\':
                return '\\';
            case '/':
                return '/';
            case 'b':
                return '\b';
            case 'f':
                return '\f';
            case 'n':
                return '\n';
            case 'r':
                return '\r';
            case 't':
                return '\t';
            case 'u':
                return this.parseUnicodeEscape();
            default:
                return malformedEvent('invalid JSON escape');
        }
    }

    private parseUnicodeEscape(): string {
        const first = this.readHexUnit();
        if (first >= 0xd800 && first <= 0xdbff) {
            if (this.text.slice(this.index, this.index + 2) !== '\\u') {
                return malformedEvent('unpaired high surrogate in a JSON string');
            }
            this.index += 2;
            const second = this.readHexUnit();
            if (second < 0xdc00 || second > 0xdfff) {
                return malformedEvent('unpaired high surrogate in a JSON string');
            }
            return String.fromCharCode(first, second);
        }
        if (first >= 0xdc00 && first <= 0xdfff) {
            return malformedEvent('unpaired low surrogate in a JSON string');
        }
        return String.fromCharCode(first);
    }

    private readHexUnit(): number {
        const hex = this.text.slice(this.index, this.index + 4);
        if (hex.length !== 4 || !/^[0-9a-fA-F]{4}$/.test(hex)) {
            return malformedEvent('invalid unicode escape in a JSON string');
        }
        this.index += 4;
        return parseInt(hex, 16);
    }

    private parseNumber(): number {
        const start = this.index;
        let negative = false;
        if (this.peek() === '-') {
            negative = true;
            this.index += 1;
        }
        const first = this.peek();
        if (first === '0') {
            this.index += 1;
            if (isDigit(this.peek())) {
                return malformedEvent('JSON numbers cannot have leading zeros');
            }
        } else if (first !== undefined && first >= '1' && first <= '9') {
            while (isDigit(this.peek())) {
                this.index += 1;
            }
        } else {
            return malformedEvent('invalid JSON number');
        }
        let float = false;
        if (this.peek() === '.') {
            float = true;
            this.index += 1;
            if (!isDigit(this.peek())) {
                return malformedEvent('invalid JSON fraction');
            }
            while (isDigit(this.peek())) {
                this.index += 1;
            }
        }
        if (this.peek() === 'e' || this.peek() === 'E') {
            float = true;
            this.index += 1;
            if (this.peek() === '+' || this.peek() === '-') {
                this.index += 1;
            }
            if (!isDigit(this.peek())) {
                return malformedEvent('invalid JSON exponent');
            }
            while (isDigit(this.peek())) {
                this.index += 1;
            }
        }
        const token = this.text.slice(start, this.index);
        const value = Number(token);
        if (!Number.isFinite(value)) {
            return malformedEvent('number ' + token + ' is out of range');
        }
        if (float) {
            return invalidEvent('float ' + token + ' is not canonical');
        }
        if (negative) {
            return invalidEvent(
                token === '-0'
                    ? 'negative zero is not a canonical number'
                    : 'negative integers are not canonical',
            );
        }
        if (!Number.isSafeInteger(value) || value > MAX_SAFE_INTEGER) {
            return invalidEvent(
                'integer ' + token + ' exceeds the canonical safe integer range',
            );
        }
        return value;
    }

    private expectWord(word: string): void {
        if (this.text.slice(this.index, this.index + word.length) !== word) {
            return malformedEvent('invalid JSON literal');
        }
        this.index += word.length;
    }

    private skipWhitespace(): void {
        while (this.index < this.text.length) {
            const character = this.text[this.index];
            if (
                character !== ' ' &&
                character !== '\t' &&
                character !== '\n' &&
                character !== '\r'
            ) {
                return;
            }
            this.index += 1;
        }
    }

    private peek(): string | undefined {
        return this.index < this.text.length ? this.text[this.index] : undefined;
    }
}
`;

mkdirSync(generatedDir, { recursive: true });
const contractFile = join(generatedDir, 'agentContract.ts');
const canonicalFile = join(generatedDir, 'agentCanonical.ts');
writeFileSync(contractFile, contractSource);
writeFileSync(canonicalFile, canonicalSource);

console.log(
    'generate-agent-contract: wrote ' +
        relative(repoRoot, contractFile) +
        ' and ' +
        relative(repoRoot, canonicalFile) +
        ' from ' +
        relative(repoRoot, schemaPath) +
        ' (sha256:' +
        contractDigest +
        ')',
);
