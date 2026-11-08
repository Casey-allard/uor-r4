if (new URLSearchParams(globalThis.location?.search || '').get('typed') === '1') {
    import('../../src/main.ts');
}
