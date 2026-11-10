# Cursor editor/IDE interaction surface as of 2026-11-09

**Scope:** Cursor's editor/IDE interaction surface, separating the hosted web experience from the Cursor desktop app, the Cursor 3 Agents Window, Origin (Cursor's hosted code host), mobile and CLI. This is a research inventory only; no product code was changed, no account was used, and no repository was mutated.

**Evidence labels used throughout:**
- **[OFFICIAL]** Cursor or VS Code documentation/help/changelog.
- **[OBSERVED]** Public UI or a public forum report of observed behavior; not a product guarantee.
- **[INFERRED]** A conclusion drawn from cited facts, not directly stated by the vendor.
- **[UNVERIFIED]** No reliable public source found at access time.

**Access date:** 2026-11-09. Cursor docs sitemap lastmod `2026-11-09T22:10:00Z`; Cursor release notes at access time include desktop `3.27` and iOS `1.16.0`. Cursor 3 / Agents Window was released 2026-04-02 per [Cursor's Agents Window doc](https://cursor.com/docs/agent/agents-window).

**Bottom line (five findings):**
- **The desktop IDE is the only full VS Code inheritance surface:** workbench, menus, keybindings, Open VSX extensions, themes, Remote SSH/Dev Containers/WSL, LSP, debugger and notebooks.
- **The Agents Window is Cursor 3's from-scratch agent-first desktop surface:** multi-repo sidebar, file/terminal/browser/Changes/Output/canvas tabs, `Cmd+K` command palette, remote SSH, LSP; themes require **Open IDE**, and VS Code extensions are not its extension model.
- **Cursor Web (`cursor.com/agents`) is a cloud-agent workspace, not a browser copy of VS Code:** official mobile FAQ confirms editor, terminal and file browser; its editor depth, LSP, debugger, notebooks, themes and extension support are not publicly documented.
- **Inline edit is retired:** `Cmd+K` no longer opens an inline-edit widget; selecting code and pressing `Cmd+L`/`Cmd+K` opens Agent with the selection. `Tab` remains the IDE autocomplete surface.
- **Menus, commands, settings and keybindings are dynamic:** the defensible inventory is built-in commands + Cursor commands + extension/plugin/MCP/skill commands, filtered by context and account; no finite list is claimed.

---

## 1. What are the actual Cursor surfaces, and which one is "the editor"?

### Takeaway
Cursor is no longer one editor. As of 2026-11-09 there are six distinct interaction surfaces: **Cursor Web** (`cursor.com/agents`) for cloud agents, the **desktop IDE** (the VS Code-derived editor), the **desktop Agents Window** (Cursor 3's from-scratch agent-first workspace), **Origin** (`cursor.com/codebase`) for hosted code browsing/review, **mobile**, and the **CLI**. Only the desktop IDE is the VS Code fork; the Agents Window is a new interface, and Cursor Web is a browser workspace over cloud VMs, not the full VS Code workbench. **[INFERRED for the web characterization; OFFICIAL for the surfaces]**

### Cited Findings

**Cursor 3 replaced the single-surface model with an agent-first workspace.** Cursor says it "forked VS Code" originally but with Cursor 3 "took that a step further by building this new interface from scratch, centered around agents," while still allowing users to "switch back to the Cursor IDE at any time" ([Cursor 3 blog](https://cursor.com/blog/cursor-3)). The Agents Window is "Cursor's agent-first interface... a unified workspace to build with agents across repos and environments, including local, cloud, remote SSH, and more" ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).

**The desktop IDE is still the VS Code-derived surface.** Cursor's own keyboard help says "Cursor uses the same default shortcuts as VS Code, plus shortcuts for AI features" ([Cursor help: keyboard shortcuts](https://cursor.com/help/customization/keyboard-shortcuts)). The Agents Window doc distinguishes the two: "The editor works well when you want the classic IDE with VS Code extensions and flexible screen splitting to see many files at once" ([Agents Window doc](https://cursor.com/docs/agent/agents-window)). Cursor 3.0 also "Removed cloud agents from the Editor," moving that work to the Agents Window/web ([Cursor 3.0 changelog](https://cursor.com/changelog/3-0)).

**Cursor Web is a cloud-agent workspace, not merely a chat dashboard.** Cursor's mobile FAQ says the iOS app has "no editor, terminal, or file browser" and that "For full editing, use the desktop app or cursor.com/agents" ([Cursor help: mobile app](https://cursor.com/help/ai-features/mobile-app)). The iOS reference repeats: "Editor, terminal, and file browser" are things that stay on `cursor.com/agents` or the Cursor Dashboard ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)). Cursor release notes also describe the web agent workspace: "Keys like Ctrl+K and Ctrl+F in the agent terminal now go to the shell instead of triggering page shortcuts" ([release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes)), and "Cloud agent conversations at cursor.com/agents ... now use the same transcript as the Agents Window" ([release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes)).

**The Agents Window has a live cloud workspace too.** Release notes describe "the Changes panel, file tree, terminal, and forwarded ports in the Agents Window, as Cursor-hosted cloud agents do" ([release notes 3.26](https://cursor.com/docs/release-notes)); and "Open the live workspace of agents on Self-Hosted Machines... with their files, changes, a terminal, and forwarded ports" ([release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes)).

**Origin is a separate hosted code surface.** Origin is "currently released in early beta"; at `cursor.com/codebase` you can "browse files, search code, and inspect commit history on the **Code** tab," with **Pull requests** and **Settings** tabs alongside ([Origin browse](https://cursor.com/docs/origin/browse)). Pull requests have a **Diff / Tour** control, `T` toggles between them, and `Cmd+I` adds selected text to chat ([Origin code tours](https://cursor.com/docs/origin/code-tours)).

**Mobile is review-only.** Cursor for iOS is "built for directing and reviewing agents, not for editing"; there is no editor, terminal, or file browser, only changed files in a diff view and PR review/merge ([Cursor help: mobile app](https://cursor.com/help/ai-features/mobile-app)). It does support Design Mode, dictation, voice chat, MCP server selection per run, slash commands, and automations ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)).

**The CLI is a separate terminal surface, not the IDE.** Cursor CLI exposes an `agent` command, shell mode, headless mode, ACP, and a documented slash-command table ([CLI overview](https://cursor.com/docs/cli/overview); [CLI slash commands](https://cursor.com/docs/cli/reference/slash-commands)). It is useful as a parity reference for commands, but it is not a hosted editor.

### Surface matrix

| Capability | Cursor Web (`cursor.com/agents`) | Desktop IDE | Desktop Agents Window | Origin web (`cursor.com/codebase`) | iOS |
|---|---|---|---|---|---|
| Full code editor | **Yes, per official mobile FAQ** ("For full editing, use ... cursor.com/agents") — depth unverified | **Yes** (VS Code workbench) | **Yes** — file viewing/editing, `Cmd+P`, `Cmd+Shift+F`, LSP go-to-definition per Cursor 3 blog | **No editor documented**; code browsing only | **No** — diff view only |
| File tree | **Yes** (mobile FAQ/release notes) | **Yes** (Explorer) | **Yes** (file tree in live workspace) | **Yes** (Code tab tree) | **No** |
| Integrated terminal | **Yes** (release notes; forum) | **Yes** (VS Code integrated terminal) | **Yes** (terminal tab; forwarded ports) | **No** | **No** |
| Search (files/text) | **Likely yes**; exact UI unverified | **Yes** (`Ctrl+P`, `Ctrl+Shift+F`) | **Yes** (`Cmd+P`, `Cmd+Shift+F`) | **Yes** (`T` Go to file, header search) | **No** |
| Tabs | **Yes** (Changes/PR/file tabs; exact tab model unverified) | **Yes** (editor tabs/groups; Agent Tabs) | **Yes** (file, terminal, browser, Changes, Output, side chat, canvas tabs) | **Yes** (Code/PR/Settings, PR sub-tabs) | **No** |
| Source control / diff / PR | **Yes** (Changes, PR review/merge/create) | **Yes** (Source Control view, diff editor; GitHub PR via extension) | **Yes** (Changes panel, PR tab, stacked PRs) | **Yes** (PR review/merge, code tours) | **Yes** (diff review/merge only) |
| Command palette | **Not documented**; browser shortcuts may conflict | **Yes** (`Cmd+Shift+P`) | **Yes** (`Cmd+K` palette per 3.25 release notes) | **Not documented** | **No** |
| Native menu bar | **No** (browser) | **Yes** (File/Edit/Selection/View/Go/Run/Terminal/Help) | **Reduced** (observed File/Edit/View/Help) | **No** | **No** |
| Settings | **Yes** (`Cmd+,` on agents pages per release notes) | **Yes** (Cursor Settings `Cmd+Shift+J`, Settings `Cmd+,`) | **Yes** (`Cmd+,`; Settings > Agents etc.) | **Repo/PR settings only** | **Limited** (profile/version) |
| Themes | **Not documented**; Cursor help says theme controls are part of the IDE and to open IDE from the Agents Window | **Yes** (Color Theme, extensions) | **Not documented**; help says open IDE for theme controls | **No** | **No** |
| VS Code extensions | **Not documented**; web is not the IDE | **Yes** (Open VSX via Cursor marketplace proxy) | **Not documented** — Agents Window uses Cursor plugins/skills/MCP; VS Code extensions are IDE-scoped | **No** | **No** |
| LSP / go-to-definition | **Unverified** | **Yes** (VS Code language services) | **Yes** (Cursor 3 blog: "full LSPs") | **No** | **No** |
| Traditional debugger | **Unverified** | **Yes** (VS Code Run and Debug) | **Not documented**; Debug Mode is agent instrumentation, not breakpoint UI | **No** | **No** |
| Notebooks | **Unverified** | **Yes** (Jupyter support; Cursor 1.0+) | **Unverified** | **No** | **No** |
| Local filesystem | **No** — cloud VM; attachments/Remote Control only | **Yes** (OS files, Workspace Trust) | **Yes** (local and remote SSH workspaces) | **No** | **No** |
| Remote SSH / containers / WSL | **Unverified from web** | **Yes** (VS Code Remote extensions; Cursor release notes) | **Yes** (remote SSH explicitly; WSL/dev containers via release notes) | **No** | **Remote Control to a local/Remote SSH workspace** |
| Cloud agents | **Yes** (primary surface) | **No** since Cursor 3.0 ("Removed cloud agents from the Editor") | **Yes** | **Cloud agents can author PRs/tours** | **Yes** (control/review) |
| AI chat / Agent | **Yes** | **Yes** | **Yes** | **Ask Cursor** on PR tours/pages | **Yes** |
| Tab autocomplete | **Not documented**; likely no | **Yes** (Cursor Tab) | **Not documented**; agent-first UI | **No** | **No** |
| Inline edit (`Cmd+K`) | **No** (retired) | **No** (retired; replaced by Agent) | **No** (`Cmd+K` is palette) | **No** | **No** |
| Voice chat | **Yes** (voice chat works "on web") | **Yes** | **Yes** | **No** | **Yes** |
| Design Mode | **Unverified** (browser annotation is documented in Agents Window) | **No** (Design Mode is Agents Window browser) | **Yes** | **No** | **Yes** (attach/draw) |

### Inferences
- A hosted UOR-R4 Studio should model Cursor Web as a **cloud-agent workspace**: agents sidebar + transcript + prompt + Changes/PR + terminal + file tree/editor, not as a browser copy of the full VS Code workbench. The official mobile FAQ is the strongest evidence that web editing exists; it does not establish VS Code extension, theme, debugger, notebook, or LSP parity. **[INFERRED]**
- The desktop IDE is the only surface where the full VS Code inheritance claim is defensible: extension host, Open VSX, themes, menus, keybindings, debugger, notebooks, Remote Development. **[INFERRED]**
- The Agents Window is the strategic default for agent work; the IDE is now a mode you "Open IDE" into. This is a product-direction inference from Cursor 3 copy and the "Open IDE" command. **[INFERRED]**

### Gaps
- No logged-in observation of `cursor.com/agents`. The exact web editor (Monaco vs embedded VS Code vs custom), whether it supports LSP, themes, extensions, notebooks or a debugger, and its tab model are not documented publicly. **[UNVERIFIED]**
- The mobile FAQ's phrase "For full editing, use the desktop app or cursor.com/agents" is the only official statement that web has an editor; no Cursor docs page describes that editor's capabilities. **[UNVERIFIED]**
- No official statement that the Agents Window lacks a traditional debugger or notebooks; absence of documentation is not proof of absence. **[UNVERIFIED]**

---

## 2. Workbench inventory: activity bar, sidebars, file tree, search, tabs, editor

### Takeaway
The desktop IDE inherits VS Code's six-area workbench (Editor, Primary Side Bar, Secondary Side Bar, Status Bar, Activity Bar, Panel) and its file tree/search/tabs/editor-group behavior. The Agents Window and Cursor Web replace that with an agent/repo sidebar plus a tabbed workspace (file, terminal, browser, Changes, Output, side chat, canvas). Origin has a hosted Code tab with a file tree, branch control, `T` file jump, code search and commit history.

### Cited Findings

**VS Code baseline (inherited by the desktop IDE).** VS Code's UI is "divided into six main areas": Editor, Primary Side Bar, Secondary Side Bar, Status Bar, Activity Bar, and Panel. The Activity Bar is "located on the far left-hand side" and switches views; the Panel "by default... contains output, debug information, errors and warnings, and an integrated terminal." Open files "are displayed with tabbed headers (Tabs)" and editors can be split into groups side by side ([VS Code: User interface](https://code.visualstudio.com/docs/getstarted/userinterface)). Cursor's IDE is a VS Code fork, so this is the inherited baseline; Cursor's own docs describe the IDE as the place "with VS Code extensions and flexible screen splitting" ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).

**Cursor's AI sidepanel is an additional surface.** Agent opens "in sidepane with `Cmd+I`" ([Agent overview](https://cursor.com/docs/agent/overview)). `Cmd+E` toggles the "Agent layout" ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). Cursor 3.0 added "Agent Tabs in the Editor" to "view multiple chats at once, side-by-side or in a grid" ([Cursor 3.0 changelog](https://cursor.com/changelog/3-0)). Chat tabs have their own shortcuts: `Cmd+T` new chat tab, `Cmd+[` / `Cmd+]` previous/next chat, `Cmd+W` close chat ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).

**Agents Window navigation.** The Agents Window is multi-workspace, with Projects in the left-hand nav ([Projects doc](https://cursor.com/docs/agent/projects)). "If you want to view or edit files without leaving the Agents Window, you can type `Cmd+P` to search files, or `Cmd+Shift+F` to search all files" ([Agents Window doc](https://cursor.com/docs/agent/agents-window)). Release notes add: tabs can be files, terminals, browser, Changes, Output, side chats, or canvases; "Back reopens tabs you just closed" and `Cmd+Shift+T` focuses a tab Back already restored ([release notes 3.25](https://cursor.com/docs/release-notes)). The sidebar groups pinned items and repositories/workspaces/chats, and remembers collapsed/expanded sections ([release notes 3.27](https://cursor.com/docs/release-notes)).

**Cursor Web and desktop navigation.** Release notes describe the **desktop Agents Window** grouping pinned agents into "Today, Last 7 days, and an Older section"; **web grouping is unverified**. Pins are saved to the account across browsers/devices; the 75-pin limit is documented for desktop 3.22 only, and the web limit is unverified ([release notes, Week of Sep 28, 2026](https://cursor.com/docs/release-notes)). Web agent conversations use the same transcript as the Agents Window, and the agent sidebars on codebase pages support mouse wheel/Page Up/Page Down scrolling ([release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes)). The 2025 web-agent announcement describes a Kanban view of agents and rich context (images, follow-ups, parallel agents), plus diff/PR review ([Cursor blog: agents on web](https://cursor.com/blog/agent-web)). **Project/repo picker (shared cloud-agent navigation; web parity unverified).** Cursor help describes a redesigned picker: create a project without leaving the picker; connect GitHub/GitLab/Azure DevOps inline; search projects scoped by **This Computer**, **Cloud**, or a remote machine; remove a project from **Recents**; Remote Machines are consolidated in the same flow ([Cursor help: Cloud Agents](https://cursor.com/help/ai-features/cloud-agents)). Release notes 3.27 (desktop) add that the picker opens on a search field, accepts a pasted clone URL, and has a footer to clone a repository, start from scratch, or create a new repository ([release notes 3.27](https://cursor.com/docs/release-notes)); web parity is unverified.

**Origin Code tab.** "On **Code**, browse directories and open files in the tree. Switch branches from the branch control in the header. The green **Code** button opens HTTPS and Origin CLI clone setup." "Use **Go to file** (shortcut `T`) or the search field in the repository header to find files and code matches." Commit history is available per branch; an author filter is offered, but "isn't offered on the history of a single file or folder" ([Origin browse](https://cursor.com/docs/origin/browse)).

### Inferences
- The IDE's activity bar/file tree/search/tabs/editor are best treated as **VS Code inheritance**, not Cursor-specific inventions. Cursor-specific additions are the Agent sidepanel, Agent layout, Agent Tabs and AI commands. **[INFERRED]**
- The Agents Window's sidebar is closer to an agent/repo navigator than a file Explorer; the file tree is one tab among many. A hosted Studio can copy the tab model without copying VS Code's editor-group semantics. **[INFERRED]**
- Origin's `T` shortcut is a hosted code-browser convention; it is not a VS Code shortcut and should not be conflated with the IDE's `Ctrl+P`. **[INFERRED]**

### Gaps
- Cursor does not publish a complete IDE activity-bar icon list; whether Cursor adds a dedicated activity-bar Agent icon (vs only a sidepanel command) is not documented. **[UNVERIFIED]**
- The exact web tab model (file tabs vs editor groups vs only Changes/PR tabs) is not documented. **[UNVERIFIED]**
- Whether the web editor supports split view, breadcrumbs, minimap, sticky scroll or multi-cursor is not documented. **[UNVERIFIED]**

---

## 3. AI input surface: Tab, inline edit, chat/Agent, prompt controls

### Takeaway
Cursor Tab remains the IDE autocomplete surface (`Tab` accept, `Esc` reject, `Cmd+→` accept word, second `Tab` jump). Inline edit via `Cmd+K` is explicitly retired: selecting code and pressing `Cmd+L` or `Cmd+K` now opens Agent with the selection referenced. Chat/Agent is the primary input across IDE, Agents Window and web, with documented `@` context, `/` commands, image attachments, voice, mode/model pickers, queue/steer semantics and chat history. IME behavior is not officially specified and is a known weak point.

### 3.1 Tab autocomplete

**Official behavior.** Tab is "Cursor's AI-powered autocomplete" that "suggests code as you type, based on your recent edits, surrounding code, and linter errors." Suggestions appear as grayed-out text; `Tab` accepts, `Escape` rejects, `Cmd+→` (Mac) / `Ctrl+→` (Windows/Linux) accepts word-by-word. After accepting, pressing `Tab` again triggers "jump-in-file," predicting the next editing location; cross-file jumps show "a portal window at the bottom of the editor." It can modify multiple lines, add missing imports, and suggest coordinated edits across files ([Cursor help: Tab](https://cursor.com/help/ai-features/tab); [Cursor Tab doc](https://cursor.com/docs/tab/overview), page rendered in Chinese but labels English).

**Settings/labels.** Cursor Tab settings include `Cursor Tab`, `Partial Accept`, `Show Suggestions in Comments`, `Whitespace-only suggestions`, `Imports`, and `Python Auto Import (beta)` ([Cursor Tab doc](https://cursor.com/docs/tab/overview)). The status bar (bottom-right) lets you **Snooze**, **Disable globally**, or **Disable for specific extensions** ([Cursor help: Tab](https://cursor.com/help/ai-features/tab)). To remap, search for `Accept Cursor Tab Suggestions` in Keyboard Shortcuts ([Cursor help: Tab](https://cursor.com/help/ai-features/tab)).

**Scope conflicts.** `Tab` is overloaded by focus: in the chat input it is "Cycle to next message" ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)); in a code editor it accepts a Tab suggestion; in a notebook it can move cell selection/add cells ([VS Code notebooks](https://code.visualstudio.com/docs/datascience/jupyter-notebooks)). Any hosted reimplementation must scope `Tab` by focused component. **[INFERRED]**

### 3.2 Inline edit and `Cmd+K`

**Official retirement statement.** Cursor's help says: "Select the code, then press `Cmd + L` or `Cmd + K` on Mac (`Ctrl + L` or `Ctrl + K` on Windows/Linux). Agent opens with those lines referenced in the input, ready for your instructions. `Cmd + K` used to open inline edit, which has been replaced by this workflow." ([Cursor help: Agent mode](https://cursor.com/help/ai-features/agent)). The keyboard reference lists `Cmd L` or `Cmd K` as "Toggle Sidepanel (unless bound to mode)" and, under Code Selection & Context, "Add selection to new chat" ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).

**What `Cmd+K` means now by surface:**
- **Desktop IDE:** toggles the sidepanel, or with a selection adds the selection to a new chat; it does not open an inline edit widget. **[OFFICIAL]**
- **Agents Window:** `Cmd+K` opens the command palette; release notes describe "The Cmd+K palette now offers Select Model, Archive, and New Side Chat" ([release notes 3.25](https://cursor.com/docs/release-notes)). This is a direct shortcut collision with the IDE's `Cmd+K`; scope by surface. **[OFFICIAL + INFERRED]**
- **Web:** no documented inline edit; not applicable. **[UNVERIFIED]**

**Inference:** Cursor's inline-edit interaction has been replaced by "selection → Agent chat → diff review." The diff view is the review surface, not a modal inline-edit prompt. **[INFERRED]**

### 3.3 Chat/Agent panel and prompt controls

**Opening and layout.** Agent opens in the sidepanel with `Cmd+I`; `Cmd+L` or `Cmd+K` also toggle the sidepanel unless bound to a mode; `Cmd+E` toggles Agent layout ([Agent overview](https://cursor.com/docs/agent/overview); [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). In the Agents Window, the prompt sits in a tabbed workspace with Projects/agents/repos in the sidebar, an editor panel, and tabs for file/terminal/browser/Changes/Output/side chat/canvas ([Agents Window doc](https://cursor.com/docs/agent/agents-window); [release notes 3.25](https://cursor.com/docs/release-notes)).

**Prompt composition controls (documented):**
- **Mode picker dropdown** in Agent; `Shift+Tab` rotates modes; `Cmd+.` opens the Mode Menu ([Plan Mode doc](https://cursor.com/docs/agent/plan-mode); [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).
- **Model picker dropdown** at the top of the chat input; `Cmd+/` cycles models; `Cmd+Opt+/` (Mac) / `Ctrl+Alt+/` toggles models; set a default in Cursor Settings > Models ([Prompting doc](https://cursor.com/docs/agent/prompting); [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).
- **Context ring** next to the prompt shows how full the context window is; clicking opens a breakdown tray with `System prompt`, `Tools`, `Rules`, `Skills`, `MCP`, `Subagents`, `Summarized conversation`, `Conversation` ([Prompting doc](https://cursor.com/docs/agent/prompting)).
- **`@`** opens context suggestions; **`/`** opens skills/commands; **image attachment** via drag/drop or paste; **microphone** for dictation; **waveform** for voice chat; **Send/Stop** button; **queue** behavior while the agent runs ([Prompting doc](https://cursor.com/docs/agent/prompting); [Agent overview](https://cursor.com/docs/agent/overview); [Voice chat doc](https://cursor.com/docs/agent/voice-chat)).

**Exact keyboard reference for chat** ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)):

| Shortcut (Mac / Windows-Linux) | Action |
|---|---|
| `Return` / `Enter` | Queue message (default) |
| `Cmd Return` / `Ctrl Enter` when typing | Force send message |
| `Cmd Shift Backspace` / `Ctrl Shift Backspace` | Cancel generation |
| `Cmd Shift L` / `Ctrl Shift L` with code selected | Add selected code as context |
| `Cmd V` / `Ctrl V` with code or log in clipboard | Add clipboard as context |
| `Cmd Shift V` / `Ctrl Shift V` with code or log in clipboard | Add clipboard to input box |
| `Cmd Return` / `Ctrl Enter` with suggested changes | Accept all changes |
| `Cmd Shift Backspace` / `Ctrl Shift Backspace` with suggested changes | Reject all changes |
| `Tab` | Cycle to next message |
| `Shift Tab` | Rotate between Agent modes |
| `Cmd Opt /` / `Ctrl Alt /` | Model toggle |
| `Cmd N` / `Cmd R` | New chat |
| `Cmd T` / `Ctrl T` | New chat tab |
| `Cmd [` / `Ctrl [` | Previous chat |
| `Cmd ]` / `Ctrl ]` | Next chat |
| `Cmd W` / `Ctrl W` | Close chat |
| `Escape` / `Esc` | Unfocus field |

**Queue vs steer vs interrupt.** "While Agent is working, type your next instruction. Press `Enter` to add it to the queue... Press `Cmd+Enter`/`Ctrl+Enter` to send immediately, bypassing the queue" ([Agent overview](https://cursor.com/docs/agent/overview)). "Type a follow-up and hit **Send now**, or press `Enter` twice. The message is delivered at the agent's next tool call instead of cutting off work mid-action... Press `Tab` to queue the message for after the turn instead" ([Agent overview](https://cursor.com/docs/agent/overview)). Cursor 3.25 release notes say "New installs steer the running agent by default... Steer is listed first in Settings. Existing installs keep their current behavior" ([release notes 3.25](https://cursor.com/docs/release-notes)). In the CLI, `Enter` steers and `Enter` again interrupts ([Agent overview](https://cursor.com/docs/agent/overview)).

**Checkpoints.** Agent automatically creates checkpoints before significant changes; click a checkpoint in the chat timeline to preview files, then restore. Restoring "reverts files only; it does not remove messages from the conversation" ([Agent overview](https://cursor.com/docs/agent/overview)). To shorten the conversation, fork the chat from an earlier message ([Cursor help: Agent mode](https://cursor.com/help/ai-features/agent)).

**Side chats and goals.** `/side` or `/btw` opens a side chat without interrupting the main conversation; it uses the parent thread as hidden reference context and keeps its own transcript; `@`-mention the side chat in the main thread to pull context back ([Agent overview](https://cursor.com/docs/agent/overview)). `/goal` gives the agent a long-lived objective; in the CLI, `Ctrl+C` pauses the goal ([Agent overview](https://cursor.com/docs/agent/overview)). `/loop` runs a prompt or skill on a recurring/variable interval ([Agent overview](https://cursor.com/docs/agent/overview); [Cloud Agent capabilities](https://cursor.com/docs/cloud-agent/capabilities)).

### 3.4 Context pickers, `@` mentions, attachments and images

**`@` context items (official list):**
- **Files & Folders:** `@auth.ts` or `@src/components/`; type `/` after selecting a folder to navigate deeper.
- **Terminals:** `@Terminals` includes terminal output.
- **Chats:** `@Chats` references a previous conversation.
- **Git diffs:** `@Commit (Diff of Working State)` for uncommitted changes; `@Branch (Diff with Main)` for the full branch diff.
- **Browser:** `@Browser` attaches context from the built-in browser.
- Multiple items can be attached by typing `@` multiple times ([Prompting doc](https://cursor.com/docs/agent/prompting); [Cursor help: context](https://cursor.com/help/customization/context)).

**Images.** "Drag and drop an image file into the chat input" or "Paste from clipboard with `Cmd+V`/`Ctrl+V`, including screenshots" ([Prompting doc](https://cursor.com/docs/agent/prompting)). Agent-generated images are saved to the project's `assets/` folder by default and shown inline in chat ([Agent overview](https://cursor.com/docs/agent/overview)).

**Web attachment limits.** At `cursor.com/agents`, "documents and images have a 4 MB per-file limit. Videos have a separate 15 MB per-file limit. The browser rejects files over these limits before upload." Supported document formats: PDF, Markdown (`.md`/`.markdown`), plain text, CSV, JSON, HAR, XML, YAML, TOML, INI, log files. ZIP is not supported as a web document attachment. "For non-sensitive files already in source control, ask the agent to read them from its checked-out repository." If a file is too large, keep only the needed content, split it, or start the Cloud Agent from Cursor desktop because "attachment limits can differ from the web app" ([Cursor help: Cloud Agents](https://cursor.com/help/ai-features/cloud-agents)).

**Attachment validation/error states.** Release notes 3.27: attached images/videos are labeled with file name; very long image names attach instead of failing; HEIC/HEIF on Windows/Linux "now explains that those images need macOS"; slow HEIC conversions get a clear timeout; "Pasted documents with no local file attach to cloud agent prompts"; videos/documents that can't be attached show a warning instead of disappearing ([release notes 3.27](https://cursor.com/docs/release-notes)). A locked prompt "no longer accepts dropped or pasted files" ([release notes 3.26](https://cursor.com/docs/release-notes)).

### 3.5 Slash commands and Custom Modes

**Editor/Agents Window behavior.** "Type `/` in the chat input to invoke a skill. Pressing `Enter` attaches the skill to one message, and it fades as the conversation moves on. Use any skill as a Custom Mode to keep the agent focused on it while it works. Pick the skill from the `/` menu and press `Option+Enter`/`Alt+Enter` instead. You can also select **Use as Mode** from the skill entry. Inside a mode, the skill stays in context on every turn" ([Prompting doc](https://cursor.com/docs/agent/prompting)). Custom Modes are "available in the Agents Window and the CLI"; any skill with a valid frontmatter block can back a mode, and optional `icon`/`color` frontmatter styles the mode badge ([Prompting doc](https://cursor.com/docs/agent/prompting)).

**Documented built-in slash commands (CLI reference).** The CLI page is the only official exhaustive table found. It includes:

| Command | Description |
|---|---|
| `/model [filter]` | Select a model; `Tab` to edit |
| `/run-everything [on\|off\|status]` | Toggle Run Everything or show status; `/auto-run` alias |
| `/plan [prompt]` | Switch to Plan mode/show plan/submit prompt |
| `/ask` | Toggle Ask mode |
| `/debug [prompt]` | Toggle Debug mode/submit prompt |
| `/goal [objective]` | Long-lived objective |
| `/logs` | Show debug log path and copy it |
| `/update` | Update Cursor Agent |
| `/max-mode` | Toggle Max Mode on legacy plans |
| `/rename <name>` | Rename chat |
| `/clear` | New chat; `/new`, `/new-chat`, `/newchat` aliases |
| `/resume` | Open recent chats and resume one |
| `/fork` | Fork current chat |
| `/summarize` | Summarize conversation; `/compress` alias |
| `/rewind` | Jump back to a previous message |
| `/vim` | Toggle Vim keys |
| `/line-numbers` | Toggle line numbers in code blocks |
| `/show-thinking` | Toggle thinking block display |
| `/status-indicators` | Toggle terminal title status indicators |
| `/shell [command]` | Enter Shell Mode; `/sh`, `/run` aliases |
| `/about` | Show CLI version/system/account info and copy it |
| `/setup-terminal` | Configure terminal newline keybindings |
| `/help [command]` | Show help |
| `/feedback <message>` | Share feedback |
| `/open` | Open repo Git root in Cursor; `/cursor` alias |
| `/copy-request-id` | Copy last request ID |
| `/copy-conversation-id` | Copy conversation ID |
| `/logout` | Sign out |
| `/quit` / `/exit` | Exit |
| `/mcp [list\|list-tools] [identifier]` | Manage MCP servers/list tools |
| `/plugin [subcommand]` | Manage plugins/marketplaces |
| `/config` | Configure CLI settings interactively |
| `/copy` | Copy a previous user message |
| `/sandbox` | Configure sandbox mode/network |
| `/bedrock [subcommand]` | Configure Bedrock when enabled |

([CLI slash commands](https://cursor.com/docs/cli/reference/slash-commands)). The task's scope is the editor, so treat this as a **command vocabulary** reference, not proof that every CLI command is exposed identically in the IDE/web. Cursor's mobile help says "Your slash commands, skills, and automations work the same on mobile as they do locally, in the CLI, and on the web" ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)), which supports cross-surface parity for user-defined commands/skills but does not prove every CLI built-in is in the IDE palette. **[INFERRED]**

**Additional commands documented for editor/Agents Window:**
- `/side`, `/btw` — side chats ([Agent overview](https://cursor.com/docs/agent/overview)).
- `/goal` — long-lived objective ([Agent overview](https://cursor.com/docs/agent/overview)).
- `/loop` — recurring prompt/skill ([Cloud Agent capabilities](https://cursor.com/docs/cloud-agent/capabilities)).
- `/in-cloud` — hand off to a cloud subagent; `/autopilot` — put a PR on autopilot ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).
- `/remote-control` — hand a local/Remote SSH session to the cloud loop, tools stay local ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)).
- `/agent-review` — run Agent Review from the agent window input ([Agent Review doc](https://cursor.com/docs/agent/agent-review)).
- `/worktree`, `/best-of-n`, `/apply-worktree`, `/delete-worktree` — worktree workflows ([Worktrees doc](https://cursor.com/docs/configuration/worktrees)).
- `/subscribe` — built-in skill for event subscriptions ([Cloud Agent capabilities](https://cursor.com/docs/cloud-agent/capabilities)).

**Dynamic command surface.** User commands are markdown files invoked with `/`; plugins bundle rules, skills, subagents, commands, MCP servers and hooks; the Customize page manages them at user/team/workspace scope ([Customize Cursor](https://cursor.com/docs/customize-cursor)). This surface is not finite: it depends on installed plugins, team marketplace, workspace files, and user skills. **[OFFICIAL + INFERRED]**

### 3.6 Modes, model choices and histories

**Modes.** Cursor's help table:

| Mode | Best for | Can edit files? |
|---|---|---|
| **Agent** | Building features, refactoring, fixing bugs | Yes |
| **Ask** | Understanding code, exploring architecture | No (read-only) |
| **Plan** | Complex features where you want to review the approach first | Yes (after you approve the plan) |
| **Debug** | Tricky bugs that need runtime evidence | Yes |

([Cursor help: Agent mode](https://cursor.com/help/ai-features/agent)). `Shift+Tab` cycles modes; the mode picker dropdown also switches; each mode "uses its own context, so switching modes starts a fresh context window" ([Cursor help: Agent mode](https://cursor.com/help/ai-features/agent)). Plan Mode asks clarifying questions, researches, creates an editable plan, and waits for approval; plans are saved by default in the home directory, with "Save to workspace" to move them into the workspace ([Plan Mode doc](https://cursor.com/docs/agent/plan-mode)). Debug Mode generates hypotheses, adds log statements to a local debug server running in a Cursor extension, asks you to reproduce the bug, analyzes logs, makes a targeted fix, then removes instrumentation ([Debug Mode doc](https://cursor.com/docs/agent/debug-mode)).

**Models.** Use the model picker dropdown at the top of the chat input, or `Cmd+/` to cycle; the change applies to the current conversation going forward; set a default in Cursor Settings > Models. "Faster models work well for quick edits and routine tasks; more capable models are better for complex reasoning and multi-file refactoring." You can switch mid-conversation ([Prompting doc](https://cursor.com/docs/agent/prompting)). Max Mode is available only on legacy request-based plans and "extends a model's context window beyond the default limit"; on legacy plans it is billed at API rate plus 20% ([Cursor help: Max Mode](https://cursor.com/help/ai-features/max-mode)). Cloud Agents use "a curated selection of models" and let you select context window size for supported models ([Cloud Agents doc](https://cursor.com/docs/cloud-agent)).

**Histories and conversation management.** Chat tabs (`Cmd+T`), previous/next chat (`Cmd+[`/`Cmd+]`), close chat (`Cmd+W`), new chat (`Cmd+N`/`Cmd+R`), conversation search from the Agents Window (`Cmd+K` opens the command palette, then search across conversations; `Cmd+F` finds text in the open conversation) ([Agent overview](https://cursor.com/docs/agent/overview); [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). `/resume`, `/fork`, `/rewind`, `/summarize`, `/rename`, `/clear` exist in the CLI reference ([CLI slash commands](https://cursor.com/docs/cli/reference/slash-commands)); side chats are durable agent conversations ([Agent overview](https://cursor.com/docs/agent/overview)). Checkpoints restore files only; forking shortens the conversation ([Cursor help: Agent mode](https://cursor.com/help/ai-features/agent)). Pins follow the account; the desktop Agents Window groups them `Today` / `Last 7 days` / `Older`, while web grouping is unverified ([release notes, Week of Sep 28, 2026](https://cursor.com/docs/release-notes)).

### 3.7 Submission, IME, multiline and paste behavior

**Submission.** The primary semantics are: `Enter` queues while the agent is running; `Cmd+Enter`/`Ctrl+Enter` force-sends; `Enter` twice steers at the next tool call; `Tab` queues for after the turn ([Agent overview](https://cursor.com/docs/agent/overview)). New installs steer by default; existing installs keep prior behavior ([release notes 3.25](https://cursor.com/docs/release-notes)). This is a critical UX distinction for a hosted Studio: "send" is not one action but three (queue, steer, interrupt/force-send). **[INFERRED]**

**Multiline.** `Shift+Enter` is the documented/observed newline gesture in the chat input. Cursor 3.0 fixed "Shift+Enter line breaks weren't treated as multiline content, so the prompt input field could stay in an incorrect state" ([Cursor 3.0 changelog](https://cursor.com/changelog/3-0)). Community reports show `Shift+Enter` inserting a newline and, in some versions, moving the caret to the end or failing to insert at the caret ([forum 2025-09-30](https://forum.cursor.com/t/shift-enter-does-not-always-insert-new-line-at-cursor-location/135578); [forum 2025-06-12](https://forum.cursor.com/t/new-line-not-correctly-inserted-in-agent-mode-chat/103320)). The CLI has a separate `Shift+Enter`/`Option+Enter` newline setup: `/setup-terminal` detects the terminal and offers alternatives if `Shift+Enter` doesn't work ([CLI terminal setup](https://cursor.com/docs/cli/reference/terminal-setup)).

**IME.** No official Cursor doc found that specifies IME composition behavior in the chat/Agent input. Public forum reports show real problems:
- Browser agent: "When using Cursor Agent in a browser with Japanese IME... pressing the Enter key to confirm Japanese character conversion" caused unintended chat submission ([forum 2025-09-08](https://forum.cursor.com/t/japanese-ime-input-causes-unintended-chat-submission-when-pressing-enter-key-in-browser-version-agent/132905)).
- Chinese IME in the Windows chat composer intermittently stutters; composition/preedit updates fall behind keystrokes ([forum 2026-10-13](https://forum.cursor.com/t/chinese-ime-stutters-while-typing-in-the-chat-composer-windows/174514)).
- Japanese IME composition background intermittently turns black in every Cursor IDE text input, including the code editor and AI chat ([forum 2026-10-15](https://forum.cursor.com/t/japanese-ime-composition-background-turns-black-in-every-cursor-ide-text-input/174895)).

**Paste.** `Cmd+V` with code/log in the clipboard adds it as context; `Cmd+Shift+V` adds it to the input box ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). Pasting an image/screenshot attaches it ([Prompting doc](https://cursor.com/docs/agent/prompting)). Text between single backticks in the prompt renders as inline code and is sent with backticks; Backspace right after conversion restores plain backticks ([release notes 3.25/3.26](https://cursor.com/docs/release-notes)). Community reports show cursor/paste insertion-point desync in the agent chat ([forum 2026-01-27](https://forum.cursor.com/t/agent-chat-text-input-cursor-insertion-point-desync/150122)). A locked prompt refuses dropped/pasted files ([release notes 3.26](https://cursor.com/docs/release-notes)).

### Input labels and validation/error states found

| Label / state | Surface | Behavior / validation | Source |
|---|---|---|---|
| **Send** | Chat/Agent | Replaced by **Stop** while agent runs; waveform replaces Send when idle and input empty | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Stop** | Chat/Agent | Stops agent mid-task; replaces Send while running | [Cursor help: Agent mode](https://cursor.com/help/ai-features/agent) |
| **Queue** | Chat/Agent | `Enter` while running; drag to reorder; processes sequentially | [Agent overview](https://cursor.com/docs/agent/overview) |
| **Send now (force send)** | Chat/Agent | `Cmd+Enter`/`Ctrl+Enter` while typing; bypasses the queue and sends immediately | [Agent overview](https://cursor.com/docs/agent/overview) |
| **Steer** | Chat/Agent | `Enter` twice (or **Send now**); delivered at the agent's next tool call instead of interrupting; `Tab` queues instead | [Agent overview](https://cursor.com/docs/agent/overview) |
| **Start voice chat** | Chat/Agent | Waveform button only when agent idle and input empty; hidden for subagents/Project workers/archived chats/Claude Code chats/other teammate chats; IDE also hides for streaming cloud agents or dictation off | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Microphone / Start voice input / Voice Input / Record voice** | Chat/Agent/iOS | Dictation; `Cmd+Shift+Space` in IDE; hold `Ctrl+M` in Agents Window; iOS says "Record voice" | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Microphone blocked. Allow it in your system or browser settings.** | Voice chat | Error when mic permission denied | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Realtime voice requires sign-in** | Voice chat | Error when signed out/API key; button still shows | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Daily limit** | Voice chat | Call limit; tells when you can call again | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Silence hangup** | Voice chat | ~10 minutes of silence while no work in progress | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| **Annotate / Try Annotate / Design Mode** | Agents Window browser | Design Mode toggle; `Cmd+Shift+D`; `Try Annotate` preview until first use; after >5 notes only icon | [Design Mode doc](https://cursor.com/docs/agent/design-mode) |
| **Batch edits / Send note / Send / Clear All** | Design Mode | Batch off: `Return` sends note immediately ("Send note"); batch on: notes collect as numbered pins, `Send` with count sends all; text edits always wait for Send; `Clear All` removes pending | [Design Mode doc](https://cursor.com/docs/agent/design-mode) |
| **Dictate** | Design Mode note | Speak a note; button appears once note has text/CSS edits | [Design Mode doc](https://cursor.com/docs/agent/design-mode) |
| **Edit CSS properties** | Design Mode | Opens CSS inspector for selected element | [Design Mode doc](https://cursor.com/docs/agent/design-mode) |
| **Build** | Plan Mode | Approve/build the plan | [Plan Mode doc](https://cursor.com/docs/agent/plan-mode) |
| **Save to workspace** | Plan Mode | Move plan from home dir to workspace | [Plan Mode doc](https://cursor.com/docs/agent/plan-mode) |
| **Restore Checkpoint** | Chat/Agent | Hover previous message; reverts files only, not messages | [Agent overview](https://cursor.com/docs/agent/overview); [Cursor help: Agent mode](https://cursor.com/help/ai-features/agent) |
| **Accept all / Reject all** | Chat suggested changes | `Cmd+Return` accept; `Cmd+Shift+Backspace` reject | [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts) |
| **Always Run** | Terminal approval | Adds command to `permissions.json` `terminalAllowlist`; hidden when admin controls allowlist | [release notes 3.26](https://cursor.com/docs/release-notes) |
| **Allow** | Read approval card | Read outside workspace; card shows full path and why | [Run Modes doc](https://cursor.com/docs/agent/security/run-modes) |
| **Add to allowlist** | Settings | Hidden when `permissions.json` or admin controls the allowlist; editor becomes read-only | [permissions.json reference](https://cursor.com/docs/reference/permissions) |
| **Switch Model** | Usage limit notice | Offered next to Buy Credits when a model's weekly limit is hit | [release notes 3.26](https://cursor.com/docs/release-notes) |
| **Connect GitHub** | Cloud Agents | Shown when repo provider isn't connected; names repo needed | [release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes) |
| **Reconnect / Try again** | Remote workspace | Persistent "Unable to reconnect to the remote workspace" notification; manual reconnect can fail with Try again | [release notes 3.19](https://cursor.com/docs/release-notes) |
| **Too large for a tour / View files** | Origin PR | Tour too large; button returns to diff | [Origin code tours](https://cursor.com/docs/origin/code-tours) |
| **Couldn't generate the tour / Retry** | Origin PR | Tour failure; retry; check paid plan | [Origin code tours](https://cursor.com/docs/origin/code-tours) |
| **Updating… / See latest** | Origin PR | New tour being written; earlier version can move to latest | [Origin code tours](https://cursor.com/docs/origin/code-tours) |
| **Not in this version** | Origin PR | Comment on code not in the viewed version | [Origin code tours](https://cursor.com/docs/origin/code-tours) |
| **Merge disabled / blocker** | Origin PR | Rule blocks merge; blocker names branch/rulesets/fix | [release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes) |
| **Publish** | Canvas | Publish/refresh a read-only share link | [Canvas doc](https://cursor.com/docs/agent/tools/canvas) |
| **Open IDE / Open Agents Window** | Command palette | Switch surfaces | [Agents Window doc](https://cursor.com/docs/agent/agents-window) |

### Inferences
- The chat input is the highest-value surface to replicate, and its hardest parts are not visual: submission semantics (queue/steer/force-send), attachment validation, context budgeting, and IME-safe composition handling. **[INFERRED]**
- `Shift+Enter` newline plus `Enter` queue is the Cursor-specific convention; a hosted Studio that uses `Enter` to send will feel wrong to Cursor users. **[INFERRED]**
- IME-safe `Enter` handling is a hard requirement for a CJK-capable hosted Studio: ignore `Enter` while `isComposing` / keyCode 229, and do not submit on composition confirmation. The forum reports are the evidence that this is easy to get wrong. **[INFERRED]**
- The context ring categories are a good blueprint for a hosted "context usage" UI: system prompt, tools, rules, skills, MCP, subagents, summary, conversation. **[INFERRED]**

### Gaps
- No official Cursor doc specifies IME behavior; all IME evidence is community reports. **[UNVERIFIED]**
- No official doc specifies the web editor's prompt keyboard shortcuts; the keyboard reference is IDE-centric. **[UNVERIFIED]**
- The exact behavior of `Cmd+K` in the web app (palette vs browser search vs nothing) is not documented. **[UNVERIFIED]**
- Whether `Tab` queues a message in the web/Agents Window exactly as in the IDE help is only partially documented; the IDE reference says `Tab` cycles messages, while the Agent overview says `Tab` queues after the turn in the web/Agents Window rollout. Scope by surface. **[UNVERIFIED]**
- No official doc lists every user-defined/plugin command; that surface is dynamic. **[INFERRED]**

---

## 4. Menus, keyboard shortcuts, command palette and settings

### Takeaway
The desktop IDE inherits VS Code's standard menu families and keybinding system, with Cursor overrides for AI (`Cmd+I`, `Cmd+L/K`, `Cmd+E`, `Cmd+.`, `Cmd+/`, `Cmd+Shift+J`, `Cmd+Shift+Space`) and a Cursor-specific shortcut-editor chord (`Cmd+R` then `Cmd+S`). The Agents Window has a reduced native menu set and its own `Cmd+K` command palette. Cursor Web has no documented native menu bar or command palette. The command surface is dynamic: VS Code commands, Cursor commands, extension-contributed commands, keybindings, settings, MCP tools, skills and plugins all vary by context and installation.

### 4.1 Documented keyboard shortcuts (exact)

From the [Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts) (Mac shown; Windows/Linux equivalents in the page). All can be remapped in Keyboard Shortcuts settings.

**General**

| Shortcut | Action |
|---|---|
| `Cmd I` / `Ctrl I` | Toggle Sidepanel (unless bound to mode) |
| `Cmd L` or `Cmd K` / `Ctrl L` or `Ctrl K` | Toggle Sidepanel (unless bound to mode) |
| `Cmd E` / `Ctrl E` | Toggle Agent layout |
| `Cmd .` / `Ctrl .` | Mode Menu |
| `Cmd /` / `Ctrl /` | Loop between AI models |
| `Cmd Shift J` / `Ctrl Shift J` | Cursor settings |
| `Cmd Shift Space` / `Ctrl Shift Space` | Toggle Voice Mode (dictation) |
| `Cmd ,` / `Ctrl ,` | General settings |
| `Cmd Shift P` / `Ctrl Shift P` | Command palette |

**Code Selection & Context**

| Shortcut | Action |
|---|---|
| `@` | @-mentions |
| `/` | Shortcut Commands |
| `Cmd Shift L` / `Ctrl Shift L` | Add selection to Chat |
| `Cmd L` or `Cmd K` | Add selection to new chat |
| `Cmd →` / `Ctrl →` | Accept next word of suggestion |
| `Cmd Return` / `Ctrl Enter` | Search codebase in chat |
| Select code, `Cmd C`, `Cmd V` | Add copied reference code as context |
| Select code, `Cmd C`, `Cmd Shift V` | Add copied code as text context |

**Tab**

| Shortcut | Action |
|---|---|
| `Tab` | Accept suggestion |
| `Cmd →` / `Ctrl →` | Accept next word |

**Chat** — see the full table in §3.3. Key overloads to flag: `Cmd+Return` is "force send" while typing and "accept all changes" with suggested changes; `Cmd+Shift+Backspace` is "cancel generation" and "reject all changes." `Cmd+K` is sidepanel/Agent in the IDE but command palette in the Agents Window; `Cmd+R` is new chat in Cursor but often reload in Electron/browser; `Cmd+W` closes the chat, not necessarily the window. Scope every binding by focused surface. **[OFFICIAL + INFERRED]**

**Other Cursor-specific shortcuts:**

| Shortcut | Action | Source |
|---|---|---|
| `Cmd+Shift+D` | Toggle Design Mode (Agents Window browser) | [Design Mode doc](https://cursor.com/docs/agent/design-mode) |
| `Cmd+Shift+Space` | Dictation (IDE) | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| Hold `Ctrl+M` | Dictate in Agents Window | [Voice chat doc](https://cursor.com/docs/agent/voice-chat) |
| `Cmd+P` | File search (Agents Window) | [Agents Window doc](https://cursor.com/docs/agent/agents-window) |
| `Cmd+Shift+F` | Search all files (Agents Window) | [Agents Window doc](https://cursor.com/docs/agent/agents-window) |
| `Cmd+K` | Command palette (Agents Window) | [release notes 3.25](https://cursor.com/docs/release-notes) |
| `Ctrl+Tab` | Recent agents switcher (Agents Window) | [release notes 3.25](https://cursor.com/docs/release-notes) |
| `Cmd+Shift+T` | Focus a tab restored by Back (Agents Window) | [release notes 3.25](https://cursor.com/docs/release-notes) |
| `Cmd+,` | Settings on agents pages | [release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes) |
| `T` | Go to file (Origin Code tab) | [Origin browse](https://cursor.com/docs/origin/browse) |
| `T` | Toggle Diff/Tour on an Origin PR | [Origin code tours](https://cursor.com/docs/origin/code-tours) |
| `?` | Keyboard shortcut sheet on Origin PR pages | [release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes) |
| `Cmd+I` | Add selected text to chat on an Origin code tour | [Origin code tours](https://cursor.com/docs/origin/code-tours) |

**VS Code baseline shortcuts inherited by the IDE** (selected; not exhaustive): `Ctrl+Shift+P` Command Palette, `Ctrl+P` Quick Open, `Ctrl+Shift+F` search across files, `Ctrl+F` find, ``Ctrl+` `` terminal, ``Ctrl+Shift+` `` new terminal, `Ctrl+Shift+G` Source Control, `Ctrl+Shift+X` Extensions, `Ctrl+,` Settings, `Ctrl+K Ctrl+S` Keyboard Shortcuts, `F5` start/continue debugging, `F9` breakpoint, `F10` step over, `F11` step into, `Shift+F11` step out, `Ctrl+Shift+M` Problems, `F12` Go to Definition, `Ctrl+Shift+O` Go to Symbol, `Ctrl+T` Go to Symbol in workspace, `F2` Rename Symbol ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings); [VS Code debugging](https://code.visualstudio.com/docs/debugtest/debugging); [VS Code code navigation](https://code.visualstudio.com/docs/editing/editingevolved); [VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)).

**Cursor's shortcut-editor override.** Cursor help says open keyboard shortcuts with `Cmd+R` then `Cmd+S` on Mac (`Ctrl+R` then `Ctrl+S` on Windows/Linux), search for the command, click the pencil, press the new combination, press Return ([Cursor help: keyboard shortcuts](https://cursor.com/help/customization/keyboard-shortcuts)). This differs from VS Code's documented `Ctrl+K Ctrl+S` ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)). Treat this as a Cursor-specific chord, not a typo: the rendered Cursor docs page also shows `Cmd R` then `Cmd S` ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). **[OFFICIAL]**

### 4.2 Menu families

Cursor does not publish an exhaustive menu-item reference. The following table uses VS Code's documented menu items/commands where available and labels the rest as inherited/inferred. The **menu bar families** themselves are corroborated by Cursor forum reports: the IDE menu bar normally shows "File, Edit, Selection, View, Go, Run, Terminal, Help" ([forum 2025-04-03](https://forum.cursor.com/t/menu-bar-is-hidden-on-0-48/74615)); a 2026 language-pack bug report also names "File, Edit, Selection, View, Go, Terminal, Help" and notes Cursor's own Settings/account and Agent/Chat window stay untranslated ([forum 2026-07-12](https://forum.cursor.com/t/bug-japanese-language-pack-only-translates-the-run-menu-in-cursor-3-11-13/165485)). The **Agents Window** exposes a reduced native menu set: a bug report describes "the native menu bar (File / Edit / View / Help)" ([forum 2026-04-07](https://forum.cursor.com/t/bug-agents-window-shows-only-menu-bar-main-content-area-is-completely-blank/156859)). **[OBSERVED]**

| Menu | Representative items (inherited VS Code baseline unless noted) | Evidence / status |
|---|---|---|
| **File** | New Text File, New File..., New Window, Open File..., Open Folder..., Open Recent, Save, Save As..., Save All, Auto Save, Preferences (Settings, Keyboard Shortcuts, Extensions, Color Theme), Close Editor, Close Folder, Close Window, Exit/Quit | VS Code docs explicitly show **File > Open Folder**, **File > Preferences > Settings**, **File > Auto Save**, **File > Exit** / **Code > Quit** ([VS Code SCM](https://code.visualstudio.com/docs/sourcecontrol/overview); [VS Code settings](https://code.visualstudio.com/docs/configure/settings); [VS Code codebasics](https://code.visualstudio.com/docs/editing/codebasics); [VS Code themes](https://code.visualstudio.com/docs/configure/themes)). Rest **[INFERRED]** standard VS Code. |
| **Edit** | Undo, Redo, Cut, Copy, Paste, Find, Replace, Find in Files, Replace in Files, Toggle Line Comment, Toggle Block Comment, Emmet..., Delete Line, Copy Line Up/Down, Move Line Up/Down, Format Document, Format Selection | Cursor 3.27 explicitly documents **Edit > Undo** and **Edit > Redo** in the Agents Window menu bar, applying to the focused prompt/editor/terminal/browser ([release notes 3.27](https://cursor.com/docs/release-notes)). VS Code documents find/replace and Format Document/Selection ([VS Code codebasics](https://code.visualstudio.com/docs/editing/codebasics)). Rest **[INFERRED]**. |
| **Selection** | Select All, Expand Selection, Shrink Selection, Copy Line Up/Down, Move Line Up/Down, Add Cursor Above/Below, Add Cursors to Line Ends, Add Selection to Next Find Match, Select All Occurrences, Column Selection Mode, Switch to Ctrl+Click/Alt+Click for Multi-Cursor | VS Code explicitly documents **Selection > Switch to Ctrl+Click for Multi-Cursor**, **Selection > Switch to Alt+Click for Multi-Cursor**, **Selection > Column Selection Mode**, multi-cursor shortcuts and expand/shrink selection ([VS Code codebasics](https://code.visualstudio.com/docs/editing/codebasics)). Rest **[INFERRED]**. |
| **View** | Command Palette, Open View..., Appearance (Full Screen, Zen Mode, Centered Layout, Menu Bar, Primary Side Bar, Secondary Side Bar, Panel Position, Activity Bar Position, Breadcrumbs, Minimap, Sticky Scroll), Editor Layout, Problems, Output, Debug Console, Terminal, Explorer, Search, Source Control, Run, Extensions, Chat/Agent | VS Code documents **View > Appearance** for side bars, activity bar, panel, breadcrumbs, minimap, sticky scroll, and **View > Terminal** ([VS Code custom layout](https://code.visualstudio.com/docs/configure/custom-layout); [VS Code user interface](https://code.visualstudio.com/docs/getstarted/userinterface); [VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)). Cursor adds Agent/chat views; exact placement **[INFERRED]**. |
| **Go** | Back, Forward, Go to File..., Go to Symbol..., Go to Definition, Go to Type Definition, Go to Implementation, Go to References, Go to Line/Column..., Go to Bracket, Next Problem, Previous Problem, Next Change, Previous Change | VS Code documents Go to Definition (`F12`), Go to Type Definition, Go to Implementation (`Ctrl+F12`), Go to Symbol (`Ctrl+Shift+O`), Go to Symbol in workspace (`Ctrl+T`), Peek References (`Shift+F12`), Problems (`Ctrl+Shift+M`) ([VS Code code navigation](https://code.visualstudio.com/docs/editing/editingevolved)). Rest **[INFERRED]**. |
| **Run** | Start Debugging, Run Without Debugging, Stop Debugging, Restart Debugging, Open Configurations, Add Configuration..., Step Over, Step Into, Step Out, Toggle Breakpoint, New Breakpoint, Run Task..., Run Build Task..., Run Active File, Run Selected Text | VS Code documents the **Run and Debug** view, `F5` start/continue, `F9` breakpoint, `F10`/`F11`/`Shift+F11`, `Ctrl+Shift+F5` restart, `Shift+F5` stop, and **Install Additional Debuggers** from the **Run** menu ([VS Code debugging](https://code.visualstudio.com/docs/debugtest/debugging)). Run Task/Run Selected Text from **[VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)**. Rest **[INFERRED]**. |
| **Terminal** | New Terminal, Split Terminal, Run Task..., Run Build Task..., Run Active File, Run Selected Text, Show Running Tasks..., Restart Running Task..., Configure Tasks..., Toggle Terminal | VS Code documents **Terminal > New Terminal**, **View > Terminal**, ``Ctrl+` ``, ``Ctrl+Shift+` ``, split terminal, run selected text, tasks ([VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)). Rest **[INFERRED]**. |
| **Help** | Welcome, Documentation, Release Notes, Keyboard Shortcut Reference, Keyboard Shortcuts, Tips and Tricks, Join Us, Report Issue, Check for Updates, About | VS Code documents **Help > Welcome**, **Help > Editor Playground**, **Help > Keyboard Shortcut Reference** ([VS Code tips](https://code.visualstudio.com/docs/editing/getting-started/tips-and-tricks); [VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)). Rest **[INFERRED]**. |

**Cursor-specific menu additions found:**
- **macOS menu bar icon / status item**: "On macOS, the menu bar menu has a new **Show Count** option. Turn it off to keep the menu bar icon without the unread agent number beside it. It's on by default and persists across restarts" ([release notes 3.27](https://cursor.com/docs/release-notes)). This is a Cursor-specific menu-bar addition outside the IDE window.
- **Edit > Undo / Redo focus-aware behavior**: in the Agents Window, Undo/Redo "apply to the prompt, editor, terminal, or browser that had focus before you opened the menu. They undo window actions like archiving an agent only when nothing editable has focus" ([release notes 3.27](https://cursor.com/docs/release-notes)). This is a Cursor-specific semantic change to standard Edit menu items.
- **Reduced Agents Window menu**: observed native menu `File / Edit / View / Help` ([forum 2026-04-07](https://forum.cursor.com/t/bug-agents-window-shows-only-menu-bar-main-content-area-is-completely-blank/156859)); no Selection/Go/Run/Terminal in that report. **[OBSERVED]**
- **No documented Cursor top-level menu** in the IDE menu bar. Cursor's own settings/commands are reachable through `Cmd+Shift+J`, `Cmd+,`, and the command palette; no official doc enumerates a `Cursor` menu alongside File/Edit/etc. The macOS application menu (Cursor > About/Settings/Services/Hide/Quit) is standard macOS behavior, not a Cursor-specific editor menu. **[INFERRED + GAP]**

**Menu bar visibility.** VS Code documents `window.menuBarVisibility` with `classic`, `visible`, `toggle`, `compact`, `hidden`; `toggle` hides the menu and a single Alt press reveals it; `compact` moves the menu into the Side Bar ([VS Code custom layout](https://code.visualstudio.com/docs/configure/custom-layout)). Cursor forum reports use `View > Appearance > Show Menu Bar` or command palette `Toggle Menu Bar` to restore it ([forum 2026-02-26](https://forum.cursor.com/t/there-is-no-file-edit-selection-option/153031)). **[OBSERVED]**

### 4.3 Command palette: documented entries, dynamic behavior

**IDE palette.** `Cmd+Shift+P` / `Ctrl+Shift+P` opens the command palette ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). VS Code defines it as "Access all available commands based on your current context" ([VS Code tips](https://code.visualstudio.com/docs/editing/getting-started/tips-and-tricks)). The palette is therefore **not a finite list**: it includes built-in commands, Cursor commands, extension-contributed commands, and context-specific commands. VS Code's extension docs explicitly show an extension contributing `TODO-Highlight: List highlighted annotations` and `TODO-Highlight: Toggle highlight` to the palette ([VS Code extension marketplace](https://code.visualstudio.com/docs/editor/extension-marketplace)). Cursor uses Open VSX and an audited marketplace proxy, so the set of extension-contributed commands depends on what is installed and available ([Cursor help: extensions](https://cursor.com/help/customization/extensions)). **[OFFICIAL + INFERRED]**

**Cursor-specific palette entries documented across pages:**
- `Open Agents Window` — `Cmd+Shift+P → Open Agents Window` ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).
- `Open IDE` — `Cmd+Shift+P → Open IDE` ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).
- `Agents Window` — Cursor 3 blog says `Cmd+Shift+P -> Agents Window` ([Cursor 3 blog](https://cursor.com/blog/cursor-3)).
- `Toggle Design Mode` — Design Mode doc says the command palette lists the same action as **Toggle Design Mode** ([Design Mode doc](https://cursor.com/docs/agent/design-mode)).
- `Open Canvas` — listed under **View** in the command palette ([Canvas doc](https://cursor.com/docs/agent/tools/canvas)).
- `Preferences: Color Theme` — theme switching ([Cursor help: themes](https://cursor.com/help/customization/themes)).
- `Terminal: Select Default Profile` — choose agent terminal profile ([Agent overview](https://cursor.com/docs/agent/overview)).
- `Keyboard Shortcuts` — search for the Keyboard Shortcuts editor ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).
- `Toggle Menu Bar` — restore the IDE menu bar ([forum 2026-02-26](https://forum.cursor.com/t/there-is-no-file-edit-selection-option/153031)). **[OBSERVED]**
- Agent Review from the Source Control tab; `/agent-review` from the agent window ([Agent Review doc](https://cursor.com/docs/agent/agent-review)).
- `Preferences: Open User Settings` / `Preferences: Open Workspace Settings` — inherited VS Code commands ([VS Code settings](https://code.visualstudio.com/docs/configure/settings)).

**Agents Window palette.** `Cmd+K` opens the command palette; "now offers Select Model, Archive, and New Side Chat. Paste a pull request URL or type a pull request number to jump straight to that pull request. Rows fit on one line, shortcuts shown next to commands match what runs them... `Ctrl+Tab` now opens the recent agents switcher, which uses the same look as the palette" ([release notes 3.25](https://cursor.com/docs/release-notes)). This is a **different palette** from the IDE's `Cmd+Shift+P`; the shortcut collision is a platform-specific trap. **[OFFICIAL]**

**Web palette.** No Cursor doc found that documents a command palette on `cursor.com/agents`. Release notes show browser-level shortcut conflicts being resolved in favor of the terminal: "Keys like `Ctrl+K` and `Ctrl+F` in the agent terminal now go to the shell instead of triggering page shortcuts" ([release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes)). Treat any web palette/shortcut behavior as unverified. **[UNVERIFIED]**

**Dynamic command surfaces to acknowledge in any inventory:**
- VS Code built-in commands (context-sensitive).
- Cursor-specific commands (Open IDE, Open Agents Window, Toggle Design Mode, Open Canvas, Agent Review, Cursor Settings).
- Extension-contributed commands (Open VSX; depends on install).
- Language/debugger commands contributed by extensions.
- MCP tools (dynamic per server; cloud agents have an MCP dropdown at `cursor.com/agents`).
- Skills/commands/plugins (user/team/workspace; `/` menu).
- Keybindings contributed by extensions and user `keybindings.json`.
- Settings contributed by extensions and Cursor.
- Menus/context menus contributed by extensions (VS Code contribution points). **[INFERRED from VS Code extensibility]**.

### 4.4 Validation/error states in keyboard and menu handling

- **Keybinding conflicts**: VS Code's Keyboard Shortcuts editor can show "Show Same Keybindings" for conflicts; `Developer: Toggle Keyboard Shortcuts Troubleshooting` logs dispatched shortcuts ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)). Cursor inherits this editor and says all Cursor keybindings, including Cursor-specific features, can be remapped ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)).
- **`when` clauses**: VS Code bindings are context-sensitive (`editorTextFocus`, `terminalFocus`, `inDebugMode`, etc.); the same chord can do different things by focus ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)). Cursor's `Cmd+K`/`Cmd+W`/`Cmd+R` overloads are concrete examples. **[OFFICIAL + INFERRED]**
- **Menu bar hidden**: `window.menuBarVisibility` can be `hidden`/`toggle`/`compact`; users can restore via `View > Appearance > Show Menu Bar` or `Toggle Menu Bar` ([VS Code custom layout](https://code.visualstudio.com/docs/configure/custom-layout); [forum 2026-02-26](https://forum.cursor.com/t/there-is-no-file-edit-selection-option/153031)). **[OBSERVED]**
- **Platform differences**: macOS uses Cmd; Windows/Linux Ctrl/Alt; some chords differ (`Cmd+Opt+/` vs `Ctrl+Alt+/`); keyboard layout rendering can change displayed chords ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)).
- **IME + Enter**: the browser agent's Japanese IME report shows Enter can submit instead of confirming composition; a hosted implementation must guard composition events ([forum 2025-09-08](https://forum.cursor.com/t/japanese-ime-input-causes-unintended-chat-submission-when-pressing-enter-key-in-browser-version-agent/132905)). **[OBSERVED]**
- **Terminal focus routing**: in the web agent terminal, `Ctrl+K`/`Ctrl+F` must go to the shell, not page shortcuts ([release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes)). In the IDE terminal, `Ctrl+F` opens terminal find unless removed from "commands to skip shell" ([VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)). **[OFFICIAL]**
- **`Cmd+Return` overload**: force-send vs accept-all-changes; the UI must resolve by whether suggested changes are present. **[OFFICIAL]**
- **`Cmd+Shift+Backspace` overload**: cancel generation vs reject-all-changes. **[OFFICIAL]**
- **`Cmd+R`/`Cmd+W`**: new chat/close chat in Cursor docs; browser/Electron may interpret as reload/close window. Scope and prevent default where needed. **[INFERRED]**

### 4.5 Settings, themes, extensions and keyboard-shortcut surfaces

**Cursor Settings.** `Cmd+Shift+J` opens Cursor settings; `Cmd+,` opens General settings ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts)). Documented settings targets include:
- **Settings > Agents > Approvals & Execution** — Run Modes, sandboxing, read access ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Cursor Settings > Agents > Auto-Run** — browser approval modes and allow/block lists ([Browser doc](https://cursor.com/docs/agent/tools/browser)).
- **Cursor Settings > Agents > Agent Review** — review preferences; starting Cursor 3.11 moves to **Git & PRs > Pull Requests** ([Agent Review doc](https://cursor.com/docs/agent/agent-review)).
- **Cursor Settings > Models** — default model ([Prompting doc](https://cursor.com/docs/agent/prompting)).
- **Cursor Settings > Tab** — Tab settings ([Tab doc](https://cursor.com/docs/tab/overview)).
- **Voice / Agent Voice** — default voice ([Voice chat doc](https://cursor.com/docs/agent/voice-chat)).
- **Settings > Agents > Remote Control** — Remote Control toggle ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)).
- **Team settings > Security & automation** — Marketplace Install Cooldown, Require Extension Signature Verification ([Cursor help: extensions](https://cursor.com/help/customization/extensions)).
- **Web agents pages**: `Cmd+,` opens Settings ([release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes)).

Cursor does not publish one exhaustive settings index in the pages read; the above are documented anchors, not a complete list. Extension-contributed settings add more. **[INFERRED + GAP]**

**Themes.** "Theme controls and the Extensions view are part of the IDE. If you're in the Agents Window and don't see the steps below, open the command palette and run **Open IDE**, then continue." Switch via `Preferences: Color Theme`; light `Default Light Modern`, dark `Default Dark Modern` or `Cursor Dark`; `Auto Detect Color Scheme` in Settings; install more themes via Extensions (`Cmd+Shift+X`, search `theme`); font size/family/ligatures via Settings ([Cursor help: themes](https://cursor.com/help/customization/themes)). VS Code's equivalent is **File > Preferences > Theme > Color Theme** (`Ctrl+K Ctrl+T`), with `workbench.colorTheme` persisted in user settings and Marketplace themes ([VS Code themes](https://code.visualstudio.com/docs/configure/themes)). This is strong evidence that **theme controls are IDE-scoped**, not Agents Window/web. **[OFFICIAL + INFERRED]**

**Extensions.** Cursor uses **Open VSX** for third-party extensions, with Cursor's marketplace proxy (`marketplace.cursorapi.com`) applying automated malware/supply-chain analysis before search/download. "Many popular VS Code extensions are available; some Microsoft Marketplace-only extensions are replaced by audited Anysphere builds." The same `publisher.extension` name can point to different publishers/code on Open VSX vs Microsoft Marketplace. Install via Extensions (`Cmd+Shift+X`), search, **Install**; disable globally or per workspace. Enterprise controls: allowed publishers/IDs, install cooldown, required signature verification, publisher verification ([Cursor help: extensions](https://cursor.com/help/customization/extensions)). VS Code's baseline is the Extensions view (`Ctrl+Shift+X`), Marketplace browse/install, extension-contributed commands ([VS Code extension marketplace](https://code.visualstudio.com/docs/editor/extension-marketplace)). **[OFFICIAL]**

**Keyboard shortcut editor.** Cursor: `Cmd+R` then `Cmd+S`; search command; pencil; press combination; Return to save ([Cursor help: keyboard shortcuts](https://cursor.com/help/customization/keyboard-shortcuts)). VS Code: `Ctrl+K Ctrl+S`; Keyboard Shortcuts editor lists all commands with/without shortcuts; `keybindings.json`; chords; `when` clauses; conflict detection ([VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings)). **[OFFICIAL]**

### Inferences
- Menus are a **presentation layer over commands**; the command palette is the real inventory. A hosted Studio should build a command registry first and render menus/keybindings from it, rather than hard-coding menu items. **[INFERRED]**
- Do not claim a finite command list. The defensible statement is: "built-in commands plus Cursor commands plus extension-contributed commands, filtered by context." **[INFERRED]**
- The IDE and Agents Window need separate shortcut maps: `Cmd+K` is the clearest collision; `Cmd+Shift+P` vs `Cmd+K` palette is the second. **[INFERRED]**
- Themes and VS Code extensions are IDE-scoped by documentation; a hosted web Studio should not promise them without a separate compatibility layer. **[INFERRED]**

### Gaps
- No official complete menu-item list for Cursor IDE or Agents Window; the table is VS Code baseline + observed labels + inference. **[UNVERIFIED]**
- No official full Cursor Settings page index; categories are pieced from docs. **[UNVERIFIED]**
- No official web command palette/keyboard map. **[UNVERIFIED]**
- Extension-contributed menus/keybindings/settings are dynamic and not enumerable. **[INFERRED]**
- Whether Cursor's IDE adds a top-level `Cursor` menu (beyond the macOS app menu) is not documented. **[UNVERIFIED]**

---

## 5. Terminal, source control, diff and PR surfaces

### Takeaway
The desktop IDE inherits VS Code's integrated terminal and Source Control view, with Cursor adding Agent terminal execution, Agent Review in the Source Control tab, and `/agent-review`. The Agents Window and Cursor Web add agent-centric **Changes** and **PR** tabs, forwarded ports, diff review, stage/commit/PR management, and stacked PRs. Origin adds hosted PR review with **Diff / Tour**, `T`, `?` shortcut sheet and **Ask Cursor**. Mobile can review/merge PRs but has no terminal.

### Cited Findings

**IDE integrated terminal (VS Code baseline).** Open with **Terminal > New Terminal** or **View > Terminal**; ``Ctrl+` `` toggles the panel, ``Ctrl+Shift+` `` creates a terminal; profiles dropdown; split panes (`Ctrl+Shift+5`, Alt+click); terminal tabs; find `Ctrl+F`; links with Ctrl/Cmd+click; copy/paste platform standards; multi-line paste warning; `Terminal: Run Selected Text in Active Terminal`; tasks via `.vscode/tasks.json` ([VS Code terminal](https://code.visualstudio.com/docs/terminal/basics)). Cursor inherits this in the IDE. **[OFFICIAL + INFERRED]**

**Cursor agent terminal.** Agent "runs shell commands directly in your terminal." Run Modes control when commands run, when Cursor asks, and when commands enter the sandbox. The sandbox blocks unauthorized file access and network activity. Heavy shell themes (Powerlevel9k/Powerlevel10k) can interfere with inline output; use `CURSOR_AGENT` in shell config to detect Cursor and skip fancy prompts ([Agent terminal doc](https://cursor.com/docs/agent/tools/terminal)). Agent uses the first terminal profile by default; set via command palette `Terminal: Select Default Profile` ([Agent overview](https://cursor.com/docs/agent/overview)).

**Agents Window terminal/Changes/ports.** Release notes: live workspace for cloud agents includes "the Changes panel, file tree, terminal, and forwarded ports" ([release notes 3.26](https://cursor.com/docs/release-notes)); forwarded ports show a red dot and error (e.g., timed-out connection) until a connection succeeds; opening a second cloud workspace no longer breaks the first; turning off Auto-Forward Ports closes auto-forwarded ports ([release notes 3.25](https://cursor.com/docs/release-notes)); connections that get no answer close after 20 seconds instead of hanging ([release notes 3.25](https://cursor.com/docs/release-notes)).

**Web terminal.** Release notes confirm an agent terminal on the web and keyboard routing: "Keys like Ctrl+K and Ctrl+F in the agent terminal now go to the shell instead of triggering page shortcuts" ([release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes)). A forum bug report confirms the cloud-agent terminal loads on `cursor.com/agents` but can fail in the Agents Window ([forum 2026-05-19](https://forum.cursor.com/t/terminal-for-cloud-agents-not-loading-in-agents-window-but-is-loading-on-https-cursor-com-agents/160952)). **[OBSERVED]**

**Source control / diff (IDE).** VS Code's **Source Control** view (`Ctrl+Shift+G`) reviews changed files, stages changes and commits; the **Diff editor** compares versions and stages individual changes; the **Source Control Graph** shows commits/branches/incoming/outgoing; the **Timeline** view shows Git commits and local saves for one file; the **Status Bar** shows branch/sync actions ([VS Code source control](https://code.visualstudio.com/docs/sourcecontrol/overview)). Cursor adds **Agent Review**: open the **Source Control** tab and run Agent Review to compare all local changes against main; or `/agent-review`; or automatic after every commit. Review depth **Quick** vs **Deep** ([Agent Review doc](https://cursor.com/docs/agent/agent-review)). **[OFFICIAL]**

**Agents Window diffs/PR.** Cursor 3's "new diffs view allows you to edit and review changes faster with a simpler UI. When you're ready, you can stage, commit, and manage PRs" ([Cursor 3 blog](https://cursor.com/blog/cursor-3)). Release notes: PR tab remembers whether you were on **Changes / Description / Commits / Checks / Reviews** when switching agents or restarting; `Cmd+click` (`Ctrl+click` on Windows/Linux) on a diff line opens that line on GitHub or Origin; generated files start collapsed with a **Generated** badge; the Merge button stays disabled while an Origin rule blocks merge; merges that bypassed rules show in the timeline; PR chips in prompts/messages show current status (merged/closed) ([release notes 3.26](https://cursor.com/docs/release-notes)). Stacked PRs: the PRs tray groups stacked PRs on a rail; the merge button shows how many PRs it will land ([release notes 3.27](https://cursor.com/docs/release-notes)).

**Origin PR review and code tours.** On a PR's **Changes** tab, select **Tour** in the **Diff / Tour** control or press `T`; press `T` again to return to Diff. Tours group related changes, explain each group, support **Generate custom tour** (highlights, files, depth, instructions), **Ask Cursor** (`+` next to a section, or select text and **Add to chat** / `Cmd+I`), **What next?** suggestions, **Is this tour helpful?**, **Too large for a tour / View files**, **Couldn't generate the tour / Retry**, **Updating… / See latest**, **Not in this version** ([Origin code tours](https://cursor.com/docs/origin/code-tours)). Release notes add a `?` keyboard shortcut sheet on PR pages, including keys for switching tabs and moving through the diff ([release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes)). **[OFFICIAL]**

**Web PR review/merge.** The 2025 web-agent announcement says team members with repository access can "review agent diffs, pull requests, and even create pull requests directly from the web interface" ([Cursor blog: agents on web](https://cursor.com/blog/agent-web)). Release notes mention the PR tab, review agents sidebar on pull request pages, merge actions, stacked PR restack, and CI/checks ([release notes 3.26/3.25](https://cursor.com/docs/release-notes)). **[OFFICIAL]**

**Mobile PR review.** iOS can "read full diffs, commits, deployments, approvals, comments, and checks. Add or change reviewers, then ask an agent to resolve review comments or fix a failing check. You can also merge with squash, mark ready, update the branch, toggle auto-merge, publish, or close" ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)). **[OFFICIAL]**

### Inferences
- Cursor's diff/PR surface is not a generic Git GUI; it is an **agent-output review surface** with PR creation/merge, artifacts, checks, and code tours. A hosted Studio that only shows a unified diff will miss the workflow. **[INFERRED]**
- The web/Agents Window terminal is a remote PTY (cloud VM or self-hosted worker), not a local shell. Browser keyboard routing must prioritize the terminal when focused. **[INFERRED]**
- Origin's code tours are a separate review product from the IDE's diff editor; do not conflate them in a feature matrix. **[INFERRED]**

### Gaps
- No official doc details the web terminal's emulator, shell profile, resize behavior, or session persistence; the forum report calls it "tmux/WebSocket" but that is unofficial. **[UNVERIFIED]**
- No official doc lists all PR actions available on web vs Agents Window vs mobile. **[UNVERIFIED]**
- No official doc describes the Agents Window's full Source Control UI beyond Changes/PR tabs and Agent Review. **[UNVERIFIED]**

---

## 6. Remote environments, language servers, debugging, notebooks and filesystem permissions

### Takeaway
The desktop IDE can host the full VS Code Remote Development stack (SSH, Dev Containers, WSL, Tunnels), language servers, debugger and Jupyter notebooks. The Agents Window supports local, worktree, cloud and remote SSH workspaces, plus LSP go-to-definition; its **Debug Mode** is an agent instrumentation workflow, not the VS Code debugger. Cursor Web is a cloud-VM workspace; it has no documented extension host, debugger, notebook support, or local filesystem access. A static browser cannot replicate OS-level file permissions or native remote development without a backend.

### 6.1 Remote SSH, containers, WSL and tunnels

**Cursor's documented remote surfaces.**
- Cursor 3 Agents Window runs agents "locally, in worktrees, in the cloud, and on remote SSH" ([Cursor 3 blog](https://cursor.com/blog/cursor-3)); the Agents Window doc says "local, cloud, remote SSH, and more" ([Agents Window doc](https://cursor.com/docs/agent/agents-window)).
- Release notes 3.23: "Drop local files into prompts for SSH, WSL, and dev container agents. When you drop, paste, or attach a file or folder from your computer into the prompt of an agent running on a remote machine, the Agents Window copies it to that machine and mentions the copy" ([release notes 3.23](https://cursor.com/docs/release-notes)).
- Release notes 3.24: "Connecting to an SSH, container, or tunnel remote no longer hangs when the connection drops during the handshake"; "Open in Browser works for HTML files on remote and cloud machines" ([release notes 3.24](https://cursor.com/docs/release-notes)).
- Release notes 3.20: "Edit Workspace for SSH and WSL multi-root workspaces... Edit Workspace is still not offered for dev containers" ([release notes 3.20](https://cursor.com/docs/release-notes)).
- Release notes 3.19: reconnect notification with **Reconnect** / **Try again** for remote workspaces or cloud agents ([release notes 3.19](https://cursor.com/docs/release-notes)).
- Remote Control: "Use a local or Remote SSH workspace. Both workspace types are supported, and the project doesn't need a Git remote"; agent loop runs in Cursor's cloud, tool calls run on your computer; only available in the Agents Window; `/remote-control`; requires Cursor 3.9.8+ ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)).

**VS Code baseline.** The Remote Development extension pack includes **Remote - SSH**, **Dev Containers**, **WSL**, and **Remote - Tunnels**; extensions install VS Code Server on the remote OS and run commands/extensions there, so "no source code needs to be on your local machine" ([VS Code Remote Development](https://code.visualstudio.com/docs/remote/remote-overview)). Cursor's IDE, being a VS Code fork with extension support, can use this stack; Cursor release notes confirm SSH/container/tunnel/WSL remote windows ([release notes 3.24/3.20](https://cursor.com/docs/release-notes)). **[OFFICIAL + INFERRED]**

**Remote-host dependency.** In the Agents Window, a remote SSH workspace still needs the remote host reachable; reconnect notifications appear when it isn't ([release notes 3.19](https://cursor.com/docs/release-notes)). In Cursor Web, cloud agents run in Cursor-managed VMs or self-hosted machines; the web app is not documented as a Remote SSH client. **[INFERRED/UNVERIFIED]**

### 6.2 Language servers and code intelligence

**Cursor 3 Agents Window.** Cursor says files can be viewed and "go to definition in the editor with full LSPs" ([Cursor 3 blog](https://cursor.com/blog/cursor-3)). The Agents Window doc says you can "view or edit files without leaving the Agents Window" and search files with `Cmd+P`/`Cmd+Shift+F` ([Agents Window doc](https://cursor.com/docs/agent/agents-window)); the LSP claim is from the Cursor 3 blog, not repeated in the doc.

**VS Code baseline.** Language services provide suggestions, parameter hints and navigation: `F12` Go to Definition, `Ctrl+F12` Go to Implementation, `Ctrl+Shift+O` Go to Symbol, `Ctrl+T` Go to Symbol in workspace, `Shift+F12` Go to References/Peek, `F2` Rename Symbol, `Ctrl+Shift+M` Problems, Code Actions/Quick Fixes ([VS Code code navigation](https://code.visualstudio.com/docs/editing/editingevolved)). Extensions provide the language servers; without the extension, the language features are absent. **[OFFICIAL]**

**Web.** No Cursor doc states whether `cursor.com/agents` runs LSPs for the editor. The web workspace may provide syntax highlighting and basic editing, but full LSP parity is unverified. **[UNVERIFIED]**

### 6.3 Debugging: two different things called "debug"

**Cursor Debug Mode (agent workflow).** Debug Mode "generates hypotheses, adds log statements, and uses runtime information to pinpoint the exact issue before making a targeted fix." It asks you to reproduce the bug; instrumentation sends data to "a local debug server running in a Cursor extension"; after the fix it removes instrumentation ([Debug Mode doc](https://cursor.com/docs/agent/debug-mode)). This is an **agent mode**, not a breakpoint/watch UI, and its "local debug server running in a Cursor extension" implies an IDE extension dependency. Whether Debug Mode works in the web/Agents Window without that extension is not documented. **[OFFICIAL + UNVERIFIED]**

**VS Code debugger (IDE).** VS Code includes a debugger for JavaScript/TypeScript/Node.js; other languages install debugger extensions. `F5` start/continue, `F9` breakpoint, `F10` step over, `F11` step into, `Shift+F11` step out, `Ctrl+Shift+F5` restart, `Shift+F5` stop; **Run and Debug** view (`Ctrl+Shift+D`), Variables, Watch, Call Stack, Debug Console (`Ctrl+Shift+Y`), breakpoints (conditional, triggered, inline, function, data, logpoints), multi-target debugging ([VS Code debugging](https://code.visualstudio.com/docs/debugtest/debugging)). Cursor's IDE inherits this; the **Run** menu includes **Install Additional Debuggers** ([VS Code debugging](https://code.visualstudio.com/docs/debugtest/debugging)). **[OFFICIAL + INFERRED]**

**Distinction for a hosted Studio:** "Debug mode" in Cursor chat ≠ "debugger" in the IDE. A hosted Studio can offer the former (agent instrumentation) without implementing the Debug Adapter Protocol; it cannot claim the latter without a debug adapter and runtime. **[INFERRED]**

### 6.4 Notebooks

**Cursor support.** Cursor 1.0 (June 4, 2025) added "Agent in Jupyter Notebooks": "Agent will now create and edit multiple cells directly inside of Jupyter" (initially only Sonnet models) ([Cursor 1.0 changelog](https://cursor.com/changelog/1-0)). Release notes 3.26 still mention "notebook edits keep fields the agent didn't touch" as a fix ([release notes 3.26](https://cursor.com/docs/release-notes)), confirming notebook editing remains a supported IDE path in 2026. Public forum reports show Jupyter rendering/Tab issues in 2026 ([forum 2026-04-06](https://forum.cursor.com/t/jupyter-notebook-rendering-takes-forever-eternity-asgard-time/156852); [forum 2026-01-12](https://forum.cursor.com/t/issues-using-cursor-tab-in-jupyter-notebooks/148577)). **[OFFICIAL + OBSERVED]**

**VS Code baseline.** Jupyter notebooks are supported natively through the Jupyter extension; create/open `.ipynb`, select a kernel with the kernel picker, run cells with `Ctrl+Enter` (run selected), `Shift+Enter` (run and insert below), `Alt+Enter` (run and insert below, focus stays); cell modes (unselected/command/edit), `A`/`B` add cells, `J`/`K` move, `Esc` command mode, `Enter` edit mode; Workspace Trust/Restricted Mode blocks execution and hides rich outputs in untrusted workspaces ([VS Code Jupyter notebooks](https://code.visualstudio.com/docs/datascience/jupyter-notebooks)). **[OFFICIAL]**

**Web/Agents Window.** No Cursor doc found that documents a notebook editor in Cursor Web or the Agents Window. The mobile app has no editor at all. Treat notebooks as **desktop IDE** unless separately verified. **[UNVERIFIED]**

### 6.5 Local filesystem and permissions

**Desktop IDE.** The agent can read/write workspace files; Run Modes and sandboxing constrain it:
- **Auto-review**: allowlisted calls run; other shell commands run in the sandbox when possible; calls outside the sandbox go to the Auto-review classifier; classifier runs on Cursor backend and can make read-only `ReadFile`, `Grep`, `Glob`, `ListDir` calls on your machine ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Allowlist**: only allowlisted actions; optional sandbox ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Run Everything**: every tool call runs automatically; no sandbox/classifier ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Read Access**: **System** (agent can read outside workspace without approval; default) vs **Workspace** (outside reads ask first; Read Allowlist for folders/globs/`~`); approval card shows full path and why; searches skip outside files; sandboxed commands see less of the machine ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Sandbox**: macOS Seatbelt via `sandbox-exec`; Linux Landlock + seccomp; workspace read/write; protected paths `.git/config`, `.git/hooks`, `.vscode`, `.cursorignore`, sensitive Cursor config; network blocked by default then opened by network mode/`sandbox.json`; `/tmp` writable; AppArmor setup for remote environments/CLI ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Other protections**: Browser Protection, File-Deletion Protection, External-File Protection ([Run Modes doc](https://cursor.com/docs/agent/security/run-modes)).
- **Workspace Trust / Restricted Mode**: VS Code blocks terminal opening and notebook execution in Restricted Mode ([VS Code terminal](https://code.visualstudio.com/docs/terminal/basics); [VS Code Jupyter notebooks](https://code.visualstudio.com/docs/datascience/jupyter-notebooks)). Cursor inherits this IDE behavior. **[INFERRED]**

**Cursor Web.** Cloud agents run in isolated VMs with cloned repos, dependencies, secrets and network access; the agent never asks you to approve an action because it runs in its own dedicated machine ([Cloud Agents doc](https://cursor.com/docs/cloud-agent); [Run Modes doc](https://cursor.com/docs/agent/security/run-modes)). Attachments are uploaded; the browser cannot read local files except through user-mediated file pickers/drag-drop. Remote Control can run tool calls on your computer while the agent loop stays in the cloud ([Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)). **[OFFICIAL]**

**Browser limits.** No Cursor doc mentions the File System Access API or a browser filesystem permission model. A static browser can only use user-mediated upload/drag-drop/paste and, if implemented, the File System Access API with explicit user grants; it cannot replicate OS-level TCC/ACL permissions. **[INFERRED/UNVERIFIED]**

### 6.6 What a static browser could replicate, and what it cannot

| Layer | Replicable in a browser? | Why / dependency |
|---|---|---|
| Prompt/chat UI, streaming transcript, mode/model pickers | **Yes** | WebSocket/SSE + client state; Cursor Web already does this |
| `@` context picker, attachments, slash commands | **Yes** | File metadata/search APIs; upload; command registry |
| File tree and read-only code view | **Yes** | Backend tree API + editor component (Monaco/CodeMirror) |
| Full editable code editor | **Yes, with caveats** | Monaco/CodeMirror; Cursor's web editor depth unverified |
| Diff/PR review, stage/commit/merge | **Yes** | Git host API + diff renderer; Cursor Web does this |
| Terminal | **Yes, with backend** | xterm.js + PTY over WebSocket; Cursor Web does this for cloud VMs |
| Command palette | **Yes** | Client command registry; dynamic commands need extension model |
| Themes | **Yes** | CSS variables/editor theme; but Cursor docs scope theme controls to the IDE |
| VS Code extension host / Open VSX | **No, not natively** | Requires extension host, Node/proxy, web-compatible extensions; Cursor Web has no documented extension support |
| LSP / go-to-definition | **Partial** | Needs language servers on backend or WASM; Cursor Web unverified |
| Debug Adapter Protocol / breakpoints | **No, not without backend** | Requires debug adapter + runtime; Cursor Web unverified |
| Jupyter notebooks / kernels | **No, not without backend** | Requires kernel process; Cursor Web unverified |
| Local filesystem / OS permissions | **No** | Browser sandbox; user-mediated uploads only |
| Remote SSH / Dev Containers / WSL | **No, not from a static page** | Requires local agent/server; Cursor Web not documented as remote SSH client |
| Cloud agents / VM execution / secrets | **No, backend required** | Cursor-managed or self-hosted workers; model routing |
| Voice chat | **Yes, with a realtime backend** | Cursor uses Grok Voice; transport is not documented (WebRTC is likely); mic permission required |
| Native menu bar / OS-level shortcuts | **No** | Browser chrome; can emulate in-page menus/shortcuts only |
| IME-safe composition | **Yes, but fragile** | Must handle `compositionstart/update/end`, keyCode 229; Cursor forum shows failures |

### Inferences
- A hosted UOR-R4 Studio can credibly replicate the **agent workspace** (prompt, context, diff, PR, terminal, file tree, editor) and a command palette, but not the **VS Code extension/LSP/debugger/notebook/local-FS** stack without a substantial backend or web-compatible language services. **[INFERRED]**
- Remote SSH/WSL/containers are a desktop capability because they need a local agent/server; a browser would need a hosted tunnel/worker model (like Cursor's cloud agents or self-hosted machines). **[INFERRED]**
- The "local filesystem permission" problem is not just a browser API issue; it is a trust/approval UX problem (read boundary, allowlists, approvals). Cursor's Run Modes are a useful blueprint. **[INFERRED]**

### Gaps
- No official doc describes Cursor Web's editor/LSP/debugger/notebook capabilities. **[UNVERIFIED]**
- No official doc confirms whether Debug Mode works outside the IDE extension. **[UNVERIFIED]**
- No official doc lists web keyboard shortcuts beyond the terminal routing note. **[UNVERIFIED]**
- No official doc describes browser filesystem permissions or File System Access API usage. **[UNVERIFIED]**

---

## 7. What is dynamic, platform-dependent, account-dependent or unverified

### Takeaway
The stable part of the inventory is the **shape**: surfaces, input controls, modes, context types, shortcuts, diff/PR workflow. The unstable part is the **enumeration**: command palette entries, menu items, settings, keybindings, models, skills/commands, MCP tools, extension-contributed UI, account entitlements and feature flags. Any claim of an exhaustive finite list would be false.

### Cited Findings

**Dynamic by extensions.** VS Code extensions contribute commands to the Command Palette (e.g., `TODO-Highlight: ...`) and can contribute menus/keybindings/settings/views via the extension API ([VS Code extension marketplace](https://code.visualstudio.com/docs/editor/extension-marketplace)). Cursor uses Open VSX and its own marketplace proxy; the available extension set and the exact `publisher.extension` code can differ from Microsoft Marketplace ([Cursor help: extensions](https://cursor.com/help/customization/extensions)). Therefore extension-contributed commands/menus are not finite and not identical across Cursor/VS Code. **[OFFICIAL + INFERRED]**

**Dynamic by skills/commands/plugins.** User commands are markdown files invoked with `/`; plugins bundle rules, skills, subagents, commands, MCP servers and hooks; the Customize page manages them at user/team/workspace scope ([Customize Cursor](https://cursor.com/docs/customize-cursor)). The `/` menu is therefore a runtime inventory, not a fixed list. **[OFFICIAL]**

**Dynamic by MCP.** Cloud agents have an MCP dropdown at `cursor.com/agents`; MCP servers can be HTTP or stdio, with OAuth; team admins configure shared servers ([Cloud Agents doc](https://cursor.com/docs/cloud-agent); [Cloud Agent capabilities](https://cursor.com/docs/cloud-agent/capabilities)). MCP tools are dynamic per server. **[OFFICIAL]**

**Dynamic by models/account.** The model picker lists models available to the account; cloud agents use a curated selection; Max Mode is legacy-plan-only; usage limits can offer **Switch Model**; team admins can block models ([Prompting doc](https://cursor.com/docs/agent/prompting); [Max Mode help](https://cursor.com/help/ai-features/max-mode); [release notes 3.26](https://cursor.com/docs/release-notes); [Run Modes doc](https://cursor.com/docs/agent/security/run-modes)). **[OFFICIAL]**

**Dynamic by account/plan/privacy.** Cloud Agents require an eligible paid plan; web voice chat requires a signed-in account (API-key sessions can't start); Cloud Agents need cloud data storage, so Privacy Mode (Legacy) blocks them; SSO/team membership/repo access gate viewing and follow-ups ([Cloud Agents doc](https://cursor.com/docs/cloud-agent); [Voice chat doc](https://cursor.com/docs/agent/voice-chat); [Cursor help: mobile app](https://cursor.com/help/ai-features/mobile-app); [Cursor for iOS reference](https://cursor.com/docs/cloud-agent/mobile)). **[OFFICIAL]**

**Dynamic by platform.** macOS uses `Cmd`; Windows/Linux use `Ctrl`/`Alt`; menu bar behavior differs (native macOS menu vs Windows/Linux menu bar); WSL is Windows-only; Remote Development differs by platform; `Cmd+Opt+/` vs `Ctrl+Alt+/`; keyboard layout changes displayed chords ([Cursor keyboard reference](https://cursor.com/docs/reference/keyboard-shortcuts); [VS Code keybindings](https://code.visualstudio.com/docs/getstarted/keybindings); [VS Code Remote Development](https://code.visualstudio.com/docs/remote/remote-overview)). **[OFFICIAL]**

**Dynamic by release channel/version.** Cursor release notes show rapid UI changes: 3.27 menu bar Show Count, Edit > Undo/Redo; 3.26 live workspace; 3.25 Cmd+K palette; 3.24 Open in Browser; 3.23 SSH/WSL/dev container file drop; 3.20 Edit Workspace; 3.19 reconnect. Cursor 3.0 removed cloud agents from the Editor. Cursor 3 is April 2, 2026. Treat any 2025-era Cursor UI description as historical. **[OFFICIAL]**

### Explicitly unverified / do not claim
- **Complete menu-item lists** for Cursor IDE or Agents Window. **[UNVERIFIED]**
- **Complete command palette list** for any Cursor surface. **[UNVERIFIED]**
- **Complete settings index** for Cursor. **[UNVERIFIED]**
- **Web editor depth**: Monaco vs embedded VS Code vs custom; LSP, themes, extensions, debugger, notebooks, multi-cursor, split view. **[UNVERIFIED]**
- **Web command palette / full keyboard map**. **[UNVERIFIED]**
- **Tab autocomplete in web/Agents Window**. **[UNVERIFIED]**
- **Inline edit** beyond "retired"; no current inline-edit UI documented. **[UNVERIFIED]**
- **IME behavior** in chat/Agent input; only community reports. **[UNVERIFIED]**
- **Exact `Shift+Enter` behavior** on web/Agents Window vs IDE; release note confirms multiline fix but not per-surface behavior. **[UNVERIFIED]**
- **`Cmd+K` in web**. **[UNVERIFIED]**
- **Whether `Cmd+Shift+P` works in Agents Window**. **[UNVERIFIED]**
- **Whether themes/extensions are available in web/Agents Window**; docs scope them to the IDE. **[INFERRED, not explicitly "no"]**
- **Whether Debug Mode works outside the IDE extension**. **[UNVERIFIED]**
- **Whether notebooks work in web/Agents Window**. **[UNVERIFIED]**
- **Whether web can open Remote SSH workspaces**. **[UNVERIFIED]**
- **Exact PR actions on web vs Agents Window vs mobile**. **[UNVERIFIED]**
- **Account-level feature flags, regional availability, pricing/usage**. **[UNVERIFIED]**

### Source conflicts / ambiguities to preserve
- Cursor help says "Cursor uses the same default shortcuts as VS Code" ([help](https://cursor.com/help/customization/keyboard-shortcuts)), but Cursor overrides `Cmd+K`/`Cmd+L` (sidepanel/Agent), `Cmd+E` (Agent layout), `Cmd+.` (Mode Menu), `Cmd+/` (model cycle), `Cmd+Shift+J` (Cursor settings), `Cmd+Shift+Space` (dictation), and uses `Cmd+R` then `Cmd+S` for keyboard shortcuts. The "same defaults" claim is a baseline, not an exact map. **[OFFICIAL + INFERRED]**
- Cursor docs say `Cmd+K` "Toggle Sidepanel" in the IDE, but the Agents Window release notes say `Cmd+K` is the command palette. This is surface-specific, not a contradiction. **[OFFICIAL]**
- Cursor 3.0 says cloud agents were removed from the Editor, but the IDE still has Agent Tabs and local Agent chats; "cloud agents" specifically moved. **[OFFICIAL]**
- The web-agent 2025 blog describes a Kanban-style agent dashboard, while 2026 docs describe editor/terminal/file browser on `cursor.com/agents`. The web product evolved; do not use the 2025 blog as the current UI description. **[OFFICIAL + INFERRED]**

---

## 8. Coverage, source register and gaps for the lead

### Takeaway
Coverage is strong on official Cursor docs for Agent/Agents Window/web/mobile/Origin, exact shortcuts, modes, context, attachments, run modes, worktrees and extensions, and on VS Code docs for inherited workbench/keybindings/terminal/SCM/remote/debug/notebooks/settings/themes. The weakest areas are the logged-in web editor, exact menu enumerations, IME/multiline per-surface behavior, and the boundary between Cursor plugins and VS Code extensions in the Agents Window.

### Coverage summary
- **Cursor official docs/help/changelog/blog:** 30+ pages read, including `agent/*`, `cloud-agent/*`, `origin/*`, `reference/*`, `configuration/*`, `cli/*`, `customize-cursor`, `release-notes`, `changelog/3-0`, `blog/cursor-3`, `blog/agent-web`, `llms.txt`, `sitemap.xml`.
- **VS Code official docs:** 12+ pages read for the inherited baseline (UI, keybindings, terminal, SCM, remote, notebooks, debugging, layout, settings, extensions, editing, navigation, themes, tips).
- **Public forum reports:** 8 used only for observed UI/error behavior (menu bar, IME, terminal, Shift+Enter, paste desync, Jupyter).
- **Public logged-out UI:** `cursor.com/agents` landing/prompt and notices; no authenticated workspace.
- **Local source log:** `local/kimi/cursor-hosted-research/editor-inputs/source-log.md` (URLs, claim/date table; not for commit).
- **Not accessed:** logged-in `cursor.com/agents`, desktop app, Agents Window live, extension marketplace runtime, full release-note history, pricing/account dashboard, private/team-specific features.

### Source register (primary)
| Source | Type | What it anchors |
|---|---|---|
| https://cursor.com/docs/agent/agents-window | Cursor official | Agents Window, Open IDE, Cmd+P/Cmd+Shift+F, GA date |
| https://cursor.com/docs/agent/overview | Cursor official | Agent, tools, queue/steer, side chats, conversation search, /goal |
| https://cursor.com/docs/agent/prompting | Cursor official | @ mentions, / skills, Custom Modes, images, voice, context ring, models |
| https://cursor.com/docs/agent/plan-mode | Cursor official | Plan mode, Shift+Tab, plan save |
| https://cursor.com/docs/agent/design-mode | Cursor official | Annotate, Cmd+Shift+D, gestures, batch edits, shortcuts |
| https://cursor.com/docs/agent/voice-chat | Cursor official | Voice chat, dictation, errors, shortcuts |
| https://cursor.com/docs/agent/debug-mode | Cursor official | Debug Mode instrumentation |
| https://cursor.com/docs/agent/agent-review | Cursor official | /agent-review, Source Control tab, Quick/Deep |
| https://cursor.com/docs/agent/projects | Cursor official | Projects, coordinator, Listening pill |
| https://cursor.com/docs/agent/security/run-modes | Cursor official | Run Modes, sandbox, read access, approvals |
| https://cursor.com/docs/reference/permissions | Cursor official | permissions.json, precedence, read-only UI |
| https://cursor.com/docs/configuration/worktrees | Cursor official | Worktrees, /worktree, /best-of-n |
| https://cursor.com/docs/reference/keyboard-shortcuts | Cursor official | Exact shortcut tables |
| https://cursor.com/docs/tab/overview | Cursor official | Tab behavior/settings |
| https://cursor.com/docs/agent/tools/terminal | Cursor official | Agent terminal, sandbox, CURSOR_AGENT |
| https://cursor.com/docs/agent/tools/browser | Cursor official | Browser tools, approvals, allow/block |
| https://cursor.com/docs/agent/tools/canvas | Cursor official | Canvas/Open Canvas |
| https://cursor.com/docs/cloud-agent | Cursor official | Cloud Agents surfaces, VM, artifacts, MCP |
| https://cursor.com/docs/cloud-agent/capabilities | Cursor official | Computer use, artifacts, remote desktop, subscriptions |
| https://cursor.com/docs/cloud-agent/mobile | Cursor official | Web vs mobile, Remote Control |
| https://cursor.com/docs/origin/browse | Cursor official | Origin Code tab, T, search, history |
| https://cursor.com/docs/origin/code-tours | Cursor official | Diff/Tour, T, Ask Cursor, errors |
| https://cursor.com/docs/customize-cursor | Cursor official | Plugins/rules/skills/subagents/hooks/commands |
| https://cursor.com/docs/cli/reference/slash-commands | Cursor official | Built-in slash command table |
| https://cursor.com/docs/release-notes | Cursor official | 3.19–3.27 UI changes, web terminal, menu bar, live workspace |
| https://cursor.com/changelog/3-0 | Cursor official | Cursor 3, Agent Tabs, /worktree, cloud agents removed |
| https://cursor.com/blog/cursor-3 | Cursor official | New interface from scratch, LSP, diffs, plugins |
| https://cursor.com/blog/agent-web | Cursor official | Web agents, Kanban, diffs/PRs |
| https://cursor.com/help/customization/keyboard-shortcuts | Cursor official | Same defaults as VS Code + AI shortcuts |
| https://cursor.com/help/customization/themes | Cursor official | Theme controls IDE-scoped, exact commands |
| https://cursor.com/help/customization/extensions | Cursor official | Open VSX, proxy, enterprise controls |
| https://cursor.com/help/ai-features/agent | Cursor official | Cmd+K inline edit retired, modes table |
| https://cursor.com/help/ai-features/tab | Cursor official | Tab accept/reject/jump |
| https://cursor.com/help/ai-features/cloud-agents | Cursor official | Web attachment limits/formats |
| https://cursor.com/help/ai-features/mobile-app | Cursor official | Mobile has no editor/terminal/file browser; use web/desktop |
| https://code.visualstudio.com/docs/getstarted/userinterface | VS Code official | Six workbench areas |
| https://code.visualstudio.com/docs/getstarted/keybindings | VS Code official | Keyboard editor, chords, when clauses |
| https://code.visualstudio.com/docs/terminal/basics | VS Code official | Integrated terminal |
| https://code.visualstudio.com/docs/sourcecontrol/overview | VS Code official | SCM view/diff/graph/timeline |
| https://code.visualstudio.com/docs/remote/remote-overview | VS Code official | SSH/Dev Containers/WSL/Tunnels |
| https://code.visualstudio.com/docs/datascience/jupyter-notebooks | VS Code official | Notebook editor/cell shortcuts |
| https://code.visualstudio.com/docs/debugtest/debugging | VS Code official | Debugger UI/breakpoints |
| https://code.visualstudio.com/docs/configure/custom-layout | VS Code official | Side bars/menu bar/editor layout |
| https://code.visualstudio.com/docs/configure/settings | VS Code official | Settings scopes/editor |
| https://code.visualstudio.com/docs/editor/extension-marketplace | VS Code official | Extensions view/commands |
| https://code.visualstudio.com/docs/editing/codebasics | VS Code official | Multi-cursor/Selection menu/find/replace |
| https://code.visualstudio.com/docs/editing/editingevolved | VS Code official | Navigation/F12/symbols/problems |
| https://code.visualstudio.com/docs/configure/themes | VS Code official | Color Theme/menu path |
| https://code.visualstudio.com/docs/editing/getting-started/tips-and-tricks | VS Code official | Command Palette/context |
| https://forum.cursor.com/t/menu-bar-is-hidden-on-0-48/74615 | Community | Menu bar families |
| https://forum.cursor.com/t/bug-agents-window-shows-only-menu-bar-main-content-area-is-completely-blank/156859 | Community | Agents Window reduced menu |
| https://forum.cursor.com/t/japanese-ime-input-causes-unintended-chat-submission-when-pressing-enter-key-in-browser-version-agent/132905 | Community | Browser IME Enter submits |
| https://forum.cursor.com/t/chinese-ime-stutters-while-typing-in-the-chat-composer-windows/174514 | Community | IME stutter |
| https://forum.cursor.com/t/japanese-ime-composition-background-turns-black-in-every-cursor-ide-text-input/174895 | Community | IME composition rendering |
| https://forum.cursor.com/t/terminal-for-cloud-agents-not-loading-in-agents-window-but-is-loading-on-https-cursor-com-agents/160952 | Community | Web terminal vs Agents Window |
| https://forum.cursor.com/t/shift-enter-does-not-always-insert-new-line-at-cursor-location/135578 | Community | Shift+Enter newline behavior |
| https://forum.cursor.com/t/agent-chat-text-input-cursor-insertion-point-desync/150122 | Community | Paste/caret desync |

### Gaps and recommended next checks
1. **Authenticated web observation**: open `cursor.com/agents` with a real account and inventory the web editor, file tree, terminal, command palette, settings, themes, extensions, LSP, debugger, notebooks and keyboard map. Do not mutate a repository; use a scratch repo or read-only view. **[highest value]**
2. **Desktop live observation**: open the IDE and Agents Window, capture the actual menu items, Cmd+K behavior in each surface, and the reduced menu set. The current menu table is VS Code baseline + inference.
3. **IME test matrix**: Japanese/Chinese/Korean IME in IDE chat, Agents Window, web chat, web terminal; verify `Enter` during composition, `Shift+Enter`, paste, caret position, and composition styling.
4. **Web keyboard map**: test `Cmd+K`, `Cmd+Shift+P`, `Ctrl+K`, `Ctrl+F`, `Cmd+W`, `Cmd+R`, `Tab`, `Shift+Tab` in the web editor, terminal and prompt; note browser-level conflicts.
5. **Agents Window feature matrix**: confirm whether Tab autocomplete, notebooks, traditional debugger, themes and VS Code extensions are present/absent; docs only imply IDE-scoping for themes/extensions.
6. **Extension/plugin boundary**: clarify which "extensions" the Agents Window supports (Cursor plugins vs VS Code extensions) and whether Open VSX extensions run in the Agents Window editor.
7. **Menu enumeration**: capture the exact IDE menu items and Cursor-specific additions; current table is not exhaustive.
8. **Settings index**: enumerate Cursor Settings panes and per-project config files; current list is anchors, not complete.
9. **Model/plan gating**: record which models/modes/features are plan- or admin-gated; current docs mention paid Cloud Agents, Max Mode legacy, model access controls, privacy mode, SSO.
10. **Date drift**: re-check release notes; Cursor ships UI changes weekly (3.19→3.27 in the sampled window), so any UI claim older than a few weeks may be stale.

### Handoff note
The most defensible one-sentence answer to the lead is: **Cursor's editor surface is now split between a VS Code-derived desktop IDE (full workbench, extensions, themes, remote/LSP/debug/notebooks) and an agent-first stack (Agents Window + Cursor Web + Origin + mobile) whose common core is the prompt/Agent loop, context pickers, modes/models, diff/PR review, and a remote terminal; only the desktop IDE supports the full VS Code inheritance, while Cursor Web is a cloud-agent workspace with an editor/terminal/file tree whose depth is not publicly documented.**
