import { useEffect, useMemo, useRef, useState } from 'preact/hooks';
import type { ChangesSnapshot } from '../agent/client.js';
import { sha256Canonical } from '../agent/hostClient.js';
import { DiffCommentView } from './DiffComment.js';

export type DiffScope = 'last_turn' | 'branch';
export type DiffSource = 'browser' | 'host' | 'upload' | null;

export interface DiffComment {
    commentId: string;
    taskId: string;
    scope: DiffScope;
    path: string;
    line: number;
    body: string;
    headDigest: string;
    createdAtMs: number;
}

export interface DiffCommentDraft {
    path: string;
    line: number;
    body: string;
}

export interface DiffReviewProps {
    taskId: string | null;
    scope: DiffScope;
    onScope(scope: DiffScope): void;
    comments: DiffComment[];
    onComment(comment: DiffCommentDraft): void | Promise<void>;
    snapshot: ChangesSnapshot | null;
    source?: DiffSource;
    loading?: boolean;
    error?: string | null;
    commentDisabledReason?: string | null;
    idPrefix?: string;
}

export interface DiffCommentStatus {
    current: boolean;
    reason: string | null;
}

export const MAX_UPLOAD_FILES = 20;
export const MAX_UPLOAD_FILE_BYTES = 128 * 1024;
export const MAX_UPLOAD_TOTAL_BYTES = 512 * 1024;
export const MAX_REVIEW_LINES = 400;
export const MAX_COMMENT_BODY = 8192;

let nextReviewId = 0;

export function snapshotFileLines(file: Record<string, unknown>): string[] | null {
    const content = typeof file.content === 'string'
        ? file.content
        : typeof file.diff === 'string'
          ? file.diff
          : null;
    if (content === null) return null;
    return content.split('\n').slice(0, MAX_REVIEW_LINES);
}

export function diffCommentStatus(
    comment: DiffComment,
    snapshot: ChangesSnapshot | null,
): DiffCommentStatus {
    if (snapshot === null) {
        return {
            current: false,
            reason: 'No current snapshot is available; this comment is stale.',
        };
    }
    if (comment.headDigest !== snapshot.headDigest) {
        return {
            current: false,
            reason: 'The snapshot head changed; this comment is stale.',
        };
    }
    const file = snapshot.files.find((candidate) => candidate.path === comment.path);
    if (file === undefined) {
        return {
            current: false,
            reason: 'This file is no longer in this snapshot.',
        };
    }
    if (comment.line === 0) return { current: true, reason: null };
    const lines = snapshotFileLines(file);
    if (lines === null) {
        return {
            current: false,
            reason: 'This snapshot has no line mapping for the file.',
        };
    }
    if (comment.line > lines.length) {
        return {
            current: false,
            reason: `Line ${comment.line} is not present in this snapshot.`,
        };
    }
    return { current: true, reason: null };
}

function completeUtf8Prefix(bytes: Uint8Array): Uint8Array {
    let end = bytes.length;
    let index = end - 1;
    let continuation = 0;
    while (index >= 0 && (bytes[index]! & 0xc0) === 0x80 && continuation < 3) {
        index -= 1;
        continuation += 1;
    }
    if (index < 0) return bytes;
    const lead = bytes[index]!;
    let length = 0;
    if ((lead & 0x80) === 0) length = 1;
    else if ((lead & 0xe0) === 0xc0) length = 2;
    else if ((lead & 0xf0) === 0xe0) length = 3;
    else if ((lead & 0xf8) === 0xf0) length = 4;
    else return bytes;
    if (index + length > end) end = index;
    return bytes.slice(0, end);
}

/**
 * Build a branch-scope review snapshot from files the user chose locally.
 * Reads only bounded byte slices; it never reads Git or the host companion.
 */
export async function createUploadedSnapshot(files: File[]): Promise<ChangesSnapshot> {
    let truncated = files.length > MAX_UPLOAD_FILES;
    let totalBytes = 0;
    const selected = files.slice(0, MAX_UPLOAD_FILES);
    const records: Array<Record<string, unknown>> = [];
    for (const [index, file] of selected.entries()) {
        const remaining = Math.max(0, MAX_UPLOAD_TOTAL_BYTES - totalBytes);
        const cap = Math.min(MAX_UPLOAD_FILE_BYTES, remaining);
        const readLength = cap === 0 ? 0 : Math.min(file.size, cap + 1);
        const buffer = new Uint8Array(await file.slice(0, readLength).arrayBuffer() as ArrayBuffer);
        let fileTruncated = file.size > cap || buffer.byteLength > cap;
        let bytes: Uint8Array<ArrayBufferLike> = fileTruncated ? buffer.slice(0, cap) : buffer;
        if (fileTruncated) bytes = completeUtf8Prefix(bytes);
        const content = new TextDecoder('utf-8').decode(bytes);
        totalBytes += bytes.byteLength;
        if (content.split('\n').length > MAX_REVIEW_LINES) fileTruncated = true;
        if (fileTruncated) truncated = true;
        records.push({
            path: file.name.length > 0 ? file.name : `uploaded-${index + 1}.txt`,
            status: 'modified',
            content,
            truncated: fileTruncated,
        });
    }
    const headDigest = await sha256Canonical({
        files: records.map((record) => ({
            path: record.path,
            content: record.content,
        })),
    });
    return {
        scope: 'branch',
        files: records,
        headDigest,
        capturedAtMs: Date.now(),
        truncated,
    };
}

