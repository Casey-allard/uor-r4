import { createElement } from 'preact';
import type { ComponentChild } from 'preact';

interface RichMessageProps {
    content: string;
}

const inlineToken = /(`[^`\n]+`|\*\*[^*\n]+\*\*|\[[^\]\n]+\]\((?:[^()\s]|\([^()\s]*\))+\))/g;
const markdownLink = /^\[([^\]\n]+)\]\(((?:[^()\s]|\([^()\s]*\))+)\)$/;
const fenceStart = /^```([^`]*)$/;
const headingStart = /^(#{1,6})\s+(.+)$/;
const unorderedStart = /^\s*[-+*]\s+/;
const orderedStart = /^\s*\d+[.)]\s+/;
const quoteStart = /^\s*>\s?/;

function httpDestination(raw: string): string | null {
    const destination = raw.trim();
    if (!/^https?:\/\//i.test(destination)) return null;
    try {
        const parsed = new URL(destination);
        return parsed.protocol === 'http:' || parsed.protocol === 'https:' ? destination : null;
    } catch {
        return null;
    }
}

function languageClass(info: string): string | undefined {
    const identifier = (info.trim().split(/\s+/)[0] || '')
        .replace(/[^a-zA-Z0-9_-]+/g, '-')
        .replace(/^-+|-+$/g, '');
    return identifier ? `language-${identifier}` : undefined;
}

function renderInline(text: string, prefix: string): ComponentChild[] {
    const children: ComponentChild[] = [];
    let last = 0;
    let sequence = 0;
    for (const match of text.matchAll(inlineToken)) {
        const start = match.index ?? 0;
        if (start > last) children.push(text.slice(last, start));
        const token = match[0];
        const key = `${prefix}-${sequence++}`;
        if (token.startsWith('`')) {
            children.push(<code key={key}>{token.slice(1, -1)}</code>);
        } else if (token.startsWith('**')) {
            children.push(<strong key={key}>{renderInline(token.slice(2, -2), key)}</strong>);
        } else {
            const link = markdownLink.exec(token);
            const destination = link ? httpDestination(link[2]) : null;
            if (link && destination) {
                children.push(<a key={key} href={destination} target="_blank" rel="noopener noreferrer">{link[1]}</a>);
            } else {
                children.push(token);
            }
        }
        last = start + token.length;
    }
    if (last < text.length) children.push(text.slice(last));
    return children;
}

function isBlockStart(line: string): boolean {
    return fenceStart.test(line) || headingStart.test(line) || unorderedStart.test(line) || orderedStart.test(line) || quoteStart.test(line);
}

export function RichMessage({ content }: RichMessageProps) {
    const lines = content.replace(/\r\n?/g, '\n').split('\n');
    const blocks: ComponentChild[] = [];
    let index = 0;
    let sequence = 0;

    while (index < lines.length) {
        if (!lines[index].trim()) {
            index += 1;
            continue;
        }
        const key = `block-${sequence++}`;
        const fence = fenceStart.exec(lines[index]);
        if (fence) {
            const language = languageClass(fence[1]);
            const body: string[] = [];
            index += 1;
            while (index < lines.length && !/^```\s*$/.test(lines[index])) {
                body.push(lines[index]);
                index += 1;
            }
            if (index < lines.length) index += 1;
            blocks.push(<pre key={key}><code className={language}>{body.join('\n')}</code></pre>);
            continue;
        }

        const heading = headingStart.exec(lines[index]);
        if (heading) {
            blocks.push(createElement(`h${heading[1].length}`, { key }, renderInline(heading[2], key)));
            index += 1;
            continue;
        }

        if (unorderedStart.test(lines[index]) || orderedStart.test(lines[index])) {
            const ordered = orderedStart.test(lines[index]);
            const marker = ordered ? orderedStart : unorderedStart;
            const items: string[] = [];
            while (index < lines.length && marker.test(lines[index])) {
                items.push(lines[index].replace(marker, ''));
                index += 1;
            }
            blocks.push(createElement(ordered ? 'ol' : 'ul', { key }, items.map((item, itemIndex) => (
                <li key={`${key}-${itemIndex}`}>{renderInline(item, `${key}-${itemIndex}`)}</li>
            ))));
            continue;
        }

        if (quoteStart.test(lines[index])) {
            const quoted: string[] = [];
            while (index < lines.length && quoteStart.test(lines[index])) {
                quoted.push(lines[index].replace(quoteStart, ''));
                index += 1;
            }
            blocks.push(<blockquote key={key}>{quoted.map((line, lineIndex) => (
                <span key={`${key}-${lineIndex}`}>{lineIndex > 0 && <br />}{renderInline(line, `${key}-${lineIndex}`)}</span>
            ))}</blockquote>);
            continue;
        }

        const paragraph: string[] = [];
        while (index < lines.length && lines[index].trim() && !isBlockStart(lines[index])) {
            paragraph.push(lines[index]);
            index += 1;
        }
        blocks.push(<p key={key}>{paragraph.map((line, lineIndex) => (
            <span key={`${key}-${lineIndex}`}>{lineIndex > 0 && <br />}{renderInline(line, `${key}-${lineIndex}`)}</span>
        ))}</p>);
    }

    return <div className="rich-message">{blocks}</div>;
}
