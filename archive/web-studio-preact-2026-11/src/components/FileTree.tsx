import type { RefObject } from 'preact';
import { useId, useMemo, useState } from 'preact/hooks';
import type { FileNode } from './types.js';
import { Icon } from './ui/Icon.js';

export const MAX_TREE_SEARCH_RESULTS = 50;
export const MAX_TREE_SEARCH_SCAN = 2000;

export interface FileTreeMeta {
    truncated: boolean;
    truncatedReported: boolean;
    symlinksSkipped: number;
    readErrors: number;
}

export interface FileTreeProps {
    nodes: FileNode[];
    meta: FileTreeMeta;
    activePath: string | null;
    phase: 'loading' | 'ready' | 'error';
    error: string | null;
    onOpen(path: string): void;
    searchRef?: RefObject<HTMLInputElement | null>;
}

export interface TreeSearch {
    matches: FileNode[];
    scanned: number;
    truncated: boolean;
}

export function searchFileNodes(nodes: readonly FileNode[], query: string): TreeSearch {
    const needle = query.trim().toLowerCase();
    const matches: FileNode[] = [];
    let index = 0;
    while (index < nodes.length && index < MAX_TREE_SEARCH_SCAN && matches.length < MAX_TREE_SEARCH_RESULTS) {
        const node = nodes[index]!;
        if (node.path.toLowerCase().includes(needle)) matches.push(node);
        index += 1;
    }
    return { matches, scanned: index, truncated: index < nodes.length };
}

interface Branch {
    name: string;
    path: string;
    directories: Branch[];
    files: FileNode[];
}

function byName(left: { name: string }, right: { name: string }): number {
    return left.name.localeCompare(right.name);
}

function buildTree(nodes: readonly FileNode[]): Branch {
    const root: Branch = { name: '', path: '', directories: [], files: [] };
    const directories = new Map<string, Branch>([['', root]]);
    const ensure = (path: string): Branch => {
        const existing = directories.get(path);
        if (existing) return existing;
        const segment = path.split('/').pop()!;
        const parent = ensure(path.includes('/') ? path.slice(0, path.lastIndexOf('/')) : '');
        const branch: Branch = { name: segment, path, directories: [], files: [] };
        parent.directories.push(branch);
        directories.set(path, branch);
        return branch;
    };
    for (const node of nodes) {
        const parentPath = node.path.includes('/') ? node.path.slice(0, node.path.lastIndexOf('/')) : '';
        if (node.kind === 'directory') ensure(node.path);
        else ensure(parentPath).files.push(node);
    }
    return root;
}

function sortTree(branch: Branch): void {
    branch.directories.sort(byName);
    branch.files.sort(byName);
    for (const directory of branch.directories) sortTree(directory);
}

export function FileTree({ nodes, meta, activePath, phase, error, onOpen, searchRef }: FileTreeProps) {
    const titleId = useId();
    const statusId = useId();
    const [query, setQuery] = useState('');
    const searching = query.trim().length > 0;
    const search = useMemo(() => (searching ? searchFileNodes(nodes, query) : null), [nodes, query, searching]);
    const tree = useMemo(() => {
        const built = buildTree(nodes);
        sortTree(built);
        return built;
    }, [nodes]);

    const status = search
        ? search.matches.length === 0
            ? search.truncated
                ? `No matches in the first ${search.scanned} loaded files. The search scans at most ${MAX_TREE_SEARCH_SCAN} entries.`
                : `No loaded files match "${query.trim()}".`
            : search.truncated
                ? `Showing the first ${search.matches.length} matches. Refine the search to narrow the results.`
                : `${search.matches.length} of ${nodes.length} loaded files match "${query.trim()}".`
        : phase === 'loading'
            ? 'Loading files…'
            : '';

    const fileButton = (node: FileNode) => (
        <button
            type="button"
            class={`tree-button tree-file${activePath === node.path ? ' is-active' : ''}`}
            aria-current={activePath === node.path ? 'true' : undefined}
            onClick={() => onOpen(node.path)}
        >
            <span class="tree-name">{node.name}</span>
        </button>
    );

    const branchList = (branch: Branch): preact.JSX.Element => (
        <ul class="tree-list">
            {branch.directories.map((directory) => (
                <li class="tree-item" key={directory.path}>
                    <div class="tree-row">
                        <span class="tree-button tree-folder">
                            <Icon name="folder" />
                            <span class="tree-name">{directory.name}</span>
                        </span>
                    </div>
                    {directory.directories.length > 0 || directory.files.length > 0 ? branchList(directory) : null}
                </li>
            ))}
            {branch.files.map((file) => <li class="tree-item" key={file.path}>{fileButton(file)}</li>)}
        </ul>
    );

    return (
        <section class="file-tree" aria-labelledby={titleId}>
            <h2 id={titleId}>Files</h2>
            <label class="visually-hidden" for={`${titleId}-search`}>Search files</label>
            <input
                id={`${titleId}-search`}
                ref={searchRef}
                type="search"
                aria-label="Search files"
                aria-describedby={statusId}
                placeholder="Search files"
                style={{ fontSize: '16px' }}
                value={query}
                onInput={(event) => setQuery(event.currentTarget.value)}
            />
            <p id={statusId} role="status">{status}</p>
            {error !== null && <p class="workbench-alert" role="alert">{error}</p>}
            {meta.truncated && (
                meta.truncatedReported
                    ? <p class="tree-notice">The file list is incomplete: the companion returned the first {nodes.length} entries.</p>
                    : <p class="tree-notice">The companion returned exactly {nodes.length} entries without a truncation flag; the listing may be capped.</p>
            )}
            {meta.symlinksSkipped > 0 && (
                <p class="tree-notice">
                    {meta.symlinksSkipped} symbolic {meta.symlinksSkipped === 1 ? 'link was' : 'links were'} skipped.
                </p>
            )}
            {meta.readErrors > 0 && (
                <p class="tree-notice">
                    {meta.readErrors} {meta.readErrors === 1 ? 'entry could not be read' : 'entries could not be read'}.
                </p>
            )}
            {search
                ? search.matches.length > 0 && (
                    <ul class="tree-list" aria-label="Search results">
                        {search.matches.map((node) => (
                            <li class="tree-item" key={node.path}>
                                {node.kind === 'directory'
                                    ? <span class="tree-button tree-folder"><Icon name="folder" /><span class="tree-name">{node.path}</span></span>
                                    : fileButton(node)}
                            </li>
                        ))}
                    </ul>
                )
                : phase === 'ready' && nodes.length === 0 && (
                    <p class="hint">No files are listed for this session yet. Opening an uploaded file adds it here; a connected companion lists its project folder.</p>
                )}
            {!search && phase === 'ready' && nodes.length > 0 && branchList(tree)}
        </section>
    );
}
