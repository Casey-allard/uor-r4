import test from 'node:test';
import assert from 'node:assert/strict';
import { createStreamMetrics } from './stream_metrics.js';

test('live native token IDs and client nanoseconds produce exact measured rates', () => {
    const metrics = createStreamMetrics();
    metrics.update({ uor: { completion_ids: [340] }, elapsedNs: 2000000, firstTokenNs: 1000000, chars: 2 });
    assert.deepEqual(metrics.snapshot(), {
        completionTokens: 1,
        tokensPerSecondMilli: 500000,
        firstTokenNs: 1000000,
        elapsedNs: 2000000,
        promptTokens: null,
        totalTokens: null,
        finishReason: null,
        exact: true,
    });
    assert.equal(metrics.text(), '1 token · 500.0 tok/s · TTFT 1.000 ms · 2.000 ms · prompt/total usage unavailable');
});

test('final native usage and server timing remain exact and include the finish reason', () => {
    const metrics = createStreamMetrics();
    metrics.update({
        usage: { prompt_tokens: 12, completion_tokens: 3, total_tokens: 15 },
        uor: { completion_ids: [340, 16, 1], elapsed_ns: 5000000, first_token_ns: 2000000, tokens_per_second_milli: 600000, stop: 'eos' },
        finishReason: 'stop',
        chars: 2,
    });
    assert.deepEqual(metrics.snapshot(), {
        completionTokens: 3,
        tokensPerSecondMilli: 600000,
        firstTokenNs: 2000000,
        elapsedNs: 5000000,
        promptTokens: 12,
        totalTokens: 15,
        finishReason: 'stop',
        exact: true,
    });
    assert.equal(metrics.text(), '3 tokens · 600.0 tok/s · TTFT 2.000 ms · 5.000 ms · 12/3/15 · stop');
});

test('empty token arrays and empty usage objects stay character-only', () => {
    const metrics = createStreamMetrics();
    metrics.update({ uor: { completion_ids: [] }, usage: {}, chars: 17 });
    assert.equal(metrics.snapshot().exact, false);
    assert.equal(metrics.text(), '17 chars · usage unavailable');
});

test('partial usage and rate-only payloads never produce null or fabricated exact usage', () => {
    const metrics = createStreamMetrics();
    metrics.update({ usage: { prompt_tokens: 12, total_tokens: 15 }, chars: 4 });
    assert.equal(metrics.snapshot().exact, false);
    assert.equal(metrics.text(), '4 chars · usage unavailable');
    assert.equal(metrics.text().includes('null'), false);
    const rateOnly = createStreamMetrics();
    rateOnly.update({ uor: { tokens_per_second_milli: 500000 }, chars: 7 });
    assert.equal(rateOnly.snapshot().exact, false);
    assert.equal(rateOnly.text(), '7 chars · usage unavailable');
});

test('exact live completion keeps partial final prompt and total usage explicitly unavailable', () => {
    const metrics = createStreamMetrics();
    metrics.update({ uor: { completion_ids: [1, 2, 3] }, elapsedNs: 3000000, chars: 2 });
    metrics.update({ usage: { completion_tokens: 3 }, chars: 2 });
    assert.equal(metrics.snapshot().exact, true);
    assert.equal(metrics.text(), '3 tokens · 1000.0 tok/s · 3.000 ms · prompt/total usage unavailable');
    assert.equal(metrics.text().includes('null'), false);
});

test('final usage recomputes a stale client rate when no server rate exists', () => {
    const metrics = createStreamMetrics();
    metrics.update({ uor: { completion_ids: [1] }, elapsedNs: 2000000, chars: 1 });
    assert.equal(metrics.snapshot().tokensPerSecondMilli, 500000);
    metrics.update({ usage: { prompt_tokens: 4, completion_tokens: 2, total_tokens: 6 }, uor: { completion_ids: [1, 2], elapsed_ns: 10000000 }, chars: 2 });
    assert.equal(metrics.snapshot().tokensPerSecondMilli, 200000);
});

test('character-only and unavailable metrics never masquerade as token usage', () => {
    const metrics = createStreamMetrics();
    metrics.update({ chars: 1234 });
    assert.equal(metrics.snapshot().exact, false);
    assert.equal(metrics.text(), '1,234 chars · usage unavailable');
    assert.equal(metrics.text().includes('tok/s'), false);
});
