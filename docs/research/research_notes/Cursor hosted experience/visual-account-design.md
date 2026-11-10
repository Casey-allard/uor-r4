# Cursor visual/interaction design and account/admin surfaces

Scope: public Cursor web surfaces and official Cursor documentation, observed on **2026-11-09** (America/New_York). No account was used, no login was attempted beyond loading public URLs, and no product code was changed. The latest changelog entry at capture time was dated **Nov 6, 2026** ("Voice Chat with Agents"), so the public surface was current within three days.

Evidence labels used below:

- **[Observed]** — measured or read from a public page in this session's fresh headless Chrome (Chrome 157.0.8103.39, fresh `/tmp` profile, no user profile), or read from a screenshot saved under `local/kimi/cursor-hosted-research/visual-account-design/`.
- **[Official]** — stated in Cursor's own docs/help/changelog/pricing copy at the linked URL.
- **[Inferred]** — my synthesis; not a fact stated by Cursor.
- **[Unverified]** — not observable without an account, a paid team, or an accessibility audit.

Measurement method: Chrome DevTools Protocol at 1440×1000 and 390×844 CSS px, DPR 1; `getComputedStyle` values read from the live public page; full-page screenshots captured with `captureBeyondViewport`. Computed colors are reported as `rgb()` and hex equivalents; no CSS was copied from source files and no values are estimated from marketing imagery. Exact values apply to the date and viewport measured and may change.

A source-set caveat that matters for any Studio plan: this 2026 Cursor surface is presented as part of **"The X Subscription."** Cursor docs route Teams billing, seats, and identity to `grok.com/dashboard`, and the pricing page mixes Cursor and Grok/X benefits. I report those labels verbatim and flag the inconsistency rather than resolving whether Cursor is a standalone product or an X bundle. See "Source conflicts and ambiguity" under Q5.

---

## Q1. What exact public visual and interaction patterns can be observed or officially established?

### Takeaway

Cursor's public web is a **dark-only, warm-neutral system**: near-black `#14120B` canvas, off-white `#EDECEC` text, a single orange accent (`#EB5600`) used at least for focus rings, restrained 4 px card radii, pill-shaped primary actions, and a compact 52 px top bar. Marketing uses **CursorGothic**; docs uses **cursorSans** with **berkeleyMono** for code. At 390 px the marketing header collapses to logo + "Sign in" + hamburger and the docs sidebar collapses to logo + search icon + hamburger.

### Cited Findings

**Color and theme**

