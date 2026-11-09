import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './tests/e2e', outputDir: '../../local/kimi/harness-parity/playwright-output', preserveOutput: 'always', fullyParallel: false, workers: 1, reporter: 'list',
    use: { baseURL: 'http://127.0.0.1:4173', browserName: 'chromium', headless: true, viewport: { width: 1440, height: 900 }, launchOptions: { executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' }, screenshot: 'only-on-failure' },
    webServer: { command: 'npm run build && ruby -run -ehttpd dist -p 4173', url: 'http://127.0.0.1:4173/?typed=1', reuseExistingServer: false, timeout: 120000 },
});
