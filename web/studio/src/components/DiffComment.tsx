import type { DiffComment } from './DiffReview.js';

export interface DiffCommentViewProps {
    comment: DiffComment;
    staleReason?: string | null;
}

export function DiffCommentView({ comment, staleReason = null }: DiffCommentViewProps) {
    const stale = staleReason !== null;
    return <article class="diff-comment" data-diff-comment data-diff-stale={stale ? 'true' : 'false'}>
        <p class="diff-comment-meta">
            {stale ? 'Stale local comment' : 'Local comment'} · {comment.line === 0 ? 'whole file' : `line ${comment.line}`}
        </p>
        {stale && <p class="diff-comment-stale">{staleReason}</p>}
        <p class="diff-comment-body">{comment.body}</p>
    </article>;
}