function sourceLabel(source: DiffSource): string {
    if (source === 'upload') return 'Uploaded file';
    if (source === 'host') return 'Companion branch';
    if (source === 'browser') return 'Browser last turn';
    return 'Changes';
}

export function DiffReview({
    taskId,
    scope,
    onScope,
    comments,
    onComment,
    snapshot,
    source = null,
    loading = false,
    error = null,
    commentDisabledReason = null,
    idPrefix,
}: DiffReviewProps) {
    const instanceRef = useRef<string | null>(null);
    if (instanceRef.current === null) {
        nextReviewId += 1;
        instanceRef.current = idPrefix ?? `diff-review-${nextReviewId}`;
    }
    const instance = instanceRef.current!;
    const headingId = `${instance}-heading`;
    const summaryId = `${instance}-summary`;
    const formHeadingId = `${instance}-form-heading`;
    const fileId = `${instance}-file`;
    const lineId = `${instance}-line`;
    const lineHelpId = `${instance}-line-help`;
    const bodyId = `${instance}-body`;
    const errorId = `${instance}-error`;
    const bodyRef = useRef<HTMLTextAreaElement | null>(null);
    const lineRef = useRef<HTMLInputElement | null>(null);
    const [filePath, setFilePath] = useState('');
    const [lineValue, setLineValue] = useState('0');
    const [body, setBody] = useState('');
    const [formError, setFormError] = useState('');
    const [status, setStatus] = useState('');

    const files = useMemo(
        () => snapshot === null ? [] : snapshot.files,
        [snapshot],
    );
    const paths = useMemo(
        () => files.map((file) => typeof file.path === 'string' ? file.path : ''),
        [files],
    );
    const selectedPath = paths.includes(filePath) ? filePath : paths[0] ?? '';
    const selectedFile = files.find((file) => typeof file.path === 'string' && file.path === selectedPath);
    const selectedLines = selectedFile === undefined ? null : snapshotFileLines(selectedFile);

    useEffect(() => {
        setFormError('');
        setStatus('');
    }, [snapshot?.headDigest, scope]);

    const scopedComments = comments.filter((comment) => comment.scope === scope);
    const current: DiffComment[] = [];
    const stale: Array<{ comment: DiffComment; reason: string }> = [];
    for (const item of scopedComments) {
        const state = diffCommentStatus(item, snapshot);
        if (state.current) current.push(item);
        else stale.push({ comment: item, reason: state.reason ?? 'This comment is stale.' });
    }
    const commentsByPath = new Map<string, DiffComment[]>();
    for (const item of current) {
        const list = commentsByPath.get(item.path) ?? [];
        list.push(item);
        commentsByPath.set(item.path, list);
    }

    async function submit(event: Event): Promise<void> {
        event.preventDefault();
        setFormError('');
        setStatus('');
        const text = body.trim();
        if (text.length === 0) {
            setFormError('Write a comment before adding it.');
            bodyRef.current?.focus();
            return;
        }
        if (text.length > MAX_COMMENT_BODY) {
            setFormError(`Comments are limited to ${MAX_COMMENT_BODY} characters.`);
            bodyRef.current?.focus();
            return;
        }
        if (selectedPath.length === 0) {
            setFormError('Choose a file from this snapshot.');
            return;
        }
        const line = Number(lineValue);
        if (!Number.isSafeInteger(line) || line < 0) {
            setFormError('Line must be 0 for a file comment or a visible line number.');
            lineRef.current?.focus();
            return;
        }
        if (line > 0 && selectedLines === null) {
            setFormError('This file has no line mapping in the snapshot. Use 0 for a file comment.');
            lineRef.current?.focus();
            return;
        }
        if (line > 0 && selectedLines !== null && line > selectedLines.length) {
            setFormError(`Line ${line} is not present in this snapshot.`);
            lineRef.current?.focus();
            return;
        }
        try {
            await onComment({ path: selectedPath, line, body: text });
            setBody('');
            setStatus('Comment added locally.');
            bodyRef.current?.focus();
        } catch (failure) {
            setFormError(failure instanceof Error ? failure.message : String(failure));
        }
    }

    return <section class="diff-review" aria-labelledby={headingId} data-diff-review>
        <header class="diff-review-header">
            <h2 id={headingId}>Review changes</h2>
            <p>Local review only. Comments are stored in this browser and are never sent to a service.</p>
            <div role="group" aria-label="Change source">
                <button type="button" aria-pressed={scope === 'last_turn'} onClick={() => onScope('last_turn')}>Last agent turn</button>
                <button type="button" aria-pressed={scope === 'branch'} onClick={() => onScope('branch')}>Branch</button>
            </div>
            <p id={summaryId} class="diff-review-summary">
                {sourceLabel(source)} · {files.length} changed file{files.length === 1 ? '' : 's'}
                {snapshot === null ? '' : ` · snapshot ${snapshot.headDigest.slice(0, 20)}`}
            </p>
        </header>
        {loading && <p class="diff-review-loading">Loading changes…</p>}
        {error !== null && <p class="diff-review-error" role="alert">{error}</p>}
        {snapshot?.truncated === true && <p class="diff-review-truncated" role="status" data-diff-truncated>
            This snapshot is truncated. It is not the complete set of changed files or lines.
        </p>}
        {!loading && error === null && snapshot === null && <p class="diff-review-empty">
            {taskId === null ? 'Open a task or upload a diff to review changes.' : 'No snapshot is available for this change source.'}
        </p>}
        {snapshot !== null && <ul class="diff-file-list" aria-label="Changed files">
            {files.map((file, index) => {
                const path = typeof file.path === 'string' ? file.path : `file-${index + 1}`;
                const state = typeof file.status === 'string'
                    ? file.status
                    : typeof file.change === 'string'
                      ? file.change
                      : 'changed';
                const lines = snapshotFileLines(file);
                const fileComments = commentsByPath.get(path) ?? [];
                return <li key={`${path}:${index}`} data-diff-file={path} class="diff-file">
                    <h3><span class="diff-file-path">{path}</span> <span class="diff-file-status">{state}</span></h3>
                    {lines === null
                        ? <p class="diff-file-no-lines">This source lists the changed file only; it has no line content.</p>
                        : <ol class="diff-lines" aria-label={`Lines in ${path}`}>
                            {lines.map((line, lineIndex) => <li key={lineIndex} data-diff-line={lineIndex + 1}><code>{line.length === 0 ? ' ' : line}</code></li>)}
                        </ol>}
                    {fileComments.length > 0 && <div class="diff-file-comments" data-diff-current-comments>
                        {fileComments.map((item) => <DiffCommentView key={item.commentId} comment={item} />)}
                    </div>}
                </li>;
            })}
        </ul>}
        {stale.length > 0 && <section class="diff-stale-comments" aria-label="Stale local comments" data-diff-stale-comments>
            <h3>Stale local comments</h3>
            <p>These comments no longer map to the current snapshot. They were not moved to another line.</p>
            {stale.map(({ comment: item, reason }) => <DiffCommentView key={item.commentId} comment={item} staleReason={reason} />)}
        </section>}
        {snapshot !== null && paths.length > 0 && commentDisabledReason !== null && <p class="diff-comment-unavailable">{commentDisabledReason}</p>}
        {snapshot !== null && paths.length > 0 && commentDisabledReason === null && <form class="diff-comment-form" aria-labelledby={formHeadingId} onSubmit={(event) => { void submit(event); }}>
            <h3 id={formHeadingId}>Add a local comment</h3>
            <label for={fileId}>File</label>
            <select id={fileId} value={selectedPath} onChange={(event) => setFilePath(event.currentTarget.value)}>
                {paths.map((path) => <option key={path} value={path}>{path}</option>)}
            </select>
            <label for={lineId}>Line (0 for the whole file)</label>
            <input
                ref={lineRef}
                id={lineId}
                type="number"
                min={0}
                step={1}
                value={lineValue}
                aria-describedby={`${lineHelpId}${formError.length > 0 ? ` ${errorId}` : ''}`}
                aria-invalid={formError.length > 0}
                onInput={(event) => setLineValue(event.currentTarget.value)}
            />
            <p id={lineHelpId} class="diff-comment-help">Use 0 for a file-level comment, or a line shown in this snapshot.</p>
            <label for={bodyId}>Comment</label>
            <textarea
                ref={bodyRef}
                id={bodyId}
                required
                maxLength={MAX_COMMENT_BODY}
                value={body}
                aria-invalid={formError.length > 0}
                aria-describedby={formError.length > 0 ? errorId : undefined}
                onInput={(event) => setBody(event.currentTarget.value)}
            />
            {formError.length > 0 && <p id={errorId} class="diff-comment-error" role="alert">{formError}</p>}
            <button type="submit">Add local comment</button>
            <p class="diff-comment-status" role="status" aria-live="polite">{status}</p>
        </form>}
    </section>;
}
