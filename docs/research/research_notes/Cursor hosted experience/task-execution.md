# Cursor Hosted Agents — task execution surfaces, actions, and states (research notes, 2026-11-09)

**Scope.** Cursor's hosted/cloud coding agents (formerly "Background Agents") and the surfaces that list, open, run, review, and continue them: `cursor.com/agents`, the desktop **Agents Window**, **Cursor for iOS**, and the API/SSE layer. Projects (coordinator + workers) are treated only where they change the task UI; they are a separate orchestration layer.

**Method.** Primary sources only for behavior: Cursor docs, Cursor help center, Cursor blog, the Cloud Agents weekly release notes (including the dated Markdown mirror), and the general Cursor release-notes timeline. One public-surface probe: an isolated built-in browser opened `https://cursor.com/agents` with no profile, cookie, or login. No real account, task, repository, paid service, or mutation was used.

**Labels used on every claim.**
- `[observed]` — directly seen in the isolated browser on 2026-11-09.
- `[official-documented]` — stated by Cursor docs/help/blog/release notes; section and date given.
- `[inferred]` — a conclusion drawn from cited facts, not itself stated.
- `[unverified]` — plausible or commonly assumed, but not established by a primary source I could read.
- `[conflicting]` — two official sources disagree; both are cited and the newer/more specific one is identified.
- `[negative finding]` — a primary-source search found no evidence for the claim; this is not proof the feature does not exist.

**Date boundary.** Docs were retrieved 2026-11-09. The Cloud Agents weekly release notes' newest entry is **Week of Oct 19, 2026**; no entries exist there for the weeks of Oct 26 or Nov 2. The general release-notes timeline runs to version **v2026.10.15** and **3.27**. A few claims come from older version-tagged entries (3.20–3.26); those are labeled by version, not by calendar date, because the page does not print dates next to versions.

**Source key.**

