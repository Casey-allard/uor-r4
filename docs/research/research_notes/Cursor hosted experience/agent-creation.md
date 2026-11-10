# Cursor hosted Agents: landing, navigation and task creation (as of 2026-11-09)

Scope: the hosted/browser Agents experience at `cursor.com`, its public entry points, signed-in navigation, composer and task-creation inputs. No account was used, no login was attempted, and no task, repository or account mutation was performed. Evidence labels: **[OBS]** dated public UI observed on 2026-11-09; **[DOC]** official Cursor documentation or release notes; **[INF]** explicit inference; **[UNV]** not verifiable without an account or task execution.

## What are the hosted Agents entry points and public-vs-login boundaries?

### Takeaway
`cursor.com/agents` is the canonical hosted-app URL, but it is not a public landing page. Every app route probed returns `307` to `https://cursor.com/api/auth/login?redirect_uri=...`, which then hands off to WorkOS AuthKit at `authenticator.cursor.sh`; only marketing pages such as `/cloud`, `/product`, `/mobile`, `/pricing`, `/for/cloud-agents` and `/docs/cloud-agent` are public.

### Cited Findings
- The public homepage navigation contains **“Cloud Agents” → `/agents`**, **“Cloud” → `/cloud`**, **“Mobile” → `/mobile`**, **“Automations” → `/automate`**, **“Sign in” → `/dashboard`**, and **“Get started” → `/login`** — [cursor.com](https://cursor.com/) [OBS 2026-11-09].
- Route probe on 2026-11-09 (GET, desktop Chrome UA): `/agents`, `/agents/new`, `/dashboard`, `/dashboard/cloud-agents`, `/dashboard/integrations`, `/automations`, `/codebase`, `/login` and `/signup` all return `307` with a `Location` of `https://cursor.com/api/auth/login?redirect_uri=<original path>` — [cursor.com/agents](https://cursor.com/agents) [OBS]. `/login` preserves `redirect_uri=https://cursor.com/agents`; `/signup` adds `&signup=true`; `/marketplace/automations` instead redirects to `/marketplace#automations` — evidence file `06-route-probe-2026-11-09.txt`.
- Following `/agents` in a real browser reaches `https://authenticator.cursor.sh/?client_id=client_01GS6W3C96KW4WRS6Z93JCE2RJ&redirect_uri=https%3A%2F%2Fcursor.com%2Fapi%2Fauth%2Fcallback&state=...&authorization_session_id=...` with page title **“Sign in”** — [cursor.com/agents](https://cursor.com/agents) [OBS 2026-11-09]. This identifies the auth boundary as WorkOS AuthKit, not a Cursor-rendered login form.
- Observed sign-in page text, at 1280×720 viewport (2000×1125 image): **“Welcome to Cursor”**, **“The new way to build software”**, five icon-only provider buttons (Google, GitHub, Apple, Slack and X glyphs), field label **“Email”**, placeholder **“Your email address”**, button **“Continue with email”**, link **“Don't have an account? Sign up”**, and footer **“Terms of Service and Privacy Policy”** — [authenticator.cursor.sh](https://authenticator.cursor.sh/) [OBS 2026-11-09]; screenshot `04-signin-2026-11-09.png`.
- No password field, SSO button, passkey button or organisation selector is visible on the first sign-in step — [authenticator.cursor.sh](https://authenticator.cursor.sh/) [OBS]. The five provider buttons have no visible text labels; identification is by brand glyph [OBS].
- A headless Chrome run with a fresh `--user-data-dir` did **not** reach the app: it received a Cloudflare “Performing security verification / Just a moment...” interstitial on `authenticator.cursor.sh` (HTTP 403 to curl). The built-in Chromium browser, with its own profile, passed that check and reached the WorkOS sign-in page — [authenticator.cursor.sh](https://authenticator.cursor.sh/) [OBS]. This is a bot-protection boundary, not product behaviour.
- Official docs list seven launch surfaces: **Cursor for iOS**, **Cursor Web (`cursor.com/agents`)**, **Cursor Desktop** (select **Cloud** under the agent input), **Slack** (`@cursor`), **GitHub or Bitbucket** (`@cursor` on a PR/issue), **Linear** (`@cursor`), and the **API** — [Cloud Agents § How to access](https://cursor.com/docs/cloud-agent) [DOC].
- Cloud Agents require a qualifying plan: “Premium and higher, Teams, Enterprise, or a legacy paid plan. Free and Plus don't include them.” The iOS page separately lists Premium, Super, Ultra, Teams, Enterprise and legacy Start/Pro/Pro+/Ultra — [Cloud Agents § Troubleshooting](https://cursor.com/docs/cloud-agent); [Cursor for iOS § Availability](https://cursor.com/docs/cloud-agent/mobile) [DOC 2026-11-09].
- Privacy Mode is a hard gate. Docs state Cloud Agents “need cloud data storage to run” and that Privacy Mode (Legacy) users must switch before starting agents — [Cursor for iOS § Availability](https://cursor.com/docs/cloud-agent/mobile); [Cloud Agents § Troubleshooting](https://cursor.com/docs/cloud-agent) [DOC].
- A text-extraction fetch of `/agents` (no browser) returned app-like strings including **“Ask Cursor to build, fix bugs, explore”**, **“Dashboard”**, **“Running on AutoUsage limits reached. This Agent is running on Auto for free.”** and **“Privacy Mode change required”**. A real browser reached the WorkOS redirect (headless Chrome was blocked by Cloudflare before the app), so this extraction is treated as an unverified extraction artifact, **not** as current UI evidence [UNV]. It is recorded only because it may indicate string fragments that the report writer should not reuse as fact.

### Inferences
- The hosted product has no public, crawlable “landing page” at `/agents`; the effective landing for an unauthenticated visitor is the WorkOS sign-in screen. Any replication should treat authentication as the first screen, not as a modal over a public agent page [INF from route and browser evidence].
- `/dashboard` is the public “Sign in” destination while `/login` is the “Get started” destination that returns to `/agents`; these two entry points likely differ in onboarding copy and post-auth landing state, but that cannot be confirmed without an account [INF].
- Because `/agents/new` exists as a redirect target, a distinct new-task route probably exists behind auth, but whether it is a separate screen, a modal, or the default `/agents` state is unknown [INF/UNV].

### Gaps
- Exact post-login landing route and state (`/agents` list vs. a blank composer vs. last-opened agent) — unverified.
- Whether new accounts see onboarding, a team/workspace picker, a spend-limit dialog, a privacy-mode dialog, or a repository-connect prompt first — unverified.
- SSO/organisation discovery after email submission, MFA and passkey flows — unverified; the task forbids entering credentials.
- Whether `/agents/new` renders anything different from `/agents` — only the redirect is observable.
- The text-extraction strings above conflict with real-browser behaviour; source and age of that artifact are unknown.

## What navigation, sidebar, menu items, buttons, icons, labels, placeholders and defaults are documented or publicly observable?

### Takeaway
The public marketing chrome is fully observable, while the signed-in app chrome is documented only in fragments: a top navigation that includes a **Dashboard** tab, an agents sidebar with search/source filter/pinned agents and **Today / Last 7 days / Older** groups, an **Open sidebar** button, **Cmd+,** Settings, a profile-photo menu on mobile, and a set of Cloud Agents dashboard sections. Exact ordering, icons and labels of the app’s primary nav are not publicly documented.

### Cited Findings
**Public marketing navigation** [OBS 2026-11-09, cursor.com]:
- Primary items: **Models**, **Grok**, **Evals**, **Product**, **Agents**, **Cloud**, **Grok Bot**, **Mobile**, **Automations**, **CLI**, **Marketplace**, **Review**, **Enterprise**, **Pricing**, **Resources**, **Changelog**, **Blog**, **Docs**, **Community**, **Help**, **Workshops**, **Forum**, **Careers**.
- Utility items: **Sign in** (`/dashboard`), **Contact sales**, **Download**, **Download for macOS**.
- Footer product items include **Cloud Agents** (`/agents`) and **Composer** — [cursor.com](https://cursor.com/) [OBS].

**Sign-in screen** [OBS 2026-11-09]: labels listed in the previous section; the page has no visible “Continue with SSO” or workspace field.

**Signed-in app: top navigation and account menu** [DOC]:
- A **Dashboard tab** exists in the agents top navigation. The mobile changelog says the phone layout uses “a centered top navigation without the Dashboard tab, and a logo that takes you back to your agents” — [Cloud Agents release notes, Week of Aug 31, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- **Settings** opens with **Cmd+,** (**Ctrl+,** on Windows/Linux) on the agents pages; the collapsed sidebar no longer opens on hover, and a user must click the **Open sidebar** button — [Cloud Agents release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Opening a conversation only auto-collapses the sidebar on windows narrower than **1024 px** — [same entry](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- On mobile, the menu button shows the user’s **profile photo** — [Cloud Agents release notes, Week of Aug 31, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].

**Signed-in app: agents sidebar / list** [DOC]:
- The app has an **agents sidebar** and an **agents navigation drawer**; it lists **pinned agents** and supports agent switching — [Cloud Agents release notes, Week of Sep 28, 2026](https://cursor.com/docs/release-notes/cloud-agents); [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- The sidebar groups agents into **Today**, **Last 7 days**, and an **Older** section that starts collapsed — [Week of Sep 28, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- The agents list has **Search** and a **source filter**; on mobile these sit inline with the list — [Week of Aug 31, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- The list includes promotion slots, a **Cursor Learn banner**, and a **connect source control prompt**; these can shift layout while loading — [Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Agent pages show a header with **status**, the **starting prompt**, and the **repository name** at the top; hovering the repository name reveals the environment used for that run — [Cloud Agents § Environments](https://cursor.com/docs/cloud-agent); [Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- On pull-request pages, a floating **agent sidebar/dock** opens above the input, shows how many agents are working or need attention, and its header reads **“Waiting on”** when the open agent is idle but child agents are still working — [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Codebase pages also have agent sidebars whose transcripts scroll with the mouse wheel and Page Up/Page Down — [Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].

**Signed-in app: agent view and composer region** [DOC]:
- The transcript is shared with the Agents Window; the first message, including images, appears immediately — [Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- **Create repo** appears **above the agent input** when an Origin draft repository is ready to be published — [Cloud Agent setup § Start from scratch](https://cursor.com/docs/cloud-agent/setup) [DOC].
- **Send**, **Stop**, a **waveform** button labelled **Start voice chat**, and a **microphone** button for dictation occupy the same trailing control area; the waveform replaces Send only when the agent is idle and the input is empty — [Voice chat § Start a voice chat](https://cursor.com/docs/agent/voice-chat); [Voice Chat with Agents, Nov 6, 2026](https://cursor.com/changelog/voice-chat) [DOC].
- The composer supports repo command pills and slash commands; repo command pills pass their contents to the agent — [Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Marketing mock on the official Cloud page shows a top strip **“Agent” / “Dashboard”**, list headings **“Today” / “This Week” / “This Month”**, task rows with model badges such as **“2 models”**, and the input placeholder **“Add a follow up...”** — [cursor.com/cloud](https://cursor.com/cloud) [OBS 2026-11-09]. This is a **marketing mock**, not the live app; its grouping (“This Week / This Month”) conflicts with the release-note grouping (“Last 7 days / Older”), so it must not be used as layout truth.
- The same Cloud page’s automation mock shows the composer placeholder **“Enter prompt text... (type @ for tools & MCPs, / for skills and commands)”**, a model pill **“Grok 4.7”**, **“Tools”**, **“Send to Slack”**, **“Add Tool or MCP”**, and an opened slash command **“/apply-report-format”** — [cursor.com/cloud](https://cursor.com/cloud) [OBS]. Again a mock, but useful evidence of the product’s own naming for `@` tools/MCPs and `/` skills/commands.

**Cloud Agents dashboard** (`cursor.com/dashboard/cloud-agents`) [DOC]:
- Anchors/sections: **`#environments`**, **`#self-hosted`** — [Cloud Agent setup](https://cursor.com/docs/cloud-agent/setup); [Cursor for iOS § Remote Control](https://cursor.com/docs/cloud-agent/mobile) [DOC].
- Environments view lists saved environments scoped to one repo or a repo group; an environment page shows repositories, snapshot vs `.cursor/environment.json`, install script, runtime/build secrets, network access, version history, Builds and setup runs — [Cloud Agents settings § Environment management](https://cursor.com/docs/cloud-agent/settings) [DOC].
- Environment actions: **Update with Agent**, **New Setup Run**, **Restore**; Builds actions: **Trigger build**, **Verify with Agent**, **Fix Build**, **Ask Agent** (logs) — [settings](https://cursor.com/docs/cloud-agent/settings); [Cloud Agents release notes, Week of Sep 28, 2026](https://cursor.com/docs/release-notes/cloud-agents); [Cursor web release notes, Week of Sep 21, 2026](https://cursor.com/docs/release-notes/web) [DOC].
- Environments list has a **Config as code** badge and a **Config as code** switch/filter; the environment name can be renamed with a **pencil** button — [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Settings sections include **Preferences**, **Security**, and pages such as **My machines** and **Self-Hosted**; section anchors are shareable — [Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Automations live at **`/automations`**, not under `/agents`; the page has **Mine**, **Team**, **Explore** tabs and a **Share** menu on team automation headers — [Automations](https://cursor.com/docs/cloud-agent/automations); [Cursor web release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes/web) [DOC].

### Inferences
- The desktop app top nav most likely contains the Cursor logo (to agents), an **Agent/Agents** destination and a **Dashboard** tab; other items such as Automations or Codebase are plausible but not documented as top-nav tabs [INF]. The only directly evidenced tab is **Dashboard**.
- The signed-in landing is probably the agents list with a new-task composer, because the app is described as “agents list”, “agents sidebar” and “composer” on `cursor.com/agents`; however, no official page states the exact composition of `/agents` when no agent is selected [INF/UNV].
- The marketing mock’s “Agent / Dashboard” pair may mirror the real top nav, but because the same mock uses different date grouping and a different placeholder, it should be treated as directional only [INF].

### Gaps
- Exact primary-nav item set, order, icons, active-state styling, and whether Automations/Codebase/Cloud Agents appear as top-level tabs — unverified.
- Exact button labels for creating a new agent (e.g. “New Agent”, “New chat”, or an always-present composer) — unverified.
- Real composer placeholder and helper text on the signed-in page — unverified; only mock and stale-extraction strings exist.
- Sidebar width, icon set, row layout, hover actions (rename, archive, pin), context-menu items and empty states — unverified.
- Which items appear in the profile menu, workspace/team switcher, and plan/usage menu — unverified.
- Keyboard focus order and ARIA roles/names for the nav, sidebar and composer — unverified.

## What are the composer inputs and task-creation controls?

### Takeaway
The documented task-creation model is: **instruction text + optional images** → **repository/repositories or Start from scratch** → **branch/starting ref or PR URL** → **model and model params** → **environment/worker** → **mode (plan vs agent)** → **MCP servers** → **attachments and secrets** → **send / queue**. The web composer is documented to expose repository, branch, model, environment, MCP, attachments, dictation and voice; exact visual grouping and placeholder text are not public.

### Cited Findings

| Control | Type / placement | Exact or documented label(s) | Behaviour, defaults and limits | Source | Status |
|---|---|---|---|---|---|
| Instruction text | Multiline text input / main composer | Actual web placeholder **unverified**. Official mock: **“Ask Cursor to build, plan, fix...”**; automation mock: **“Enter prompt text... (type @ for tools & MCPs, / for skills and commands)”**; stale extraction: **“Ask Cursor to build, fix bugs, explore”** | Required `prompt.text` in API; task instruction is the primary input; supports `@` context and `/` skills | [Prompting agents](https://cursor.com/docs/agent/prompting); [Cloud Agents API § Create An Agent](https://cursor.com/docs/cloud-agent/api/endpoints); [cursor.com/cloud](https://cursor.com/cloud) | DOC for behaviour; UNV for real placeholder |
| Repository | Picker / chip above or in composer | **“Start from scratch”** in the repository picker; otherwise choose a connected repository | Start from scratch creates a draft **Origin** repo in the background; needs a plan with Cloud Agents + Origin (not Free/Plus); team default repo can skip the choice; API allows up to 500 repos | [Cloud Agent setup § Start from scratch](https://cursor.com/docs/cloud-agent/setup); [Cloud Agents settings § Default settings](https://cursor.com/docs/cloud-agent/settings); [API § repos](https://cursor.com/docs/cloud-agent/api/endpoints) | DOC |
| Branch / ref | Picker or text field next to repo | **Branch**, **starting ref** (API `startingRef`) | Branch name or full 40-char SHA; blank → repository default branch; new auto-generated `cursor/...` branch unless `workOnCurrentBranch: true`; `prUrl` overrides starting ref | [API § Create An Agent](https://cursor.com/docs/cloud-agent/api/endpoints); [Cloud Agents release notes, Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents) | DOC |
| Model | Dropdown at the top of the chat input | **Model picker dropdown**; model pill in mock **“Grok 4.7”** | Curated cloud model list; per-model params for reasoning effort, context window and Fast; omit → user default → team default → system default; `Cmd+/` cycles models (Agent docs); iOS always uses max context | [Prompting agents § Changing models](https://cursor.com/docs/agent/prompting); [API § model](https://cursor.com/docs/cloud-agent/api/endpoints); [Cloud Agents § Models](https://cursor.com/docs/cloud-agent); [Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile) | DOC; web placement inferred from shared composer docs |
| Mode | Mode selector / keyboard cycle | **Plan** vs **Agent** | API `mode` defaults to `agent`; `plan` explores and drafts a plan before coding; Plan mode is reached with **Shift+Tab** in the IDE; a web mode control is not documented | [API § mode](https://cursor.com/docs/cloud-agent/api/endpoints); [Plan mode](https://cursor.com/help/ai-features/plan-mode) | DOC for API/IDE; UNV for web control |
| Environment / worker | Picker near repo/model | **Environment**; named **cloud**, **pool**, **machine** targets | Resolution order: personal saved environment → team saved environment → `.cursor/environment.json` (single-repo only); no repo/environment → team default no-repo environment; environment picker lists unnamed envs by repo names like “web-app +3” | [Cloud Agent setup § Environment resolution order](https://cursor.com/docs/cloud-agent/setup); [API § env](https://cursor.com/docs/cloud-agent/api/endpoints); [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents) | DOC |
| Images | Drag/drop, paste, attach button | **Drag and drop**; **Paste from clipboard with Cmd+V** | API: max 5 images, 15 MB each; PNG/JPEG/GIF/WebP; web docs/images 4 MB per file | [Prompting agents § Image input](https://cursor.com/docs/agent/prompting); [API § prompt.images](https://cursor.com/docs/cloud-agent/api/endpoints); [Cloud Agent web attachment limits](https://cursor.com/help/ai-features/cloud-agents) | DOC (general Agent); web limit DOC |
| Documents / video | File attachment | **Attach** (label unverified) | Documents/images 4 MB per file; videos 15 MB per file; PDF, MD, TXT, CSV, JSON, HAR, XML, YAML, TOML, INI, LOG; ZIP unsupported; browser rejects before upload | [Cloud Agent web attachment limits](https://cursor.com/help/ai-features/cloud-agents); [Agent troubleshooting](https://cursor.com/help/troubleshooting/agent-issues) | DOC |
| Links | No dedicated field documented | — | URLs can be included in prompt text; the API image input also accepts an `http(s)` URL; no link-preview or URL-validation control is documented | [API § prompt.images](https://cursor.com/docs/cloud-agent/api/endpoints) | INF/UNV |
| `@` context | Mention menu in the input | **`@`** | Suggestions for files/folders, **@Terminals**, **@Chats**, **@Commit (Diff of Working State)**, **@Branch (Diff with Main)**, **@Browser**; automation mock says `@` for tools & MCPs | [Prompting agents § @ mentions](https://cursor.com/docs/agent/prompting); [cursor.com/cloud](https://cursor.com/cloud) | DOC for Agent; web parity UNV |
| Slash commands / skills | `/` menu | **`/`**; mock command **“/apply-report-format”** | `Enter` attaches a skill to one message; `Option+Enter` / `Alt+Enter` keeps it as a mode; repo command pills pass contents to the agent | [Prompting agents § Custom Modes](https://cursor.com/docs/agent/prompting); [Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents) | DOC; web parity partly inferred |
| MCP servers | **MCP dropdown** on `cursor.com/agents` | **MCP dropdown** | Select MCP servers per run at launch; personal and team servers; HTTP and stdio; OAuth; built-in Cursor Cloud MCP; team admins manage shared servers under **Dashboard → Plugins & MCPs** | [Cloud Agents § MCP support](https://cursor.com/docs/cloud-agent); [Capabilities § MCP tools](https://cursor.com/docs/cloud-agent/capabilities); [Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile) | DOC |
| Secrets / env vars | Dashboard and environment pages, not the composer | **Secrets**; **Environment Variable**, **Runtime Secret**, **Build Secret** | Workspace/team-scoped and environment-scoped; injected at agent start; running agents do not pick up new secrets; runtime secrets redacted as `[REDACTED]`; API session `envVars` max 50 | [Cloud Agent setup § Environment variables and secrets](https://cursor.com/docs/cloud-agent/setup); [Security and network § Secret protection](https://cursor.com/docs/cloud-agent/security-network); [API § envVars](https://cursor.com/docs/cloud-agent/api/endpoints) | DOC |
| Dictation | Microphone button next to Send | **Microphone**; **Start voice input** (Agents Window), **Voice Input** (IDE), **Record voice** (iOS) | Turns speech into text in the composer; user reviews before sending; `Cmd+Shift+Space` in IDE; hold `Ctrl+M` in Agents Window | [Prompting agents § Voice input](https://cursor.com/docs/agent/prompting); [Voice chat § Voice chat and dictation](https://cursor.com/docs/agent/voice-chat) | DOC |
| Voice chat | Waveform button where Send normally is | **Start voice chat** | Only when agent idle and input empty; realtime spoken conversation; Grok Voice; voices include Ara, Eve, Leo, Rex, Sal; hidden in subagents/workers/archived/teammate chats | [Voice chat](https://cursor.com/docs/agent/voice-chat); [Voice Chat with Agents, Nov 6, 2026](https://cursor.com/changelog/voice-chat) | DOC |
| Send / queue / stop | Trailing button in composer | **Send**, **Stop**, **Send now** | Enter queues a follow-up while the agent works; `Cmd+Enter` sends immediately; double-Enter or **Send now** steers at the next tool call; Stop replaces Send while running | [Agent overview § Queued messages](https://cursor.com/docs/agent/overview) | DOC for Agent; new-task submit key UNV |
| Create repo | Button above the agent input | **Create repo**, **Other**, **Private**, **Internal**, **Create Origin repo** | Suggested names or custom name; letters/digits/hyphens/underscores, max 100 chars; Private = only you, Internal = team can view/edit | [Cloud Agent setup § Save the work to a repository](https://cursor.com/docs/cloud-agent/setup) | DOC |
| Auto PR / reviewer | API option; UI toggle location unverified | `autoCreatePR`, `skipReviewerRequest` | Whether to open a PR on completion; whether to request the user as reviewer; PRs now open **ready for review** by default, not draft | [API § Create An Agent](https://cursor.com/docs/cloud-agent/api/endpoints); [Cloud Agents release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents) | DOC for API/default; UI UNV |

**Keyboard submission and queueing detail** [DOC]:
- While an agent is running: `Enter` adds the message to the queue; `Cmd+Enter` sends immediately and appends to the most recent user message; `Enter` twice sends a steering follow-up; **Send now** does the same — [Agent overview § Queued messages](https://cursor.com/docs/agent/overview).
- These bindings are documented for Agent/IDE; the exact binding for submitting a **brand-new** cloud task from the web is not stated [UNV].

**Validation and error messages** [DOC unless noted]:
- File too large on web: **`<filename> is too large (4MB limit)`** — [Agent troubleshooting](https://cursor.com/help/troubleshooting/agent-issues).
- ZIP as a web document attachment is unsupported; browser rejects over-limit files before upload — [Cloud Agent web attachment limits](https://cursor.com/help/ai-features/cloud-agents).
- Fork PR automations fail with **“Fork pull requests not supported”** — [Automations § GitHub triggers](https://cursor.com/docs/cloud-agent/automations).
- Worker cannot start: **“Cloud Agent could not get a worker to start your session; please retry in a few minutes.”** — [Cloud Agents release notes, Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- Microphone denied: **“Microphone blocked. Allow it in your system or browser settings.”** — [Voice chat § Requirements](https://cursor.com/docs/agent/voice-chat).
- Voice chat without account auth: **“Realtime voice requires sign-in”** — [Voice chat § FAQ](https://cursor.com/docs/agent/voice-chat).
- Self-hosted workspace attach: **“Workspace runs on a self-hosted worker”** — [Week of Sep 7, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- GitHub access failures: the **Connect GitHub** card names the repository the agent needs; errors explain missing app access, SAML SSO, or IP allow-list blocks — [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents); [GitHub § Troubleshooting](https://cursor.com/docs/integrations/github).
- Model policy block: **“Model blocked”** for managed automations — [Cursor web release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes/web).
- No branch given: not an error; Cursor “boots the default branch when no branch is given” — [Week of Sep 14, 2026](https://cursor.com/docs/release-notes/cloud-agents).

**Documented defaults** [DOC]:
- Model: user default → team default → system default — [API § model](https://cursor.com/docs/cloud-agent/api/endpoints); a **Default model** can be set in Cloud Agents settings — [settings § Default settings](https://cursor.com/docs/cloud-agent/settings).
- Repository: **Default repository** can be set; when empty, agents ask the user to choose — [settings](https://cursor.com/docs/cloud-agent/settings).
- Base branch: blank means the repository’s default branch — [settings](https://cursor.com/docs/cloud-agent/settings).
- Environment: personal → team → committed `.cursor/environment.json`; new saved environments default **environment.json location** to **“None (skip environment.json)”** — [setup § Environment resolution order](https://cursor.com/docs/cloud-agent/setup).
- Mode: `agent` by default — [API § mode](https://cursor.com/docs/cloud-agent/api/endpoints).
- PR style: ready for review by default — [Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- First use: user is asked to set a spend limit — [Cloud Agents § Billing](https://cursor.com/docs/cloud-agent).

### Inferences
- The API’s `Create An Agent` body is the best available proxy for the composer’s task-creation state: prompt, images, model/params, name, env, repos, starting ref/PR URL, work-on-current-branch, auto-PR, reviewer request, env vars, MCP servers, subagents and mode. Not all of these are necessarily exposed in the web UI (e.g. `customSubagents`, `envVars`, `agentId`) [INF].
- The web composer probably reuses the shared Agent composer (model picker at top, `@` and `/` menus, image paste/drag), but the docs describe those in IDE/Agents Window terms; web parity should be validated against a real account [INF].
- The “Start from scratch” path is the closest documented equivalent to a “no repository” task-creation preset; it is not a prompt template, but it does create an Origin draft repo automatically [INF].

### Gaps
- Real placeholder/helper text on the signed-in composer; whether it is “Ask Cursor to build…” and whether it changes by mode/model.
- Whether the web composer exposes Plan mode, thinking effort, Fast tier, context-window selection, subagents, session env vars, or auto-PR toggles, and in what grouping.
- Whether attachments are a paperclip button, drag-drop zone, paste-only, or all three; attachment count limits in the UI (API allows 5 images).
- Whether links get previews, validation, or a separate URL field.
- IME composition behaviour (Chinese/Japanese/Korean), Shift+Enter vs Enter, multiline growth, paste sanitisation and drag-drop of non-image files — no official documentation found.
- Exact inline validation for empty prompt, invalid branch, inaccessible repo, unsupported file type, oversized video, or expired auth.
- Whether the composer persists drafts, supports history/undo, or offers prompt templates/presets.

## What task-creation options exist for parallel/background/cloud agents, schedules, presets, permissions, secrets, environment setup and GitHub integration?

### Takeaway
Hosted Cursor is designed for many parallel cloud agents on isolated Ubuntu VMs, with per-run repo/branch/model/environment selection, team-scoped visibility and secrets, optional self-hosted workers, and a separate Automations surface for schedules and event triggers. “Presets” exist as automation schedule presets and marketplace templates, not as a documented composer prompt library.

### Cited Findings
**Parallel and background execution** [DOC]:
- “You can run as many agents as you want in parallel, and they do not require your local machine to be connected to the internet” — [Cloud Agents § Why use Cloud Agents?](https://cursor.com/docs/cloud-agent).
- Each agent runs in an isolated Ubuntu VM with cloned repos, installed dependencies, secrets, startup commands and network access; Cursor manages provisioning, isolation, snapshots, startup, artifacts and capacity — [Cloud Agents § Environments / Runtime](https://cursor.com/docs/cloud-agent).
- Multi-repo environments clone each selected repo into the agent machine and can open PRs in the repos it changes; API allows up to 500 repos — [setup § Multi-repo environments](https://cursor.com/docs/cloud-agent/setup); [API § repos](https://cursor.com/docs/cloud-agent/api/endpoints).
- Agents can use computer control (click, type, screenshots), remote desktop control, artifacts (screenshots/videos/logs), and MCP tools — [Capabilities](https://cursor.com/docs/cloud-agent/capabilities) [DOC].
- Projects add a coordinator/worker model with a task tracker, List and Board views, parent/subtask status propagation, and worker topic names — [Projects](https://cursor.com/docs/agent/projects); [Cloud Agents release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents) [DOC].
- Team members can view an agent only if they belong to the same Cursor team **and** have their own access to the repository; viewing is read-only unless an admin enables team follow-ups — [Cloud Agents § Artifacts and remote desktop control](https://cursor.com/docs/cloud-agent) [DOC].

**Schedules and automations** [DOC]:
- Automations are created at **`cursor.com/automations`**, in the Agents Window, with the **`/automate`** skill, or from a template in the **Cursor Marketplace** — [Automations § Getting started](https://cursor.com/docs/cloud-agent/automations).
- Scheduled triggers run on a recurring schedule; the editor offers **preset options** or a **cron expression**, and runs may start late but never before the indicated time (up to 5 minutes late, per release notes) — [Automations § Scheduled triggers](https://cursor.com/docs/cloud-agent/automations); [Cursor web release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes/web).
- Trigger sources: GitHub, GitLab, Bitbucket Cloud, Slack, webhooks, Linear, Sentry, PagerDuty; GitHub supports draft opened, PR opened/pushed/merged, push to branch, comment, labels, CI completed, issue comment, PR review comment/submitted, review thread updated, workflow run completed — [Automations § Triggers](https://cursor.com/docs/cloud-agent/automations).
- Automation tools include PR creation, PR comments, reviewer requests, Slack send/read, MCP servers, memories and computer use — [Automations § Tools](https://cursor.com/docs/cloud-agent/automations).
- Automation settings choose model, repositories (**No repository**, **Single repository**, **Multi-repo environment**), and **Share** options: **Run as Me / Service account**, **Private / Members can view / Members can edit**, plus **Copy link** — [Automations § Automation settings / Share](https://cursor.com/docs/cloud-agent/automations).
- Automation tabs: **Mine**, **Team**, **Explore** (template gallery); team admins land on Team, others on Mine — [Cursor web release notes, Week of Oct 5, 2026](https://cursor.com/docs/release-notes/web).

**Presets and templates** [DOC]:
- Automation schedule presets and the **Explore** template gallery / Marketplace automations are documented — [Automations](https://cursor.com/docs/cloud-agent/automations); [Marketplace automations](https://cursor.com/marketplace#automations).
- No official documentation was found for reusable prompt presets, saved composer templates, or a “task preset” control on the Agents page — [UNV/GAP].

**Permissions** [DOC]:
- Plan gate: Premium and higher, Teams, Enterprise, or legacy paid; Free and Plus excluded — [Cloud Agents § Troubleshooting](https://cursor.com/docs/cloud-agent).
- Repository requirement: read-write privileges to the repo and any dependent repos/submodules — [Cloud Agents § Repository provider connection](https://cursor.com/docs/cloud-agent).
- Team visibility: same team plus repository access; viewing read-only; **Team follow-ups** has three modes: **Disabled**, **Service accounts only**, **All** — [Cloud Agents § Share agents with your team](https://cursor.com/docs/cloud-agent); [settings § Team follow-ups](https://cursor.com/docs/cloud-agent/settings).
- Admin security toggles: **Display agent summary**, **Display agent summary in external channels**, **Team follow-ups**, **Automatically allow Projects to place their work**, **Computer use** — [settings § Security settings](https://cursor.com/docs/cloud-agent/settings).
- Network modes: **Allow all network access**, **Default + allowlist**, **Allowlist only** — [settings § Network access settings](https://cursor.com/docs/cloud-agent/settings).
- Remote Control is gated separately and must be enabled per team/self-hosted settings — [Cursor for iOS § Remote Control](https://cursor.com/docs/cloud-agent/mobile).

**Secrets** [DOC]:
- Secrets are managed at the **Secrets** tab/Cloud Agents dashboard and can be team, personal or environment-scoped; environment-scoped secrets apply to every repo in that environment and never reach Self-Hosted Machines — [setup § Environment variables and secrets](https://cursor.com/docs/cloud-agent/setup).
- Secret types: **Environment Variable**, **Runtime Secret** (redacted as `[REDACTED]` in tool results/transcript/commits), **Build Secret** (Docker build only) — [Security and network § Secret protection](https://cursor.com/docs/cloud-agent/security-network).
- Precedence: values passed at agent start (`envVars`) → environment secrets → user personal secrets → team secrets — [API § Secrets](https://cursor.com/docs/cloud-agent/api/endpoints).
- Team admins can turn on **Disable personal secrets** — [Cloud Agents release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents).

**Environment setup** [DOC]:
- Two main paths: agent-led setup from the Cloud Agents dashboard (under 10 minutes) or manual Dockerfile via `.cursor/environment.json`; both support an install script — [setup § Environment setup options](https://cursor.com/docs/cloud-agent/setup).
- Environment JSON fields: `build.dockerfile`, `build.context`, `install`, `snapshot`, `start`, `terminals`; paths relative to `.cursor`; schema at `cursor.com/schemas/environment.schema.json` — [setup § Configuration in code](https://cursor.com/docs/cloud-agent/setup).
- Builds prepare the environment in the background; failed builds do not replace the active build; Builds tab supports logs, trigger, pin, start agent, **Fix Build**, **Verify with Agent** — [setup § How Builds use the install script](https://cursor.com/docs/cloud-agent/setup); [Cloud Agents settings](https://cursor.com/docs/cloud-agent/settings).
- Resource limits: default VM profile; Enterprise can request more; self-serve custom resources “coming soon” — [setup § Resource limits](https://cursor.com/docs/cloud-agent/setup).
- Start from scratch creates a draft Origin repo; “Create repo” publishes it, with **Private** or **Internal** visibility and a 100-character name — [setup § Start from scratch](https://cursor.com/docs/cloud-agent/setup).

**GitHub integration** [DOC]:
- Connect at **`cursor.com/dashboard/integrations`**; the GitHub row has **Connect** (or **Manage Connections** when connected), **Disconnect Account**, and for GitHub Enterprise **Manage apps**; **Source Control** section groups it — [GitHub § Setup](https://cursor.com/docs/integrations/github).
- The first Connect links a GitHub account without installing the app; users can connect a second Cursor account to a GitHub account that already has the app — [GitHub § Setup](https://cursor.com/docs/integrations/github).
- `@cursor` on a GitHub PR or issue launches an agent; GitHub Enterprise Server is supported; no public webhook URL is required because GitHub events pull through the Cursor account — [Cloud Agents § How to access](https://cursor.com/docs/cloud-agent); [Agent SDK changelog](https://cursor.com/changelog/agent-sdk) [DOC].
- PRs from Cloud Agents open **ready for review** by default; a codebase setting **Never open PRs as drafts** can force that — [Cloud Agents release notes, Week of Oct 19, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- GitHub permissions requested include repository contents, pull requests, checks, custom repository roles and organization custom properties; “least privilege” claimed — [GitHub § Permissions](https://cursor.com/docs/integrations/github).
- Sharing a GitHub installation with another team uses **Share with another team** on the GitHub/GitHub Enterprise row; the receiving team can use every repository on the installation and is billed to that team — [GitHub § Share an installation](https://cursor.com/docs/integrations/github).

### Inferences
- The hosted experience is intentionally “many small task surfaces” (web, desktop, iOS, Slack, GitHub, Linear, API) sharing one backend; the web app is the management and environment-configuration surface, not the only place tasks start [INF].
- Automations are the closest thing to scheduled/preset task creation; they are a separate product surface with their own editor, permissions and billing identity, not a mode of the `/agents` composer [INF].
- Because Cloud Agents can run without a repository (Start from scratch or no-repo automation), Cursor has a first-class “no codebase” task path; replication should not assume a repo is mandatory [INF].

### Gaps
- Exact schedule presets and timezone handling; whether cron uses UTC or account timezone.
- Maximum number of concurrent agents per user/team and queue behaviour at the cap — not documented in the pages read.
- Whether Projects/workers are available on all Cloud Agents plans or only certain tiers.
- Exact UI for auto-PR, reviewer request, subagents, session env vars and model params on the web.
- Self-hosted machine/pool setup details beyond the dashboard anchors; not needed for the hosted replication but relevant to parity.

## What responsive/mobile behaviour and accessibility/keyboard details are publicly established?

### Takeaway
Cursor documents a distinct phone layout for the web agents pages and a separate native iOS app; keyboard coverage is partial (Settings, sidebar, queue/send, model cycling, dictation), and there is no public accessibility conformance statement, ARIA map, IME note or mobile breakpoint table.

### Cited Findings
**Mobile web** [DOC]:
- “On mobile, the agents pages on cursor.com use slightly larger text, a centered top navigation without the Dashboard tab, and a logo that takes you back to your agents. Search and the source filter now sit inline with the agents list, the composer has a taller input, and the menu button shows your profile photo.” — [Cloud Agents release notes, Week of Aug 31, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- Opening a conversation only collapses the sidebar on windows narrower than **1024 px** — [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- Dashboard phone fixes: tab strips scroll to screen edges and tap targets are larger (Week of Oct 12, 2026); the navigation drawer can be swiped left to close (Week of Oct 5, 2026); the menu button sits at the left of the top bar (Week of Sep 28, 2026) — [Cursor web release notes](https://cursor.com/docs/release-notes/web) [DOC].

**Native iOS** [DOC]:
- Separate app, iPhone/iPad iOS 26.0+, English only, Android planned — [Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile).
- Pick repository and branch, send a task, supervise; models use maximum context; dictation with live transcription; voice chat including Siri; MCP servers selectable per run; Design Mode attachments; push notifications; up to eight Live Activities — [Cursor for iOS § Getting started / What you can do](https://cursor.com/docs/cloud-agent/mobile).
- iPad layout keeps chats in a sidebar, opens review next to chat, full-width diffs, Apple Pencil annotation — [Cursor for iOS § iPad layout](https://cursor.com/docs/cloud-agent/mobile).
- Web-only: editor/terminal/file browser, secrets/environments, MCP management, source-control setup, automations/rules/skills config, admin/billing/usage — [Cursor for iOS § What lives on the web](https://cursor.com/docs/cloud-agent/mobile).

**Keyboard** [DOC]:
- **Cmd+, / Ctrl+,** opens Settings on the agents pages — [Week of Oct 12, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- **Open sidebar** button replaces hover-to-open; sidebar auto-collapse only below 1024 px — [same](https://cursor.com/docs/release-notes/cloud-agents).
- `Cmd+K` and `Ctrl+F` inside the agent terminal go to the shell, not the page — [Week of Oct 5, 2026](https://cursor.com/docs/release-notes/cloud-agents).
- `Enter` queues, `Cmd+Enter` sends immediately, `Enter` twice steers, **Send now** steers — [Agent overview](https://cursor.com/docs/agent/overview).
- `Cmd+/` cycles models; `Cmd+Shift+Space` toggles IDE dictation; hold `Ctrl+M` in the Agents Window to talk — [Prompting agents](https://cursor.com/docs/agent/prompting); [Voice chat](https://cursor.com/docs/agent/voice-chat).
- Automations list rows are keyboard reachable with **Tab**; usage-event table columns can be reordered by drag or keyboard — [Cursor web release notes, Week of Oct 12, 2026](https://cursor.com/docs/release-notes/web).
- Plan mode is cycled with **Shift+Tab** from the chat input in the IDE — [Plan mode](https://cursor.com/help/ai-features/plan-mode).

**Paste, drag-drop and dictation** [DOC]:
- Images can be dragged into the chat input or pasted with **Cmd+V**, including screenshots — [Prompting agents § Image input](https://cursor.com/docs/agent/prompting).
- Dictation is a microphone icon in the chat input; user reviews transcription before sending — [Prompting agents § Voice input](https://cursor.com/docs/agent/prompting).

### Inferences
- The web app is responsive down to phone widths, but the mobile web layout is a reduced management/direction surface; deep configuration remains on desktop/web and iOS intentionally omits admin/secrets/environment management [INF].
- Keyboard support appears aimed at power users (Settings, sidebar, model cycling, queueing) rather than full keyboard-only operation; no evidence of roving tabindex, skip links, or focus traps [INF].
- Because the composer is shared across surfaces, `@`, `/`, drag-drop and paste likely work on web, but the docs describe them for Agent generally, not the hosted web app specifically [INF].

### Gaps
- Breakpoints other than the 1024 px sidebar collapse; tablet layout; landscape phone layout.
- ARIA roles/names, screen-reader announcements for streaming transcripts, focus order, focus return after dialogs, colour contrast, reduced-motion support, zoom/reflow behaviour — no official accessibility documentation found [UNV].
- IME composition, dead keys, right-to-left text, multiline Enter/Shift+Enter, paste sanitisation, and drag-drop of folders/non-image files — no documentation found [UNV].
- Whether the mobile web composer supports voice chat/dictation and file attachments at parity with desktop [UNV].
- Localisation of the web app; iOS is documented as English-only [DOC for iOS, UNV for web].

## Which claims cannot be verified without an account or real task execution?

### Takeaway
Almost all of the **signed-in** experience is unverified: the landing composition, real labels/placeholders, default selections, control grouping, validation copy, permission prompts, environment build behaviour, model routing and task execution. What is verified is the auth boundary, the public marketing chrome, and the official documentation/release-note descriptions of the app.

### Cited Findings
**Verified without an account** [OBS/DOC]:
- The `/agents` → `/api/auth/login` → `authenticator.cursor.sh` redirect chain and the sign-in screen’s exact labels — [OBS 2026-11-09].
- Public marketing navigation and the `/cloud`, `/product`, `/mobile`, `/pricing`, `/for/cloud-agents`, `/docs/cloud-agent` routes — [OBS/DOC].
- The existence and text of the official docs, API schema and weekly release notes cited above — [DOC].
- That headless Chrome is challenged by Cloudflare on the auth domain, while the built-in browser reaches WorkOS — [OBS].

**Not verified (require an account, and in many cases a real task)** [UNV]:
- Post-login landing: whether `/agents` shows an empty composer, the agents list, the last agent, or an onboarding checklist.
- Actual primary-nav items/order/icons; whether **Automations**, **Codebase**, **Cloud Agents** or **Marketplace** are top-level tabs; profile-menu contents; workspace/team switcher.
- Real composer placeholder, helper text, character counter, attachment button icon, model/mode/environment grouping, and whether plan mode is exposed.
- Default values actually preselected in the UI (model, repo, branch, environment, context window, Fast/effort).
- Whether `@`, `/`, drag-drop, paste, dictation and voice chat work identically on web; their exact menus and empty states.
- Inline validation for empty prompt, bad branch, inaccessible repo, unsupported file, oversized video, expired auth, plan limit, or privacy-mode block.
- Permission prompts for repo access, GitHub App installation, MCP OAuth, microphone, spend limit, and team follow-ups.
- Environment build screens, logs, build pinning, **Fix Build** and **Verify with Agent** behaviour.
- Actual task execution: branch creation, commits, PR opening, artifacts, remote desktop, computer use, review/merge, model routing and billing.
- Private/enterprise behaviour: SSO, SCIM, service accounts, audit logs, IP allow-lists, self-hosted machines/pools, customer-managed keys.
- Mobile web at real phone widths and the native iOS app (not installed; iOS 26+ requirement).

### Inferences
- For a replication plan, the highest-risk unknowns are not the documented fields but the **information architecture** of the signed-in landing and the **default state** of the composer; both should be treated as design decisions to validate with a low-cost account-based observation before locking a spec [INF].
- The official docs are sufficient to define a functional task-creation contract (prompt, repo, branch, model, env, mode, MCP, attachments, secrets, auto-PR), but not to pixel-match Cursor’s UI [INF].

### Gaps
- No public screenshot or recording of the current signed-in `/agents` landing was found in the docs, help center or official blog pages fetched. The official `/cloud` page uses a marketing mock, not a live capture.
- No official accessibility statement, design-system documentation, or component library for the web agents app was found.
- No public API/OpenAPI schema for the web app’s own UI state (only the Cloud Agents REST API, which is a related but separate contract).

### Evidence ledger and method (2026-11-09)
- **Public route probe**: `06-route-probe-2026-11-09.txt` — GET requests with desktop Chrome UA; records `307` locations for `/agents`, `/agents/new`, `/dashboard`, `/dashboard/cloud-agents`, `/dashboard/integrations`, `/automations`, `/codebase`, `/login`, `/signup`, `/marketplace/automations`.
- **Sign-in observation**: `04-signin-2026-11-09.png` — 1280×720 viewport, 2× scale; WorkOS page reached from `/agents` in the built-in Chromium browser; no credentials entered.
- **Headless bot-check evidence**: `01-agents-desktop-2026-11-09.png`, `02-agents-mobile-2026-11-09.png`, `03-agents-dom-2026-11-09.html` — fresh Chrome profile at 1440×1100 and 390×844; Cloudflare interstitial on `authenticator.cursor.sh`; not product UI.
- **Marketing mock evidence**: `05-cloud-marketing-2026-11-09.html` — official `/cloud` page HTML containing the “Agent / Dashboard / Today / This Week” mock and the automation composer placeholder; labeled as mock, not live app.
- **Docs/release notes read** (all fetched 2026-11-09): Cloud Agents overview, setup, settings, capabilities, mobile, API endpoints, automations, security/network; Agent overview, prompting, voice chat, Plan mode; GitHub integration; Cloud Agent help and troubleshooting; Cloud Agents release notes (weeks of Aug 31–Oct 19, 2026); Cursor web release notes (weeks of Aug 31–Oct 19, 2026); voice-chat changelog (Nov 6, 2026); public homepage and `/cloud`.
- **No account, login, task, repository, secret, payment or mutation was used.** No Cursor source code or proprietary assets were copied; screenshots are of public pages only and are stored under `local/kimi/cursor-hosted-research/agent-creation/`, not for commit.
- **Worktree base**: `/Users/casey.allard/uor-r4/.worktrees/kimi-cursor-hosted-research`, branch `kimi/cursor-hosted-research`. This note is the only repository file I wrote; `uor-r4-baseline.md` in the same directory was present from another researcher and was not touched.
