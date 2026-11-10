# Cursor Integrations and Hosted-Platform Capabilities (as of 2026-11-09)

*Method: all Cursor documentation pages below were fetched on 2026-11-09 (America/New_York) as Markdown from `cursor.com/<path>.md`, with HTTP status, byte size and SHA-256 recorded in `local/kimi/cursor-hosted-research/integrations-platform/manifest.tsv` (127 unique 200 responses). Page text is quoted or paraphrased only from those captures; marketing/legal pages were fetched as HTML. Labels: `[official-documented]` = stated on a Cursor page; `[observed]` = directly observed in the fetch (status, index, count); `[inferred]` = my reasoning over cited facts; `[unverified]` = asserted or implied but not confirmed. No account, credential, repository, webhook or paid service was connected or exercised.*

## 1. Which exact integrations, configuration inputs, permissions, environment variables/secrets and user controls are officially documented?

### Takeaway
Cursor documents a wide first-party integration surface, but nearly every integration is a **delivery channel into one hosted execution product — Cloud Agents** — rather than an independent capability. Configuration is predominantly **file-based and repo-committed** (`.cursor/environment.json`, `mcp.json`, `hooks.json`, `cli.json`, `sandbox.json`, `permissions.json`), with a smaller set of dashboard-managed secrets and org policies; permissions are coarse provider-level grants, and most controls are plan-gated to Teams/Enterprise.

### Cited Findings

**A. Integration and surface inventory** `[official-documented]`

