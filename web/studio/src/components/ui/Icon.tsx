const paths: Record<string, string> = {
    agent: 'M4 5h16v11H8l-4 3V5Zm4 4h8M8 12h5',
    workspace: 'M3 7h7l2 2h9v10H3V7Zm0 5h18',
    changes: 'M5 4h14v16H5V4Zm4 4h6M9 12h6M9 16h4',
    terminal: 'm5 7 5 5-5 5m7 0h7',
    models: 'M12 3v4m0 10v4M3 12h4m10 0h4M6 6l3 3m6 6 3 3m0-12-3 3m-6 6-3 3',
    guide: 'M5 4h13v15H5V4Zm3 4h7M8 12h7M8 16h5',
    plus: 'M12 5v14M5 12h14',
    split: 'M4 5h16v14H4V5Zm8 0v14',
    close: 'm6 6 12 12M18 6 6 18',
    folder: 'M3 7h7l2 2h9v10H3V7Z',
    command: 'm9 6-4 6 4 6m6-12 4 6-4 6',
    settings: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Zm0-5v3m0 12v3M3 12h3m12 0h3M5.6 5.6l2.1 2.1m8.6 8.6 2.1 2.1m0-12.8-2.1 2.1m-8.6 8.6-2.1 2.1',
    inspector: 'M4 4h16v16H4V4Zm5 0v16M10 9h6m-6 4h6',
    browser: 'M3 5h18v14H3V5Zm0 4h18M6 7h.01M9 7h.01',
    send: 'm4 4 17 8-17 8 3-8-3-8Zm3 8h14',
    attach: 'm8 12 6-6a4 4 0 0 1 6 6l-8 8a6 6 0 0 1-8-8l8-8',
    save: 'M5 4h12l3 3v13H4V4h1Zm3 0v6h8V4M8 20v-6h8v6',
    chevron: 'm8 10 4 4 4-4',
};

export function Icon({ name, size = 16 }: { name: keyof typeof paths | string; size?: number }) {
    return <svg aria-hidden="true" viewBox="0 0 24 24" width={size} height={size} fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round"><path d={paths[name] || paths.guide} /></svg>;
}