- **[Observed]** Marketing body background `rgb(20, 18, 11)` = `#14120B`; body text `rgb(237, 236, 236)` = `#EDECEC`; `color-scheme: dark`; `html` background is transparent and the body carries the color. Same values on `/pricing`, `/docs`, `/changelog`. ([homepage](https://cursor.com/), [pricing](https://cursor.com/pricing), [docs](https://cursor.com/docs))
- **[Observed]** Raised surfaces use `rgb(27, 25, 19)` = `#1B1913` with a 1 px `rgb(42, 40, 34)` = `#2A2822` border (docs cards, pricing plan cards, nav dropdown panels). Pricing plan card measured 318×671 px with 4 px radius and 13.4–15 px padding. ([pricing](https://cursor.com/pricing), [docs](https://cursor.com/docs))
- **[Observed]** Docs focus ring on the skip link is a two-layer shadow: `rgb(255,255,255) 0 0 0 2px, rgb(235,86,0) 0 0 0 4px`; the orange is `#EB5600`. This is the clearest measured accent color on the public site. ([docs](https://cursor.com/docs))
- **[Observed]** Marketing exposes semantic color variables that alias a theme layer (`--color-bg`, `--color-text-primary`, `--color-text-muted`, `--color-border`, `--color-accent`), plus timeline colors `#DFA88F` (thinking), `#9FC9A2` (grep), `#9FBBE0` (read), `#C0A8DD` (edit). These are runtime token names/values, not a complete palette. ([homepage](https://cursor.com/))
- **[Observed]** Docs exposes `--radius: .625rem` (10 px) and light-theme token values (`--muted-foreground: #7A7974`, `--border: #E2E2DF`) even though the rendered page is dark. The rendered docs cards used 4 px, so the 10 px token is not the card radius on that page. ([docs](https://cursor.com/docs))
- **[Observed]** All sampled public pages render dark, and no web theme toggle appeared in the marketing header or docs shell. **[Unverified]** No official page documents a web light/dark toggle; theme documentation is editor-side only (see Q4). ([themes help](https://cursor.com/help/customization/themes))

**Typography**

| Element | Desktop 1440 (observed) | Mobile 390 (observed) |
|---|---|---|
| Body font | `CursorGothic, "CursorGothic Fallback", system-ui, "Helvetica Neue", Helvetica` | same |
| Body size / line-height | 16 px / 24 px | 15 px / 22.5 px |
| Marketing H1 | 26 px / 32.5 px, weight 400, letter-spacing −0.325 px | 24.375 px / 30.47 px, −0.305 px |
| Marketing section H2 | 36 px / 43.2 px, weight 400 | 33.75 px / 40.5 px |
| Pricing H1 ("Pricing") | 52 px / 59.8 px, weight 400, −1.3 px | not measured |
| Pricing closing CTA | 72 px, weight 400 | not measured |
| Docs font | `cursorSans` (400/700 loaded), `berkeleyMono` for mono | same |
| Docs H1 | 33.75 px / 40.5 px, weight 400, −0.675 px | 33.75 px / 40.5 px |
| Docs body | 15 px / 24.375 px | same |
| Docs content H2 | 20.625 px / 26.8 px | same |
| Docs sidebar group labels | 12 px | hidden/collapsed |
| Docs code font | `berkeleyMono` 400 (loaded on demand) | same |

Sources: measured on [homepage](https://cursor.com/) and [docs](https://cursor.com/docs); screenshots `desktop-home-full-1440-2026-11-09.png`, `mobile-home-full-390-2026-11-09.png`, `desktop-docs-full-1440-2026-11-09.png`, `mobile-docs-full-390-2026-11-09.png`.

- **[Observed]** Marketing H1 is weight 400, not bold; hierarchy comes from size and spacing. The pricing H1 is much larger (52 px) than the homepage H1 (26 px), so the marketing system is page-specific rather than one fixed H1 size. ([homepage](https://cursor.com/), [pricing](https://cursor.com/pricing))
- **[Observed]** Marketing supports a 72 px display size for the closing CTA ("Get started with Cursor."), showing an intentional display step beyond H1. ([pricing](https://cursor.com/pricing))
- **[Observed]** Docs group labels are 12 px but are marked up as `H2` elements *before* the page `H1`; the rendered heading order on `/docs` is `H2 × 13` (sidebar groups) then `H1: Cursor Documentation`. This is an accessibility/heading-outline defect in the public docs shell, not a visual one. ([docs](https://cursor.com/docs))

**Layout, hierarchy, spacing**

- **[Observed]** Marketing header: 52 px tall, 20 px horizontal padding, full-width nav centered, right cluster "Sign in" / "Contact sales" / "Download" (pill). Nav items are 14 px, 34 px tall, 15 px horizontal padding, 0 radius; dropdown disclosure chevrons are separate 10×34 px buttons. ([homepage screenshot](https://cursor.com/), `desktop-home-1440x1000-2026-11-09.png`)
- **[Observed]** Marketing content gutter: 70 px at 1440 (content H1 x=70, width 658); 19 px at 390 (H1 x=19, width 353). Hero section top padding 112 px desktop / 105 px mobile; bottom 67.2 px / 63 px. ([homepage](https://cursor.com/))
- **[Observed]** Marketing page height: 8,249 px desktop vs 10,046 px mobile for the same content; mobile stacks rather than hides sections. ([homepage](https://cursor.com/))
- **[Observed]** Pricing: "Individual / Teams" segmented control 207×42 px with a pill track (`oklab(... / 0.05)` over the page bg) and a 113×38 px selected segment on `#1B1913`; plan cards in a four-up row at 1440 with the first card at 318×671. The capture showed Free, Plus, Premium ("Most popular" tag), and Super; docs also document Ultra, whose position at 1440 was not captured. ([pricing](https://cursor.com/pricing), [models & pricing](https://cursor.com/docs/models-and-pricing))
- **[Observed]** Plan CTA labels are "Get started", "Get Plus", "Get Premium", "Get Super"; the primary CTA is a light pill (bg `#EDECEC`, text `#14120B`), secondary actions are transparent with text only. ([pricing](https://cursor.com/pricing))
- **[Observed]** Docs is a three-column shell: header 49 px; left sidebar 309 px wide (x=0–309, card background `#1B1913`), content starts at x=309 and the H1 at x=395, right rail with "On this page" and "Copy page". Sidebar items are 260×29 px; cards are 224×160 px, 4 px radius, 1 px `#2A2822` border. ([docs](https://cursor.com/docs), `desktop-docs-full-1440-2026-11-09.png`)
- **[Observed]** Docs header: logo + "Docs / API / Learn / Help" tabs (36 px tall), centered search button 320×36 px with 8 px radius and placeholder "Search docs... ⌘K", right "Sign in" / "Download". ([docs](https://cursor.com/docs))
- **[Observed]** Docs right rail copy: "On this page", "Copy page"; docs content cards use the labels "Get started", "Models & Pricing", "Changelog", "Understand your code", "Plan and build features", "Find and fix bugs", "Review changes". ([docs](https://cursor.com/docs), screenshot)
- **[Observed]** Changelog uses a left date rail ("Nov 6, 2026"), a narrow article column, a small category label "Changelog", a large article H1, and section H2s ("Start a call", "Talk while the agent works", "Powered by Grok Voice"). The article column measured visually at roughly 640 px inside a 1440 page; this was read from the screenshot, not `getComputedStyle`, so treat the width as approximate. ([changelog](https://cursor.com/changelog), `desktop-changelog-1440x1000-2026-11-09.png`)
- **[Observed]** Marketing body copy can be muted via alpha (`rgba(237,236,236,0.55)` for the "Software creation is changing…" paragraph); composited over `#14120B`, that is approximately **5.5:1**, which passes WCAG AA for normal text. Do not read the raw alpha as 15.9:1. ([homepage](https://cursor.com/))

**Borders, radii, shadows**

- **[Observed]** Marketing cards: 4 px radius, 1 px `#2A2822` border, no shadow. Plan cards: 4 px radius, no border in the measured style (background contrast only). Search field: 8 px radius. Primary buttons and segmented controls: fully rounded pills (computed radius `3.35544e+07px`). ([pricing](https://cursor.com/pricing), [docs](https://cursor.com/docs))
- **[Observed]** No drop shadows were found on the sampled cards; depth is carried by background steps (`#14120B` page → `#1B1913` surface) and 1 px borders. ([homepage](https://cursor.com/), [pricing](https://cursor.com/pricing))
- **[Observed]** The 404 page uses a large perspective "404" illustration with a thin yellow/orange edge highlight, black/near-black background, body copy "The page you're looking for has wandered off, but don't worry!", "Let's get you back on track.", and a small "Take me home" button. ([cursor.com/sign-in](https://cursor.com/sign-in) returns this 404, not a login form)

**Icons and imagery**

- **[Observed]** Header logo is a white geometric mark plus the uppercase wordmark "CURSOR"; mobile keeps the mark + wordmark at 390. Nav uses text plus `↓` glyphs for disclosures, `→` for links, `✓` for plan features, `×` for the cookie close, and a two-line hamburger at 390. No icon font or third-party icon set was identifiable from the public DOM. ([homepage](https://cursor.com/), screenshots)
- **[Observed]** Marketing uses a large embedded product screenshot/recording in the hero (Agents Window with task list, chat, diff; "Grok 4.7" in the composer) and a logo wall (Stripe, Datadog, Linear, Nvidia, Figma, Ramp, Decagon, Adobe), each image with an `alt` attribute. ([homepage](https://cursor.com/))
- **[Official]** The changelog documents a waveform button labeled **"Start voice chat"** that "sits where Send usually is, next to the microphone button for dictation"; voices include Ara, Eve, Leo, Rex, and Sal. ([changelog, Nov 6 2026](https://cursor.com/changelog))

**Motion**

- **[Observed]** Homepage: 139 elements with a non-zero `transition-duration` (unique values 0.1, 0.12, 0.14, 0.15, 0.2, 0.25, 0.5, 0.6 s) and 54 with a running `animation` (0.25–3.1 s). Docs: 260 transitioning elements (0.1–0.25 s) and 3 animations (0.4 s). (`interaction-home-1440.json`, `interaction-docs-1440.json`; local evidence, not committed) ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Observed]** Under emulated `prefers-reduced-motion: reduce`: homepage animations dropped from 54 to 39, but transitions stayed at 139; docs stayed at 3 animations / 260 transitions. Partial reduced-motion handling exists on marketing; no CSS-level reduction was observed in docs. This measures computed CSS only, not whether JS pauses motion. ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Inferred]** Transitions are used mainly for hover/state affordances (0.1–0.25 s); longer 1–3 s animations belong to hero/product demos. Motion is not a primary hierarchy device.

**Focus, keyboard, accessibility semantics**

- **[Observed]** Marketing: pressing `Tab` focuses the visually revealed skip link "Skip to content" (113×27 px), then the logo link; both show `outline: 2px solid #EDECEC`, `outline-offset: 2px`, no box-shadow. ([homepage](https://cursor.com/), keyboard test in `interaction-home-1440.json` scope)
- **[Observed]** Docs: `Tab` focuses "Skip to main content" (182×40 px, revealed from a 1×1 px hidden state) with a white 2 px inner ring plus orange 4 px outer ring; the second focusable link shows only `outline: auto 1px` and no shadow, so focus treatment is inconsistent within docs. ([docs](https://cursor.com/docs))
- **[Observed]** Homepage: exactly one `H1`, 28 images with zero missing `alt`, 62 `aria-label`/`aria-labelledby` attributes, 9 landmark elements, and a `Skip to content` link. Docs: one `H1` (but preceded by 13 sidebar `H2`s), 25 images with zero missing `alt`, 102 aria labels, 4 landmarks, and a visually hidden skip link. ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Observed]** Docs exposes a cookie dialog with `role="dialog"`, accessible name "Privacy Preference Center", switches and a live status region; the banner copy is "Essential cookies keep the site working and stay on. Optional cookies help with performance and advertising — accept, reject, or manage them." with buttons "Reject All", "Accept All Cookies", "Cookies Settings". Desktop banner sits bottom-right; mobile is full-width with stacked buttons. ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Observed]** Many pointer targets are compact: marketing nav 34 px tall, docs header 36 px, docs sidebar 29 px, logo link 24 px tall, disclosure chevrons 10×34 px. These exceed WCAG 2.5.8's 24×24 minimum for most items but are below the 44×44 touch-target guideline; the 10 px chevron is the notable exception. ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Unverified]** No public accessibility statement, VPAT, or WCAG conformance claim was found on cursor.com in this session. A search for "Cursor accessibility statement WCAG" returned unrelated sites only. Absence is a gap, not evidence of non-conformance.

**Public route behavior (important for evidence boundaries)**

- **[Observed]** `https://cursor.com/sign-in` returns a **404**, not a sign-in form: title "Cursor - The best way to code with AI", body copy quoted above, "Take me home" button. ([cursor.com/sign-in](https://cursor.com/sign-in), `desktop-signin-1440-2026-11-09.png`)
- **[Observed]** `https://cursor.com/dashboard` and `https://cursor.com/agents` redirect to `https://authenticator.cursor.sh/?client_id=…&redirect_uri=https%3A%2F%2Fcursor.com%2Fapi%2Fauth%2Fcallback…`, which served a bot-check interstitial titled "Just a moment..." with "Performing security verification" and then "Verification successful. Waiting for authenticator.cursor.sh to respond". No login form, dashboard, or agent UI was observable. ([dashboard](https://cursor.com/dashboard), [agents](https://cursor.com/agents), `desktop-dashboard-1440-2026-11-09.png`, `desktop-agents-1440-2026-11-09.png`)
- **[Inferred]** The auth host uses a separate design system (system-ui, 40 px/600 H1, pure black) and is not evidence of Cursor product UI. It is also a gate that a fresh headless profile cannot pass without a real interactive session; I did not attempt to bypass it.

### Inferences

- Cursor's public design language is deliberately **low-chrome**: near-black canvas, one surface step, hairline borders, 4 px radii, no shadows, and a single warm accent. Hierarchy is typographic (size, spacing, weight 400) rather than color or elevation.
- The orange accent is used sparingly and functionally (focus ring, "Most popular" tag), not as a large fill. A hosted Studio could adopt the same discipline: neutral shell, one accent reserved for focus/selection/warnings, and no gradient decoration.
- The 52 px header / 70 px gutter / 4 px card system is a coherent desktop grid; the 390 px layout preserves the same order and simply stacks (header 56 px, gutter 19 px, 4-column logo wall). That is a simpler responsive strategy than a separate mobile design.
- Docs is a different sub-brand: sans-serif `cursorSans` rather than `CursorGothic`, 3-column reference layout, 10 px radius token, and denser 29 px rows. A Studio product with both marketing and docs should decide whether to unify these or keep the marketing/docs split intentional.
- **Recommendation, not evidence:** for a UOR-R4 Studio, borrow the token structure (canvas, surface, border, text, muted, accent, radius) and the focus-ring pattern, not Cursor's exact fonts or brand colors. The measured white+orange focus ring is unusually visible on dark and is the strongest accessibility pattern observed.

### Gaps

- No public page exposes the full design-token set, font files, weights, or spacing scale. Only computed values on sampled elements are known.
- Hover, active, disabled, loading, error, empty, and validation states were not captured; only default and keyboard-focus states.
- No mobile pricing capture; the four-up plan row's overflow behavior at 390 is unverified.
- No tablet breakpoint (768/1024) measured, so the exact responsive breakpoints are unknown.
- The changelog article column width is approximate (screenshot reading), not a computed measurement.
- Marketing focus behavior was tested with a single `Tab` sequence; full keyboard order, focus trapping in menus/dialogs, and screen-reader announcements were not audited.
- The cookie dialog's actual consent behavior and persistence were not tested.

---

## Q2. What account/dashboard/settings/admin menus, sections, controls, default values, plans/limits, settings texts, team administration and destructive actions are documented?

### Takeaway

Cursor documents a **team dashboard** with ten top-level areas (Overview, Settings, Members, Audit Log, Integrations, Cloud Agents, Automations, Active Directory Management, Usage, Billing & Invoices), a **three-role model** (Member, Admin, Unpaid Admin), **four seat types** (Premium, Super, Ultra, Free; legacy Standard/Premium), org-level SSO/SCIM/groups above teams, and a personal layer (profile, privacy, API keys, billing, spend limits/alerts, account deletion). Exact dashboard nav labels are not shown publicly; the docs give section headings and in-page control labels. Anything behind `cursor.com/dashboard` is unverified visually.

### Cited Findings

**A. Team dashboard — documented sections** ([Dashboard docs](https://cursor.com/docs/account/teams/dashboard))

- **[Official]** The dashboard "lets you access billing, set up usage-based pricing, and manage your Team." Documented top-level sections:
  - **Overview** — "quick summary of your team's activity, usage statistics, and recent changes."
  - **Settings** — team-wide preferences and security.
  - **Members** — invite users, control access, role-based permissions, monitor activity.
  - **Audit Log** — security/administrative events; Enterprise only, admins only.
  - **Integrations** — VCS, project management, developer services.
  - **Cloud Agents** — status, logs, resource usage.
  - **Automations** — recurring cloud agents; Bugbot, Security Agents, PR Routing & Approval.
  - **Active Directory Management** — enterprise SSO/provisioning.
  - **Usage** — AI requests, model usage, resource consumption.
  - **Billing & Invoices** — subscription, payment methods, billing history, invoices, usage-based pricing.
- **[Official]** The Settings section includes these documented subsections:
  - **Privacy Settings** — data sharing/training, team-wide privacy enforcement.
  - **Usage-Based Pricing Settings** — enable usage-based pricing, monthly team spending limit, "only admins can modify these settings."
  - **Cursor Router** — enable routing for Auto, choose optimization modes members can select, routed-model visibility, Auto as team default.
  - **Team Marketplaces** — import private marketplaces from GitHub or use Default; **Marketplace Access** for whole team / selected Organization Groups / SCIM directory group; **Allow Members to Publish**; Teams 1 marketplace, Enterprise unlimited.
  - **Sync Skills for Cloud Agents** — opt-in sync of `~/.cursor/skills/`; admins can disable under **Security & Identity**.
  - **Bedrock IAM Role** — AWS Bedrock IAM roles.
  - **Single Sign-On (SSO)**.
  - **Protected Git Scopes** — lock a Git org/group/namespace to the Cursor organization.
  - **Cursor Admin API Keys**.
  - **Active Sessions** — monitor/manage active user sessions.
  - **Invite Code Management**.
  - **API Endpoints** — REST endpoints; all Team and Enterprise except AI Code Tracking (Enterprise).
- **[Official]** Enterprise-only Settings subsections:
  - **Model Access Control** — "Team Settings → Models"; org can widen per cohort via "Organization → Groups → Models"; team and group access combine as a union (most-permissive wins).
  - **Enhanced Spend Limits** — per-member overrides, group limits via directory sync, default per-member caps.
  - **Auto Run Configuration** — automatic command execution policy.
  - **Repository Blocklist**.
  - **MCP Configuration** — MCP server trust management.
  - **Cursor Ignore Configuration**.
  - **.cursor Directory Protection**.
  - **AI Code Tracking API** — Enterprise only.
  - **Audit Log** — "tamper-proof records"; filter by application; CSV export includes application; Enterprise only.
  - **SCIM** — Enterprise.
- **[Inferred]** These are documentation headings, not confirmed left-nav labels. A Studio inventory should treat them as the information architecture Cursor documents, not as a screenshot-verified menu.

**B. Members, roles, and seats** ([Members docs](https://cursor.com/docs/account/teams/members))

- **[Official]** Roles:
  - **Member** (default): full paid features, no billing/admin dashboard, can see own usage and remaining included usage.
  - **Admin**: full features, add/remove members, modify roles, SSO, top-ups and monthly additional usage limit, team analytics.
  - **Unpaid Admin**: not billable, no paid features, same administrative capabilities; requires at least one paid user.
- **[Official]** Seat tiers on The X Subscription: **Premium $30/user/mo** (Premium pool), **Super $100/user/mo** (3.75× Premium), **Ultra $200/user/mo** (12.5× Premium), **Free $0** (no team allowance; draws from personal plan/free allowance; assigned by email invite or Seat menu, not invite links). Legacy Teams: **Standard $40**, **Premium $120 (5× Standard)**.
- **[Official]** Exact member-management labels: **"Invite Members"**, **"Invite Link"**, context menu → **"Remove"**, context menu → **"Change role"**, **"Seat"** menu next to a name under **Members** (legacy path shown as `grok.com/dashboard`), **"Show org admins"** toggle, **"View API keys"** in a member's context menu, **"Allow Personal API Key Creation"** under the **"Personal API Keys"** section of the **"Team"** tab under **"API keys"**.
- **[Official]** Role comparison table (Member / Admin / Unpaid Admin): Members can invite members and set usage controls; Admins/Unpaid Admins can remove members, change roles, access admin dashboard, configure SSO/security, manage billing, view analytics, manage access, set usage controls.
- **[Official]** Constraints and billing: at least one Admin and one paid member at all times; removing a member does not delete their Cursor account or data; deleting the entire team permanently deletes all associated data; removed members who used credits keep the seat until cycle end; mid-cycle adds are pro-rated; role changes bill from the change date; monthly/yearly renewal stays on the original signup date.
- **[Official]** Seat changes: upgrades immediate with pro-rated charge; downgrades apply at period end and can be cancelled by picking the current tier again.
- **[Official]** Yearly billing: go to `cursor.com/dashboard/billing`, click **"Upgrade Now"** on the green banner; save 20%; no yearly→monthly mid-plan switch.
- **[Observed]** The public `/pricing` page shows a Free/Plus/Premium/Super row; the docs table adds Ultra and Teams. See Q4 for the plan conflict.
- **[Inferred]** "Member" is both a role and a generic word for a person; a Studio UI should separate "role" from "seat" the way Cursor does, because the same person can be an Admin on a Free seat.

**C. Identity: SSO, SCIM, organizations** ([SSO](https://cursor.com/docs/account/teams/sso), [SCIM](https://cursor.com/docs/account/teams/scim), [Organizations](https://cursor.com/docs/enterprise/organizations))

- **[Official]** SSO is **SAML 2.0**, no extra cost on Teams/Enterprise; exact UI: go to **Single Sign-On (SSO) settings** at `cursor.com/dashboard/team-settings#single-sign-on-sso`; find section **"Single Sign-On (SSO)"**; click **"Configure"** next to **"SSO-Provider Connection Settings"**; click **"Configure"** next to **"Domain Verification Settings"** to verify domains. IdP setup uses JIT provisioning. Once domain verification + connection are active, users on the domain are required to sign in with SSO; there is **no separate enforcement toggle**. Multiple domains must each be verified separately.
- **[Official]** SCIM 2.0 requires Enterprise + active SSO. Exact path: `cursor.com/dashboard/members?subtab=active-directory`, or dashboard settings → **"Members & Groups"** tab → **"Directory Groups"** subtab. SCIM users/groups are **read-only** in Cursor; changes sync from the IdP in real time; group spend limits use the highest applicable limit; roles are configured in Cursor unless a directory group is mapped to an Organization Group with a role, in which case sync manages roles and dashboard editing is locked.
- **[Official]** Organizations sit above teams: multiple teams, one default/root team, org-level SSO, org-level SCIM directory, Organization Groups, org admin role. Exact labels: **"SCIM Directory"** section in Organization **Settings** → **"Sync Directory"**; **"Auto Add Users to Root Team"** defaults **on**; **"Merge into default IDP"** from the **⋯** menu on an identity-provider card; confirmation checks prerequisites and asks the admin to acknowledge the merge **can't be undone**; while running, the card shows a **"Merging…"** badge and settings are locked.
- **[Official]** Org roles: org admins manage organization settings/membership/shared identity and can view teams; team admins manage their own teams; team admin does not grant org admin. On linked teams, org admins can manage Team MCP servers and team plugin variables like team admins.
- **[Official]** Limits combine **most-permissive-wins**: team default → per-user override → directory group → organization group; spend limits use the highest value; per-user override wins over defaults and directory groups.

**D. Analytics and audit** ([Analytics](https://cursor.com/docs/account/teams/analytics), [Compliance & monitoring](https://cursor.com/docs/enterprise/compliance-and-monitoring))

- **[Official]** Analytics at `cursor.com/dashboard/analytics`; team admins see all users, members see themselves (and leaderboard entries). Data collected from client 1.5+. Filters: up to **10 users** and **90 continuous days**, plus active-directory groups and dates; gear icon selects **timezone** and whether **weekends** are shown. CSV: each chart has a bottom-right button; page header download icon downloads all charts.
- **[Official]** Documented charts: AI Share of Committed Code, Agent Edits, Tab Completions, Messages Sent, Active Users, Daily Usage (All/Tab/Agent/DAU), Usage Leaderboard (Chats, Tab Completions, Agent Lines of Code; top ten + filtered users), Repository Insights, Conversation Insights, Compare, Cloud Agent Usage (Agents Created, Pull Requests, Lines of Code), Cloud Agent Top Repositories, Top Cloud Agent Users, Client Versions.
- **[Official]** Conversation Insights is **enabled by default for Enterprise**; disable via **"Disable Conversation Insights"** in team settings. It classifies work on-device across Category, Work Type, Complexity, Specificity; free during preview, charged from Jan 1, 2026. Grok Bot Conversation Insights adds a **Cursor / Grok Bot** source toggle at the top of Conversation Insights.
- **[Official]** Audit log: Enterprise only, admin only; view at `cursor.com/dashboard/audit-log`; filter by **date range, event type, actor, application**; export filtered result to CSV; CSV includes Application and User ID. Events cover sign-ins, membership, roles, API keys, settings, integrations, Cloud Agent environments, Grok Bot configuration, Origin repositories; each event records time, actor, client IP, product surface when available. Payloads carry identifiers/names/changed settings, never prompts, agent output, generated code, or credentials. Failed sign-ins are not logged; secrets log name/scope, not values.
- **[Official]** Audit API: both endpoints default to **last 7 days**, accept up to a **30-day window** and **500 events/page**, filter by `eventTypes` and `users`, return oldest first. Streaming destinations: Splunk HEC, Sumo Logic, HTTPS webhook (Cribl, SentinelOne, Rapid7, Cortex XDR, Google Chronicle), S3.
- **[Official]** Analytics API and Admin API are separate; Analytics API is Enterprise-only; Admin API is available on Team and Enterprise except AI Code Tracking (Enterprise).

**E. Cloud Agents admin settings** ([Cloud Agents settings](https://cursor.com/docs/cloud-agent/settings))

- **[Official]** `cursor.com/dashboard/cloud-agents`; **Environments** view lists saved environments; open one to review repositories, snapshot vs `.cursor/environment.json`, install script, runtime/build secrets, network access, version history, Builds, setup runs. Actions: **"Update with Agent"**, **"New Setup Run"**, **"Restore"** from version history.
- **[Official]** **Builds** tab: inspect logs, trigger a Build, choose/pin active Build, start an agent from a specific Build.
- **[Official]** Defaults: **Default model**, **Default repository** (empty = agents ask user), **Base branch** (blank = repository default).
- **[Official]** Network modes: **Allow all network access**, **Default + allowlist**, **Allowlist only**; users, team admins, environment owners can configure; environment can inherit or define its own.
- **[Official]** Security toggles (admin only): **Display agent summary**, **Display agent summary in external channels**, **Team follow-ups** (Disabled / Service accounts only / All), **Automatically allow Projects to place their work** (off by default), **Computer use** (Enterprise only).
- **[Official]** Remote Control: enable in Agents Window under **Settings > Agents**; admin enables for team at **Cursor Dashboard → Cloud Agents → Self-Hosted**; requires Cursor 3.9.8+; unavailable when privacy settings disable cloud data storage; **"Keep this computer awake"** under Settings > Agents.
- **[Official]** Team follow-ups risk text is unusually explicit: enabling them lets a member influence an agent running with another user's secrets; "Treat this setting with the same care you would give shared SSH keys or service credentials."

**F. Personal profile** ([Profiles help](https://cursor.com/help/account-and-billing/profiles))

- **[Official]** Public profile at `cursor.com/@yourhandle`; claim at `cursor.com/profile`; handle rules: lowercase letters, numbers, single hyphens, min 3 characters, unique, first come first served, **cannot be changed after claiming**; up to **four links**; shows name, profile picture, join date, usage data.
- **[Official]** **"Public profile"** toggle: on = anyone with link; off = team members can still view while signed in, non-team only owner. Visibility levels: **Public**, **Team** (default for team members), **Private**.
- **[Official]** Team admin **"Profile visibility"** policy caps visibility at Private / Team (default) / Public; most restrictive policy applies across teams. Deleting the account deletes the profile and frees the handle.

**G. Privacy and security** ([Privacy help](https://cursor.com/help/security-and-privacy/privacy))

- **[Official]** **Privacy Mode** means code is never used for training by Cursor or model providers. Enable: **Cmd+Shift+J** (Mac) / **Ctrl+Shift+J** (Windows/Linux) → **General** in the sidebar → toggle **Privacy Mode**.
- **[Official]** For teams, Privacy Mode is enabled by default for all team members; admins can enforce organization-wide via `cursor.com/dashboard` so members cannot disable it. Enterprise: on by default.
- **[Official]** ZDR does **not** apply when you use your own API keys; data follows the provider. Some models require provider-side retention, are off by default, and need admin approval. Enterprise adds audit logs, granular model controls, SOC 2 Type II, CMEK for Cloud Agent data.
- **[Official]** Cloud Agents need cloud data storage; iOS app prompts to opt in to Privacy Mode with **"Update"** then **"Switch to Privacy Mode"**; cannot switch back to Legacy Privacy Mode afterward.

**H. API keys** ([API keys help](https://cursor.com/help/models-and-usage/api-keys))

- **[Official]** Add your own provider key: **Cursor Settings > Models** → choose provider (OpenAI, Anthropic, Google, Azure, AWS Bedrock) → paste key → **"Save"**. Keys appear in the model picker; invalid keys fail until updated/removed. Keys are not stored on Cursor servers; sent per request over encrypted connections.
- **[Official]** Custom keys work with chat models only; Tab completion uses Cursor's built-in models. Individual plans: own-key requests don't draw included usage. Teams/Enterprise: Cursor Token Rate **$0.25 per million tokens** (input, output, cached) applies to eligible third-party models even with your own key; exempt when cache-read price is strictly below $0.10/M across every tier; paid from team prepaid credit, or "Other Models" under **Spending** on legacy.
- **[Official]** Team key administration: Enterprise admins review/revoke personal keys under **Members** → **API Keys** page, or **"View API keys"** from a member's context menu; filter by member; revoke one or all; revoked keys fail immediately; audit event `user_api_key` with action `revoke`; **"Allow Personal API Key Creation"** can block new keys while existing keys keep working.

**I. Billing, plans, spend, cancellation, deletion**

- **[Official]** Billing: on The X Subscription, plan/payment/invoices/prepaid credit live at `grok.com/dashboard/billing`; **"Payment details"**, **"Invoices"** tab. Legacy: `cursor.com/dashboard/billing` → **"Manage in Stripe"** → Stripe portal; on-demand card at **"Spending"** on `cursor.com/dashboard`. ([Billing help](https://cursor.com/help/account-and-billing/billing))
- **[Official]** Plan table: Free $0 (limited), Plus $8/mo (base pool), Premium $30/mo (4× Plus), Super $100/mo (15× Plus), Ultra $200/mo (50× Plus), Teams $30/$100/$200 per seat/mo, Enterprise custom pooled. Annual: Plus $80, Premium $300, Super $1,000, Ultra $2,000. Included pool is weekly and shared across Cursor/Grok; third-party models excluded, paid at API list price from prepaid credit; Cursor Token Rate for Teams/Enterprise. ([Models & pricing](https://cursor.com/docs/models-and-pricing))
- **[Official]** Plus restrictions: only Cursor models (Grok 4.7, 4.6, 4.5, Composer 2.5) at fixed settings — **Fast is off**, **effort is medium**, Grok 4.7 uses the **256k** context window; no Auto, Cloud Agents, Automations, Bugbot, Origin, third-party models, or own API keys. ([Models & pricing](https://cursor.com/docs/models-and-pricing))
- **[Official]** Top-ups: prepaid credit pays for Cursor models beyond the weekly pool and third-party models; **"Auto Top-up"** refills below a threshold; a **monthly additional usage limit** caps paid additional usage; all three under **"Additional usage credits"** at `grok.com/dashboard/billing`; Teams admins manage a shared balance and one team cap. **[Unverified]** A credit-expiry policy is documented, but the exact term was **not independently established in the retained captures**; treat it as single-source/cohort-dependent and unverified, not a current universal policy. ([Models & pricing](https://cursor.com/docs/models-and-pricing), [Billing help](https://cursor.com/help/account-and-billing/billing))
- **[Official]** Spend limits: dashboard **Spending** tab; on-demand must be enabled; set monthly limit for self or team; **"No Limit"** removes it; changes immediate. Limit reached → AI stops for that user, other members unaffected, user sees a notification, resumes next cycle. Enterprise: member overrides in **Members**, group overrides in **Groups** (`?subtab=active-directory`), team limit in **Spending**; highest applicable wins. **Dynamic Spend Limits** toggle adjusts the team limit with seat count. ([Spend limits](https://cursor.com/help/account-and-billing/spend-limits))
- **[Official]** Spend alerts: team dashboard `cursor.com/dashboard/spending` → **"Add Alert"** → team-level or member-level → threshold; email when exceeded; admins choose whether member alerts go to admins, member, or both. On X Subscription: **Billing › Alerts** at `grok.com/dashboard/billing/alerts`; additional usage must be on; individual plans have no alerts. ([Spend alerts](https://cursor.com/help/account-and-billing/spend-alerts))
- **[Official]** Cancel: X Subscription → `grok.com/dashboard/billing` → options on **"Current plan"** card → cancel → confirm. Legacy → `cursor.com/dashboard/billing` → **"Manage in Stripe"** → **"Cancel Subscription"** → confirm. Access continues to period end; account reverts to Free/Hobby; cancellation does not delete account or local files. Cancel option hidden if plan lapsed, unpaid invoice, no admin access, or store-billed. ([Cancel help](https://cursor.com/help/account-and-billing/cancel))
- **[Official]** Delete account: cancel subscription first; `cursor.com/dashboard` → scroll to **"Advanced Account Settings"** → **"Delete Account"** → confirm. Deletes account and all associated data, chat history, settings; all data removed within **30 days**; local files untouched. ([Delete account](https://cursor.com/help/account-and-billing/delete-account))
- **[Official]** If switching to The X Subscription, Cursor/SuperGrok/paid X Premium and Grok Bot iOS/Android subscriptions are cancelled and unused time refunded; an App Store-billed Cursor subscription must be cancelled by the user in Apple settings; unpaid on-demand is billed in one final invoice and the switch waits for payment. ([Cancel help](https://cursor.com/help/account-and-billing/cancel))

**J. Rules, team rules, memories**

- **[Official]** Four rule types: **Project Rules** (`.cursor/rules/*.mdc`), **User Rules** (global, **Customize → Rules**), **Team Rules** (dashboard, Team/Enterprise), **AGENTS.md**. ([Rules docs](https://cursor.com/docs/rules))
- **[Official]** Rule creation: `/create-rule` in chat, or **Customize → Rules → "Add Rule"**. Rule type dropdown: **Always Apply**, **Apply Intelligently**, **Apply to Specific Files**, **Apply Manually**.
- **[Official]** Team Rules at `cursor.com/dashboard/team-content`; creation dialog exact checkboxes: **"Enable this rule immediately"** (unchecked = draft) and **"Enforce this rule"** (required, cannot be disabled in Customize). Non-enforced team rules can be toggled off by users under **Team Rules** in Customize. Precedence: **Team Rules → Project Rules → User Rules**.
- **[Official]** `AGENTS.md` supported in project root and subdirectories; nested instructions combine with more specific taking precedence.
- **[Official]** Customize page: sidebar entry **Customize**; install plugins/skills/MCPs, team marketplace, team leaderboard, filter by scope (you / workspace / team), plugin canvases. Components: Plugins, Rules, Skills, Subagents, Hooks, Commands. ([Customize Cursor](https://cursor.com/docs/customize-cursor))
- **[Unverified]** **Memories**: there is no current `Memories` page in Cursor's docs index (`/llms.txt`), and the Rules page does not mention memories. A Cursor forum post (2025-06-22) says memories can be edited/removed in **Cursor > Settings > Rules**; a third-party guide (2026-10-21 update) says the old Memories docs page redirects to Rules. No official current label "Rules & Memories" was found. ([Cursor forum](https://forum.cursor.com/t/about-cursors-memory-record-feature/107355), [third-party summary](https://supermemory.ai/blog/cursor-memory-via-mcp/), [docs index](https://cursor.com/llms.txt))
- **[Unverified]** **Beta**: no official settings-menu page documents a Beta tab. A Cursor docs code sample references internal `settingsBetaTab.tsx`/`settingsGeneralTab.tsx` files, which implies a Beta settings surface exists but does not establish its menu label. Community posts describe **Settings → Beta** and "Indexing & Docs between Tab and Beta". Treat as secondary evidence only. ([Rules docs code sample](https://cursor.com/docs/rules), [forum result](https://forum.cursor.com/t/codebase-indexing-settings-tab-hidden-by-disable-codebase-indexing-feature-gate-please-re-enable/165859))
- **[Unverified]** **Sign out**: no official account-menu documentation was found. The public account-mismatch page says "please sign out below and then sign back in with the same account you are using in the app" and shows a **"Cancel"** control; a search result also surfaces the label **"Sign out & switch account"**, but the fetched page text did not include that button. A 2024 community post says Cursor Settings → General can manage the account or log out. ([account mismatch](https://cursor.com/mismatching-client-account), [community post](https://forum.cursor.com/t/change-account-in-cursor-app/11357))

**K. Appearance and themes (editor, not web)** ([Themes help](https://cursor.com/help/customization/themes))

- **[Official]** Theme switching is part of the **IDE**, not the Agents Window: command palette (`Cmd/Ctrl+Shift+P`) → `Preferences: Color Theme`; light themes include **"Default Light Modern"**, dark themes **"Default Dark Modern"** and **"Cursor Dark"**; arrow keys preview. Follow OS: Settings (`Cmd+,` / `Ctrl+,`) → search **"Auto Detect Color Scheme"** → enable.
- **[Official]** Font size: Settings → search **"Font Size"** (editor and terminal separately). Font family: search **"Font Family"** (e.g., `JetBrains Mono`, `Fira Code`). Ligatures: search **"Font Ligatures"** → `true`. More themes via Extensions (`Cmd/Ctrl+Shift+X`) → search `theme` → **"Install"**.
- **[Inferred]** The web dashboard has no documented theme toggle; observed pages are dark-only. Editor theme settings do not establish web appearance settings.

**L. Menu-item evidence matrix (the parent's requested list)**

| Menu item | Best available evidence | Status |
|---|---|---|
| Profile | `cursor.com/profile`; handle, 4 links, **Public profile** toggle, visibility Public/Team/Private, admin **Profile visibility** | **[Official]** personal page; **dashboard menu label unverified** |
| General | **Cursor Settings → General** contains **Privacy Mode** | **[Official]** for that item; full General tab inventory **[Unverified]** |
| Models | **Cursor Settings → Models** hosts provider API keys; model picker; team **Team Settings → Models** | **[Official]** |
| Rules / Memories | **Customize → Rules → Add Rule**; Team Rules dashboard; no current Memories doc | Rules **[Official]**; Memories **[Unverified]** |
| Beta | No official page; docs code sample names a beta settings tab; community reports Settings → Beta | **[Unverified]** (secondary only) |
| Usage / Billing / Plans | **Usage** section on dashboard; **Spending** tab; `dashboard/billing`; **Manage in Stripe**; plan table | **[Official]** |
| API keys | **Cursor Settings → Models** for personal provider keys; **API Keys** dashboard page; **Personal API Keys** → **Team** tab; **Cursor Admin API Keys** | **[Official]** |
| Privacy / Security | **Privacy Mode**, team enforcement, ZDR exceptions, CMEK, audit log, SSO/SCIM | **[Official]** |
| Team / Members / Roles / SSO / SCIM / Audit | Dashboard sections and role/seat/SSO/SCIM/audit docs | **[Official]**; visual menu **[Unverified]** |
| Notifications | Spend-alert emails; editor notification at usage limit; iOS push/Live Activities | **Partial [Official]**; general notification preferences **[Unverified]** |
| Help / Docs / Theme | Top nav **Docs / API / Learn / Help**; docs search; editor theme settings | **[Official]** for nav; account-menu Help/Theme **[Unverified]** |
| Sign out | No official account-menu doc; account-mismatch copy; community post | **[Unverified]** |

### Inferences

- Cursor separates **three layers** that a Studio should keep distinct: personal (profile, privacy, keys, billing), team (members, seats, rules, cloud agents, usage), and organization (identity, groups, pooled usage, audit). The same person can hold different roles per layer.
- The most consequential admin controls are **not** cosmetic: model access, network access, team follow-ups, auto-run, repository blocklist, `.cursor` protection, personal key creation, and privacy enforcement. These are the settings a hosted Studio's admin surface must expose with clear risk copy.
- Cursor's copy uses **imperative labels** ("Invite Members", "Add Alert", "Change role", "Remove", "Delete Account") and **status labels** ("Merging…", "Most popular", "No Limit"). This is a consistent, plain-language pattern.
- Destructive actions follow a **two-step pattern**: choose the action from a menu/card, then confirm in a dialog; the identity-provider merge explicitly says it "can't be undone." Account deletion requires cancelling the subscription first, which is a deliberate dependency chain.
- Defaults are mostly **safe/restrictive**: Privacy Mode on for teams/Enterprise, team follow-ups not assumed open, project placement off, non-enforced team rules disableable, Conversation Insights on for Enterprise (the notable exception, with an opt-out).

### Gaps

- No screenshot or DOM of any authenticated dashboard page; all dashboard "menus" are documentation headings unless an exact in-page label is quoted.
- No evidence of the dashboard's left-nav order, iconography, responsive behavior, empty states, or table design.
- Default values are undocumented for many toggles: Team follow-ups default, network-access default, seat default, Auto Run default, marketplace access default, session timeout, notification preferences.
- No official documentation of a general notifications settings page, a Beta tab, a Memories page, or a Sign out control.
- Exact usage-pool dollar amounts/credits are not published; only multipliers and per-model token prices.
- Audit-log retention period was not found; only API window limits (7-day default, 30-day max).
- No documentation of SCIM group-to-role mapping UI beyond the Organization Group mapping description.

---

## Q3. What responsive/mobile layout and accessibility behavior is established for hosted Agents vs marketing pages vs desktop?

### Takeaway

Responsive behavior is **established and measured for marketing and docs**, **documented for the native iOS app**, and **unverified for the hosted web Agents dashboard** because `/agents` is auth-gated. Marketing and docs both collapse navigation at 390 px rather than shrinking the desktop nav. Accessibility evidence is mixed: strong skip links and a high-visibility docs focus ring, but a heading-order defect in docs, compact pointer targets, and only partial CSS-level reduced-motion handling.

### Cited Findings

**Marketing (measured)**

- **[Observed]** Desktop 1440: header 52 px; nav visible with "Models / Product / Enterprise / Pricing / Resources" and right "Sign in / Contact sales / Download"; H1 26 px; body 16 px; hero top padding 112 px; page height 8,249 px. ([homepage](https://cursor.com/), `desktop-home-full-1440-2026-11-09.png`)
- **[Observed]** Mobile 390: header 56 px; desktop nav removed; right side is "Sign in" text + two-line hamburger; H1 24.375 px; body 15 px; hero shows a single light pill **"Get started →"** instead of desktop's "Download for macOS" + "Request a demo"; logo wall becomes 4 columns × 2 rows; page height 10,046 px. ([homepage](https://cursor.com/), `mobile-home-full-390-2026-11-09.png`)
- **[Observed]** Mobile cookie banner spans the viewport width with stacked full-width buttons ("Reject All", "Accept All Cookies", "Cookies Settings") and a close X; desktop places it bottom-right as a card. ([homepage](https://cursor.com/), screenshots)
- **[Inferred]** The mobile marketing page is not a reduced desktop: it reorders content, drops secondary CTAs, and keeps the same section sequence. The breakpoint is somewhere below 1440 and above 390; exact breakpoints were not measured.

**Docs (measured)**

- **[Observed]** Desktop: header 49 px; fixed 309 px left sidebar; content x=309, H1 x=395; right rail with "On this page"/"Copy page"; content cards 224×160; sidebar rows 260×29; search 320×36 with `⌘K`. ([docs](https://cursor.com/docs), `desktop-docs-full-1440-2026-11-09.png`)
- **[Observed]** Mobile 390: header is logo + search icon + hamburger; sidebar hidden; single-column content with 16 px side padding; H1 33.75 px; body 15 px; the captured full-page height was 844 px, equal to the viewport. Whether the shell scrolls internally or content is lazy-loaded was not determined, so only the first viewport is evidenced. ([docs](https://cursor.com/docs), `mobile-docs-full-390-2026-11-09.png`)
- **[Observed]** On mobile the cookie banner overlays the lower content and includes the same three controls. ([docs](https://cursor.com/docs))
- **[Inferred]** Docs uses a **drawer/hamburger** pattern for the 309 px sidebar on mobile, while search remains a top-level icon. This is the standard reference-docs responsive pattern; the open drawer itself was not captured.

**Hosted Agents (documented, not visually verified)**

- **[Official]** `cursor.com/agents` is the web home for cloud agents, same backend as the desktop Agents Window and iOS app. The iOS app is a native app for controlling cloud and local agents, iPhone/iPad **iOS/iPadOS 26.0+**, **English only**, Android planned. ([Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile))
- **[Official]** iOS interaction: pick repository/branch; send task; watch chat stream; tap subagent card for child transcript; review diffs/commits/deployments/approvals/comments/checks; add/change reviewers; merge with squash, mark ready, update branch, toggle auto-merge, publish, close; slash commands/skills/automations work; Design Mode attaches photos/camera/files and lets you point/click/draw; dictation with live transcription; voice chat incl. Siri; MCP servers per run; push notification when an agent finishes a turn; up to **eight agents** tracked with Live Activities/Dynamic Island.
- **[Official]** iPad layout: chats in sidebar while following several agents; review next to chat; diffs at full width; screenshot + tap/draw with Apple Pencil.
- **[Official]** Mobile is **cache-first**: reads local data so inbox/conversations open fast, then syncs when connection returns.
- **[Official]** Web-only surfaces (explicitly not in the iOS app): editor/terminal/file browser; secrets and environments; MCP server management; source-control setup; automations/rules/skills config; **admin, billing, and usage**. This is the closest official statement of a hosted-vs-mobile responsibility split. ([Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile))
- **[Official]** Remote Control: agent loop moves to Cursor's cloud, tool calls stay on the user's computer; requires Cursor 3.9.8+, Cloud Agents access, Remote Control enabled under **Settings > Agents**, admin enablement at **Cloud Agents → Self-Hosted**, cloud data storage, local or Remote SSH workspace, computer awake/online. ([Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile))
- **[Unverified]** The responsive layout, navigation, breakpoints, touch behavior, and accessibility of the **web** hosted Agents dashboard. `/agents` redirected to the auth host; no screenshot of the signed-in surface was obtainable.

**Accessibility (measured + documented)**

- **[Observed]** Marketing: visible-on-focus skip link; keyboard focus ring `2px solid #EDECEC` with 2 px offset on skip link and logo; 1 H1; 28/28 images with alt; 62 aria labels; 9 landmarks. ([homepage](https://cursor.com/))
- **[Observed]** Docs: hidden-until-focus skip link that becomes 182×40; focus ring white 2 px + orange 4 px; 1 H1 but 13 sidebar H2s before it; 25/25 images with alt; 102 aria labels; 4 landmarks; cookie dialog with role/name and switches. ([docs](https://cursor.com/docs))
- **[Observed]** Docs focus treatment is inconsistent: the second focusable link showed only `outline: auto 1px` and no box-shadow, while the skip link had the strong two-layer ring. ([docs](https://cursor.com/docs))
- **[Observed]** Partial reduced motion: marketing animations 54 → 39 under `prefers-reduced-motion: reduce`; transitions unchanged (139); docs animations/transitions unchanged (3/260). ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Observed]** Compact targets: 34–36 px header controls, 29 px sidebar rows, 24 px logo, 10×34 px disclosure chevrons; 44×44 touch guidance is not met by most header controls. ([homepage](https://cursor.com/), [docs](https://cursor.com/docs))
- **[Unverified]** No official statement of Dynamic Type or VoiceOver support was found; the iOS docs only state iOS/iPadOS 26.0+ and English. ([Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile))
- **[Unverified]** No public accessibility statement/VPAT/WCAG claim; no screen-reader, zoom, high-contrast, or keyboard-trap audit was performed.

### Inferences

- Cursor treats **marketing, docs, and product** as three responsive contexts: marketing collapses to a brand + sign-in + hamburger; docs collapses to a search + hamburger reference shell; the native app uses platform navigation (sidebar on iPad, inbox on iPhone) rather than a web breakpoint.
- A hosted Studio should budget for a **mobile read/review surface** and a **desktop admin surface**, because Cursor explicitly keeps admin/billing/usage web-only while agent direction/review goes mobile. That is a product-boundary decision, not just CSS.
- The docs heading order (H2 sidebar before H1) is a concrete, fixable accessibility defect; the skip link and focus ring are concrete patterns worth copying.
- Reduced motion should be handled for both transitions and animations, not only keyframe animations; Cursor's public site is only partially there.

### Gaps

- Exact responsive breakpoints for marketing/docs; only 1440 and 390 were measured.
- No tablet or 768/1024 capture; no landscape mobile.
- No signed-in web Agents dashboard, so its responsive behavior and accessibility are unknown.
- No iOS app screenshots or accessibility labels; the iOS section is documentation, not observation.
- No screen-reader or keyboard-trap test; no color-contrast audit of every token pair (only sampled text pairs).
- No public accessibility conformance report found.

---

## Q4. What settings affect model/provider choice, privacy, memories/rules, usage/cost, API access and appearance?

### Takeaway

Cursor exposes model/provider control at **four levels** (individual model picker, team model access, organization-group model access, and Cloud Agent default model), privacy at **individual + team enforcement** levels, rules at **user/project/team** levels with an enforcement switch, cost at **usage pool + prepaid credit + spend limits + alerts**, API access through **personal provider keys + personal Cursor keys + admin API keys + service accounts**, and appearance only in the **desktop editor**, not the web dashboard.

### Cited Findings

| Setting / control | Where it lives | What it changes | Documented default | Source |
|---|---|---|---|---|
| Model picker / Auto / Cursor Router | Editor model picker; team **Settings → Cursor Router** | Which model answers; Auto routing; optimization modes members can select; routed-model visibility; Auto as team default | Not stated | [Dashboard docs](https://cursor.com/docs/account/teams/dashboard) |
| Team model access | **Team Settings → Models** | Which models members may use; union with org groups (most-permissive wins) | Not stated | [Dashboard docs](https://cursor.com/docs/account/teams/dashboard) |
| Org-group model access | **Organization → Groups → Models** | Widens/narrows model access per cohort | Not stated | [Dashboard docs](https://cursor.com/docs/account/teams/dashboard) |
| Cloud Agent default model | **Cloud Agents** settings → **Default model** | Model used when a run doesn't specify one | Empty/ask? Not stated | [Cloud Agents settings](https://cursor.com/docs/cloud-agent/settings) |
| Cloud Agent default repo / base branch | Same | Skips repo choice; branch agents fork from | Empty = repo default | [Cloud Agents settings](https://cursor.com/docs/cloud-agent/settings) |
| Personal provider keys | **Cursor Settings → Models** | Routes OpenAI/Anthropic/Google/Azure/Bedrock requests through your key; provider bills you | None | [API keys help](https://cursor.com/help/models-and-usage/api-keys) |
| Bedrock IAM role | Team **Settings → Bedrock IAM Role** | AWS Bedrock access via IAM | None | [Dashboard docs](https://cursor.com/docs/account/teams/dashboard) |
| Privacy Mode | **Cursor Settings → General**; team dashboard enforcement | No training by Cursor/providers; team-wide enforcement | On by default for teams/Enterprise | [Privacy help](https://cursor.com/help/security-and-privacy/privacy) |
| ZDR exceptions | Model/provider policy | Own API keys follow provider policy; retention-required models off by default, admin approval | Off by default | [Privacy help](https://cursor.com/help/security-and-privacy/privacy) |
| CMEK | Enterprise | Customer-managed keys for Cloud Agent data | Not stated | [Privacy help](https://cursor.com/help/security-and-privacy/privacy) |
| Conversation Insights | Team settings → **Disable Conversation Insights** | On-device classification of agent work; Grok Bot toggle | **On** for Enterprise | [Analytics docs](https://cursor.com/docs/account/teams/analytics) |
| Profile visibility | Team settings **Profile visibility**; personal **Public profile** | Caps profile at Private/Team/Public | Team for team members | [Profiles help](https://cursor.com/help/account-and-billing/profiles) |
| User rules | **Customize → Rules** | Global instructions to Agent | None | [Rules docs](https://cursor.com/docs/rules) |
| Project rules | `.cursor/rules/*.mdc` | Path/description/manual-scoped instructions | None | [Rules docs](https://cursor.com/docs/rules) |
| Team rules | `cursor.com/dashboard/team-content` | Team-wide rules; **Enable this rule immediately**; **Enforce this rule** | Non-enforced = user can disable | [Rules docs](https://cursor.com/docs/rules) |
| Rule precedence | Prompt assembly | **Team → Project → User**; merged, earlier wins on conflict | — | [Rules docs](https://cursor.com/docs/rules) |
| Usage pool | Plan | Weekly included usage for Cursor models, shared across Cursor/Grok | Tier-dependent | [Models & pricing](https://cursor.com/docs/models-and-pricing) |
| Prepaid credit / Auto Top-up / monthly cap | **Additional usage credits** at `grok.com/dashboard/billing` | Beyond-pool usage; threshold refill; monthly cap | None | [Models & pricing](https://cursor.com/docs/models-and-pricing) |
| Spend limit | Dashboard **Spending** tab | Monthly on-demand/credit cap; **No Limit** removes | No limit unless set | [Spend limits](https://cursor.com/help/account-and-billing/spend-limits) |
| Dynamic spend limit | **Spending** tab toggle | Scales team limit with seat count | Not stated | [Spend limits](https://cursor.com/help/account-and-billing/spend-limits) |
| Spend alerts | **Add Alert**; X Subscription **Billing › Alerts** | Email at threshold; team/member-level; recipient choice | None | [Spend alerts](https://cursor.com/help/account-and-billing/spend-alerts) |
| Personal Cursor API keys | **API Keys** dashboard page | Scripts/integrations; Enterprise admins can review/revoke; **Allow Personal API Key Creation** | Creation allowed unless blocked | [Members docs](https://cursor.com/docs/account/teams/members) |
| Admin API keys | Team **Settings → Cursor Admin API Keys** | Programmatic admin/usage/audit access | None | [Dashboard docs](https://cursor.com/docs/account/teams/dashboard) |
| Service accounts | Enterprise `account/enterprise/service-accounts` | Non-human identities for agents/automation | Not stated | [Enterprise index](https://cursor.com/llms.txt) |
| Theme / font | Editor **Settings** / command palette | Light/dark theme, font size/family/ligatures, OS auto-detect | Not stated | [Themes help](https://cursor.com/help/customization/themes) |
| Web appearance | — | No documented web theme control; observed dark-only | Dark | [Observed] |

- **[Official]** The Cursor Token Rate is **$0.25/M tokens** (input, output, cached) for eligible third-party models on Teams/Enterprise, including own-key requests; Cursor models and Cost mode are exempt; models with cache-read price strictly below $0.10/M across every tier are exempt. ([API keys help](https://cursor.com/help/models-and-usage/api-keys), [Models & pricing](https://cursor.com/docs/models-and-pricing))
- **[Official]** On Plus, model settings are fixed: Fast off, effort medium, Grok 4.7 256k context. On Premium+, Fast/effort/third-party models become available; third-party usage is billed at API list price from prepaid credit. ([Models & pricing](https://cursor.com/docs/models-and-pricing))
- **[Official]** Rules **do not** impact Cursor Tab or other AI features; they apply to Agent (Chat). ([Rules docs](https://cursor.com/docs/rules))
- **[Official]** Team Rules are included in Agent context "across all repositories and projects for that team"; enforced rules cannot be disabled in Customize. ([Rules docs](https://cursor.com/docs/rules))
- **[Inferred]** There is no single "Settings" page that owns all of these; the model is federated across editor settings, team dashboard, organization settings, and billing. A Studio should decide whether to keep that federation or present a single settings hub with clear scope labels (You / Team / Organization).

### Inferences

- The **scope label is the hard part**: the same conceptual setting (model access, spend limit, rules) exists at user, team, and org levels, and Cursor resolves conflicts with "most-permissive wins" for access/limits but "team wins" for rules. A Studio needs an explicit precedence model per setting, not one global rule.
- Cost controls are layered deliberately: included pool → prepaid credit → monthly cap → spend limit → alert. Each layer has a different failure mode (stop, bill, notify). A Studio should expose the layer, not just a dollar number.
- Privacy is **default-on for teams** and enforceable; individual own-key use is the documented exception. A Studio handling proprietary code should make the same distinction visible at the point of key entry.
- Appearance is a desktop-editor concern; the web account surface is dark-only. Do not promise a web theme toggle without evidence.

### Gaps

- Exact default values for Team follow-ups, network access, seat assignment, Auto Run, marketplace access, and session timeout are not documented.
- No documentation of a notification-preferences page beyond spend alerts and iOS push.
- No official "Memories" settings page or label; the Rules page is the only current official rule surface.
- No official Beta-tab documentation.
- No documented web theme toggle, accent customization, or density setting.
- No public explanation of how model access conflicts resolve between team and org beyond "union / most-permissive."

---

## Q5. What is unverified without an account or paid team?

### Takeaway

Everything **inside** `cursor.com/dashboard`, `cursor.com/dashboard/team-settings`, `cursor.com/dashboard/members`, `cursor.com/dashboard/analytics`, `cursor.com/dashboard/audit-log`, `cursor.com/dashboard/billing`, `cursor.com/dashboard/cloud-agents`, `cursor.com/agents`, and `cursor.com/profile` is visually unverified. The public auth host returned a bot-verification interstitial, and `/sign-in` is a 404. Documentation establishes labels and behavior, but not layout, navigation, default states, or responsive behavior. Paid-team-only features (audit log, SCIM, CMEK, org groups, pooled usage) are documented but cannot be observed without an Enterprise account.

### Cited Findings

- **[Observed]** `/dashboard` and `/agents` redirect to `authenticator.cursor.sh` and show "Just a moment..." / "Performing security verification" / "Verification successful. Waiting for authenticator.cursor.sh to respond"; no form or dashboard content. ([dashboard](https://cursor.com/dashboard), [agents](https://cursor.com/agents), screenshots `desktop-dashboard-1440-2026-11-09.png`, `desktop-agents-1440-2026-11-09.png`)
- **[Observed]** `/sign-in` is a 404 page, not an auth page. ([cursor.com/sign-in](https://cursor.com/sign-in), `desktop-signin-1440-2026-11-09.png`)
- **[Official]** The dashboard is the documented home for billing, usage-based pricing, team management, members, audit log, integrations, cloud agents, automations, active directory, usage, and billing/invoices. ([Dashboard docs](https://cursor.com/docs/account/teams/dashboard))
- **[Official]** Audit logs, SCIM, CMEK, organization groups, pooled usage, and the AI Code Tracking API are Enterprise-only. ([Compliance](https://cursor.com/docs/enterprise/compliance-and-monitoring), [SCIM](https://cursor.com/docs/account/teams/scim), [Organizations](https://cursor.com/docs/enterprise/organizations), [Privacy](https://cursor.com/help/security-and-privacy/privacy))
- **[Official]** Spend alerts and limits are documented with exact labels, but the live dashboard values (current spend, thresholds, seat counts) require an account. ([Spend alerts](https://cursor.com/help/account-and-billing/spend-alerts), [Spend limits](https://cursor.com/help/account-and-billing/spend-limits))
- **[Official]** Profile visibility/handle claiming requires sign-in at `cursor.com/profile`; the public profile page itself was not loaded for a real handle. ([Profiles help](https://cursor.com/help/account-and-billing/profiles))
- **[Official]** The iOS app requires an App Store install and sign-in; no app UI was observed. ([Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile))

### Source conflicts and ambiguity

- **[Official/Observed]** Cursor's 2026 surface is presented as **"The X Subscription"** at $8/$30/$100/$200, with Teams seats at $30/$100/$200 and billing routed to `grok.com/dashboard`. The pricing page also lists Grok/X benefits and says "Only Grok models are available on this plan" for Plus. This is materially different from the legacy Cursor Pro/Pro+/Ultra model and from the separate [usage-limits help page](https://cursor.com/help/models-and-usage/usage-limits), which still says "Pro ($20/mo): $20 of API agent usage", "Pro Plus ($60/mo): $70", "Ultra ($200/mo): $400", monthly reset, and "Max Mode" surcharge. The docs do label Pro/Pro+/Ultra/Start/Standard/Premium seats as **legacy plans that keep working**, so both can be true for different cohorts — but the help page does not label itself legacy, and the plan names collide (legacy "Premium" seat vs current "Premium" plan). **Do not treat either page as the single current plan truth without checking the account cohort.** ([Pricing](https://cursor.com/pricing), [Models & pricing](https://cursor.com/docs/models-and-pricing), [Usage limits](https://cursor.com/help/models-and-usage/usage-limits), [Members/legacy seats](https://cursor.com/docs/account/teams/members))
- **[Official/Observed]** Docs sometimes route Cursor team actions to `grok.com/dashboard` (seat menu, billing, API key settings) and sometimes to `cursor.com/dashboard` (audit log, analytics, cloud agents). Whether this is a migration in progress is **not established**; the inconsistency is in the source set, not a finding about the product.
- **[Observed/Unverified]** The docs index (`/llms.txt`) has no `Memories` page, while community/third-party sources describe Memories and a "Rules & Memories" section. Whether this is a documentation gap or a rename is not established; the current official surface is Rules. ([docs index](https://cursor.com/llms.txt), [forum post](https://forum.cursor.com/t/about-cursors-memory-record-feature/107355), [third-party summary](https://supermemory.ai/blog/cursor-memory-via-mcp/))
- **[Official]** SSO docs say there is "no separate enforcement toggle" once domain verification + connection are active, while other docs/help refer to "SSO enforcement options." The SSO page is the more specific and current statement; treat "enforcement" as automatic by domain, not a toggle.
- **[Inferred]** The auth host and marketing site use different design systems (system-ui/pure black vs CursorGothic/warm black). Do not use the auth interstitial as evidence of Cursor's product design.

### Gaps

- Dashboard navigation labels, order, icons, layout, density, responsive behavior, empty/loading/error states.
- Account menu contents: profile, general, models, rules/memories, beta, usage/billing/plans, API keys, privacy/security, notifications, help/docs/theme, sign-out.
- Sign-out flow and whether it is per-device or global.
- Beta tab label and contents.
- Memories label, location, and CRUD UI.
- Notification preferences beyond spend alerts and iOS push.
- Default states for team follow-ups, network access, seat assignment, Auto Run, marketplace access, session timeout.
- Audit-log retention; audit-log visual filters/columns.
- Exact plan pool amounts and overage prices for the current X Subscription (only multipliers and per-model token prices are public).
- SCIM/org-group UI beyond the documented paths; role-mapping screen.
- Mobile app UI, accessibility labels, Dynamic Type/VoiceOver support.
- Any paid-team feature behavior under load or failure.

### Source index (all accessed 2026-11-09)

Public UI: [cursor.com](https://cursor.com/), [pricing](https://cursor.com/pricing), [docs](https://cursor.com/docs), [changelog](https://cursor.com/changelog), [sign-in (404)](https://cursor.com/sign-in), [dashboard (auth gate)](https://cursor.com/dashboard), [agents (auth gate)](https://cursor.com/agents), [account mismatch](https://cursor.com/mismatching-client-account).

Docs/help: [docs index](https://cursor.com/llms.txt), [team dashboard](https://cursor.com/docs/account/teams/dashboard), [members/roles/seats](https://cursor.com/docs/account/teams/members), [teams setup](https://cursor.com/docs/account/teams/setup), [SSO](https://cursor.com/docs/account/teams/sso), [SCIM](https://cursor.com/docs/account/teams/scim), [analytics](https://cursor.com/docs/account/teams/analytics), [organizations](https://cursor.com/docs/enterprise/organizations), [admin setup](https://cursor.com/docs/enterprise/admin-setup-guide), [compliance/audit](https://cursor.com/docs/enterprise/compliance-and-monitoring), [cloud agent settings](https://cursor.com/docs/cloud-agent/settings), [Cursor for iOS](https://cursor.com/docs/cloud-agent/mobile), [rules](https://cursor.com/docs/rules), [customize](https://cursor.com/docs/customize-cursor), [models & pricing](https://cursor.com/docs/models-and-pricing), [profiles](https://cursor.com/help/account-and-billing/profiles), [billing](https://cursor.com/help/account-and-billing/billing), [cancel](https://cursor.com/help/account-and-billing/cancel), [delete account](https://cursor.com/help/account-and-billing/delete-account), [spend limits](https://cursor.com/help/account-and-billing/spend-limits), [spend alerts](https://cursor.com/help/account-and-billing/spend-alerts), [privacy](https://cursor.com/help/security-and-privacy/privacy), [API keys](https://cursor.com/help/models-and-usage/api-keys), [usage limits](https://cursor.com/help/models-and-usage/usage-limits), [themes](https://cursor.com/help/customization/themes).

Secondary (labeled as such in text): [Cursor forum memories post](https://forum.cursor.com/t/about-cursors-memory-record-feature/107355), [forum Beta/indexing post](https://forum.cursor.com/t/codebase-indexing-settings-tab-hidden-by-disable-codebase-indexing-feature-gate-please-re-enable/165859), [forum sign-out post](https://forum.cursor.com/t/change-account-in-cursor-app/11357), [third-party Memories summary](https://supermemory.ai/blog/cursor-memory-via-mcp/).

Local evidence (not for commit): `local/kimi/cursor-hosted-research/visual-account-design/` — desktop/mobile PNGs and JSON measurements listed inline above.