| Surface | Direction / role | Documented auth and permission model | Availability / status |
|---|---|---|---|
| GitHub.com and GitHub Enterprise Server | Clone, branch, push, PRs, review comments, issues, CI status, mergeability | Cursor GitHub App installed by a Cursor admin + GitHub org admin; per-user Git account is a second layer; access inherited from triggering user | Cloud Agents require Premium+, Teams, Enterprise, or legacy paid plan; Free/Plus excluded ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)) |
| GitLab.com and GitLab self-hosted | Same as GitHub | OAuth application; redirect `https://cursor.com/gitlab-connected`; `Trusted: true`, `Confidential: true`; scopes `api` and `write_repository`; paid GitLab plan (Premium/Ultimate) required because project access tokens are unavailable on Free | Self-hosted requires Teams or Enterprise ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)) |
| Bitbucket Cloud | Clone/push/PRs and Bugbot comments | Personal OAuth connection per developer + workspace-level Cursor app ("Forge Apps") for app identity; permissions: Account, Repositories, Pull requests, Pull request write, Webhooks | Public beta; `bitbucket.org` only ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)) |
| Bitbucket Data Center | Bugbot only | Dedicated service account + HTTP access token; repository admin per repo | Teams or Enterprise; **Cloud Agents not supported** ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)) |
| Azure DevOps Services | Clone, branch, PRs, Bugbot | Microsoft Entra OAuth; Bugbot uses a Cursor-provisioned Entra service principal requiring tenant admin consent (`client_id=29df4809-337a-4255-bd57-d4a1476b0e65`, redirect `https://cursor.com/azure-devops-connected`, Azure DevOps API resource `499b84ac-1321-427f-aa17-267ca6975798`) | Public beta; `dev.azure.com` only, **Azure DevOps Server not supported**; Automations, Bugbot autofix, Security Agents unsupported ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)) |
| Slack | `@Cursor` in channels/DMs; thread context; status; PR link | Slack app installed by workspace admin; dashboard account linking; 18 scopes listed (see D) | Teams follow-ups controlled separately; channel settings apply to public channels only ([slack.md](https://cursor.com/docs/integrations/slack.md)) |
| Microsoft Teams | `@Cursor` in personal chat, team channels, group chats | Teams app; 6 permissions: `identity`, `messageTeamMembers`, `ChannelMessage.Read.Group`, `ChatMessage.Read.Chat`, `ChannelSettings.Read.Group`, `TeamSettings.Read.Group` | ([microsoft-teams.md](https://cursor.com/docs/integrations/microsoft-teams.md)) |
| Jira | Assign work item to Cursor or `@Cursor` comment; Rovo chat follow-ups | Atlassian Marketplace app; either service-account auth or per-user auth via **Require individual authentication** | Cursor Teams/Enterprise; Jira Commercial Cloud with Rovo; **no Atlassian HIPAA, FedRAMP or Government Cloud** ([jira.md](https://cursor.com/docs/integrations/jira.md)) |
| Linear | Delegate issue or `@Cursor` comment; status; auto-PR | OAuth workspace connection by a Cursor admin; account linking on first use | ([linear.md](https://cursor.com/docs/integrations/linear.md)) |
| Notion | Delegate specs/tasks/comments to an agent | Cursor **User API key** entered into Notion; GitHub connected for PRs | Beta; Notion Business or Enterprise plans only ([notion.md](https://cursor.com/docs/integrations/notion.md)) |
| Tailscale | Lets **Rollouts** reach tailnet-only MCP servers | Tagged ephemeral device (`tag:cursor`), OAuth client with `auth_keys` scope, ACL to MCP server tag/port | Beta on Teams and Enterprise; HTTPS MCP only (stdio has no tailnet address) ([tailscale.md](https://cursor.com/docs/integrations/tailscale.md)) |
| MCP servers | Tools/resources/prompts/roots/elicitation/MCP Apps | `stdio` (local, manual auth), `SSE` and `Streamable HTTP` (remote, OAuth); static OAuth client credentials supported | Capabilities say SSE and `mcp-remote` are **not supported inside cloud agents**, but the API schema enumerates `sse`; cloud SSE is **unverified**. HTTP/stdio are the documented supported transports ([mcp.md](https://cursor.com/docs/mcp.md), [capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md), [api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)) |
| Plugins / Skills / Hooks / Subagents / Rules | Extensibility packages and policy hooks | Cursor Plugins (`.cursor-plugin/plugin.json`) and Agent Plugins (`plugin.json`); marketplaces reviewed; install modes Default Off / Default On / Required | Team marketplaces: 1 on Teams, unlimited on Enterprise ([plugins.md](https://cursor.com/docs/plugins.md)) |
| CLI (`agent`) | Interactive, print/headless, CI | Browser login or `CURSOR_API_KEY`; permission tokens in `~/.cursor/cli-config.json` or `.cursor/cli.json` | ([cli/overview.md](https://cursor.com/docs/cli/overview.md), [cli/reference/authentication.md](https://cursor.com/docs/cli/reference/authentication.md)) |
| Cloud Agents API | Programmatic agents, runs, environments, secrets, workers/pools, models, repos | Basic or Bearer with user or service-account API key | **v1 public beta**; webhooks "coming soon" on v1, legacy v0 supports them ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)) |
| TypeScript/Python SDK, SDK Bridge | Local or cloud agent runs from code | `CURSOR_API_KEY`; SDK Bridge adds a per-process loopback bearer token | User and service-account keys; **Team Admin API keys not supported yet** ([sdk/typescript.md](https://cursor.com/docs/sdk/typescript.md), [sdk/bridge.md](https://cursor.com/docs/sdk/bridge.md)) |
| Agent SDK (`@grok/agent-sdk`) | Define/deploy agents as Markdown + TypeScript | `agent-sdk login`; hosted deployments get an alias token; secrets set via CLI | Cursor-managed hosting or self-host ([agent-sdk.md](https://cursor.com/docs/agent-sdk.md), [agent-sdk/deployment.md](https://cursor.com/docs/agent-sdk/deployment.md)) |
| Automations | Scheduled/event-triggered cloud agents | GitHub/GitLab/Bitbucket/Slack/Linear/Sentry/PagerDuty/webhook triggers; run as Me or Service account | ([cloud-agent/automations.md](https://cursor.com/docs/cloud-agent/automations.md)) |
| Origin | Cursor's git forge: repos, mirror from GitHub, PRs, code tours, Origin Apps | Early beta; namespace claimed once and not changeable during beta | Premium+; not Free/Plus ([origin.md](https://cursor.com/docs/origin.md)) |
| Apps | Cloud agents build and publish hosted web apps | Dedicated hosting account per team/user, auto-provisioned Postgres, Cursor sign-in for private/team apps, team/personal secrets | Cloud agents only, **not supported locally** ([cloud-agent/apps.md](https://cursor.com/docs/cloud-agent/apps.md)) |
| Mobile | iOS native app for cloud and local agents; Android via `cursor.com/agents` PWA | Cursor account, SSO if enforced | iOS/iPadOS 26.0+, English; Android planned ([cloud-agent/mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)) |
| Web | `cursor.com/agents` | Cursor account | Same backend as desktop Agents Window and iOS ([cloud-agent/mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)) |

**B. Configuration inputs and exact keys** `[official-documented]`

- **`.cursor/environment.json`** — `build.dockerfile`, `build.context`, `snapshot` (e.g. `snapshot-20260212-...`), `install`, and (from the startup section) `start` and `terminals`. Paths in `build` are relative to the repo's `.cursor` folder; when `context` is omitted it defaults to `.cursor`; `.`, `./`, `..` mean repo root; `install` runs from project root. Full schema at `https://www.cursor.com/schemas/environment.schema.json` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- **Environment resolution order** — 1) personal saved environment, 2) team saved environment, 3) `.cursor/environment.json` (single-repo runs only). Saved environments start with "environment.json location = None (skip environment.json)" ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- **Desktop/project MCP** — `.cursor/mcp.json` (project) and `~/.cursor/mcp.json` (global); remote entries use `url`, `headers`, optional `auth.CLIENT_ID`, `auth.CLIENT_SECRET`, `auth.scopes`; stdio entries use `type: "stdio"`, `command`, `args`, `env`, `envFile`; interpolation supports `${env:NAME}`, `${userHome}`, `${workspaceFolder}`, `${workspaceFolderBasename}`, `${pathSeparator}`, `${/}` ([mcp.md](https://cursor.com/docs/mcp.md)).
- **CLI permissions** — `~/.cursor/cli-config.json` (global) or `<project>/.cursor/cli.json`; `permissions.allow` / `permissions.deny` with `Shell(commandBase)`, `Read(pathOrGlob)`, `Write(pathOrGlob)`, `WebFetch(domainOrPattern)`, `Mcp(server:tool)`; deny beats allow ([cli/reference/permissions.md](https://cursor.com/docs/cli/reference/permissions.md)).
- **MCP auto-run allowlist** — `~/.cursor/permissions.json` `mcpAllowlist` array using `server:tool`, `server:*`, `*:tool`, `*:*`; precedence: team dashboard > `~/.cursor/permissions.json` > editor settings ([model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).
- **Sandbox** — `~/.cursor/sandbox.json` (all projects) and `<project>/.cursor/sandbox.json` (project, merged with project taking priority); controls network policy, extra readable/writable paths, temp writes, shared build caches; admin policy and hardcoded rules layer on top ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).
- **Hooks** — `.cursor/hooks.json` (project), `~/.cursor/hooks.json` (user, not available in cloud), team and enterprise hooks via dashboard on Enterprise ([hooks.md](https://cursor.com/docs/hooks.md)).
- **Steering/context files** — `AGENTS.md` (cloud agents read it; docs recommend a "Cursor Cloud specific instructions" section), `.cursorignore` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md), [agent/security.md](https://cursor.com/docs/agent/security.md)).
- **Agent SDK hosting** — `defineAgent({ hosting: { egressDomains: [...], secretNames: [...] } })`; secrets set with `agent-sdk secret set <slug> NAME`; commit names, never values ([agent-sdk/deployment.md](https://cursor.com/docs/agent-sdk/deployment.md)).
- **MDM policies** — `AllowedTeamId` (comma-separated team IDs), `AllowedExtensions` (JSON string; allowlist model, `{"*": true}` to keep allow-all), `WorkspaceTrustEnabled` (boolean), `UpdateMode`; corresponding client settings `cursorAuth.allowedTeamId`, `extensions.allowed`, `security.workspace.trust.enabled` ([identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md), [security-hardening.md](https://cursor.com/docs/enterprise/security-hardening.md)).
- **Tenancy header** — `x-cursor-allowed-tenants: v1.orgs=<ids>;teams=<ids>` on traffic to `api2.cursor.sh` (including sign-in) and `cursor.com`, injected by a TLS-terminating proxy; up to 8 copies, each must allow; bad format is refused rather than treated as absent ([tenancy-restriction.md](https://cursor.com/docs/enterprise/tenancy-restriction.md)).

**C. Secrets and environment variables** `[official-documented]`

- Three secret types for Cloud Agents: **Environment Variable** (visible to agent; for non-sensitive config), **Runtime Secret** (loaded as env var but redacted from tool results, transcript, commits and commit messages as `[REDACTED]`; still visible to a user in the terminal), **Build Secret** (Docker build only, via `RUN --mount=type=secret`) ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Secret scopes: team secrets, user/personal secrets, and environment-scoped secrets; environment-scoped secrets "never reach Self-Hosted Machines"; Build Secrets never reach workers ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md), [self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- Injection timing: "Secrets are injected when an agent starts. Agents already running won't pick up new secrets" ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- `CURSOR_AWS_ASSUME_IAM_ROLE_ARN` enables STS role assumption; Cursor sets `AWS_CONFIG_FILE`, `AWS_PROFILE=cursor-cloud-agent`, `AWS_SDK_LOAD_CONFIG=1`; STS credentials expire after 1 hour and refresh when within 15 minutes of expiry ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- OIDC: agents mint RS256 JWTs over `POST /v1/tokens/oidc` on a Unix socket at `CURSOR_AGENT_SOCKET` (managed VM: `/run/cursor/api.sock`); 5-minute lifetime, no refresh, 30 tokens/min per agent, 8 concurrent connections ([identity.md](https://cursor.com/docs/cloud-agent/identity.md)).
- Other documented variables: `CURSOR_API_KEY` (CLI, SDK, workers, Agent SDK), `CURSOR_AGENT` (set in shell when Cursor runs), `AGENT_SDK_BEARER_TOKEN`, `AGENT_ALIAS_URL` / `AGENT_ALIAS_TOKEN`, `CURSOR_SDK_BRIDGE_LOG=1`, `NO_OPEN_BROWSER=1`, `HTTPS_PROXY`/`https_proxy` (self-hosted worker), `ALL_PROXY`/`HTTP_PROXY`/`HTTPS_PROXY` (Tailscale userspace), `CF_ACCESS_CLIENT_ID`/`CF_ACCESS_CLIENT_SECRET` (Cloudflare Access), `TOTP_SECRET` (for `oathtool --totp -b "$TOTP_SECRET"`).
- Headers: `X-Agent-Alias-Token` and `X-BDK-Session-Key` (Agent SDK hosted routes), `CF-Access-Client-Id`/`CF-Access-Client-Secret`, `x-cursor-allowed-tenants` ([deployment.md](https://cursor.com/docs/agent-sdk/deployment.md), [guides/webhooks.md](https://cursor.com/docs/agent-sdk/guides/webhooks.md), [setup.md](https://cursor.com/docs/cloud-agent/setup.md)).

**D. Permissions requested per integration** `[official-documented]`

- **GitHub App** (8 purposes): Repository access (clone, create working branches), Pull requests, Issues, Checks and statuses, Actions and workflows, Administration (read branch protection/required checks), Custom repository roles, Organization custom properties. The page states all follow least privilege but does **not** publish GitHub scope strings ([github.md](https://cursor.com/docs/integrations/github.md)).
- **Slack** (18 scopes): `app_mentions:read`, `channels:history`, `channels:join`, `channels:read`, `chat:write`, `files:read`, `files:write`, `groups:history`, `groups:read`, `im:history`, `im:read`, `im:write`, `mpim:history`, `mpim:read`, `reactions:read`, `reactions:write`, `team:read`, `users:read` ([slack.md](https://cursor.com/docs/integrations/slack.md)). `[observed]` count from the table.
- **GitLab**: `api`, `write_repository`; app must be Trusted and Confidential ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)).
- **Azure DevOps**: organizations/projects listing, code repositories (clone/create branches), pull requests (open/update/merge) ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).
- **Microsoft Teams**: 6 permissions listed above ([microsoft-teams.md](https://cursor.com/docs/integrations/microsoft-teams.md)).
- **Jira**: identify user, read work item fields/descriptions/comments, post status/completion/PR links, receive assignment/mention events; exact scope strings are shown only in the Atlassian Marketplace prompt, not in the docs ([jira.md](https://cursor.com/docs/integrations/jira.md)).
- **Linear**: read issues, write issue status (per the enterprise integration summary) ([model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).

**E. User and admin controls** `[official-documented]`

- Run Modes and approvals: terminal commands require approval by default; Auto-review runs allowlisted calls, sandboxes shell commands when possible, and routes the rest through a classifier; the docs state Auto-review "is not a security boundary" ([agent/security.md](https://cursor.com/docs/agent/security.md), [run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).
- Sandbox: workspace read/write; protected paths include `.git/config`, `.git/hooks`, `.vscode`, `.cursorignore`; network blocked by default then opened by mode; `/tmp` writable unless disabled ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).
- Read Access: **System** (default; reads outside workspace without approval) vs **Workspace** (asks outside workspace; Read Allowlist); requires Cursor 3.23+ ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).
- Protections that can force approval even in an auto mode: Browser Protection, File-Deletion Protection, External-File Protection ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).
- Cloud agent controls: network access mode (user/environment/team, lockable on Enterprise), team follow-ups (Disabled / Service accounts only / All), display agent summary, display summary in external channels, computer use (Enterprise only), spend limits, retention policy ([settings.md](https://cursor.com/docs/cloud-agent/settings.md), [security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Sharing: agents visible to same Cursor team **and** viewer must connect their own SCM account and have repo access; viewing is read-only unless team follow-ups enabled ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- Enterprise: model access (team baseline + Organization Groups union), BYOK restriction, MCP allowlist and per-server tool/network policy, repository blocklist (editor/CLI only), Protected Git Scopes, SSO/SCIM, MDM, audit streaming, CMEK, US data residency ([model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).

### Inferences
- `[inferred]` The integration surface is deliberately "thin": Cursor owns the agent runtime and repo access, while chat/issue tools are notification and trigger channels. A Studio plan that needs first-class issue-tracker behavior cannot rely on Cursor's integrations for data ownership; it would own that state itself.
- `[inferred]` Because secrets are injected only at agent start and environment config resolves personal → team → repo file, per-tenant isolation in a hosted Studio maps naturally to one saved environment per tenant/repo group rather than per-request secret passing.
- `[inferred]` The documented controls split cleanly into "deterministic" (hooks, sandbox, approvals, network allowlists, Git scopes) and "best-effort" (Auto-review, rules, `.cursorignore`, browser origin allowlist). Any Studio security claim should be built on the former.

### Gaps
- GitHub App **scope strings** and installation-token behavior are not published; only purpose-level permissions.
- Jira OAuth scopes are not enumerated in docs.
- No documentation found for **remote SSH, WSL, or dev containers** as Cursor surfaces: seven candidate URLs returned 404 (`/docs/configuration/ssh`, `/docs/remote-development`, `/docs/configuration/remote-development`, `/docs/configuration/containers`, `/docs/configuration/wsl`, `/docs/agent/remote-development`, `/docs/reference/ssh`) and the docs index contains no such page. Absence from the index is not proof the product lacks the feature `[unverified]`.
- JetBrains and Xcode integration pages were captured but not analyzed in depth; their capability/permission details are not covered here.
- Grok Bot (a separate Cursor product with hosted computers, `*.cursorvm.com`, its own network policy, approvals, inference hooks and team setup scripts) appears throughout enterprise docs but was out of scope; its integration surface is not inventoried here.

## 2. How do background/cloud agents execute code, install dependencies, access network, use secrets, persist state and open PRs?

### Takeaway
A cloud agent is a **stateful, per-agent Ubuntu VM** built from a snapshot ("Build"), with dependency installation done ahead of time by an `install` script, long-running processes started by `start`/`terminals`, **automatic terminal execution**, internet on by default, secrets injected as environment variables at start, and handoff by pushing a branch and opening a signed PR. State persists as conversation history (indefinite by default), disk snapshots (30-day inactivity window), and optional automation memories.

### Cited Findings

**Lifecycle** `[official-documented]`
- Six stages: Start (web, IDE, CLI, API, Slack, linked issue/PR) → Provision (isolated VM, clone authorized repo) → Run (code/tools, stream progress) → Persist (conversation, metadata, artifacts) → Hand off (push branch, open PR; ready for review unless draft requested) → Recycle (hibernate then delete on lifecycle timers) ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Cloud agents run in "isolated Ubuntu machines" and "the same agent fundamentals" as local agents ([setup.md](https://cursor.com/docs/cloud-agent/setup.md), [cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- Isolation: per-agent VM boundary, Firecracker-based microVMs, and a **separate AWS account** from the rest of Cursor production ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Cursor-hosted VMs are **x86_64 (amd64)**; arm64 requires a self-hosted worker ([choose-runtime.md](https://cursor.com/docs/cloud-agent/self-hosted/choose-runtime.md)).

**Dependencies and environment builds** `[official-documented]`
- Two setup paths: agent-driven setup in the dashboard (agent installs deps, verifies, creates first Build) or a Dockerfile referenced from `.cursor/environment.json` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- Build lifecycle: trigger (recurring, config change, manual, agent-requested) → prepare (base image, clone every repo at default branch, run `install` to completion) → snapshot disk with environment version and exact commit SHA → activate → agents start from active Build ([builds.md](https://cursor.com/docs/cloud-agent/builds.md)).
- "Builds preserve disk state only. Running processes, exported shell variables, and in-memory caches don't continue into an agent run. Start services with `start` or `terminals`" ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- `terminals` run in a `tmux` session shared by user and agent ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- Failed Build does not replace the active Build; agents continue from the last successful one ([builds.md](https://cursor.com/docs/cloud-agent/builds.md)).
- Docker is supported but runs inside another container layer; complex setups need `fuse-overlayfs` and `iptables-legacy`; a sample Dockerfile pins `docker-ce=5:28.5.2-1~ubuntu.24.04~noble` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- Resource limits: "Each cloud agent runs on a default VM profile with limited memory and CPU"; Enterprise can request increases; self-serve custom resources "coming soon" ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- Multi-repo environments clone each selected repo and are reused for later runs and automations on the same repo group ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).

**Network access** `[official-documented]`
- "The agent has internet access by default." Three modes: **Allow all network access**, **Default + allowlist**, **Allowlist only**. Even in Allowlist only, Cursor services and SCM providers remain reachable ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- The "Default" domains are the same default allowlist used by the desktop sandbox; one team allowlist controls both ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Enterprise can **lock** the network policy so it overrides user settings; a locked setting does not reach Self-Hosted Machines ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Egress IPs are published as JSON at `https://cursor.com/docs/ips.json`, with clusters keyed `us3p`, `us4p`, `us5p`; Cursor warns not to use IP allowlisting as the primary control ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- **Git egress proxy**: only git traffic uses a narrower IP set (24 addresses listed, of which 21 are pre-allocated and not yet serving) `[observed]` count; other requests including GitHub API and `gh` use Cloud Agent ranges ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Artifact uploads go to `cloud-agent-artifacts.s3.us-east-1.amazonaws.com`; the docs warn not to wildcard `*.s3.us-east-1.amazonaws.com` because it opens egress to every bucket in the region ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Private network options: Tailscale **userspace networking** (`tailscaled --tun=userspace-networking`, SOCKS5 on 1055, HTTP proxy on 1054), Cloudflare Tunnel (`cloudflared` outbound; Access service tokens as secrets), or private connectivity (AWS PrivateLink / Cloudflare Tunnel) for GHES, GitLab Enterprise, Bitbucket Data Center, Artifactory/Nexus and webhook traffic ([setup.md](https://cursor.com/docs/cloud-agent/setup.md), [private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).
- PrivateLink specifics: Cursor-to-Git direction via your endpoint service; Git-to-Cursor direction via Cursor's endpoint service `com.amazonaws.vpce.us-east-1.vpce-svc-054b15427d4bea2b7` / `vpce-svc-054b15427d4bea2b7` with private DNS `api2.cursor.sh`; Cursor subnet CIDRs `10.2.8.0/21`, `10.2.24.0/21`, `10.2.40.0/21`; no self-signed certs, SSH, custom ports or IPv6-only endpoints ([private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).
- Google Private Service Connect is **not offered** ([private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).

**Execution autonomy** `[official-documented]`
- "The agent auto-runs all terminal commands, letting it iterate on tests. This differs from the foreground agent, which requires user approval for every command." The docs explicitly name data-exfiltration/prompt-injection risk ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Mitigations named: egress controls, Runtime Secrets, `.cursorignore`, human PR review, signed commits, hooks, Bugbot/Security Agents ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).

**Secrets** `[official-documented]`
- Three types and scopes as in Q1; encrypted at rest and in transit; "not visible to anyone other than the Cloud Agent user" ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- OIDC tokens for cloud federation instead of long-lived keys; claims include `cloud_agent_id`, `agent_runtime` (`managed`/`self_hosted`), `team_id`, `repo_url`/`repo_urls`, `branch_name`, `source` ([identity.md](https://cursor.com/docs/cloud-agent/identity.md)).
- AWS assume-role flow with external ID and Cursor principal `arn:aws:iam::289469326074:role/roleAssumer` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).

**Persistence and state** `[official-documented]`
- Four data classes: runtime workspace (VM; recycled after idle, timer refreshes on follow-up), VM snapshots (encrypted, rolling 30 days of inactivity), conversation state (Cursor backend, per-agent keys, **indefinite by default**), secrets/tokens (until deleted) ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Delete Agent API removes transcript and artifacts but **not** snapshots ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Enterprise retention: **Indefinite** or **90 days**; custom windows in early access; switching back to Indefinite does not restore deleted data ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Automations add **Memories**: named entries (`MEMORIES.md` by default) stored outside the working filesystem, persisted across runs, editable/deletable in UI ([automations.md](https://cursor.com/docs/cloud-agent/automations.md)).
- Agent SDK hosting: sessions, `ctx.host.kv`, and `ctx.host.files` survive restarts and redeploys; other disk writes may disappear ([agent-sdk/deployment.md](https://cursor.com/docs/agent-sdk/deployment.md)).
- Self-hosted: full checkout, build cache and machine-local credentials stay on the machine; the worker sends file contents, terminal output, diffs, screenshots, local MCP results and routing metadata to Cursor ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).

**PRs, commits, CI** `[official-documented]`
- Agents push a branch and open a PR "ready for review unless you ask for a draft" ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Every commit is signed with an **HSM-backed Ed25519** key and shows "Verified" on GitHub; satisfies signed-commit branch protection with no setup ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Commit author is `Cursor Agent <cursoragent@cursor.com>`, with the starting person as `Co-authored-by` when resolvable; on GitLab, the project access token's bot user; on Bitbucket Cloud, PRs are attributed to the person who started the agent ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md), [gitlab.md](https://cursor.com/docs/integrations/gitlab.md), [bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)).
- CI autofix: automatically tries to fix CI failures on PRs it created; **GitHub Actions only**; skips if a human pushed, a follow-up was sent, the check already fails on the base commit, or after **10** CI-failure follow-ups; disable per agent with `@cursor autofix off` ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md)).
- Subscriptions: agent subscribes to GitHub PR/CI, Slack thread/channel, Linear issue, or timers; lasts at most **180 days**; bursts coalesce; not available to service-account-owned agents ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md)).
- Artifacts (screenshots, videos, logs) attach to the PR; posting them into GitHub PR descriptions uses long unguessable **public** URLs because GitHub's image proxy requires public URLs ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md)).
- Remote desktop control and port forwarding let a human test the app in the agent VM ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md), [setup.md](https://cursor.com/docs/cloud-agent/setup.md)).

**Self-hosted execution** `[official-documented]`
- "Cursor runs the agent loop, inference, and planning. Your worker performs file edits and terminal commands. It also runs computer-use tools and local MCP servers" ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- Worker holds a long-lived **outbound HTTPS** connection; no inbound ports, public IPs or VPN tunnels. Required hosts: `api2.cursor.sh`, `api2direct.cursor.sh`, `agent-stores.s3.us-east-1.amazonaws.com`, `cloud-agent-artifacts.s3.us-east-1.amazonaws.com`; proxy via `HTTPS_PROXY`/`https_proxy` ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- Live workspace (`--connect-workspace`, Linux only, Cursor 3.25+) gives files, terminal and forwarded ports; runs as the worker's OS user; downloads server builds from `cursor.blob.core.windows.net` and refuses unsigned builds ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- Limits: up to 10,000 workers per user and per team; Team Pools require Enterprise and a service-account API key ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).

### Inferences
- `[inferred]` The Build abstraction is the main cost/latency lever: dependency work is moved off the critical path, but any Studio that needs per-tenant toolchains inherits a build farm and snapshot store, not just a container scheduler.
- `[inferred]` Because the agent auto-runs commands and has internet by default, a hosted Studio's default posture should be **Allowlist only + locked policy + Runtime Secrets**, accepting that Cursor services/SCM remain reachable by design.
- `[inferred]` "Conversation indefinite, snapshots 30 days" means a Studio must implement its own tenant-level retention above Cursor's, because Cursor's 90-day Enterprise cap applies per team, not per Studio tenant.
- `[inferred]` Self-hosted workers move only tool execution; inference and planning remain Cursor-hosted, so self-hosting does **not** produce an air-gapped product.

### Gaps
- VM CPU/RAM/disk sizes, GPU availability, per-run wall-clock limits, concurrency quotas and build minutes are not published (only "default VM profile" and Enterprise uplift on request).
- Exact base image/OS version is not pinned beyond "Ubuntu"; Docker sample implies Ubuntu 24.04 (noble) but that is an example, not a platform guarantee.
- No documented egress bandwidth/latency figures.
- The `ips.json` payload was not fetched during this research; only its documented shape and example are cited.
- Whether Builds run for every plan tier or only where Cloud Agents are available is implied but not stated per-tier.

## 3. What GitHub/GitLab/Slack/Linear/API/MCP/CLI/mobile surfaces and actions exist, and what is not supported?

### Takeaway
GitHub is the reference provider and the only one with the full trigger, CI-autofix and Security Agents feature set; GitLab, Bitbucket Cloud and Azure DevOps are partial, and each has an explicit exclusion list. The API, MCP, CLI and mobile surfaces are real but bounded: the Cloud Agents API v1 is public beta without webhooks; MCP in cloud agents documents HTTP/stdio, while SSE is contested (capabilities say unsupported, API schema enumerates `sse`) and therefore **unverified**; mobile is iOS-only native (Android via PWA).

### Cited Findings

**GitHub** `[official-documented]`
- Actions: clone, create working branches, open/update PRs, leave review comments (Bugbot), track issues, report checks/statuses, monitor CI and trigger re-runs, read branch protection/required checks, read custom repository roles and org custom properties ([github.md](https://cursor.com/docs/integrations/github.md)).
- Kick off from a GitHub PR or issue by commenting `@cursor` ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- Triggers available to Automations: draft opened, PR opened, PR pushed, PR merged, push to branch, comment added, PR label changed, issue label changed, CI completed, issue comment, PR review comment, PR review submitted, review thread updated, workflow run completed (push-started only) ([automations.md](https://cursor.com/docs/cloud-agent/automations.md)).
- Enterprise controls: IP allow list via installed GitHub App or explicit IPs; AWS PrivateLink / Cloudflare Tunnel for GHES; Protected Git Scopes; team installation sharing ([github.md](https://cursor.com/docs/integrations/github.md)).

**GitLab** `[official-documented]`
- Actions: clone, branch, MRs, Bugbot comments; commit author is the project access token bot user; if the bot identity can't be resolved, falls back to `Cursor Agent <cursoragent@cursor.com>` and push rules may reject ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)).
- Extra Automations triggers beyond core: MR label changed, MR approved ([automations.md](https://cursor.com/docs/cloud-agent/automations.md)).
- Requires paid GitLab plan; self-hosted requires Teams/Enterprise; OAuth token endpoint redirects restricted to same scheme/host/port, max 5 ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)).

**Bitbucket** `[official-documented]`
- Cloud: clone, push, PRs, Bugbot comments; identity split (app for comments/status, user for git/PR); one Bitbucket workspace per Cursor team; `Switch workspace` disconnects the app ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)).
- Data Center: Bugbot only; comment commands `bugbot run` / `cursor review` **do not run**; Autofix **not supported** ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)).
- Automations: Bitbucket Cloud only; no label or inline review-comment triggers; PR triggers don't run on fork PRs (except merged) ([automations.md](https://cursor.com/docs/cloud-agent/automations.md)).

**Azure DevOps** `[official-documented]`
- Cloud Agents clone/branch/PR; Bugbot reviews and posts comments/build status `cursor-bugbot/review`; on-demand `cursor review` / `bugbot run` works only when the commenter's ADO sign-in address matches a Cursor account in the owning team ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).
- Not supported: Automations, Bugbot autofix, Security Agents; Auto-Enable for New Repositories; Automatically Learn Rules; personal Bugbot settings; personal-scope repos; ADO Server ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).

**Slack** `[official-documented]`
- Commands: `@Cursor [prompt]`, `@Cursor settings`, options (`repo`, `env`/`environment`, `branch`, `model`, `autopr`, `worker`/`machine`, `pool`, `self_hosted`/`sh`, `channel`), `@Cursor agent`, `@Cursor list my agents`, `@Cursor pool [set|unset]`; context menu Add follow-up / Switch repository / Delete / View request ID / Give feedback; DMs start a new agent per top-level message ([slack.md](https://cursor.com/docs/integrations/slack.md)).
- Routing: message content → recent activity → routing rules → channel default → default repository; `env` beats `repo`; inline options beat modal defaults; options inside triple-backtick code blocks are ignored ([slack.md](https://cursor.com/docs/integrations/slack.md)).
- Limits: channel settings and channel default pools apply to **public channels only**; Slack triggers see only public channels; external-channel summaries are a separate privacy toggle ([slack.md](https://cursor.com/docs/integrations/slack.md), [automations.md](https://cursor.com/docs/cloud-agent/automations.md)).

**Linear** `[official-documented]`
- Actions: delegate issue to Cursor, `@Cursor` comment, follow-up comments, real-time status, auto-PR; config keys `repo`, `branch`, `model` via `[key=value]`, issue labels, or project labels; repo labels must be a group named exactly `repo` with `owner/repo` labels ([linear.md](https://cursor.com/docs/integrations/linear.md)).
- Limit: triage rules need a human assignee for rules to fire ([linear.md](https://cursor.com/docs/integrations/linear.md)).

**API** `[official-documented]`
- Cloud Agents API v1 (public beta): create/list/get agent, follow-up/steer, list/get runs, stream run, cancel run, usage, list/download artifacts, archive/unarchive/delete, environments CRUD + history/builds, secrets (personal/team/environment) list/set/delete, user-scoped worker token, workers/pools/pending requests/claim/release/fail, API key info, list models, list GitHub repositories ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- Auth: Basic or Bearer; user or service-account key; keys beginning `grok_` get `403 key_not_supported` for worker tokens ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- **v1 webhooks are "coming soon"; the legacy v0 API still supports them** ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- Other APIs: Admin, Analytics, AI Code Tracking, Bugbot (Enterprise, Basic auth); Origin API (early beta, Bearer via Origin CLI/App); SDKs ([api.md](https://cursor.com/docs/api.md)).
- The APIs are explicitly **not** a model-inference or chat-completions API ([api.md](https://cursor.com/docs/api.md)).

**MCP** `[official-documented]`
- Supported transports in the IDE: stdio, SSE, Streamable HTTP; protocol features Tools, Prompts, Resources, Roots, Elicitation, MCP Apps extension ([mcp.md](https://cursor.com/docs/mcp.md)).
- Cloud agents: HTTP (recommended; config never in VM, tool calls proxied through backend) and stdio (server runs in VM; agent can read its config/env) are the documented supported transports. The capabilities page says **SSE and `mcp-remote` are not supported**, while the API schema enumerates `sse`; **cloud SSE support is unverified**. OAuth is per-user even for team servers; `env`, `headers`, `CLIENT_SECRET` are write-only/redacted after save ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md), [api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- OAuth: authorization code + PKCE; RFC 9728 protected-resource metadata; RFC 8707 resource indicator; dynamic client registration as a public client; fixed redirects `https://www.cursor.com/agents/mcp/oauth/callback` (web/agents) and `http://localhost:8787/callback` (desktop); refresh-token renewal; `offline_access` may be needed ([mcp.md](https://cursor.com/docs/mcp.md)).
- Cloud MCP sign-ins are stored on Cursor servers, separate from local sign-ins ([mcp.md](https://cursor.com/docs/mcp.md)).

**CLI** `[official-documented]`
- Interactive, Plan/Ask/Agent modes, print mode `-p/--print`, `--force`/`--yolo` for file changes, output formats text/json/stream-json, `--stream-partial-output`, image/media via file paths ([cli/overview.md](https://cursor.com/docs/cli/overview.md), [cli/headless.md](https://cursor.com/docs/cli/headless.md)).
- Auth: `agent login` browser flow, `agent status`, `agent logout`, `NO_OPEN_BROWSER=1`, `CURSOR_API_KEY`, `--api-key` ([cli/reference/authentication.md](https://cursor.com/docs/cli/reference/authentication.md)).
- Permission tokens and precedence as in Q1; print mode can use write/shell tools gated by `permissions.allow`/`deny` and `--force` ([cli/reference/permissions.md](https://cursor.com/docs/cli/reference/permissions.md)).

**Mobile** `[official-documented]`
- iOS/iPadOS 26.0+, English; run agents on Cloud, Team Pool or My Machines; any cloud-agent model at max context; live stream; follow-ups; subagent transcripts; review/merge PRs (squash, ready, update branch, auto-merge, publish, close); slash commands, skills, automations; Design Mode; dictation; voice chat incl. Siri; MCP selection; push + Live Activities for up to 8 agents; cache-first ([cloud-agent/mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)).
- Android: `cursor.com/agents` PWA; native app planned ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).

**Explicitly not supported (consolidated)** `[official-documented]`
- Legacy Privacy Mode for Cloud Agents; Cloud Agents need cloud code storage ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- SSE inside cloud agents is **unverified**: the capabilities page says unsupported while the API schema enumerates `sse`; `mcp-remote` is not documented as supported ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md), [api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- Azure DevOps Server; ADO Automations/autofix/Security Agents; ADO personal-scope repos ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).
- Bitbucket Data Center Cloud Agents; Bitbucket DC comment commands and Autofix ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md)).
- GitLab Free (no project access tokens) ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)).
- Jira on Atlassian HIPAA/FedRAMP/Government Cloud ([jira.md](https://cursor.com/docs/integrations/jira.md)).
- Cloud Agents API v1 webhooks (coming soon) ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- Team Admin API keys in SDK/SDK Bridge ([sdk/typescript.md](https://cursor.com/docs/sdk/typescript.md), [sdk/bridge.md](https://cursor.com/docs/sdk/bridge.md)).
- Google Private Service Connect for private connectivity ([private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).
- Self-signed certificates, SSH, custom ports, IPv6-only endpoints on private connectivity paths ([private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).
- Apps feature outside cloud agents ("not supported locally") ([cloud-agent/apps.md](https://cursor.com/docs/cloud-agent/apps.md)).
- `sessionStart`, `sessionEnd`, `beforeMCPExecution`/`afterMCPExecution`, Tab hooks, `workspaceOpen`, and prompt-based hooks in cloud agents ([hooks.md](https://cursor.com/docs/hooks.md)).

### Inferences
- `[inferred]` Feature parity is a real product risk: a Studio that promises "works with your SCM" will behave differently on GitHub vs every other provider, and Azure DevOps/Bitbucket DC users would lose automations, autofix and security review entirely.
- `[inferred]` The absence of v1 webhooks means a hosted Studio cannot rely on Cursor pushing run events; it must poll the API or use the v0 surface until v1 webhooks ship.
- `[inferred]` Because cloud MCP OAuth tokens live on Cursor servers, a Studio cannot centrally revoke a user's MCP grant through its own IdP; it must go through Cursor or the provider.

### Gaps
- API rate limits and concurrency limits for the Cloud Agents API are not covered here (the API overview points to a rate-limits section that was not read in full).
- The CLI GitHub Actions page and JetBrains/Xcode integration pages were captured but not analyzed.
- No documented list of which models are available to cloud agents; the docs say "a curated selection" and "any model available for cloud agents" without enumerating.
- Whether `@cursor` comment triggers work on GitHub Enterprise Server PRs is implied by feature parity but not stated explicitly `[unverified]`.

## 4. What privacy, retention, authentication, authorization, SSO/SCIM/audit and security boundaries are documented?

### Takeaway
Cursor documents a layered enterprise story: **Privacy Mode + ZDR by default for most models**, explicit exceptions for a few non-ZDR models and BYOK; **SAML SSO, SCIM 2.0, three RBAC roles, MDM policies and a network tenancy header**; **audit logs with a named event taxonomy streamed to SIEM/S3**, separate from OpenTelemetry usage export; and cloud-agent isolation as per-agent Firecracker microVMs in a separate AWS account with no Cursor employee access. The boundaries that matter most for a Studio are: retention is team-scoped, data residency has named exclusions, and several agent guardrails are explicitly best-effort.

### Cited Findings

**Privacy and model data** `[official-documented]`
- With Privacy Mode on, "your code is never used for training by Cursor or other AI model providers"; Privacy Mode is on by default for Enterprise ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- Most models run under Cursor ZDR agreements; a few require provider retention — Claude Fable 5.2/5.1/5 — and requests fail until an admin approves the model's retention policy; opting in applies to the whole team; security-guardrail trips route to Claude Opus ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- BYOK: Cursor's ZDR does **not** apply; data handling follows the provider; the key is not stored on Cursor servers but is sent with every request because all requests route through Cursor's backend ([help/models-and-usage/api-keys.md](https://cursor.com/help/models-and-usage/api-keys.md)).
- Data Use page: with Privacy Mode off, Cursor may use codebase data, prompts, editor actions and snippets to improve/train; some inference providers may temporarily store inputs/outputs and delete after use; file contents are temporarily cached encrypted with client-generated keys that exist only for the request ([cursor.com/data-use](https://cursor.com/data-use)).
- Cloud Agents are "the only feature that requires Cursor to store code"; if policy prohibits code storage, don't enable Cloud Agents ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- Legacy Privacy Mode is not supported for Cloud Agents ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).

**Retention and deletion** `[official-documented]`

| Data | Where | Retention |
|---|---|---|
| Runtime workspace | Agent VM | Recycled after idle; timer refreshes on follow-up ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)) |
| VM snapshots (incl. cloned code) | Encrypted snapshot/cache layer | Rolling 30 days of inactivity; each start/resume extends; automatic deletion ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)) |
| Conversation state | Cursor backend, per-agent keys | Indefinite by default; deletable via Delete Agent API ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)) |
| Secrets/tokens | Encrypted credential stores | Until deleted/removed ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)) |
| Cloud Agent conversations (Enterprise policy) | Cursor backend | Indefinite or 90 days; custom windows early access ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)) |
| Shared transcripts (Enterprise) | Cursor | 30–365 days, off by default ([security-hardening.md](https://cursor.com/docs/enterprise/security-hardening.md)) |
| Account deletion | Cursor | Removed within 30 days ([security-hardening.md](https://cursor.com/docs/enterprise/security-hardening.md)) |
| Grok Bot conversations incl. Google data (adjacent product) | Cursor | Team retention 30–365 days ([cursor.com/privacy](https://cursor.com/privacy)) |

**Residency, encryption, certifications** `[official-documented]`
- US-only data residency: inference, processing and storage in the US; **10% uplift** on eligible model pricing; Enterprise, per team; up to two weeks to enable; EU/UK inference-only on request, broader EU in development ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- US residency exclusions: SSO/authentication (routes through WorkOS), BYOK, custom models via OpenAI-compatible base URL or third-party gateway, MCP/connectors and `@Web`, Bugbot/code review, shared links, Slack- or web-triggered Cloud Agents ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- Encryption: TLS 1.2+ in transit, AES-256 at rest with per-agent keys; Enterprise CMEK/BYOK for Cloud Agent server-side encryption ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md), [privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- Certifications: AIUC-1, ISO/IEC 27001:2022, ISO/IEC 42001:2023, SOC 2 Type II; at-least-annual third-party penetration testing; no infrastructure in China and no China-headquartered subprocessors ([cursor.com/security](https://cursor.com/security)).
- Trust Center at `trust.cursor.com` is cited for certificates, subprocessors and architecture; **the site returned "Page not found" to a non-JS fetch**, so its contents were not verified `[unverified]`.

**Authentication and authorization** `[official-documented]`
- SAML 2.0 SSO with Okta, Azure AD, Google Workspace, OneLogin; can require SSO and block password auth ([identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md)).
- SCIM 2.0 provisioning on Enterprise with SSO; org-level SCIM directory groups and Organization Groups; team membership can be driven by groups ([identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md)).
- RBAC: three team roles — Members, Admins, Unpaid Admins ([identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md)).
- MDM: allowed team IDs, allowed extensions, workspace trust; admin portal pushes extension policy to clients; MDM policy overrides client settings ([identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md)).
- Network tenancy restriction: `x-cursor-allowed-tenants` header, checked per request, not stored on the token; same account off-network is unrestricted ([tenancy-restriction.md](https://cursor.com/docs/enterprise/tenancy-restriction.md)).
- Cloud Agent access: org-level app install + per-user SCM account; "Access is inherited, never widened" — an agent can only reach repos the triggering user could already reach ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Protected Git Scopes: lock a Git org/group/namespace to the Cursor organization; requires Cursor team admin + Git provider admin; works with cloud and self-hosted GitHub and GitLab ([model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).
- Repository blocklist: applies to editor and CLI, **not** Cloud Agents, Automations or Bugbot ([model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).
- Service accounts: Enterprise; no seat; team usage pool; API keys shown once, rotatable, suspendable, archivable; spend limits; repo scope; team-level GitHub integration required ([service-accounts.md](https://cursor.com/docs/account/enterprise/service-accounts.md)).
- OIDC: Cursor-signed RS256 JWTs, issuer `https://api.cursor.com`, discovery `https://api.cursor.com/.well-known/openid-configuration`, JWKS `https://api.cursor.com/keys`; claim issuer `https://api.cursor.com/self-hosted/claims` ([identity.md](https://cursor.com/docs/cloud-agent/identity.md)).

**Audit and monitoring** `[official-documented]`
- Audit logs record sign-ins, membership, roles, API keys, settings, integrations, Cloud Agent environments, Grok Bot configuration and Origin repositories; each event has time, actor, client IP and surface when available; payloads carry identifiers/names/settings, **never prompts, agent output, generated code or credentials** ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- Not logged: agent responses/prompts/generated code; Grok Bot shell/browser actions (separate Action Recording over OTel); Cloud Agent **automation runs**; failed sign-in attempts; secret values (`cloud_agent_secret` records name/scope; `cloud_agent_secret_read` records only a count) ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- Access: Enterprise + admin; dashboard filter/export CSV; Admin API `GET /teams/audit-logs` and `GET /organizations/audit-logs`; default last 7 days, max 30-day window, 500 events/page, oldest first ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- Streaming: Splunk HEC, Sumo Logic, HTTPS webhook (Cribl, SentinelOne, Rapid7, Cortex XDR, Google Chronicle), S3; contact `hi@cursor.com` to set up ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- OpenTelemetry export is a **separate pipeline** for usage/activity telemetry (tokens, tool calls, cost, API/cloud-agent logs, Action Recording) ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- Event taxonomy includes auth/membership (`login`, `logout`, `add_user`, `remove_user`, `credentials_revoked`, `update_user_role`, `privacy_mode`, `user_spend_limit`), API keys/service accounts (`team_api_key`, `user_api_key`, `organization_api_key`, `service_account`, `api_key`, `cloud_agent_sub_token`), settings/integrations (`team_settings`, `team_rule`, `team_repo`, `team_hook`, `mcp_server_config`, `mcp_authentication`, `slack_account_link`, `team_marketplace`, `team_data_export_download`), directory/org groups ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).
- `credentials_revoked` records what happened to sessions, repo grants, cloud-agent git tokens and personal API keys; `sessions: retained`/`repo_grants: retained` means the person still belongs to another team in the org ([compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md)).

**Security boundaries and known limits** `[official-documented]`
- Cloud Agents: no Cursor employee access to code in VMs; access attempts monitored; VM account separate from production ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Auto-review is best-effort, not a security boundary; `.cursorignore` blocks agent read/context but "terminal and MCP tools can't honor it"; browser origin allowlist is best-effort and bypassable via link navigation, redirects and JS navigation ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md), [security-hardening.md](https://cursor.com/docs/enterprise/security-hardening.md), [browser.md](https://cursor.com/docs/agent/tools/browser.md)).
- Prompt injection/exfiltration is named as the main cloud-agent risk; layers are egress controls, Runtime Secrets, `.cursorignore`, PR review, signed commits, hooks ([cloud-agent/security.md](https://cursor.com/docs/cloud-agent/security.md)).
- Artifacts posted to GitHub use public unguessable URLs; blocking the artifact host disables uploads but not the session ([capabilities.md](https://cursor.com/docs/cloud-agent/capabilities.md), [security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).

### Inferences
- `[inferred]` For a Studio with regulated tenants, Cursor's team-scoped retention and per-team residency mean the Studio's tenant model must align 1:1 with Cursor teams; otherwise one tenant's retention or residency setting cannot be independently enforced.
- `[inferred]` Audit completeness is insufficient for "who changed what code": prompts/code/agent output are explicitly excluded, and automation runs are excluded. A Studio needing full provenance must add hooks or its own event log.
- `[inferred]` Because the repository blocklist does not cover Cloud Agents/Automations/Bugbot, it is not a sufficient "no sensitive repos" control; Protected Git Scopes is the stronger primitive.
- `[inferred]` The tenancy header is a network control, not an account control; it does not protect a user who takes their token off-network.

### Gaps
- Trust Center contents (certificates, subprocessor list, architecture) were not retrievable without JS; only the security page's summary is cited.
- No documentation found on audit-log retention period (how long Cursor keeps events), only query windows.
- No documented data-retention period for self-hosted worker logs or local MCP results.
- The exact set of "models with data retention" beyond the three Fable models is not enumerated in the page read.
- HIPAA BAA and cyber-safeguards pages were captured but not analyzed.
- No documented breach-notification SLA; `/security` only states critical incidents are communicated by email.

## 5. Which capabilities fundamentally cannot run entirely in a static browser, and what alternatives exist?

### Takeaway
Almost none of Cursor's execution plane can run in a static browser. The browser can host **UI, OAuth public-client flows, and streaming views**, but repository credentials, GitHub App installation tokens, webhooks, long-running VMs, dependency installation, terminal/computer use, secret storage, commit signing and org policy all require a **hosted backend**; local files, local MCP stdio servers and local terminal access require a **local companion** (desktop app, CLI worker, or SDK bridge).

### Cited Findings

**What the browser already does in Cursor's own architecture** `[official-documented]`
- Cursor Web (`cursor.com/agents`) is a client for cloud agents, sharing a backend with the desktop Agents Window and iOS ([cloud-agent/mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)).
- The in-editor Browser is "a secure web view ... controlled using an MCP server running as an extension"; it has a per-session random auth token and per-tab random ID; cookies/localStorage/IndexedDB persist per workspace ([browser.md](https://cursor.com/docs/agent/tools/browser.md)).
- MCP OAuth explicitly distinguishes web and desktop callbacks: `https://www.cursor.com/agents/mcp/oauth/callback` vs `http://localhost:8787/callback`; dynamic client registration registers as a public client with no secret ([mcp.md](https://cursor.com/docs/mcp.md)).
- OAuth uses authorization code + PKCE, RFC 9728 metadata and RFC 8707 resource indicators ([mcp.md](https://cursor.com/docs/mcp.md)).

**What requires a hosted backend** `[official-documented]`
- Cloud Agents: Cursor provisions and manages VMs, snapshots, startup, artifacts and capacity; code is stored in Cursor's AWS infrastructure ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md), [security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Builds: clone repos, run `install`, snapshot disk; requires server-side compute and storage ([builds.md](https://cursor.com/docs/cloud-agent/builds.md)).
- GitHub App: installation, repo/PR/issue webhooks, CI events and branch-protection reads happen outside the browser and require a server-side app identity ([github.md](https://cursor.com/docs/integrations/github.md)); that installation tokens are server-side is `[inferred]` from the app model.
- GitLab self-hosted OAuth app uses a **client secret** and a fixed redirect to `cursor.com/gitlab-connected` ([gitlab.md](https://cursor.com/docs/integrations/gitlab.md)).
- Slack/Teams/Jira/Linear/Notion integrations are installed into the vendor workspace and deliver events to Cursor's backend; Slack triggers only see public channels ([slack.md](https://cursor.com/docs/integrations/slack.md), [automations.md](https://cursor.com/docs/cloud-agent/automations.md)).
- Webhook triggers and Agent SDK custom channels need a public endpoint and signature verification (`publicEndpoint()`, `sharedSecretAuth()`, `hmacSignatureAuth()`) ([automations.md](https://cursor.com/docs/cloud-agent/automations.md), [agent-sdk/guides/webhooks.md](https://cursor.com/docs/agent-sdk/guides/webhooks.md)).
- Secret storage/redaction and OIDC minting are backend/socket services ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md), [identity.md](https://cursor.com/docs/cloud-agent/identity.md)).
- Signed commits use an HSM-backed Ed25519 key held by Cursor ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Artifact upload/storage to S3 and hosted Apps (dedicated hosting account, Postgres, Cursor sign-in) ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md), [cloud-agent/apps.md](https://cursor.com/docs/cloud-agent/apps.md)).
- Agent SDK managed hosting builds a pushed Git ref and serves it at a stable alias URL; sessions/`kv`/`files` persist across redeploys ([agent-sdk/deployment.md](https://cursor.com/docs/agent-sdk/deployment.md)).

**What requires a local companion** `[official-documented]`
- Local agent loop and filesystem access: SDK "Local" runtime runs the loop inline in your Node process and reads files from disk — but **inference still goes through Cursor's hosted models** ([sdk/typescript.md](https://cursor.com/docs/sdk/typescript.md)).
- SDK Bridge: a local loopback HTTP/1.1 server (`127.0.0.1`) with a per-process bearer token, Connect/protobuf `sdk.v1`, and callback servers for custom tools/stores ([sdk/bridge.md](https://cursor.com/docs/sdk/bridge.md)).
- CLI: `agent` runs locally; `agent login` stores credentials locally; permission tokens govern shell/read/write/webfetch/MCP ([cli/reference/authentication.md](https://cursor.com/docs/cli/reference/authentication.md), [cli/reference/permissions.md](https://cursor.com/docs/cli/reference/permissions.md)).
- Self-Hosted Machines: a worker (`agent worker start`) holds an outbound HTTPS connection; Cursor never connects into the network; local MCP servers and computer-use run on the worker ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- MCP stdio servers run as local child processes and cannot run in a browser ([mcp.md](https://cursor.com/docs/mcp.md)).
- Browser automation is an MCP server running as an extension, not a browser-native API ([browser.md](https://cursor.com/docs/agent/tools/browser.md)).
- Local sandbox (`sandbox.json`, AppArmor setup for remote environments and CLI) enforces filesystem/network limits on the machine ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md)).

**What requires a third-party service** `[official-documented]`
- SCM hosting and PR/CI: GitHub/GitLab/Bitbucket/Azure DevOps ([github.md](https://cursor.com/docs/integrations/github.md)).
- Chat/issue delivery: Slack, Teams, Jira, Linear, Notion ([slack.md](https://cursor.com/docs/integrations/slack.md), [notion.md](https://cursor.com/docs/integrations/notion.md)).
- Private networking: Tailscale or Cloudflare (or AWS PrivateLink) ([setup.md](https://cursor.com/docs/cloud-agent/setup.md), [private-connectivity.md](https://cursor.com/docs/cloud-agent/private-connectivity.md)).
- App publishing: Vercel for "Publish" from the Agents Window, or Cursor Apps hosting ([setup.md](https://cursor.com/docs/cloud-agent/setup.md), [cloud-agent/apps.md](https://cursor.com/docs/cloud-agent/apps.md)).
- Identity: customer IdP for SAML/SCIM; WorkOS is named as Cursor's identity provider for SSO ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).

### Inferences
- `[inferred]` A "static browser only" Studio can be at most a **control surface** for agents that already exist: it can render conversations, diffs, artifacts and approvals by calling a backend, but it cannot create a sandbox, clone a repo, hold a GitHub App key, receive a webhook, or run a terminal.
- `[inferred]` Three viable architectures follow from the docs: (a) **hosted control plane + Cursor Cloud Agents API** — fastest, but tenant isolation, retention and audit are constrained by Cursor's team model; (b) **local companion + user's own credentials** — maximum data control, but no unattended/background execution and no multi-user sharing; (c) **self-hosted worker pool + Cursor-hosted inference** — keeps code and tool execution in the operator's network while still depending on Cursor for the agent loop and models.
- `[inferred]` OAuth client secrets and API keys cannot safely live in browser JavaScript; any Studio that authenticates to GitHub/GitLab on behalf of users needs a server-side token exchange, even if the rest of the UI is static. This is a security inference, not a Cursor-documented constraint.
- `[inferred]` The MCP OAuth design (public client + PKCE + fixed redirects) is the one integration pattern that genuinely works browser-first; it is a model for how a Studio could let users connect third-party tools without a backend secret, provided the provider supports dynamic registration or a user-supplied client ID.

### Gaps
- Cursor does not document an official **embed/white-label/partner** API for third-party web apps; whether `cursor.com/agents` can be iframed or resold is `[unverified]`.
- No CORS, token-exchange or browser-session documentation for calling the Cloud Agents API from a web origin.
- No documented WebSocket/SSE limits or long-poll behavior for the web client.
- The practical offline/degraded behavior of Cursor Web is not documented beyond iOS being "cache-first" ([mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)).
- No documented static-export or self-hosted web UI option.

## 6. What is unverified or plan/version/account dependent?

### Takeaway
A large fraction of the surface is gated by **plan tier, beta status, region, or client version**, and several documented details are internally inconsistent or unretrievable. The most consequential dependencies for a Studio plan are Cloud Agents (Premium+), Team Pools/service accounts/audit/CMEK/residency/model controls (Enterprise), and the beta status of the API, Bitbucket Cloud, Azure DevOps, Notion, Tailscale and Origin.

### Cited Findings

**Plan gating** `[official-documented]`
- Cloud Agents: Premium and higher, Teams, Enterprise, or legacy paid; **Free and Plus excluded** ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- Jira integration: Cursor Teams and Enterprise only ([jira.md](https://cursor.com/docs/integrations/jira.md)).
- Notion: Cursor users on Notion Business or Enterprise ([notion.md](https://cursor.com/docs/integrations/notion.md)).
- Tailscale: beta on Teams and Enterprise ([tailscale.md](https://cursor.com/docs/integrations/tailscale.md)).
- Team Pools: Cursor Enterprise + service-account API key ([self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md)).
- Service accounts, audit logs, CMEK, US data residency, model access control, BYOK restriction, MCP allowlist, Protected Git Scopes: Enterprise ([service-accounts.md](https://cursor.com/docs/account/enterprise/service-accounts.md), [compliance-and-monitoring.md](https://cursor.com/docs/enterprise/compliance-and-monitoring.md), [privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md), [model-and-integration-management.md](https://cursor.com/docs/enterprise/model-and-integration-management.md)).
- Origin: Premium+, Super, Ultra, Teams, Enterprise, legacy paid; **not Free/Plus** ([origin.md](https://cursor.com/docs/origin.md)).
- Team marketplaces: 1 on Teams, unlimited on Enterprise ([plugins.md](https://cursor.com/docs/plugins.md)).
- Computer use in cloud agents: "available to enterprise teams only" for the team setting ([settings.md](https://cursor.com/docs/cloud-agent/settings.md)).

**Beta / early-access status** `[official-documented]`
- Cloud Agents API **v1 public beta**; v0 legacy still available ([api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- Bitbucket Cloud **public beta**; Azure DevOps **public beta** ([bitbucket.md](https://cursor.com/docs/integrations/bitbucket.md), [azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).
- Notion **beta**; Tailscale **beta**; Origin **early beta** ([notion.md](https://cursor.com/docs/integrations/notion.md), [tailscale.md](https://cursor.com/docs/integrations/tailscale.md), [origin.md](https://cursor.com/docs/origin.md)).
- Cursor-configured Dockerfiles: **private beta for Enterprise**, request via account rep/`hi@cursor.com` ([setup.md](https://cursor.com/docs/cloud-agent/setup.md)).
- Custom Cloud Agent retention windows: **early access for select Enterprise teams** ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Browser Origin Allowlist: must be enabled for the org before it appears ([browser.md](https://cursor.com/docs/agent/tools/browser.md)).

**Version and platform dependencies** `[official-documented]`
- Read Access requires Cursor **3.23+**; live workspace requires **3.25+** and Linux; admin extension policy requires client **2.1+**; iOS/iPadOS **26.0+**; Agent SDK requires **Node 22.13+** and "never Bun" ([run-modes.md](https://cursor.com/docs/agent/security/run-modes.md), [self-hosted.md](https://cursor.com/docs/cloud-agent/self-hosted.md), [identity-and-access-management.md](https://cursor.com/docs/enterprise/identity-and-access-management.md), [mobile.md](https://cursor.com/docs/cloud-agent/mobile.md), [agent-sdk/deployment.md](https://cursor.com/docs/agent-sdk/deployment.md)).
- Cursor-hosted VMs are x86_64 only; arm64 needs self-hosted workers ([choose-runtime.md](https://cursor.com/docs/cloud-agent/self-hosted/choose-runtime.md)).
- Azure DevOps integration is `dev.azure.com` only; `*.visualstudio.com` URLs must be converted ([azure-devops.md](https://cursor.com/docs/integrations/azure-devops.md)).

**Documentation inconsistencies / unverified items** `[observed]` / `[unverified]`
- Four `https://cursor.com/docs/help/...` URLs that I constructed returned 404; the same pages exist at `https://cursor.com/help/...` and were fetched successfully. The docs index uses the latter form.
- Service accounts doc example posts to `https://api.cursor.com/agents` while the v1 reference uses `/v1/agents`; the doc does not say which is current ([service-accounts.md](https://cursor.com/docs/account/enterprise/service-accounts.md), [api/endpoints.md](https://cursor.com/docs/cloud-agent/api/endpoints.md)).
- `https://cursor.com/docs/ips.json` is a JSON endpoint under a docs path; the docs cite it but the payload was not fetched here ([security-network.md](https://cursor.com/docs/cloud-agent/security-network.md)).
- Trust Center (`trust.cursor.com`) returned "Page not found" to a non-JS fetch; certificates/subprocessors/architecture could not be verified ([cursor.com/security](https://cursor.com/security)).
- The claim that managed Cloud Agents "cover the requirements of over 80% of our customers" is vendor marketing without a cited study ([choose-runtime.md](https://cursor.com/docs/cloud-agent/self-hosted/choose-runtime.md)).
- Model availability for cloud agents is described only as "a curated selection"; no list is published ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- EU/UK residency is "on request" / "in active development"; no date or commitment ([privacy-and-data-governance.md](https://cursor.com/docs/enterprise/privacy-and-data-governance.md)).
- Android native app is "planned" with no date ([cloud-agent.md](https://cursor.com/docs/cloud-agent.md)).
- Whether Cursor Web supports every desktop feature (e.g. local terminal, local files) is not documented; the mobile page describes web as a full cloud-agent client, not a local one ([mobile.md](https://cursor.com/docs/cloud-agent/mobile.md)) `[unverified]`.

### Inferences
- `[inferred]` A Studio plan should treat every integration except GitHub + Slack as **conditional**: available only on certain plans, in beta, or with a reduced feature set.
- `[inferred]` Enterprise-only controls (audit streaming, CMEK, residency, SCIM, model governance) are the actual product boundary for regulated customers; a Studio built on non-Enterprise Cursor cannot offer those guarantees itself.
- `[inferred]` Beta status of the Cloud Agents API plus "webhooks coming soon" implies the API contract may change; a Studio should isolate Cursor API calls behind an adapter and version-pin assumptions.

### Gaps
- No public SLA/uptime, status-page history, or incident-notification commitment was found in the pages read.
- Pricing, included usage and overage behavior were not analyzed (models-and-pricing captured but not read in full).
- No documentation found for rate limits, idempotency keys, or pagination limits beyond the audit-log 500/page and 30-day window.
- No documentation found for data-processing locations beyond US residency and the statement that personal data may be processed in the US and other jurisdictions ([cursor.com/privacy](https://cursor.com/privacy)).
- Whether Cursor offers a partner/reseller/embedded agreement for a hosted Studio is not documented in the pages read `[unverified]`.
