function finite(value) {
    if (value === null || value === undefined || value === '') return null;
    return Number.isFinite(Number(value)) ? Number(value) : null;
}

function grouped(value) {
    return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function duration(ns) {
    const milliseconds = Number(ns) / 1000000;
    if (milliseconds >= 1000) return (milliseconds / 1000).toFixed(3) + ' s';
    return milliseconds.toFixed(3) + ' ms';
}

export function createStreamMetrics() {
    let state = {
        completionTokens: null,
        tokensPerSecondMilli: null,
        firstTokenNs: null,
        elapsedNs: null,
        promptTokens: null,
        totalTokens: null,
        finishReason: null,
        exact: false,
    };
    let chars = null;
    let serverRateProvided = false;

    function update(input = {}) {
        const usage = input.usage && typeof input.usage === 'object' ? input.usage : null;
        const uor = input.uor && typeof input.uor === 'object' ? input.uor : null;
        if (Array.isArray(uor && uor.completion_ids) && uor.completion_ids.length > 0) {
            state.completionTokens = uor.completion_ids.length;
            state.exact = true;
        }
        if (usage) {
            const completion = finite(usage.completion_tokens);
            if (completion !== null) state.completionTokens = completion;
            state.promptTokens = finite(usage.prompt_tokens);
            state.totalTokens = finite(usage.total_tokens);
            if (completion !== null && state.promptTokens !== null && state.totalTokens !== null) state.exact = true;
        }
        const serverRate = finite(uor && uor.tokens_per_second_milli);
        if (serverRate !== null) {
            state.tokensPerSecondMilli = serverRate;
            serverRateProvided = true;
        }
        const first = finite(uor && uor.first_token_ns) ?? finite(input.firstTokenNs);
        if (first !== null) state.firstTokenNs = first;
        const elapsed = finite(uor && uor.elapsed_ns) ?? finite(input.elapsedNs);
        if (elapsed !== null) state.elapsedNs = elapsed;
        if (typeof input.finishReason === 'string') state.finishReason = input.finishReason;
        else if (typeof (uor && uor.stop) === 'string') state.finishReason = uor.stop;
        if (!serverRateProvided && state.completionTokens !== null && state.elapsedNs > 0) {
            state.tokensPerSecondMilli = Math.floor(state.completionTokens * 1000000000000 / state.elapsedNs);
        }
        if (Number.isFinite(Number(input.chars))) chars = Number(input.chars);
        return snapshot();
    }

    function snapshot() {
        return { ...state };
    }

    function text() {
        if (!state.exact) {
            const parts = [];
            if (chars !== null) parts.push(grouped(chars) + ' chars');
            if (state.elapsedNs !== null) parts.push(duration(state.elapsedNs));
            parts.push('usage unavailable');
            return parts.join(' · ');
        }
        const parts = [(state.completionTokens || 0) + ' token' + (state.completionTokens === 1 ? '' : 's')];
        if (state.tokensPerSecondMilli !== null) parts.push((state.tokensPerSecondMilli / 1000).toFixed(1) + ' tok/s');
        if (state.firstTokenNs !== null) parts.push('TTFT ' + duration(state.firstTokenNs));
        if (state.elapsedNs !== null) parts.push(duration(state.elapsedNs));
        if (state.promptTokens !== null && state.totalTokens !== null) {
            parts.push(state.promptTokens + '/' + state.completionTokens + '/' + state.totalTokens);
        } else {
            parts.push('prompt/total usage unavailable');
        }
        if (state.finishReason) parts.push(state.finishReason);
        return parts.join(' · ');
    }

    return { update, snapshot, text };
}
