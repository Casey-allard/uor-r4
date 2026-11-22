import { describe, expect, it } from 'vitest';

import type { AgentEvent } from './generated/agentContract.js';
import {
    CanonicalJsonError,
    canonicalJson,
    digestEvent,
    parseCanonicalJson,
} from './generated/agentCanonical.js';

// These exact strings and the digest are also asserted by the Rust reference
// test crates/uor-r4-studio-agent/tests/canonical_json.rs
// (cross_runtime_vectors_are_pinned). Keep both sides in step.
const EVENT_CANONICAL =
    '{"data":{"message_id":"msg_1f2e3d4c5b6a7988","text":"ordered"},"event_id":"evt_run_7f3a2c1b9d4e6a80_1_000000000006","generation":1,"project_id":"prj_4b1d9e0c7a2f6e51","recorded_at_ms":1794283200123,"run_id":"run_7f3a2c1b9d4e6a80","schema":"uor-r4.studio.agent-event/1","sequence":6,"task_id":"tsk_9a3c5e7b1d2f4a60","type":"assistant.message"}';
const EVENT_DIGEST =
    'sha256:fac58990349c5d556f0d14f927d9a37eacfe2a8d90683fcbb04a146b086e6565';

function eventWithOrder(reversed: boolean): AgentEvent {
    return {
        schema: 'uor-r4.studio.agent-event/1',
        event_id: 'evt_run_7f3a2c1b9d4e6a80_1_000000000006',
        project_id: 'prj_4b1d9e0c7a2f6e51',
        task_id: 'tsk_9a3c5e7b1d2f4a60',
        run_id: 'run_7f3a2c1b9d4e6a80',
        generation: 1,
        sequence: 6,
        recorded_at_ms: 1_794_283_200_123,
        type: 'assistant.message',
        data: reversed
            ? { text: 'ordered', message_id: 'msg_1f2e3d4c5b6a7988' }
            : { message_id: 'msg_1f2e3d4c5b6a7988', text: 'ordered' },
    };
}

function codeOf(run: () => unknown): string {
    try {
        run();
    } catch (error) {
        expect(error).toBeInstanceOf(CanonicalJsonError);
        if (error instanceof CanonicalJsonError) {
            return error.code;
        }
        throw error;
    }
    throw new Error('expected canonicalization to fail');
}

describe('canonicalJson', () => {
    it('sorts object keys by ASCII bytes and preserves array order', () => {
        expect(canonicalJson({ b: 2, a: 1, a_b: 5, 'a.b': 3, 'a-b': 4 })).toBe(
            '{"a":1,"a-b":4,"a.b":3,"a_b":5,"b":2}',
        );
        expect(canonicalJson({ z: { b: true, a: null }, y: [3, 2, 1] })).toBe(
            '{"y":[3,2,1],"z":{"a":null,"b":true}}',
        );
    });

    it('matches the Rust empty-key and control-escape vectors', () => {
        expect(canonicalJson({ '': 1, a: 2 })).toBe('{"":1,"a":2}');
        expect(canonicalJson({ a: '\u0000\u000b\u001f' })).toBe(
            '{"a":"\\u0000\\u000b\\u001f"}',
        );
        const controls = Array.from({ length: 0x20 }, (_, code) =>
            String.fromCharCode(code),
        ).join('');
        const escaped = Array.from({ length: 0x20 }, (_, code) =>
            '\\u00' + code.toString(16).padStart(2, '0'),
        ).join('');
        expect(canonicalJson({ a: controls })).toBe('{"a":"' + escaped + '"}');
        expect(canonicalJson({ a: '\u007f' })).toBe('{"a":"\u007f"}');
    });

    it('keeps non-ASCII scalar values as raw UTF-8', () => {
        expect(canonicalJson({ a: 'café ☃' })).toBe('{"a":"café ☃"}');
        expect(canonicalJson('\ud83d\ude00')).toBe('"😀"');
    });

    it('rejects floats, NaN and infinity with invalid_event', () => {
        expect(() => canonicalJson({ x: 1.5 })).toThrow(/invalid_event/);
        expect(codeOf(() => canonicalJson({ x: 1.5 }))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(Number.NaN))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(Number.POSITIVE_INFINITY))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(Number.NEGATIVE_INFINITY))).toBe('invalid_event');
        // JavaScript normalizes 1.0 and 1e3 to integers before canonicalJson
        // sees them; only the raw parser can reject their lexical forms.
        expect(canonicalJson({ x: 1.0 })).toBe('{"x":1}');
        expect(canonicalJson(1e3)).toBe('1000');
    });

    it('rejects JavaScript negative zero with Object.is', () => {
        expect(Object.is(JSON.parse('-0'), -0)).toBe(true);
        expect(Object.is(-0, 0)).toBe(false);
        expect(Object.is(JSON.parse('-0'), 0)).toBe(false);
        expect(codeOf(() => canonicalJson(JSON.parse('-0')))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson({ x: -0 }))).toBe('invalid_event');
        expect(canonicalJson(0)).toBe('0');
    });

    it('rejects negative and unsafe integers', () => {
        expect(codeOf(() => canonicalJson(-1))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(9_007_199_254_740_992))).toBe('invalid_event');
        expect(canonicalJson(9_007_199_254_740_991)).toBe('9007199254740991');
    });

    it('rejects non-ASCII, uppercase and punctuation keys', () => {
        for (const key of ['é', 'A', '$schema', 'a b', 'a/b']) {
            expect(codeOf(() => canonicalJson({ [key]: 1 }))).toBe('invalid_event');
        }
        expect(canonicalJson({ 'a-b.c_d9': 1 })).toBe('{"a-b.c_d9":1}');
    });

    it('rejects unpaired surrogates in string values', () => {
        expect(codeOf(() => canonicalJson('\ud800'))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson('\udc00'))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson({ x: 'a\ud800b' }))).toBe('invalid_event');
    });

    it('rejects values outside the JSON data model', () => {
        expect(codeOf(() => canonicalJson(undefined))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(() => 1))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(new Date(0)))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson(new Map()))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson([1, undefined]))).toBe('invalid_event');
        expect(codeOf(() => canonicalJson({ x: Symbol('x') }))).toBe('invalid_event');
    });
});

