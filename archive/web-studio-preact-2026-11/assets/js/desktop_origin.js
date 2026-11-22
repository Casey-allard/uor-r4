function desktopUrls(value, location) {
    const local = new URL(typeof location === 'string' ? location : location?.href || 'http://desktop.local/');
    const candidate = value instanceof URL ? value : new URL(String(value || ''), local);
    return { candidate, local };
}

export function isDeclaredDesktopOrigin(value, location = globalThis.location) {
    try {
        const { candidate, local } = desktopUrls(value, location);
        if (candidate.username || candidate.password) return false;
        if (local.protocol === 'tauri:') {
            if (candidate.protocol === 'tauri:') {
                return candidate.host === local.host && candidate.port === local.port;
            }
            return (candidate.protocol === 'http:' || candidate.protocol === 'https:')
                && candidate.host === 'tauri.localhost'
                && candidate.port === '';
        }
        return candidate.origin === local.origin;
    } catch (_error) {
        return false;
    }
}

export function normalizeNativeApiUrl(value, location = globalThis.document?.baseURI || globalThis.location) {
    const { candidate, local } = desktopUrls(value, location);
    if (!isDeclaredDesktopOrigin(candidate, local)) throw new Error('The native bootstrap must be same-origin.');
    if (local.protocol === 'tauri:' && candidate.protocol === 'tauri:') {
        return new URL('http://tauri.localhost' + candidate.pathname + candidate.search);
    }
    return candidate;
}
