// Minimal local declarations for the two Node built-ins used by the focused
// contract tests. This package has no @types/node, and Task 2 must not add a
// dependency; only this surface is used.
declare module 'node:crypto' {
    export interface Hash {
        update(data: Uint8Array | string): Hash;
        digest(encoding: 'hex'): string;
    }

    export function createHash(algorithm: 'sha256'): Hash;
}

declare module 'node:fs/promises' {
    export function readFile(path: URL | string): Promise<Uint8Array>;
    export function readFile(path: URL | string, encoding: 'utf8'): Promise<string>;
}

declare module 'node:path' {
    export function resolve(...paths: string[]): string;
}
