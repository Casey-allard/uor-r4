import { render } from 'preact';
import { act } from 'preact/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { RichMessage } from './RichMessage.js';

let host: HTMLDivElement;
beforeEach(() => { host = document.createElement('div'); document.body.appendChild(host); });
afterEach(() => { render(null, host); host.remove(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

function show(content: string) {
    act(() => { render(<RichMessage content={content} />, host); });
}

describe('RichMessage', () => {
    it('renders headings, lists, quotes, and fenced code as semantic blocks', () => {
        show('# Plan\n\n- first\n- second\n\n1. alpha\n2. beta\n\n> Review note\n\n```rust\nfn main() {}\n```');
        expect(host.querySelector('h1')?.textContent).toBe('Plan');
        expect(Array.from(host.querySelectorAll('ul li')).map((node) => node.textContent)).toEqual(['first', 'second']);
        expect(Array.from(host.querySelectorAll('ol li')).map((node) => node.textContent)).toEqual(['alpha', 'beta']);
        expect(host.querySelector('blockquote')?.textContent).toBe('Review note');
        const code = host.querySelector('pre code')!;
        expect(code.className).toBe('language-rust');
        expect(code.textContent).toBe('fn main() {}');
    });

    it('renders punctuation and spaced fence tags as code with sanitized language classes', () => {
        show('```c++\n# code heading\n```\n\nvisible after\n\n``` rust\nfn main() {}\n```');
        expect(host.querySelector('h1')).toBeNull();
        const codes = Array.from(host.querySelectorAll('pre code'));
        expect(codes).toHaveLength(2);
        expect(codes[0].className).toBe('language-c');
        expect(codes[0].textContent).toBe('# code heading');
        expect(codes[1].className).toBe('language-rust');
        expect(codes[1].textContent).toBe('fn main() {}');
        expect(host.querySelector('p')?.textContent).toBe('visible after');
    });

    it('renders inline formatting and HTTP(S) links without fetching', () => {
        const fetchMock = vi.fn();
        vi.stubGlobal('fetch', fetchMock);
        show('Read **bold** and `code` with [docs](https://example.com/文档) and [http](http://example.com).');
        expect(host.querySelector('strong')?.textContent).toBe('bold');
        expect(host.querySelector('p code')?.textContent).toBe('code');
        const links = Array.from(host.querySelectorAll('a'));
        expect(links.map((link) => link.getAttribute('href'))).toEqual(['https://example.com/文档', 'http://example.com']);
        for (const link of links) {
            expect(link.target).toBe('_blank');
            expect(link.rel).toBe('noopener noreferrer');
        }
        expect(fetchMock).not.toHaveBeenCalled();
    });

    it('preserves a complete balanced parenthesized link destination', () => {
        show('[wiki](https://en.wikipedia.org/wiki/Function_(mathematics))');
        const link = host.querySelector('a')!;
        expect(link.getAttribute('href')).toBe('https://en.wikipedia.org/wiki/Function_(mathematics)');
        expect(link.rel).toBe('noopener noreferrer');
    });

    it('keeps raw HTML and unsafe or relative destinations inert', () => {
        show('<script>alert(1)</script>\n<img src=x onerror=alert(1)>\n[bad](javascript:alert(1))\n[data](data:text/html,hi)\n[vb](vbscript:msgbox(1))\n[proto](//evil.example)\n[relative](/guide)\n[x](https://example.com/a(b)');
        expect(host.querySelector('script, img, a')).toBeNull();
        expect(host.textContent).toContain('<script>alert(1)</script>');
        expect(host.textContent).toContain('<img src=x onerror=alert(1)>');
        expect(host.textContent).toContain('[bad](javascript:alert(1))');
        expect(host.textContent).toContain('[data](data:text/html,hi)');
        expect(host.textContent).toContain('[vb](vbscript:msgbox(1))');
        expect(host.textContent).toContain('[proto](//evil.example)');
        expect(host.textContent).toContain('[relative](/guide)');
        expect(host.textContent).toContain('[x](https://example.com/a(b)');
    });

    it('keeps unmatched inline markup and unclosed fences safe', () => {
        show('Literal **open and `tick`\n\n```ts\nconst <Tag> = value;');
        expect(host.querySelector('strong')).toBeNull();
        expect(host.querySelector('tag')).toBeNull();
        expect(host.querySelector('pre code')?.textContent).toBe('const <Tag> = value;');
    });

    it('preserves long and Unicode content without truncation', () => {
        const long = 'x'.repeat(120001);
        show(`**${long}**\n\n你好 🙂`);
        expect(host.querySelector('strong')?.textContent).toBe(long);
        expect(host.textContent).toContain('你好 🙂');
    });
});
