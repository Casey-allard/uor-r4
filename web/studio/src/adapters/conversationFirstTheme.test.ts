// @ts-expect-error Node's filesystem API is available in Vitest but not in the browser-only type config.
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const styles = readFileSync('index.css', 'utf8');

describe('conversation-first Studio theme', () => {
    it('removes navigation, inspector, and status panels from the default chat view', () => {
        expect(styles).toContain('body[data-view="chat"] .nav-sidebar,');
        expect(styles).toContain('body[data-view="chat"] #inspector,');
        expect(styles).toContain('body[data-view="chat"] .statusbar');
        expect(styles).toMatch(/body\[data-view="chat"\] \.shell-body[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s*!important/s);
    });

    it('uses a rounded floating composer and mobile-safe message text', () => {
        expect(styles).toMatch(/body\[data-view="chat"\] \.composer-box[^}]*border-radius:\s*24px/s);
        expect(styles).toMatch(/body\[data-view="chat"\] #chatInput[^}]*font-size:\s*16px/s);
        expect(styles).toContain('body[data-view="chat"] .empty-quick-primary');
        expect(styles).toMatch(/body\[data-view="chat"\] \.view-head-actions \.ghost-btn,\s*body\[data-view="chat"\] \.view-head-actions \.badge\s*\{[^}]*display:\s*inline-flex\s*!important/s);
        expect(styles).toMatch(/body\[data-view="chat"\] #btnOpenModelSources,\s*body\[data-view="chat"\] #btnNewChat2\s*\{\s*display:\s*inline-flex\s*!important/s);
    });

    it('keeps the mobile header compact and uses system typography', () => {
        expect(styles).toMatch(/body\[data-view="chat"\]\s*\{[^}]*font-family:\s*-apple-system/s);
        expect(styles).toMatch(/body\[data-view="chat"\] #btnMobileNav\s*\{\s*display:\s*none\s*!important/s);
        expect(styles).toMatch(/@media \(max-width: 780px\)[\s\S]*body\[data-view="chat"\] \.view-head-actions \.badge\s*\{\s*display:\s*none\s*!important/s);
    });
});
