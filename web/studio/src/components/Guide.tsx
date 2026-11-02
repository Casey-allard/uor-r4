export function Guide() {
    return (
        <section aria-label="Studio guide">
            <h1>Guide</h1>
            <h2>Primary workflow</h2>
            <ol>
                <li>Open or import a workspace, then open a file in an editor tab.</li>
                <li>Attach current-file or pasted context when using an API provider.</li>
                <li>Select a local artifact or API source. Selection alone does not mean an artifact is loaded or an endpoint is reachable.</li>
                <li>Run, review the response, edit or insert it, and save through the existing workspace path.</li>
            </ol>
            <h2>Model limits</h2>
            <p>The M1 discrete model provides top-8 next-token prediction; it is not a conversational assistant. A loaded artifact with generation disabled must not be presented as chat.</p>
            <p>Local WASM prompts stay in the browser worker. Loopback, private-network, and remote API privacy are labeled separately from local execution.</p>
            <h2>Keyboard and offline behavior</h2>
            <p>Navigation, editor, model, session, palette, and dialog controls use native keyboard semantics. The legacy page makes no external requests at load; remote model and API contact remains user-initiated.</p>
            <p>Full real-browser parity, one-time browser-data migration, and production cutover are later verification stages.</p>
        </section>
    );
}