| ID | Source |
|---|---|
| S1 | [Cloud Agents docs](https://cursor.com/docs/cloud-agent) |
| S2 | [Cloud agent capabilities](https://cursor.com/docs/cloud-agent/capabilities) |
| S3 | [Agents Window docs](https://cursor.com/docs/agent/agents-window) |
| S4 | [Agent docs — how Agent works, queue, side chats](https://cursor.com/docs/agent/overview) |
| S5 | [Cloud Agents API endpoints](https://cursor.com/docs/cloud-agent/api/endpoints) |
| S6 | [Cloud Agents release notes](https://cursor.com/docs/release-notes/cloud-agents) (dated Markdown mirror: [.md](https://cursor.com/docs/release-notes/cloud-agents.md)) |
| S7 | [Cursor release notes — general timeline](https://cursor.com/docs/release-notes) |
| S8 | [Cursor for iOS docs](https://cursor.com/docs/cloud-agent/mobile) |
| S9 | [Cursor for iOS help](https://cursor.com/help/ai-features/mobile-app) |
| S10 | [Cursor for iOS launch blog](https://cursor.com/blog/ios-mobile-app) |
| S11 | [Plan Mode docs](https://cursor.com/docs/agent/plan-mode) |
| S12 | [Agent help — Stop, queue, review](https://cursor.com/help/ai-features/agent) |
| S13 | [Cloud Agents help](https://cursor.com/help/ai-features/cloud-agents) |
| S14 | [Multi-agent help](https://cursor.com/help/ai-features/multi-agent) |
| S15 | [Cursor 3 launch blog](https://cursor.com/blog/cursor-3) |
| S16 | [cursor.com/agents](https://cursor.com/agents) — observed sign-in redirect |

---

## 1. Task list, detail navigation, menus, labels, and default views

### Takeaway
The task list is an **agent/thread list**, not a run list. On the web it is an agent list at `cursor.com/agents` (the API returns it newest first); in the desktop Agents Window it is a sidebar with Pinned, repository/workspace, and Chats sections plus Today / Last 7 days / Older groups. The documented detail surface is conversation-first: a header, status, starting prompt, and a tabbed workspace for changes, PR, terminal/desktop, and artifacts. Pins are account-scoped; archive is soft delete; the list is optimized for many parallel agents rather than a single chat.

### Cited Findings
- `[official-documented]` The web task list and task detail live at `cursor.com/agents`; API examples use `https://cursor.com/agents/bc-00000000-…` as the agent URL, so the detail route is `/agents/{agentId}` with a `bc-` id — [S1](https://cursor.com/docs/cloud-agent), [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` `GET /v1/agents` lists agents **newest first**, defaults to 20, max 100, supports a `cursor` pagination cursor and `prUrl` filter, and **includes archived agents by default** (`includeArchived` defaults to `true`) — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` The desktop Agents Window sidebar groups agents into **Today**, **Last 7 days**, and an **Older** section that starts collapsed; the **web grouping is unverified**. Pins are saved to the account so they follow browsers and devices — [S6, Week of Sep 28, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The sidebar has a **Pinned** section and **Repositories**, **Workspaces**, and **Chats** headers; collapsing any of them persists across reloads/restarts, and sections expanded with **More** keep showing the same number of rows — [S7, 3.26 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` Agent rows in the sidebar are **single-line** and can show when each agent was **last updated**; a **Search** row opens the chat-history picker; row keyboard shortcuts appear only on hover; long titles fade instead of hiding the **peer count** — [S7, 3.25 › Agent and chat](https://cursor.com/docs/release-notes), [S7, 3.24 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` The **desktop 3.22 release notes** document **up to 75 pinned agents** (raised from 50); the **web limit is unverified**. The desktop sidebar shows an **unread dot**, and Project rows can show unread while children work — [S7, 3.22 › Agents](https://cursor.com/docs/release-notes), [S7, 3.21 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` Cloud Agents are built for **parallelism**: you can run as many agents as you want in parallel, each in its own VM, and they do not need your local machine online; the desktop Agents Window shows all local and cloud agents in one sidebar; worktrees give each local task an isolated Git checkout; **Best-of-N tabs** switch between candidate conversations — [S1](https://cursor.com/docs/cloud-agent), [S3](https://cursor.com/docs/agent/agents-window), [S15](https://cursor.com/blog/cursor-3), [S6, Week of Sep 28, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` A failed agent gets a **red Error status**, its own **Error group**, and an **Error** option under **Show > Status**, on by default — [S7, 3.22 › Agents](https://cursor.com/docs/release-notes).
- `[official-documented]` The built-in Cursor Cloud MCP exposes `list-cloud-agents` with filters for **source, status, date, code changes, PR creation, and archived state**; `rename-chat` changes the current run's chat title (up to **200 characters**) and the new title shows in the **agents list and conversation header**; `run-info` returns run ID, URL, repo, branch, model, owner, lifecycle status, and where the run was started (Cursor, Slack, GitHub, API, and others) — [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[official-documented]` The **Cmd+K** command palette offers **Select Model**, **Archive**, and **New Side Chat**; pasting a PR URL or typing a PR number jumps to that PR; **Ctrl+Tab** opens a recent-agents switcher with the same look — [S7, 3.25 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` Rename is available from an agent's **right-click menu**; in the desktop tab strip, **double-click a tab** to rename; hovering a chat tab/title/tile header shows the same **agent card** as the sidebar — [S7, 3.23 › Fixes](https://cursor.com/docs/release-notes), [S7, 3.24 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` Archiving is reversible with **Cmd+Z / Ctrl+Z** (or **Edit > Undo** when no text field is focused) and re-doable with **Cmd+Shift+Z**; **Archive All** acts on the group clicked; archiving an agent with running background processes asks for confirmation first — [S7, 3.26 › Navigation and layout](https://cursor.com/docs/release-notes), [S7, 3.20 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` An archived agent **remains readable but cannot accept new runs until unarchived**; archive is the documented reversible "soft delete," while `DELETE /v1/agents/{id}` is permanent and irreversible — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` A follow-up sent from a stale tab to an agent archived elsewhere **unarchives it and delivers the message**; deleted agents stay closed — [S6, Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The **New Agent** button opens a new-agent input; clicking it reliably shows the input even after a reload, workspace switch, tiled conversation, or fullscreen editor — [S7, 3.20 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` On a direct agent link, the **header, status, and starting prompt** appear before the rest of the page finishes loading — [S6, Week of Oct 19, 2026 › Web](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Opening a conversation only collapses the sidebar on windows **narrower than 1024px**; a collapsed sidebar is reopened with an **Open sidebar** button (hover no longer pops it open); **Cmd+, / Ctrl+,** opens Settings on the agents pages — [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` On phone-width web, the agents pages use larger text, a **centered top navigation without the Dashboard tab**, a logo that returns to agents, **Search and the source filter inline with the agents list**, a taller composer, and a profile-photo menu button — [S6, Week of Aug 31, 2026 › Web app](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The iOS app's primary list is the **Inbox**; pull down on it to reload agents and their open PR statuses; Projects appear in their own Inbox section; Back from a Project returns to the Inbox — [S9](https://cursor.com/help/ai-features/mobile-app), [S7, 1.16.0 › Inbox](https://cursor.com/docs/release-notes), [S7, 1.9.0 › Projects](https://cursor.com/docs/release-notes).
- `[official-documented]` Every surface lets you **filter for agents started on mobile**; agents started on mobile are tagged `source: iosApp` — [S9](https://cursor.com/help/ai-features/mobile-app), [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` The detail workspace is tabbed. Documented tab/area names include **Changes**, **PR**, **Terminal**, **Cloud Desktop**, **Canvas**, **Output**, and file/terminal/browser tabs. **Back** reopens a just-closed tab (file, terminal, browser, Changes, Output, side chat) with its navigation history; tabs restore after hiding the panel, switching agents, or restarting — [S7, 3.24 › Navigation and layout](https://cursor.com/docs/release-notes), [S7, 3.26 › Navigation and layout](https://cursor.com/docs/release-notes), [S6, Week of Oct 19, 2026 › Web](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The **PR tab** remembers whether you were on **Changes, Description, Commits, Checks, or Reviews** when you switch agents or restart — [S7, 3.26 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` The **Changes tab** keeps its **scope, branch, and picked commits** across tab restores; a **Last Agent Turn** scope shows only the latest turn's edits — [S7, 3.27 › Navigation and layout](https://cursor.com/docs/release-notes), [S7, 3.22 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` In the **desktop Agents Window**, the **status pills above the prompt** list live status first — **pending input, Changes, Agents, Terminals, Canvas, Queue** — then quick actions; they stay visible when a tray is open; the **Terminals** pill shows a spinner while a command runs. **Web parity is unverified** — [S7, 3.20 › Chat and composer](https://cursor.com/docs/release-notes).
- `[inferred]` The list's default view is the unarchived agent list, newest first, with the current or most recent task selected; the detail's default view is the conversation transcript, not the Changes tab. The sources establish the pieces but never state the default selected tab or default filter.
- `[inferred]` "Task list" and "run history" are different objects: the list shows agents/threads, while runs are turns inside an agent and are exposed through run history/API, not as top-level list rows.
- `[inferred]` Pins, archive state, and the shared transcript are the account-level continuity layer; sidebar collapse and open tabs are device-local UI state (documented as persisting across reloads/restarts, not as synced).

### Gaps
- `[unverified]` Exact visual hierarchy, spacing, icons, colors, empty states, onboarding, button placement, menu ordering, composer placeholder text, and the default tab on first open. The authenticated pages could not be viewed.
- `[unverified]` Whether the web list exposes the same **Show > Status** filter, **Error** group, and per-row "last updated" as the desktop sidebar. The release notes describe an "agents sidebar" without always naming the surface.
- `[unverified]` Whether **Delete** is exposed in the web/desktop UI. It is documented in the API/SDK and release notes say deleted agents stay closed, but I found no public UI label or confirmation text.
- `[unverified]` Whether bulk selection exists beyond **Archive All** on a sidebar group.
- `[unverified]` The exact list empty-state text and whether archived agents are hidden by default in the UI (the API includes them by default).

---

## 2. Task detail: instructions, streaming, plans/tool traces, logs, subagents, side chats

### Takeaway
The detail page is a **transcript-first** surface. The first user message appears immediately; assistant text and tool calls stream in; finished work folds into **Worked** sections with a duration; a **status line** shows what the agent is waiting for; plans appear as an editable card; subagent tool calls and messages stream inline under the parent; side chats are durable parallel conversations. Raw logs are available through run events, setup logs, and PR artifacts rather than a single obvious "Logs" tab.

### Cited Findings
- `[official-documented]` An agent is built from **instructions, tools, and a model**; tools include file search/read, web search, rules retrieval, file edit, terminal, browser control, image generation, and asking clarifying questions. **There is no limit on the number of tool calls** during a task — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` Cloud agent conversations at `cursor.com/agents`, the PR agent panel, and side chats **use the same transcript as the Agents Window**; the first message in a new agent shows up right away, images included; mouse wheel and Page Up/Page Down scroll the transcript in codebase sidebars — [S6, Week of Oct 19, 2026 › Web](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The run stream is **SSE** with events `status`, `assistant`, `tool_call`, `result`, `error`, and `done`; an optional richer `interaction_update` carries `text-delta`, `tool-call-started`, `tool-call-completed`, `step-started`, `step-completed`, and `turn-ended`. Tool calls use a stable envelope and may be **truncated** with flags — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` The stream is scoped to one run, does not replay prior runs, can resume with `Last-Event-ID`, and returns `410 stream_expired` after its retention window; the terminal state should then be read with Get A Run — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` Each finished turn shows **how long the agent worked**; the **"Worked for"** duration includes tools that ran after the final reply — [S7, 3.25 › Agent and chat](https://cursor.com/docs/release-notes), [S7, 3.22 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` In a Project chat, the coordinator's thinking and tool work **fold into a Worked section with a duration before each reply**; chats opened from history show the same view — [S7, 3.27 › Projects](https://cursor.com/docs/release-notes).
- `[official-documented]` When a subagent needs approval or an answer, the main conversation's status line says **"Waiting for approval"** or **"Waiting for response"** and shows what it is asking for; messages sent by another agent name the sending agent — [S7, 3.25 › Agent and chat](https://cursor.com/docs/release-notes).
- `[official-documented]` **Plan Mode** is entered with **Shift+Tab** or the mode picker; the agent asks clarifying questions, researches, produces a reviewable plan, and you **click to build the plan** when ready. Plans are saved in the home directory by default; **Save to workspace** moves one into the workspace — [S11](https://cursor.com/docs/agent/plan-mode).
- `[official-documented]` The plan card above the prompt has a **Minimize** button that collapses it to a **Plan pill**; right-click the Plan or Canvas pill and choose **Remove** to clear it; the plan card takes priority over follow-up suggestions — [S7, 3.23 › Chat and composer](https://cursor.com/docs/release-notes).
- `[official-documented]` Subagents run in their own context window and return a result to the main conversation; Cursor includes built-in **research, shell, and browser** subagents and supports custom subagents in `.cursor/agents/`; `/multitask` runs async subagents in parallel, and **Build in Parallel** runs independent plan steps at once — [S14](https://cursor.com/help/ai-features/multi-agent), [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` A Cloud Agent's subagent **tool calls and messages stream live under the parent's task**; cloud subagent Task cards show the subagent's name and progress even if it starts after the card appears; tapping a subagent card on iOS opens its child transcript — [S6, Week of Oct 5, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.24 › Fixes](https://cursor.com/docs/release-notes), [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` **Side chats** are opened with `/side` or `/btw` (optionally with a question) or the plus button at the top of the chat panel. A side chat is a durable agent conversation that uses the parent thread as hidden reference context and keeps its own transcript; @-mention it in the main thread to pull context back — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` **Conversation search** is opened with **Cmd/Ctrl+K** from the Agents Window and searches past transcripts using a local search index; **Cmd/Ctrl+F** finds text in the open conversation — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` **`/goal`** gives the agent a long-lived objective; in the CLI `Ctrl+C` pauses the goal; `/loop` runs a prompt or skill on a recurring/variable interval; the agent chooses when to wake if no fixed interval is given — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` **Checkpoints** are created before significant changes; click a checkpoint in the chat timeline to preview files at that point, then restore; **Restore Checkpoint** is also on previous requests or the + button when hovering a message. Restoring reverts files only, not messages — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` Cloud Agents expose **run events** through the built-in Cursor Cloud MCP: `setup_started`, `setup_completed`, `setup_failed`, `pr_created`, `pr_creation_failed`, `artifact_created`, `mcp_auth_error`. The MCP also exposes transcripts, diff metadata, environment details, and **setup logs** — [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[official-documented]` A new web agent shows its **logs and conversation** once created; the run dashboard's event list is the closest documented "logs" surface — [S6, Week of Sep 7, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents), [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[inferred]` The transcript intentionally separates **narrative** (assistant messages), **evidence of work** (Worked sections, tool calls), and **state** (status line/pills). This is a review-oriented design: the user can audit without reading every tool call.
- `[inferred]` Plans are artifacts inside a task, not a separate task type. The same task can move from plan to implementation without changing its identity.
- `[inferred]` Subagents are nested tasks rendered inline; they are not separate top-level list entries, although the Agents tray/Back navigation can focus one.

### Gaps
- `[unverified]` Exact rendering of tool calls: collapsed by default, iconography, per-call duration, argument display, and whether raw JSON is viewable.
- `[unverified]` Whether thinking/reasoning is always visible, summarized into "Worked", or model-dependent.
- `[unverified]` Whether a dedicated **Logs** tab exists. The release notes mention logs being shown, and the API/MCP expose setup logs and events, but I found no public label for a Logs tab.
- `[unverified]` Exact plan-card text, plan editing controls, and whether **Build in Parallel** appears on every plan or only multi-step plans.
- `[unverified]` Side-chat close/archive behavior and whether side chats appear in the main task list (a help-center snippet says closing a side chat archives its side-chat agent; I did not read the full page, so this is not cited as a finding).
- `[unverified]` Whether side chats can be started from the web detail page as well as the Agents Window/IDE.

---

## 3. Instructions, queued follow-ups, steering, stop/redirect, additional runs, and history

### Takeaway
Cursor exposes **three distinct message modes**: queue (default, waits for the current turn), **Send now/steer** (delivered at the agent's next tool call), and immediate `Cmd+Enter` (bypasses the queue and appends to the most recent user message). **Stop** ends the active turn; **Retry** resumes a retryable error without a new message; a normal follow-up starts a **new run** on the same agent. I found **no user-facing Pause button** for cloud tasks. History is preserved through the durable thread, search, pins, archive, and run history.

### Cited Findings
- `[official-documented]` While Agent works, **Enter** adds the message to the queue; messages appear in order below the active task and can be **dragged to reorder**; the agent processes them sequentially after finishing — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` **Cmd+Enter / Ctrl+Enter** sends immediately, bypassing the queue; the message is appended to the most recent user message and attaches to tool results; use it to interrupt or redirect — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` To **steer** without interrupting, type a follow-up and hit **Send now**, or press **Enter twice**; the message is delivered at the agent's **next tool call** instead of cutting off work mid-action. **Tab** queues the message for after the turn. Steering is available on `cursor.com/agents` and rolling out in the Agents Window; in the CLI, Enter steers at a safe boundary and Enter again interrupts — [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` The API exposes the same distinction: `POST /v1/agents/{id}/runs` without `delivery` starts a **new run**; `delivery: "steer"` sends into the running turn. A steer accepts only `prompt` and `delivery`; other top-level fields return `400 validation_error`. The response is `202` with `delivery: "steered"` (includes `runId`) or `delivery: "queued"` (no `runId`); an unavailable feature returns `403 feature_unavailable` — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` A failed steer may still have reached the agent; Cursor treats a retry as a new prompt, so retrying can **deliver the prompt twice** — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` **Stop** is the documented interrupt: "Click the **Stop** button to stop Agent mid-task. This lets you redirect or start a different approach." — [S12](https://cursor.com/help/ai-features/agent).
- `[official-documented]` On a subagent, **Stop** or **Stop all** also stops every subagent it started, including finished/idle/unloaded ones and background subagents; pending questions are cancelled; the parent's task shows as **stopped** — [S7, 3.20 › Agents](https://cursor.com/docs/release-notes), [S7, 3.20 › Agent](https://cursor.com/docs/release-notes).
- `[official-documented]` When a cloud agent stops with a **retryable error**, the Agents Window shows a **Retry** button in the transcript that **resumes the run without sending a new message**; generic failures show **"This run hit an unexpected error. Retry or send a follow-up below."**; interrupting a running agent with a new message shows status text like **"Changing course…"** — [S7, 3.20 › Cloud agents](https://cursor.com/docs/release-notes).
- `[official-documented]` Cancelling through the API is **terminal**: the run transitions to `CANCELLED` and **cannot be resumed**; to continue the conversation, create a new run on the same agent — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` A message sent **after pressing Stop** runs instead of staying queued; a follow-up sent after stopping early in the first step works; queued follow-ups no longer vanish when a turn ends or when a steer falls back to the queue — [S6, Week of Sep 14, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents), [S6, Week of Sep 28, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.22 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` A message sent just as the agent is wrapping up is answered as **one more step in the same run** instead of becoming a separate follow-up; messages sent to a running agent arrive **in the order sent** — [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S6, Week of Aug 31, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Editing an earlier message in a reopened/synced conversation resumes from that turn's state; if the state cannot be found, the edit fails and leaves the transcript intact — [S7, 3.21 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` **Fork the chat** from an earlier message is the documented way to shorten the conversation itself; restoring a checkpoint does not remove messages — [S12](https://cursor.com/help/ai-features/agent).
- `[official-documented]` The iOS app supports **follow-ups**: earlier messages load as you scroll up while live replies keep streaming; follow-ups appear when you tap Send and do not vanish or duplicate after a reconnect; open questions reappear when you reopen a chat — [S7, 1.16.0 › Fixes](https://cursor.com/docs/release-notes), [S7, 1.9.0 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` **"Pause" is not a documented user action.** The word appears only as a state: an agent "pauses" on a question or MCP sign-in, Jira runs "show as working instead of paused," and stopping an agent while it starts "pauses it once it launches." No Pause button or Pause API is documented — [S6, Weeks of Aug 31 and Sep 21, 2026](https://cursor.com/docs/release-notes/cloud-agents), [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[inferred]` The intended mental model is **one durable agent thread, many runs**: a follow-up is a new run, not a new task; Stop ends the run; Retry resumes a retryable run; a new task requires **New Agent**.
- `[inferred]` Queue vs steer vs immediate is a deliberate control gradient: default queue is safest, steer preserves in-flight work, immediate is for urgent redirection. This is a stronger design than a single "send" button.
- `[inferred]` The absence of Pause is a product choice: cloud tasks are cheap to stop and continue with a follow-up; a paused VM would create state and billing ambiguity.

### Gaps
- `[unverified]` Exact queue UI: placement, maximum length, whether queued messages can be edited/deleted individually, and what happens to the queue when the task is archived.
- `[unverified]` Whether **Stop** stops only the active run or the whole agent, and whether a stopped run is shown as `CANCELLED` or a distinct "stopped" state.
- `[unverified]` Whether **Retry** appears for every error or only errors the backend classifies as retryable; whether Retry is available after a manual Stop.
- `[unverified]` Whether a **Resume** action exists for cloud tasks. "Resume" is documented for a local extension-host crash and for SDK `Agent.resume`, not for a stopped cloud task.
- `[unverified]` Whether the web UI exposes the API's `steer`/`queued` delivery result, and how a queued steer is shown.
- `[unverified]` Whether there is a maximum number of runs per agent or a retention limit on old runs.

---

## 4. Code changes, review, diffs, file navigation, terminal/output/artifacts, GitHub PR and CI

### Takeaway
Cursor separates the **agent workspace diff** (`Changes`) from the **remote PR review object** (`PR`). The Changes tab is scopeable (Last Agent Turn, branch, picked commits) and is the fast path for reviewing what the agent just did. The PR tab is a five-part review surface (Changes, Description, Commits, Checks, Reviews) with merge/review actions. A terminal, cloud desktop, browser, forwarded ports, and artifacts (screenshots/videos/logs) cover execution evidence. The agent works on a separate branch and opens a PR; CI autofix and subscriptions keep it working after the first turn.

### Cited Findings
- `[official-documented]` Agent edits are applied as it works; review them in the diff view and reject anything unwanted; **Restore Checkpoint** rolls back files after a point without removing messages — [S12](https://cursor.com/help/ai-features/agent), [S4](https://cursor.com/docs/agent/overview).
- `[official-documented]` The **Changes tab** has a **Last Agent Turn** scope showing only the latest turn's edits; clicking **Review** or a file in the **end-of-turn summary** opens the Changes tab in that scope — [S7, 3.22 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` The Changes tab keeps its **scope, branch, and picked commits** across tab restores; the **PR tab** remembers **Changes / Description / Commits / Checks / Reviews** across agent switches and restarts — [S7, 3.27 › Navigation and layout](https://cursor.com/docs/release-notes), [S7, 3.26 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` In the PR tab, **Cmd/Ctrl+click on a diff line** opens that line on GitHub or Origin; **generated files start collapsed with a Generated badge**; the **Merge** button stays disabled while an Origin rule blocks the merge; merges that bypass rules show in the timeline; **PR chips** in prompts/messages show current status such as merged or closed — [S7, 3.26 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` A PR review can be **discarded** from the review menu in the PR header, deleting the pending review and all draft comments; **Cmd+Enter / Ctrl+Enter** submits a reply in a discussion thread; pending comments show as drafts right away — [S7, Week of Oct 12, 2026 › Web](https://cursor.com/docs/release-notes), [S7, 3.23 › Files, changes, and pull requests](https://cursor.com/docs/release-notes), [S7, Week of Sep 21, 2026 › Web](https://cursor.com/docs/release-notes).
- `[official-documented]` A **Terminal** tab is available for cloud agents; opening a terminal for a machine that is offline opens a Terminal tab that **explains why** and connects automatically once the machine is reachable; **Ctrl+K / Ctrl+F** in the agent terminal go to the shell, not page shortcuts — [S7, 3.22 › Cloud agents](https://cursor.com/docs/release-notes), [S6, Week of Sep 14, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` A **Cloud Desktop** tab streams the agent's desktop; it stops streaming while another tab is selected and reconnects with a fresh connection when you switch back; the agent **steps back when you take control**, and the desktop is released when a computer-use subagent finishes — [S7, 3.26 › Fixes](https://cursor.com/docs/release-notes), [S6, Week of Sep 21, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The editor/workspace panel includes a **file tree, Changes panel, terminal, forwarded ports, and browser tabs**; cloud agent ports that fail show a red dot and error until a connection succeeds; opening a second cloud workspace no longer breaks port forwarding for the first — [S7, 3.27 › Self-Hosted Machines](https://cursor.com/docs/release-notes), [S7, 3.26 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` **Cloud agents produce screenshots, videos, and log references** as artifacts to demonstrate their work; artifacts are attached to the PR and can optionally be embedded in the GitHub PR description via **Allow posting artifacts to GitHub** — [S2](https://cursor.com/docs/cloud-agent/capabilities), [S1](https://cursor.com/docs/cloud-agent).
- `[official-documented]` The API can **list artifacts** under the workspace `artifacts/` directory and **download** one via a temporary **15-minute** presigned S3 URL — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` Artifact cards in chat offer **Add to Chat**, **Download**, **Copy Image**, or **Copy Text**; right-clicking a file link in an agent's chat offers **Download File** (default `~/Downloads`), including files a cloud agent built in its workspace — [S7, 3.25 › Agent and chat](https://cursor.com/docs/release-notes), [S7, 3.24 › Chat and composer](https://cursor.com/docs/release-notes).
- `[official-documented]` Cloud agents clone the repo from GitHub/GitLab/Azure DevOps/Bitbucket Cloud, work on a **separate branch**, and push changes for handoff; the API's `workOnCurrentBranch` controls whether commits go to a new `cursor/...` branch or directly to the starting ref; `autoCreatePR` controls PR creation — [S1](https://cursor.com/docs/cloud-agent), [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` Every commit and PR an agent creates is **attributed and visible in Git history, with signed, verified commits**; commits are authored as the agent, not the person who started it — [S1](https://cursor.com/docs/cloud-agent), [S13](https://cursor.com/help/ai-features/cloud-agents).
- `[official-documented]` Cloud Agents open pull requests **ready for review by default**, unless the task asks for a draft; a **Never open PRs as drafts** setting keeps every PR ready for review even when a draft is requested — [S6, Week of Oct 19, 2026 › Pull requests](https://cursor.com/docs/release-notes/cloud-agents), [S7, Week of Oct 19, 2026 › Web](https://cursor.com/docs/release-notes).
- `[official-documented]` The **PR tab** exposes review actions: **merge (including squash), mark ready, update the branch, toggle auto-merge, publish, or close**; add or change reviewers; read full diffs, commits, deployments, approvals, comments, and checks — [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` **Fix Merge Conflicts** is a cloud-agent git action with a loading state; **Checkout Branch** and **Fast Forward Branch** appear only for cloud agents whose checkout can be changed — [S7, 3.23 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` Merge-conflict notices say **"at least N"** conflicted paths when the list is cut short; stacked PR blockers name the PR to restack from or the base branch to retarget; a PR with its own conflicts is not shown as only waiting on a lower stack conflict — [S6, Week of Oct 19, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.26 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` Cloud Agents **automatically try to fix CI failures** in PRs they create (GitHub Actions only). They skip if you pushed a new commit, sent a follow-up, the same check fails on the base commit, or the PR has already had **10 CI-failure follow-ups**. Disable per PR with `@cursor autofix off`, re-enable with `@cursor autofix on`, or globally in Dashboard → Cloud Agents → My Settings — [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[official-documented]` **Subscriptions** let an agent wait for GitHub PR activity, CI results, Slack replies/channel messages, Linear issue changes, or timers. Events land as follow-ups in the same conversation; bursts coalesce; a subscription lasts at most **180 days**; CI waits for every check and delivers one commit-wide result — [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[official-documented]` A Cloud Agent comment/reply/review on a **private Origin PR** ends with an **Open in Web** button that opens the agent run; comments on public repositories do not include the link — [S6, Week of Oct 19, 2026 › Pull requests](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.26 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[inferred]` **Changes** is the review surface for the agent's local workspace; **PR** is the collaboration surface for the remote branch. The product expects the PR to be the handoff object, with Changes as the fast pre-PR review.
- `[inferred]` Artifacts are treated as **evidence attached to the PR**, not as a separate deliverables system; the API's `artifacts/` directory plus PR embedding is the persistence model.
- `[inferred]` CI autofix turns the PR into a long-lived task: the agent can keep working after its first turn through subscriptions, which is why the task must remain durable and resumable.

### Gaps
- `[unverified]` Whether inline review comments can be created on the **Changes** tab or only on the **PR** tab. The docs describe review comments on PRs; the Changes tab is described as review/commit.
- `[unverified]` Whether the terminal is fully interactive, read-only, or command-only; the docs say "open a terminal" but do not state the interaction contract.
- `[unverified]` Exact **Artifacts page** URL, whether it is per-agent or global, and whether artifacts can be deleted or shared publicly from the UI.
- `[unverified]` Whether **checks/CI** are visible in the agent page or only in the PR tab.
- `[unverified]` Whether **Fix Merge Conflicts** is a button that asks the agent or a deterministic merge; the release notes only show a loading state while "the cloud agent works."
- `[unverified]` Whether the web/desktop UI exposes **Checkout Branch** and **Fast Forward Branch** for GitHub-hosted agents, or only for Cursor-hosted/Origin repositories.
- `[unverified]` Exact diff rendering: split vs unified, word-level highlights, file tree, and comment threading.

---

## 5. What users can do with running, completed, failed, cancelled, and archived tasks; what persists

### Takeaway
The durable object is the **agent/thread**; each turn is a **run**. Agent lifecycle is `ACTIVE` / `IDLE` / `ARCHIVED`; run lifecycle is `CREATING` / `RUNNING` / `WAITING_FOR_BACKGROUND_WORK` then `FINISHED` / `ERROR` / `CANCELLED` / `EXPIRED`. Archive is reversible and keeps the task readable; delete is permanent. Completed tasks can be reviewed, followed up, archived, shared, or turned into PRs; failed tasks get Retry/recovery; cancelled runs cannot be resumed, only continued with a new run. Pins and archive state follow the account; tabs, scopes, and sidebar collapse are device UI state that survives reloads.

### Cited Findings
- `[official-documented]` Agent lifecycle: **`ACTIVE`** — a turn is running, waiting on background work, or about to start; the machine stays up. **`IDLE`** — the last turn finished and follow-ups are accepted; the machine may be hibernated/snapshotted; runs that ended in a recoverable error also report `IDLE`, with run-level error detail on Get A Run. **`ARCHIVED`** — archived or expired; terminal; claims end and workspace state can be deleted — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` Run lifecycle: `CREATING`, `RUNNING`, or `WAITING_FOR_BACKGROUND_WORK` while active; `FINISHED`, `ERROR`, `CANCELLED`, or `EXPIRED` once ended. Terminal runs carry `durationMs`, `result`, and a per-agent `git` snapshot of pushed branches/PRs — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` Running: send a follow-up (new run) or steer (into the active turn), Stop, Stop all subagents, Retry a retryable error, open Terminal/Desktop, take over the desktop, or send a message while the agent works — [S4](https://cursor.com/docs/agent/overview), [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S7, 3.20 › Cloud agents](https://cursor.com/docs/release-notes).
- `[official-documented]` Completed: read the transcript, inspect Changes, review/merge the PR, download artifacts, send a follow-up, rename, pin, archive, copy PR link, open in Cursor/Web, or add to a Project — [S1](https://cursor.com/docs/cloud-agent), [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S7, 3.23 › Cloud agents](https://cursor.com/docs/release-notes), [S7, 3.23 › Files, changes, and pull requests](https://cursor.com/docs/release-notes), [S7, 3.24 › Projects and workspaces](https://cursor.com/docs/release-notes).
- `[official-documented]` Failed: red **Error** status and Error group, **Retry** in the transcript, error tray above the input with **Retry / Manage Usage / Manage Environments**, Terminal tab explanation if the machine is offline, and specific failure reasons in run history — [S7, 3.22 › Agents](https://cursor.com/docs/release-notes), [S7, 3.22 › Cloud agents](https://cursor.com/docs/release-notes), [S6, Week of Sep 7, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Cancelled: the API transitions the run to `CANCELLED` and it **cannot be resumed**; create a new run on the same agent. In the UI, Stop followed by a message runs the message; a follow-up after Stop is supported — [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S6, Week of Sep 14, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents), [S6, Week of Sep 28, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Archived: readable, cannot accept new runs until unarchived; a follow-up from a stale tab unarchives and delivers; deleted agents stay closed; archive can be undone with Cmd+Z, and Archive All can be undone — [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S6, Week of Sep 14, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.26 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` Sharing: send an agent's URL to a teammate; teammates see the conversation, code changes, and artifacts if they belong to the same Cursor team **and** have access to the repository via their own connected source-control account. Viewing is read-only unless a team admin enables **team follow-ups** — [S1](https://cursor.com/docs/cloud-agent).
- `[official-documented]` Pins follow the account across browsers/devices; sidebar grouping and collapsed sections persist across reloads/restarts; the PR tab remembers its sub-tab; Changes keeps scope/branch/picked commits; open tabs restore after hiding/switching/restart — [S6, Week of Sep 28, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S7, 3.27 › Navigation and layout](https://cursor.com/docs/release-notes), [S7, 3.26 › Files, changes, and pull requests](https://cursor.com/docs/release-notes).
- `[official-documented]` The iOS app is **cache-first**: it reads local data so the Inbox and conversations open fast, then syncs when the connection returns; the chat stream pauses while the app is in the background and catches up when you return — [S8](https://cursor.com/docs/cloud-agent/mobile), [S9](https://cursor.com/help/ai-features/mobile-app).
- `[inferred]` `IDLE` is an internal/API state; the UI likely renders it as "idle" or "done" rather than showing the enum. The exact user-facing label is not documented.
- `[inferred]` Archive is the safe default removal; delete is intentionally API-only or rare in the UI because it destroys the transcript and workspace.
- `[inferred]` The system distinguishes "cancelled" (user stopped) from "error" (failure) and "expired" (retention), because the recovery path differs: new run vs retry vs start over.

### Gaps
- `[unverified]` Whether **Delete** is exposed in the UI and what confirmation it uses.
- `[unverified]` Whether an explicit **Unarchive** action exists in the UI, or only undo/Cmd+Z and the implicit unarchive-on-follow-up.
- `[unverified]` Whether a completed task can be **resumed as a new agent** (fork) vs only followed up.
- `[unverified]` Whether **Retry** is offered for `CANCELLED`/`EXPIRED` runs.
- `[unverified]` Exact user-facing labels/colors/icons for `ACTIVE`, `IDLE`, `ERROR`, `CANCELLED`, `EXPIRED`.
- `[unverified]` Whether run history is a separate page per agent or only a dashboard table.
- `[unverified]` Whether archived tasks are hidden from the default list or shown with a filter (API includes them by default; UI behavior not documented).

---

## 6. Usage, performance, latency, tokens, and cost

### Takeaway
Cursor exposes **per-run token usage** and **wall-clock duration** through the API, and a **Runs dashboard** with total runs, success rate, recent runs, a run-history chart, and filters by trigger/status/creator. The UI also shows **"Worked for"** per turn. Billing is plan-based at the selected model's rate, with a Max Mode upcharge on legacy plans and a spend limit. I found **no per-task dollar cost**, **no token-count UI on the task page**, and **no latency percentile/SLO** documented.

### Cited Findings
- `[official-documented]` `GET /v1/agents/{id}/usage` returns token usage for the agent, broken down per run and totaled; totals include Task subagents and side chats and usage not tied to a run. Fields are `inputTokens`, `outputTokens`, `cacheWriteTokens`, `cacheReadTokens`, `totalTokens`, plus `usageUuid`; an optional `runId` scopes to one run — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` `GET /v1/agents/{id}/runs/{runId}` returns run `status`, `createdAt`, `updatedAt`, `durationMs` (terminal runs), `result`, and `git.branches[]` with `repoUrl`, `branch`, `prUrl`; the `git` snapshot is per-agent, not per-run — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[official-documented]` The Cloud Agents page opens with **Total runs**, **Success rate**, and **Recent runs** cards for a selected date range; **All Runs** opens a Runs page with a run-history chart, a runs table, and filters for **trigger, status, and creator**. Team admins see team runs; personal users see their own — [S6, Week of Aug 31, 2026 › Dashboard and admin](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Each finished turn shows **how long the agent worked**; the **"Worked for"** duration includes tools that ran after the final reply — [S7, 3.25 › Agent and chat](https://cursor.com/docs/release-notes), [S7, 3.22 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` Cloud Agents use the plan's usage at the selected model's rate; on legacy request-based plans they are priced at **API rates plus the Max Mode upcharge** and count against included requests; a larger context window can increase token usage and cost; a spend limit is requested on first use — [S1](https://cursor.com/docs/cloud-agent), [S13](https://cursor.com/help/ai-features/cloud-agents).
- `[official-documented]` Usage-limit states expose **Included Usage Reached** with **Manage Plan**, **Buy Credits**, **Switch Model**, or **Upgrade Plan** depending on plan and model; on-demand limits can be raised — [S7, 1.16.0 › Account and billing](https://cursor.com/docs/release-notes), [S7, 3.23 › Customization and settings](https://cursor.com/docs/release-notes), [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` On iOS, **admin, billing, and usage are web-only**; the app is not an admin console — [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[inferred]` The primary usage unit is **tokens per run**, with duration as a secondary performance signal; cost is derived from plan/model pricing rather than stored per task.
- `[inferred]` The Runs dashboard is the closest thing to a task-performance view, but it is aggregate and filterable, not a per-task latency breakdown.
- `[inferred]` "Worked for" is a user-facing duration, while `durationMs` is API-level wall-clock; they may not be identical because `durationMs` is computed at terminal state and "Worked for" includes post-reply tool time.

### Gaps
- `[unverified]` Whether the task detail page shows token counts, cost, or a usage tab. The API exposes them; the UI display is not documented.
- `[unverified]` Whether **durationMs** includes environment setup/queue wait or only the run's execution.
- `[unverified]` Whether usage includes failed/cancelled runs and subagent/side-chat usage in the UI totals (the API says it does for totals).
- `[unverified]` Any latency metrics: time-to-first-token, queue wait, environment build time, tool latency, or percentiles.
- `[unverified]` Any per-task dollar/credit estimate in the UI.
- `[unverified]` Whether the Runs page is available to non-admin personal users on all plans, and whether it can export.

---

## 7. Error states and recovery behavior

### Takeaway
Cursor's error UX is **typed and action-oriented**: a short human message plus one primary next step. The main recovery paths are Retry (retryable run error), follow-up (new run), Manage Usage, Manage Environments, reconnect (environment/machine), re-authenticate (MCP), and start a new agent (unrecoverable). The API adds explicit error codes and a terminal `CANCELLED` state.

### Cited Findings
- `[official-documented]` Generic run failure text: **"This run hit an unexpected error. Retry or send a follow-up below."** with a **Retry** button that resumes without a new message — [S7, 3.20 › Cloud agents](https://cursor.com/docs/release-notes).
- `[official-documented]` Cloud agent errors appear in the **tray above the input**, including while a run is retrying automatically or after its session expires; a failed run offers a single next step where one applies, such as **Retry**, **Manage Usage**, or **Manage Environments** — [S7, 3.22 › Cloud agents](https://cursor.com/docs/release-notes).
- `[official-documented]` Environment reconnect text: **"Reconnecting to your environment. This can take a few minutes."** The agent retries on its own and continues in the same environment with files intact; if it takes too long, it asks you to send the message again instead of starting over — [S6, Week of Sep 14, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Worker-start failure text: **"Cloud Agent could not get a worker to start your session; please retry in a few minutes."** A follow-up to a disconnected My Machine gets a machine-offline message after about **15 seconds**; pool machines that go offline wait for the same machine to return before moving — [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S6, Week of Sep 21, 2026 › Self-Hosted Machines](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Usage-limit errors tell you to turn on on-demand usage, raise your/team's on-demand limit, or switch to a Cursor model or Auto on a trial; switching a follow-up to Auto after a trial limit lets the agent continue — [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` **"Out of Other Models usage"** names the model class and suggests switching to a named model or raising on-demand usage; a blocked model produces **Model Blocked** as soon as a follow-up is sent — [S6, Week of Sep 28, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents), [S6, Week of Sep 21, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Repository access failures name the repository that is empty, denied, or needs connecting; an empty Cursor-hosted repository says it is empty and asks for an initial commit; a PR creation failure from GitHub rate limits says how long to wait — [S6, Weeks of Sep 14 and Sep 21, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` Out-of-memory kills are recognized and the agent is told the memory limit so it can explain the failure and retry with less memory; a machine out of memory suggests running memory-heavy steps on fewer files or with lower parallelism — [S7, 3.25 › Fixes](https://cursor.com/docs/release-notes), [S6, Week of Sep 28, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` MCP sign-in: an agent waiting for sign-in shows as **Needs Attention** in the Inbox and on the Live Activity until you tap **Skip** or **Authenticate**; the card then says authenticated or skipped; a subagent waiting on you gets a yellow status dot in the Agents pill and is marked as needing attention in the tray — [S7, 1.9.0 › Agents](https://cursor.com/docs/release-notes), [S7, 3.26 › Fixes](https://cursor.com/docs/release-notes).
- `[official-documented]` A local agent whose extension host crashes shows **"Extension host stopped unexpectedly"** with a **Resume** option and is marked unread; one system notification covers all agents that stopped if Cursor is not focused — [S7, 3.22 › Agent and chat](https://cursor.com/docs/release-notes).
- `[official-documented]` Other exact states: **"Workspace runs on a self-hosted worker"** — [S6, Week of Sep 14, 2026 › Self-Hosted Machines](https://cursor.com/docs/release-notes/cloud-agents); **"Projects are disabled for your team."** — [S6, Week of Sep 21, 2026 › Projects](https://cursor.com/docs/release-notes/cloud-agents); **"No results found"** in command-palette search — [S7, 3.25 › Navigation and layout](https://cursor.com/docs/release-notes); **"1 run"** not "1 runs" — [S6, Week of Sep 28, 2026 › Fixes](https://cursor.com/docs/release-notes/cloud-agents); **"Privacy Mode (Legacy) blocks Cloud Agents."** and the PR-page agents sidebar shows a **Retry** notice instead of loading forever — [S6, Week of Oct 12, 2026 › Agents](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` API error semantics: `400 validation_error` for invalid follow-up/steer bodies, `409 agent_busy` for a follow-up when busy, `403 feature_unavailable` for steering unavailable, `404 run_not_found`, `410 stream_expired` after stream retention; a steer retry can duplicate the prompt — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[inferred]` Errors are designed to be **recoverable in place**: Retry resumes the same run; Manage Usage/Environments opens the blocking resource; reconnect keeps the same environment; re-auth keeps the task alive.
- `[inferred]` The product avoids raw provider errors in the main transcript; it maps them to a short message plus an action, with raw detail available through events/logs.

### Gaps
- `[unverified]` The full error taxonomy exposed to users (codes vs human text) and whether errors persist in task history.
- `[unverified]` Whether logs can be downloaded for support from the task page.
- `[unverified]` Whether a failed task can be archived/deleted, and whether Retry is available after archiving.
- `[unverified]` Exact rate-limit messages and retry timers.
- `[unverified]` Behavior when the browser loses connection mid-stream beyond "reconnects" and `Last-Event-ID`.
- `[unverified]` Whether the web UI exposes `409 agent_busy` or automatically queues.

---

## 8. Cross-device continuation: web, desktop, iOS, and Remote Control

### Takeaway
All surfaces share the same backend and thread. Mobile-started agents appear on web/desktop and are tagged `source: iosApp`; pins sync at the account level; the transcript is shared. Mobile is a **review/supervision** surface, not an IDE: no editor, terminal, or file browser, but full diff/PR review and merge. Remote Control lets a local agent's loop run in the cloud while tools stay on the user's computer. Move to Cloud transfers the conversation but not dirty files.

### Cited Findings
- `[official-documented]` Cursor for iOS "runs on the same backend as cursor.com/agents and the desktop Agents Window, so the agents you start on mobile show up everywhere you work" — [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` Agents started on mobile are tagged **`source: iosApp`**; each surface lets you **filter for agents you started on mobile** — [S8](https://cursor.com/docs/cloud-agent/mobile), [S9](https://cursor.com/help/ai-features/mobile-app).
- `[official-documented]` Mobile Inbox: pull down to refresh agents and open PR statuses; Projects have their own section; **New Project** starts from a name with optional repo, branch, and model; Back from a Project returns to the Inbox — [S9](https://cursor.com/help/ai-features/mobile-app), [S7, 1.16.0 › Inbox](https://cursor.com/docs/release-notes), [S7, 1.9.0 › Projects](https://cursor.com/docs/release-notes).
- `[official-documented]` Mobile **Live Activities** track up to **eight** running agents on the Lock Screen/Dynamic Island; they need the system permission enabled in iOS Settings under Cursor — [S9](https://cursor.com/help/ai-features/mobile-app).
- `[official-documented]` Mobile push behavior is contradictory across sources: the iOS docs say "Get a push notification when an agent finishes a turn" — [S8](https://cursor.com/docs/cloud-agent/mobile); the Week of Aug 31, 2026 release note says Cloud Agents **no longer push a turn-finished notification** and only push when an agent asks a question or needs approval — [S6, Week of Aug 31, 2026 › Notifications](https://cursor.com/docs/release-notes/cloud-agents). The release note is newer and more specific; treat the iOS docs sentence as stale/conflicting.
- `[official-documented]` Mobile review: full diffs, commits, deployments, approvals, comments, checks; add/change reviewers; merge with squash, mark ready, update branch, toggle auto-merge, publish, or close — [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` Mobile limitations: no editor, terminal, or file browser; environment/secrets, MCP management, source-control setup, automations/rules/skills config, and admin/billing/usage stay on the web — [S9](https://cursor.com/help/ai-features/mobile-app), [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` **Remote Control**: enable under **Settings > Agents**, run `/remote-control` in the agent input, send the next message; the session appears in the iOS Inbox. The agent loop runs in Cursor's cloud while terminal commands, file edits, tests, and git run on the user's computer; the computer must stay awake/online; Cursor 3.9.8+; supported plans; admins can enable/disable for Teams/Enterprise; not available when privacy settings disable cloud data storage — [S8](https://cursor.com/docs/cloud-agent/mobile), [S9](https://cursor.com/help/ai-features/mobile-app).
- `[official-documented]` **Move to Cloud** transfers conversation history/context but **does not snapshot dirty files or uncommitted edits**; commit or stash first. Cloud-to-local moves the session back to the computer — [S13](https://cursor.com/help/ai-features/cloud-agents), [S15](https://cursor.com/blog/cursor-3).
- `[official-documented]` The Agents Window shows **all local and cloud agents in one sidebar**, including agents kicked off from mobile, web, desktop, Slack, GitHub, and Linear; moving local↔cloud is a first-class handoff — [S15](https://cursor.com/blog/cursor-3), [S3](https://cursor.com/docs/agent/agents-window).
- `[official-documented]` iPad layout keeps chats in a sidebar, opens a review next to a chat, gives file diffs full width, and supports Apple Pencil drawing on screenshots — [S9](https://cursor.com/help/ai-features/mobile-app), [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[official-documented]` Tabs in the Agents Window restore after hiding the panel, switching agents, or restart; PR tab sub-tab, Changes scope/branch/picked commits, canvases' Rendered/Source view, expanded folders, and agent-opened tabs like **Agent Changes** and **Desktop** come back — [S7, 3.27 › Navigation and layout](https://cursor.com/docs/release-notes).
- `[official-documented]` iOS chat stream **pauses while the app is in the background and catches up when you return**; the app is cache-first and syncs when the connection returns; pull down on the Inbox to refresh — [S9](https://cursor.com/help/ai-features/mobile-app), [S8](https://cursor.com/docs/cloud-agent/mobile).
- `[inferred]` Cross-device continuity is **thread-based**, not device-based: the task id is the unit of continuity; source tags and pins are filters/organization, not separate task copies.
- `[inferred]` Mobile is deliberately a **supervisory client**; the absence of a terminal/editor avoids pretending a phone is an IDE and keeps the product's review loop coherent.
- `[inferred]` Remote Control is the bridge for local-only work; Move to Cloud is the bridge for cloud-only work; the two together cover both directions.

### Gaps
- `[unverified]` Exact sync latency and conflict resolution when the same task is edited on two devices.
- `[unverified]` Whether draft prompts, queued messages, and notification read state sync across devices.
- `[unverified]` Whether mobile can start a task from a PR/issue link or only from the repo picker.
- `[unverified]` Exact Live Activity states/labels and how they map to task statuses.
- `[unverified]` Whether Remote Control works with all privacy modes (the docs say it does not when cloud data storage is disabled).
- `[unverified]` Whether the web/desktop can take over a Remote Control session, or only the phone.
- `[unverified]` Android: docs say "not yet" with no date — [S9](https://cursor.com/help/ai-features/mobile-app).

---

## 9. Coverage, source ledger, conflicting reports, and what is unverifiable

### Takeaway
Coverage is **strong for documented behavior and exact strings**, **weak for visual/interaction detail**, and **absent for the authenticated product**. The only direct observation was the sign-in wall. Every claim about layout, default views, menu contents, or control placement beyond the quoted labels is either inferred or unverified. The largest conflict is mobile push-on-finish vs the newer no-turn-finished-push release note. The largest negative finding is that **Pause is not a documented user action**.

### Cited Findings
- `[observed]` On 2026-11-09, an isolated built-in browser opened `https://cursor.com/agents` and was redirected to `https://authenticator.cursor.sh/?client_id=…&redirect_uri=https%3A%2F%2Fcursor.com%2Fapi%2Fauth%2Fcallback…` with page title **"Sign in"**. No credentials were entered, no profile was used, and no login bypass was attempted — [S16](https://cursor.com/agents).
- `[official-documented]` The Cloud Agents weekly release notes are labeled by the Monday starting each week; the newest entry is **Week of Oct 19, 2026**, and the page's dated Markdown mirror confirms entries for Aug 31, Sep 7, Sep 14, Sep 21, Sep 28, Oct 5, Oct 12, and Oct 19, 2026 — [S6](https://cursor.com/docs/release-notes/cloud-agents).
- `[official-documented]` The general release-notes timeline is version-tagged, not date-tagged, and runs through **v2026.10.15** and **3.27**; older entries used here are labeled by version (3.20–3.26) — [S7](https://cursor.com/docs/release-notes).
- `[official-documented]` Cursor 3 / Agents Window became generally available on **April 2, 2026**; the Agents Window is the agent-first interface with multi-workspace, new diffs view, parallel agents, local/cloud handoff, cloud subagents, and worktrees — [S3](https://cursor.com/docs/agent/agents-window), [S15](https://cursor.com/blog/cursor-3).
- `[official-documented]` "Cloud Agents" were formerly called **Background Agents** — [S1](https://cursor.com/docs/cloud-agent).
- `[official-documented]` Team viewing is read-only by default; a team admin can enable **team follow-ups**; a viewer must connect their own source-control account and have repository access — [S1](https://cursor.com/docs/cloud-agent).
- `[conflicting]` Mobile docs say push on **turn finish** — [S8](https://cursor.com/docs/cloud-agent/mobile); the newer Week of Aug 31, 2026 note says **no turn-finished push**, only questions/approvals — [S6](https://cursor.com/docs/release-notes/cloud-agents). Do not treat "push on every finish" as current.
- `[official-documented]` The iOS help and release notes document **voice chat**, **Siri** launch, **Design Mode**, and voice-call **Live Activities**; exact availability may be plan/version-gated — [S9](https://cursor.com/help/ai-features/mobile-app), [S7, 1.16.0](https://cursor.com/docs/release-notes).
- `[negative finding]` No primary source documents a **Pause** button, **Pause** API, or user-initiated pause/resume for cloud tasks. "Pause" appears only as a state in integration notes — [S6](https://cursor.com/docs/release-notes/cloud-agents), [S2](https://cursor.com/docs/cloud-agent/capabilities).
- `[negative finding]` No primary source documents a **per-task dollar cost** or **token-count UI**; token usage is API-level, and billing is plan-level — [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S1](https://cursor.com/docs/cloud-agent).
- `[negative finding]` No primary source documents a **Logs tab** by that exact label; logs are exposed through run events, setup logs, Terminal, and PR artifacts — [S2](https://cursor.com/docs/cloud-agent/capabilities), [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[negative finding]` No primary source documents a **Delete** control in the web/desktop task UI; delete is documented in the API/SDK — [S5](https://cursor.com/docs/cloud-agent/api/endpoints).
- `[inferred]` The safest replication target is the **documented state machine + transcript + review handoff + cross-device continuity**, not an attempt to pixel-match an unobservable UI.
- `[inferred]` A plan should explicitly decide whether to expose runs as a first-class object. Cursor exposes runs in the API/dashboard but keeps the task list agent-centric; that is a deliberate abstraction.

### Inferences
- `[inferred]` Public documentation is sufficient to replicate the **behavior and state model** of Cursor's hosted tasks, but not to pixel-match the authenticated UI. A UOR-R4 plan should separate "behavioral parity" (buildable now) from "visual parity" (needs an account walkthrough).
- `[inferred]` The documented labels are unusually specific for a public docs set (for example "Waiting on", "Input Needed", "Changing course…", "This run hit an unexpected error. Retry or send a follow-up below."). That suggests the strings are intentional product copy, not incidental; a replication should treat them as a vocabulary to design around, not necessarily to copy verbatim.
- `[inferred]` The conflict over mobile push, plus the absence of Pause/Delete/Logs labels, indicates the public docs are a composite of docs, release notes, and help pages at different freshness levels. Any decisive UX decision should be verified on a real account/device before freeze.

### Coverage matrix

| Surface | What is documented | What is not observable without account |
|---|---|---|
| Web `cursor.com/agents` | URL, list API semantics, header/status/prompt, transcript, web/mobile layout, source filter; status pills documented for desktop, web parity unverified | Exact layout, default tab, menus, empty states, composer placeholder, visuals |
| Desktop Agents Window | Sidebar groups, pin/archive/rename/search, Error/Show filter, tabs, Changes/PR/Terminal/Desktop, Worked sections, Retry/Stop | Exact layout, icons, default views, menu contents, keyboard map beyond quoted shortcuts |
| iOS | Inbox, Live Activities, push, review/merge, no editor/terminal, Remote Control, source tag | Exact screens, labels, empty states, notification permission flow, sync behavior |
| Slack / GitHub / Linear | Entry points and agent URL sharing; `@cursor` launches; PR comments/`Open in Web` | Exact card/menu text and state transitions; not probed |
| API / SSE | Full agent/run lifecycle, follow-up/steer, cancel, archive/delete, usage, artifacts, stream events | Nothing about visual UI; error taxonomy is API-specific |

### Gaps
- `[unverified]` Every authenticated visual and interaction detail: exact control text not quoted above, default selected tab, menu ordering, iconography, color tokens, hover/focus states, empty/loading/error illustrations, responsive breakpoints beyond 1024px, composer placeholder, onboarding.
- `[unverified]` Whether the web detail page exposes **Changes/PR/Terminal/Desktop/Artifacts** exactly as the desktop Agents Window does; the docs often describe the Agents Window specifically.
- `[unverified]` Whether the task list shows **runs** as expandable children, or only the latest run.
- `[unverified]` Whether **team follow-ups**, **shared agents**, and **read-only viewing** have distinct UI states/badges.
- `[unverified]` Exact **notification** defaults, permission prompts, and whether browser push exists alongside iOS push.
- `[unverified]` Exact **usage/cost** UI on the task page, and whether per-run token counts are ever shown to non-API users.
- `[unverified]` Whether **Pause** or **Resume** exists behind a feature flag, enterprise setting, or newer release not covered by the retrieved notes.
- `[unverified]` Whether **Android** has shipped since the help page's "not yet."
- `[unverified]` The current production behavior on 2026-11-09 may differ from docs if a release is rolling out; the newest Cloud Agents note is Oct 19, 2026, and docs can lag.

---

## 10. Replication checklist for a UOR-R4 hosted-task experience (inferred)

> Everything in this section is `[inferred]` from the cited findings; it is a planning aid, not a claim about Cursor.

| Layer | Minimum objects/states | Minimum actions | Evidence anchor |
|---|---|---|---|
| Task identity | Task/agent id (`bc-…` style), title, repo(s), source, created/updated, pinned, archived, unread | create, rename, pin/unpin, archive/unarchive, delete (API/if exposed), share link, filter by source/status | [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S6](https://cursor.com/docs/release-notes/cloud-agents) |
| Runs | Run id, status (`CREATING`/`RUNNING`/`WAITING_FOR_BACKGROUND_WORK`/`FINISHED`/`ERROR`/`CANCELLED`/`EXPIRED`), timestamps, duration, result, branch/PR | follow-up (new run), steer, cancel, retry, read stream | [S5](https://cursor.com/docs/cloud-agent/api/endpoints) |
| Transcript | User/assistant messages, tool calls, thinking/"Worked" blocks, status line, subagent nesting, side chats | send, queue, reorder queue, send now, stop, retry, fork, search, restore checkpoint | [S4](https://cursor.com/docs/agent/overview), [S7, 3.25](https://cursor.com/docs/release-notes) |
| Review | Changes scopes (last turn/branch/commits), PR sub-tabs, diff comments, review state | review, comment, approve/request changes, discard review, merge/close | [S7, 3.22/3.26](https://cursor.com/docs/release-notes), [S8](https://cursor.com/docs/cloud-agent/mobile) |
| Execution evidence | Terminal, cloud desktop, browser, ports, artifacts (screenshots/videos/logs) | open, take control, download, copy, attach to chat/PR | [S2](https://cursor.com/docs/cloud-agent/capabilities), [S5](https://cursor.com/docs/cloud-agent/api/endpoints) |
| Notifications | Inbox, unread, Live Activities (≤8), push for questions/approvals, run-finished chime (coordinator only) | mark read, open, act | [S6, Aug 31](https://cursor.com/docs/release-notes/cloud-agents), [S9](https://cursor.com/help/ai-features/mobile-app) |
| Usage | Per-run tokens, total tokens, duration, success rate, filters | view, filter, export (team) | [S5](https://cursor.com/docs/cloud-agent/api/endpoints), [S6, Aug 31](https://cursor.com/docs/release-notes/cloud-agents) |
| Cross-device | Account-scoped pins, shared transcript, source tags, archive semantics, Remote Control/Move to Cloud | continue, filter, hand off | [S8](https://cursor.com/docs/cloud-agent/mobile), [S9](https://cursor.com/help/ai-features/mobile-app) |

**Highest-risk unknowns before building.** The default task-detail view, the exact list/menu hierarchy, whether runs are visible as children, whether Pause/Resume exist, and whether logs/artifacts have dedicated tabs. These should be resolved with a real account walkthrough before any pixel-level replication; the documented state machine and action set are sufficient to build the underlying product without them.