describe('parseCanonicalJson', () => {
    it('sorts and unescapes raw JSON exactly like Rust', () => {
        expect(parseCanonicalJson('{"b":2,"a":{"d":4,"c":3}}')).toBe(
            '{"a":{"c":3,"d":4},"b":2}',
        );
        expect(
            parseCanonicalJson(
                '{"a":"caf\\u00e9 \\"q\\" \\\\ line\\nend","z":"\\u0000\\u001f"}',
            ),
        ).toBe('{"a":"café \\"q\\" \\\\ line\\u000aend","z":"\\u0000\\u001f"}');
        expect(parseCanonicalJson('"caf\\u00e9"')).toBe('"café"');
        expect(parseCanonicalJson('{"x":"\\ud83d\\ude00"}')).toBe('{"x":"😀"}');
        expect(parseCanonicalJson('{"a":1} \n\t')).toBe('{"a":1}');
        expect(parseCanonicalJson('null')).toBe('null');
        expect(parseCanonicalJson('0')).toBe('0');
    });

    it('rejects raw nested duplicate keys before object conversion', () => {
        for (const text of [
            '{"a":1,"a":2}',
            '{"outer":{"a":1,"a":2}}',
            '[{"x":1,"x":1}]',
            '{"a":{"b":{"c":1,"c":1}}}',
            '{"a":1,"\\u0061":2}',
        ]) {
            expect(codeOf(() => parseCanonicalJson(text))).toBe('invalid_event');
        }
        expect(parseCanonicalJson('{"a":1,"b":2}')).toBe('{"a":1,"b":2}');
    });

    it('rejects non-canonical numbers lexically', () => {
        for (const text of [
            '{"x":1.5}',
            '{"x":1.0}',
            '{"x":-1}',
            '{"x":-0}',
            '{"x":9007199254740992}',
            '{"x":1e3}',
            '{"x":-9007199254740991}',
        ]) {
            expect(codeOf(() => parseCanonicalJson(text))).toBe('invalid_event');
        }
        expect(parseCanonicalJson('{"x":0,"y":9007199254740991}')).toBe(
            '{"x":0,"y":9007199254740991}',
        );
    });

    it('matches the Rust number-overflow boundary', () => {
        expect(codeOf(() => parseCanonicalJson('{"x":1e308}'))).toBe('invalid_event');
        expect(codeOf(() => parseCanonicalJson('{"x":1e400}'))).toBe('malformed_event');
        expect(codeOf(() => parseCanonicalJson('{"x":-1e400}'))).toBe('malformed_event');
        expect(codeOf(() => parseCanonicalJson('{"x":' + '9'.repeat(400) + '}'))).toBe(
            'malformed_event',
        );
    });

    it('rejects malformed text and unpaired surrogates', () => {
        for (const text of [
            '{"x":1',
            '\ufeff{"x":1}',
            '{"x":1} trailing',
            '{"x":"\\ud800"}',
            '{"x":"\\udc00"}',
            '{"x":"\ud800"}',
            '{"x":1,}',
            "",
            '[] []',
            '{"x":01}',
        ]) {
            expect(codeOf(() => parseCanonicalJson(text))).toBe('malformed_event');
        }
    });

    it('rejects non-ASCII keys after parsing', () => {
        expect(codeOf(() => parseCanonicalJson('{"é":1}'))).toBe('invalid_event');
    });

    it('matches the Rust 127-level nesting limit', () => {
        const depth127 = '['.repeat(127) + '0' + ']'.repeat(127);
        const depth128 = '['.repeat(128) + '0' + ']'.repeat(128);
        expect(parseCanonicalJson(depth127)).toBe(depth127);
        expect(codeOf(() => parseCanonicalJson(depth128))).toBe('malformed_event');
    });

    it('preserves prototype-like keys without polluting objects', () => {
        expect(parseCanonicalJson('{"__proto__":{"polluted":true},"a":1}')).toBe(
            '{"__proto__":{"polluted":true},"a":1}',
        );
        expect(({} as Record<string, unknown>).polluted).toBeUndefined();
    });
});

describe('digestEvent', () => {
    it('matches the pinned Rust canonical string and digest independent of insertion order', async () => {
        const forward = eventWithOrder(false);
        const reversed = eventWithOrder(true);
        expect(canonicalJson(forward)).toBe(EVENT_CANONICAL);
        expect(await digestEvent(forward)).toBe(EVENT_DIGEST);
        expect(await digestEvent(reversed)).toBe(EVENT_DIGEST);
    });

    it('rejects an event containing a non-canonical value', async () => {
        const event = eventWithOrder(false) as unknown as Record<string, unknown>;
        await expect(
            digestEvent({ ...event, sequence: 1.5 } as unknown as AgentEvent),
        ).rejects.toBeInstanceOf(CanonicalJsonError);
    });
});
