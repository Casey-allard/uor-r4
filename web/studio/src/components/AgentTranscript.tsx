import type { ChatMessage } from '../adapters/workflow.js';
import { RichMessage } from './RichMessage.js';
import { Icon } from './ui/Icon.js';

interface AgentTranscriptProps { messages: ChatMessage[]; canRun: boolean; busy?: boolean; copy(text: string): void | Promise<void>; insert(text: string): void; download(text: string): void; retry(text: string): void }

export function AgentTranscript({ messages, canRun, busy = false, copy, insert, download, retry }: AgentTranscriptProps) {
    return <div class="agent-transcript" aria-label="Conversation messages" aria-live="polite" aria-busy={busy ? 'true' : 'false'}>
        {messages.length === 0 && <div class="agent-empty"><span class="agent-empty-mark" aria-hidden="true">U</span><h2>What should we work on?</h2><p>No messages yet</p><p>Ask a question, inspect a workspace, or choose a build task. Model and tool activity will appear here with its actual capability and privacy state.</p></div>}
        {messages.map((message, index) => {
            const previous = message.role === 'assistant' ? messages.slice(0, index + 1).reverse().find((item) => item.role === 'user') : undefined;
            return <article class="transcript-message" data-role={message.role}>
                <header><span class="message-role">{message.role === 'user' ? 'You' : 'R4 Harness'}</span>{message.tool && <span class="message-tool">{message.tool}</span>}</header>
                <div class="message-content"><RichMessage content={message.content} /></div>
                <footer class="message-actions">
                    <button type="button" onClick={() => void copy(message.content)}><Icon name="changes" />Copy</button>
                    <button type="button" onClick={() => insert(message.content)}><Icon name="workspace" />Insert into editor</button>
                    <button type="button" onClick={() => download(message.content)}><Icon name="save" />Save as file</button>
                    {message.role === 'assistant' && <button type="button" disabled={!canRun} onClick={() => previous && retry(previous.content)}>Retry</button>}
                </footer>
            </article>;
        })}
    </div>;
}
