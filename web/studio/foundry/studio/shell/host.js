import {chatModelRoute,LOCAL_MODEL,UOR_R4_PROVIDER,UOR_R4_DEFAULT_MODEL} from './chat-model-route.mjs';
import * as uorR4 from './uor-r4-runtime.mjs';
import {createMvmHost} from './mvm-host.mjs';
import {createLinuxVmHost} from './linux-vm-host.mjs';
import {createStitchNetwork} from './stitch-network.mjs';
import {createGitHubVault} from './github-vault.mjs';
import {createGitHubClient} from './github-client.mjs';
import {runUorR4AgentTurn} from './r4-agent-loop.mjs';
import {createThemeLibrary} from "./theme-library.mjs";
import {themesData} from "./builtin-themes.mjs";
import {changeUserSettings} from "./settings-transactions.mjs";
import {createPromptLibrary} from "./prompt-library.mjs";
import {readWorkingTree} from "./working-tree.mjs";
import {createWindowDelivery} from "./window-delivery.mjs";
import {createWindowCoordinator,windowJournalStore,windowRecoveryScopes,readWindowJournal} from "./window-coordination.mjs";
import {VisibleEntitySchema,ChatTabOwnershipSchema,WindowInterestSchema,QueryInvalidationScopeSchema,QueryInvalidationBatchSchema} from "./chat-contracts.mjs";
import {prepareMessageRestoration} from "./message-restoration.mjs";
import {prepareSnapshot} from "./snapshot-restore.mjs";
import {createVersionPreviewController} from "./version-preview-controller.mjs";
import {previewProjectStore,previewFailure} from "./version-preview-store.mjs";
import {previewWithGuest} from "./version-preview-codec.mjs";
import {versionAddress,listStoredVersions,setStoredVersionMetadata,verifiedVersionChanges,resolveStoredVersion} from "./version-history.mjs";
import {createChatQueueController} from "./chat-queue-controller.mjs";
import {readStoredChatQueue,commitStoredChatQueue,bindQueuePayload,chatQueueFailure} from "./chat-queue-store.mjs";
import {VersionPreviewIntentEventSchema,VersionPreviewRemoteSnapshotSchema,ChatStreamIntentEventSchema,SerializableChatTurnIntentSchema,serializeImmutableChatTurnPayload} from "./chat-contracts.mjs";
import {queueWithGuest} from "./chat-queue-codec.mjs";
import {createDiscovery} from "./application-discovery.mjs";
import {mountApplicationDiscovery} from "./discovery-ui.mjs";
import {routeWithGuest} from './routing-codec.mjs';
import {runBuildWork,buildWorkStore} from './build-work.mjs';
import {withPortableBuildPermission, portablePermissionFailure} from "./portable-build-permission.mjs";
import {commandRegistry, invokeBoundCommand} from "./command-registry.mjs";
import {interactionPreset, interactionModule, validateInteractionDocument, interactionFailure} from "./interaction-application.mjs";
import {runStoredInteraction} from "./interaction-state.mjs";
import {mountInteractionApplications, openInteractionApplication} from "./interaction-ui.mjs";
import { admitEventWithGuest } from "./event-codec.mjs";
import { loadBrowserCompiler } from "./browser-compiler.mjs";
import { browserSettings, assertBrowserSettingsPatch } from "./privacy-policy.mjs";
import { prepareProjectImport } from "./project-import.mjs";
import { withBuildModules } from "./build-modules.mjs";
import { captureBuildSource, assertBuildSourceCurrent, requiresRulePublication } from "./build-source.mjs";
import { applyStoredProjectPatch, assertGenerationActive } from "./generation-patch.mjs";
import { editStoredProject, captureProjectSource, commitPreparedProject } from "./project-edit.mjs";
import {modulePresentation} from "./module-presentation.mjs";
import { loadProjectScaffold, prepareExistingProject, prepareNewProject, createStoredProject, createStoredChat } from "./project-creation.mjs";
import { storageRequest } from "./storage-request.mjs";
import { executeWorkWithGuest } from "./work-codec.mjs";
// The browser host: the surface src/preload.ts exposes as window.electron, implemented on the page
// with no server. Rules come from the generated core (core.wasm: preimage, restore, network, view);
// addresses are BLAKE3 over the bytes the core spells; records live in IndexedDB. Adapters only:
// nothing here decides, it derives, stores and forwards.
//
// Slices 1 and 2: settings, apps and their files, versions as kappa snapshots, chats as records,
// the preview worker, and the app run machine (build in the tab, serve at the version's address).
// The chat stream is the next slice.

import { packBytes, unpackBytes, RepresentationError } from "./exact-representation.mjs";
import { admitWithGuest } from "./composition-codec.mjs";
import { readBoundRules, readBoundRulesSnapshot, saveBoundRules, sameRuleReservation } from "./rules-source.mjs";
import {browserChatMode,assertBrowserChatModes,browserChatModeSettings} from "./browser-chat-modes.mjs";
import {chatExecutionPolicy,chatExecutionPrompt,readOnlyChatText} from "./chat-execution.mjs";
import {updateChatPreferences} from "./chat-preferences.mjs";
import { renameStoredProject, restoreStoredProject, deleteStoredProject, deleteStoredChat, updateStoredChat } from "./project-lifecycle.mjs";
import { invokeHost, unwrapHost } from "./host-boundary.mjs";
import * as inference from "./inference.js";
import { r4Decide, r4Refusal, r4Request, r4Spec } from "./r4.mjs";
import { viewReady } from "./inference.js";
import { install, installed, encodeView, b64, unb64, sha256 } from "./holo.js";
import { admitPublication } from "./publication-identity.mjs";
import * as prism from "./prism.js";
import * as live from "./live/project.mjs";
import * as lexlean from "./live/lexlean.mjs";
import { canonicalEditPath, planCanonicalEdit } from "./canonical-edits.mjs";
import {applicationShape, activeApplicationModule, retiredApplicationPaths, boundApplicationState, shapeFailure} from './application-shape.mjs';
import { executeInteraction } from "./interaction-codec.mjs";
import { copyEventCommand, readEventJournal, updateEventJournal } from "./event-delivery.mjs";
import { authorizePreview, commitPreviewPermission, previewPermissionFailure, mountPreviewPermissions } from "./preview-permissions.mjs";
import { copyGenerationInput, assertCanonicalGeneration } from "./canonical-generation.mjs";
import * as appShape from "./live/app.mjs";
import { frameHtml, FONTS } from "./live/frame-html.mjs";
import { applyOps, createEditor, PatchError } from "./live/edit.mjs";
import * as covers from "./covers.mjs";
import { createProbe, K } from "./telemetry/probe.mjs";
import {publishGallery, readGallery, galleryFailure} from "./gallery-store.mjs";

const BASE = new URL("./", import.meta.url); // the shell's directory, wherever it is served
const listeners = new Map();
const handlers = new Map();
const seen = new Map();
let relayWindowEvent=()=>{};
const emitLocal = (channel, payload) => { const set = listeners.get(channel); if (set) for (const fn of set) { try { fn(payload); } catch (e) { console.error(e); } } };
const emit = (channel,payload)=>{emitLocal(channel,payload);relayWindowEvent(channel,payload);};

// ---- the generated core, the addresses and the store come from the shared inference module: one
// core.wasm, one BLAKE3, one database ("hologram-forge" v2) for the builder's records and the seals.
const enc = new TextEncoder(), dec = new TextDecoder();
let core = null;
const coreReady = inference.coreReady().then((c) => (core = c));
const kappa = inference.kappa;
const dbReady = inference.workspace();
// The Prism R4 decision is read once, while the studio starts, so a first turn does not wait to fetch it.
r4Spec().catch(() => {});
async function tx(store, mode, fn) { return storageRequest(await dbReady, store, mode, fn); }
const get = (store, key) => tx(store, "readonly", (s) => s.get(key));
const put = (store, key, value) => tx(store, "readwrite", (s) => s.put(value, key));
const del = (store, key) => tx(store, "readwrite", (s) => s.delete(key));
const all = (store) => tx(store, "readonly", (s) => s.getAll());
const keys = (store) => tx(store, "readonly", (s) => s.getAllKeys());
// An app's files are the keys "<id>:<path>": read by key range instead of walking every app's keys.
const appRange = (appId) => IDBKeyRange.bound(appId + ":", appId + ":￿");

const themeLibrary=createThemeLibrary(dbReady,themesData);
handlers.set("get-themes", () => themeLibrary("catalog"));
handlers.set("get-custom-themes", () => themeLibrary("list"));
handlers.set("create-custom-theme", input => themeLibrary("create",input));
handlers.set("update-custom-theme", input => themeLibrary("update",input));
handlers.set("delete-custom-theme", input => themeLibrary("delete",input));
handlers.set("set-app-theme", input => themeLibrary("select",input));
handlers.set("get-app-theme", input => themeLibrary("selected",input));
const promptLibrary=createPromptLibrary(dbReady);
handlers.set("prompts:list", () => promptLibrary("list"));
handlers.set("prompts:create", input => promptLibrary("create",input));
handlers.set("prompts:update", input => promptLibrary("update",input));
handlers.set("prompts:delete", input => promptLibrary("delete",input));

// ---- settings
const DEFAULT_SETTINGS = {
  // Default to the real UOR-R4 V13 Qwen3-0.6B Foundry integer WASM engine in this browser.
  selectedModel: { name: "uor-r4:qwen3-0.6b-foundry", provider: "uor-r4" }, providerSettings: {}, telemetryConsent: "opted_out", telemetryUserId: "web",
  hasRunBefore: true, experiments: {}, enableProLazyEditsMode: false, enableProSmartFilesContextMode: false, selectedChatMode: "build",
  enableAppBlueprint: false, enableTestingForNewApps: false, enableAutoUpdate: false, releaseChannel: "stable", isRunning: false,
  enableSandboxScriptExecution: false, enableMcpToolSearch: false, enableCodeExplorer: false, runTypeScriptForWholeProject: false,
  enableMultiWindow: false, enableExplorerSubagent: false, enableAutoReview: false, enableReviewButton: false,
  enableImplementerSubagent: false, enableAdvancedSubagents: false, autoFixReviewIssues: false, autoApproveNonSchemaSql: true,
  autoExpandPreviewPanel: true, enableContextCompaction: true, enablePnpmMinimumReleaseAgeWarning: false,
  previewIdleTimeoutPolicy: "default", nodeRuntimePreference: "system", disablePreviewNodeAutoInstall: true,
  // The scaffold every project starts from is a Vite React app, which is the template this names.
  selectedTemplateId: "react",
};
// A stored record from an earlier shell may lack a key a newer renderer reads (providerSettings, say): the
// defaults fill what is missing, the record keeps what it has. One mode is implemented here, the build
// turn that writes files, so that is the mode the record always carries: an agent mode would promise
// tool calls this host does not make.
handlers.set("get-user-settings", async () => {
  const stored = (await inference.settingsGet()) || {};
  // Migrate removed dummy/legacy models (e.g. local foundry:reader, synthetic-seed-5, or cloud models)
  // onto the real UOR-R4 V13 Qwen3-0.6B Foundry model once per stored settings record.
  if (
    stored.selectedModel &&
    (stored.selectedModel.provider !== "uor-r4" ||
      stored.selectedModel.name === "uor-r4:synthetic-seed-5" ||
      stored.selectedModel.name === "synthetic-seed-5")
  ) {
    return changeUserSettings(dbReady, { selectedModel: DEFAULT_SETTINGS.selectedModel }, (current, captured) => browserSettings(browserChatModeSettings({ ...DEFAULT_SETTINGS, ...current, ...captured })));
  }
  return browserSettings(browserChatModeSettings({ ...DEFAULT_SETTINGS, ...stored }));
});
handlers.set("set-user-settings", (patch) => { assertBrowserSettingsPatch(patch); assertBrowserChatModes(patch); return changeUserSettings(dbReady, patch || {}, (current,captured) => browserSettings(browserChatModeSettings({ ...DEFAULT_SETTINGS, ...current, ...captured }))); });
// What the renderer takes as "set up": the studio's own model is the only provider, and it is always
// set up, so no connect dialog ever stands between the first prompt and the answer. Whether this
// browser can run it is the R4 route's question (Local or NoGpu), never a provider or key question.
handlers.set("get-env-vars", () => ({ UOR_R4_WASM: "on this device", HOLOGRAM_DEVICE: "on this device" }));
handlers.set("get-system-platform", () => "web");
handlers.set("get-app-version", () => ({ version: "1.15.0-web" }));
handlers.set("native-theme:get-state", () => ({ shouldUseDarkColors: matchMedia("(prefers-color-scheme: dark)").matches }));
handlers.set("get-initial-load-telemetry-context", () => ({ isFirstSession: false, previousSessionAppSize: null }));
// The build engine is esbuild in the tab; the renderer asks for Node before it shows a preview, so the
// answer names the engine that will run: no Node is installed or needed.
handlers.set("nodejs-status", () => ({ nodeVersion: "browser (esbuild in the tab)", pnpmVersion: "browser", nodeDownloadUrl: "", source: "system", nodePath: null, managedNodeInstalled: false, managedNodeVersion: null, systemNodeTooOld: false, managedNodeSupported: false }));
// A window here is a tab, and its session id has to be a UUID: the renderer parses it, and "web-1"
// was refused, so the window session never initialised and every screen that keys off it -- the
// version controls among them -- reported itself unavailable. The id lasts as long as the tab, which
// is what a window session is, so a reload finds the same one.
const windowsStore=dbReady.then(windowJournalStore);
const windowSchemas={visible:VisibleEntitySchema.array().max(100),tabs:ChatTabOwnershipSchema.array().max(100),interest:WindowInterestSchema,scopes:QueryInvalidationScopeSchema.array().min(1).max(128),batch:QueryInvalidationBatchSchema};
const windowCoordination=createWindowCoordinator({store:{read:()=>windowsStore.then(s=>s.read()),commit:(...args)=>windowsStore.then(s=>s.commit(...args))},locks:navigator.locks,channel:typeof BroadcastChannel==="function"?new BroadcastChannel("foundry:windows:v1"):null,storage:{getItem:key=>sessionStorage.getItem(key),setItem:(key,value)=>sessionStorage.setItem(key,value)},mintId:()=>crypto.randomUUID(),validate:(kind,value)=>windowSchemas[kind].safeParse(value),deliver:createWindowDelivery({emit:emitLocal,chatMachine:chatMachineOf,chatActor:id=>chatActorId(id),appMachine:machineOf}),exists:entity=>get(entity.kind==="app"?"apps":"chats",entity.id).then(Boolean)});
handlers.set("window-infrastructure:bootstrap", input => windowCoordination.bootstrap(input));
handlers.set("window-infrastructure:set-focused", () => windowCoordination.setFocused());
handlers.set("window-infrastructure:set-visible-entities", input => windowCoordination.setVisible(input));
handlers.set("window-infrastructure:set-chat-tab-ownership", input => windowCoordination.setTabs(input));
for (const c of ["add-log", "renderer:error-toast-ready", "clear-logs"]) handlers.set(c, () => undefined);
relayWindowEvent=(channel,payload)=>{windowCoordination.relay(channel,payload);if(channel==="version-created"&&Number.isSafeInteger(payload?.appId))void windowCoordination.invalidate([{family:"apps"},{family:"app",appId:payload.appId},{family:"versions",appId:payload.appId}]).catch(()=>{});if(channel==="chat:stream:end"&&Number.isSafeInteger(payload?.chatId))void windowCoordination.invalidate([{family:"chat",chatId:payload.chatId},{family:"chats"}]).catch(()=>{});};
window.addEventListener("pagehide",()=>windowCoordination.dispose(),{once:true});
handlers.set("does-release-note-exist", () => ({ exists: false }));
handlers.set("get-user-budget", () => null);
handlers.set("get-subscription-status", () => null);
// Apps live in this browser's store, not in a folder: the path is the store's name and it is not a
// place the visitor can point elsewhere.
handlers.set("get-custom-apps-folder", () => ({ path: inference.DB, isPathAvailable: true, isPathDefault: true }));
handlers.set("get-node-path", () => null);
// What this build is, for the settings page's own debug panel: no Node, no auto updater, the engine
// named where a Node path would be, and the chosen model as the renderer spells it.
handlers.set("get-system-debug-info", async () => {
  const s = (await get("settings", "user")) || DEFAULT_SETTINGS;
  const m = s.selectedModel || {};
  return { nodeVersion: null, pnpmVersion: null, nodePath: "esbuild in the tab", telemetryId: "web", telemetryConsent: "opted_out", telemetryUrl: "", prismVersion: "1.15.0-web", platform: "web", architecture: navigator.userAgent.includes("WOW64") || navigator.userAgent.includes("x86_64") ? "x86_64" : "unknown", logs: "", updaterLogs: null, selectedLanguageModel: (m.provider ? m.provider + "/" : "") + (m.name || "") };
});
const UOR_R4 = { id: UOR_R4_PROVIDER, name: "UOR-R4 Integer WASM", type: "custom", hasFreeTier: true, envVarName: "UOR_R4_WASM" };
async function getUorR4Models() {
  const cat = await uorR4.getCatalog();
  return (cat.models || []).map((m) => ({
    apiName: m.apiName,
    displayName: m.label || m.id,
    description: m.note || "In-browser UOR-R4 integer WASM model",
    contextWindow: m.id === "m1-discrete-trunk-seed-1" ? 512 : 4096,
  }));
}
handlers.set("get-language-model-providers", () => [UOR_R4]);
handlers.set("get-language-models", async ({ providerId }) => (providerId === UOR_R4_PROVIDER ? await getUorR4Models() : []));
handlers.set("get-language-models-by-providers", async () => ({ [UOR_R4_PROVIDER]: await getUorR4Models() }));

// ---- apps and files. A file is stored under "<appId>:<path>"; the app record carries no files,
// get-app lists them. Every edit seals a version: the address of the project state.
const nextId = async (store) => { const ks = await keys(store), high = await get("refs", `allocation:${store}`); const maximum = Math.max(0, high || 0, ...ks.map(Number)); if (!Number.isSafeInteger(maximum) || maximum < 0 || maximum >= Number.MAX_SAFE_INTEGER) throw new Error("Identifier allocation failed"); return maximum + 1; };
const nulls = { githubOrg: null, githubRepo: null, githubBranch: null, supabaseProjectId: null, supabaseParentProjectId: null, supabaseOrganizationSlug: null, neonProjectId: null, neonDevelopmentBranchId: null, neonPreviewBranchId: null, neonActiveBranchId: null, selectedDatabaseBranchType: null, vercelProjectId: null, vercelProjectName: null, vercelDeploymentUrl: null, vercelTeamId: null, installCommand: null, startCommand: null, chatContext: null, isFavorite: false, testingEnabled: false, collectionId: null };
const withDates = (a) => ({ ...a, createdAt: new Date(a.createdAt), updatedAt: new Date(a.updatedAt), resolvedPath: a.path });
async function filesOf(appId) {
  const ks = (await tx("files", "readonly", (s) => s.getAllKeys(appRange(appId)))).map((k) => k.slice(String(appId).length + 1)).sort();
  return ks;
}
let scaffold = null;
async function loadScaffold() {
  if (!scaffold) scaffold = await loadProjectScaffold(new URL("scaffold/", BASE));
  return scaffold;
}
handlers.set("list-apps", async () => ({ apps: (await all("apps")).map(withDates) }));
handlers.set("get-app", async (id) => { const a = await get("apps", id); if (!a) throw new Error("no app " + id); return { ...withDates(a), files: await filesOf(id), frameworkType: "vite", supabaseProjectName: null, vercelTeamSlug: null }; });
handlers.set("check-app-name", async ({ appName }) => ({ exists: (await all("apps")).some((a) => a.name === appName) }));
handlers.set("preview-app-folder-name", async ({ appName }) => ({ folderName: appName }));
const pendingCreations = new Map();
handlers.set("create-app", async ({ name, initialChatMode, firstPromptCreationOperationId }) => {
  await coreReady;
  const capturedMode = browserChatMode(initialChatMode,await get("settings","user"));
  const prepared = await prepareNewProject({ name, initialChatMode: capturedMode, files: await loadScaffold() }, { core, derive: kappa });
  const created = await createStoredProject(await dbReady, prepared, nulls, themesData);
  if (typeof firstPromptCreationOperationId === "string" && firstPromptCreationOperationId) {
    pendingCreations.set(firstPromptCreationOperationId, { kind: "app", appId: created.app.id, chatId: created.chatId });
  }
  emit("version-created", { appId: created.app.id, oid: created.oid });
  return { app: withDates(created.app), chatId: created.chatId };
});
handlers.set("read-app-file", async ({ appId, filePath }) => { const c = await get("files", appId + ":" + filePath); if (c === undefined) throw new Error("no file " + filePath); return c; });
handlers.set("edit-app-file", async (input) => {
  input = structuredClone(input);
  await coreReady;
  if (input?.filePath === "prism/interaction.json") {
    const edit = structuredClone(input), source = await captureProjectSource(await dbReady, edit.appId);
    const document = await validateInteractionDocument(edit.content, kernelReady);
    const files = Object.fromEntries(source.files);
    files["prism/interaction.json"] = JSON.stringify(document, null, 2) + "\n";
    files["prism/Application.lex.tex"] = interactionModule(document.graph);
    files["prism/InteractionModel.lex.tex"] = await loadInteractionModelSource();
    const prepared = await prepareExistingProject({name:source.name,files}, {core,derive:kappa}, source.reference);
    const oid = await commitPreparedProject(await dbReady, edit.appId, ["prism/interaction.json", "prism/Application.lex.tex", "prism/InteractionModel.lex.tex"], source, prepared);
    emit("version-created", {appId:edit.appId,oid});return {};
  }
  if (["prism/Application.lex.tex", "prism/InteractionModel.lex.tex"].includes(input?.filePath) && await get("files", input.appId + ":prism/interaction.json") !== undefined) throw interactionFailure("interactionApplicationMismatch");
  if (canonicalEditPath(input?.filePath)) {
    const source = await captureProjectSource(await dbReady, input.appId);
    const plan = planCanonicalEdit(input, source, { lexlean, prism });
    if (plan) {
      const options = {source, onCommit: oid => emit('version-created', {appId:input.appId, oid})};
      if (plan.shape === 'stateful') {
        await appAfterTurn(input.appId, '', {...options, moduleText:plan.moduleText});
        await refreshPreview(input.appId);
      } else {
        await rulesAfterTurn(input.appId, plan.text, [], options);
        await refreshLive(input.appId);
      }
      return plan.warning === null ? {} : { warning: plan.warning };
    }
  }
  const oid = await editStoredProject(await dbReady, input, { core, derive: kappa });
  emit("version-created", { appId: input.appId, oid });
  return {};
});
handlers.set("delete-app", async ({ appId }) => { const deletedChats=(await all("chats")).filter(chat=>chat.appId===appId).map(chat=>chat.id); await deleteStoredProject(await dbReady, appId); liveEditors.delete(appId); coverSources.delete(appId); clearTimeout(coverTimers.get(appId)); coverTimers.delete(appId); await covers.forget(BASE, appId); for(const chatId of deletedChats){queueChannel?.postMessage({chatId});await chatQueue.refresh(chatId).catch(()=>{});} });
handlers.set("rename-app", async (input) => {
  await coreReady;
  await renameStoredProject(await dbReady, input, { core, derive: kappa, onCommit: oid => emit("version-created", { appId: input.appId, oid }) });
  return { app: withDates(await get("apps", input.appId)) };
});
// A search matches a project by name, by the title of one of its chats, or by a line in one of their
// messages, and says which: the renderer shows the matched title and message under the project's name.
handlers.set("search-app", async (q) => {
  const needle = String(q || "").toLowerCase();
  const chats = await all("chats");
  const out = [];
  for (const a of await all("apps")) {
    const mine = chats.filter((c) => c.appId === a.id);
    const chat = mine.find((c) => String(c.title || "").toLowerCase().includes(needle));
    let message = null;
    for (const c of mine) {
      const hit = (c.messages || []).find((m) => String(m.content || "").toLowerCase().includes(needle));
      if (hit) { message = String(hit.content); break; }
    }
    if (!needle || a.name.toLowerCase().includes(needle) || chat || message) {
      out.push({ ...withDates(a), matchedChatTitle: chat ? String(chat.title || "") : null, matchedChatMessage: message });
    }
  }
  return out;
});
// A file matches by its path or by a line of its text, and the renderer shows the matching line with
// what surrounds it, so the snippets are cut here rather than left out.
handlers.set("search-app-files", async ({ appId, query }) => {
  const needle = String(query || "").toLowerCase();
  const out = [];
  for (const p of await filesOf(appId)) {
    const snippets = [];
    if (needle) {
      const lines = String((await get("files", appId + ":" + p)) || "").split(/\r?\n/);
      for (let i = 0; i < lines.length && snippets.length < 8; i += 1) {
        const at = lines[i].toLowerCase().indexOf(needle);
        if (at < 0) continue;
        snippets.push({ before: lines[i].slice(Math.max(0, at - 40), at), match: lines[i].slice(at, at + needle.length), after: lines[i].slice(at + needle.length, at + needle.length + 40), line: i + 1 });
      }
    }
    if (!needle || p.toLowerCase().includes(needle) || snippets.length) out.push({ path: p, matchesContent: snippets.length > 0, snippets });
  }
  return out;
});
handlers.set("app:get-current-commit-hash", async ({ appId }) => (await get("refs", appId + ":main")) || null);

// ---- moving in. A browser keeps this store per origin, so apps made on the site Forge was served from
// before stay there; its moved page writes them to a file (hologram-forge/export/1) and Import App on the
// home page hands that file here. Ids are renumbered past the ones in use, so nothing here is replaced;
// a name already taken gets " (moved)". The file carries apps, their files, chats, versions, rules state
// and the objects those versions name; cached rule/proof status is ignored and must be rechecked. It never carries settings (keys), the audit, seals, grants or
// installed holos, and anything else in it is ignored. Every object must derive to its address and
// every referenced snapshot must have complete closure; any defect refuses the whole import. Everything is written in one transaction: a failure leaves the store as it was.
const EXPORT_SCHEMA = "hologram-forge/export/1";
handlers.set("forge:import", async (input) => {
  await coreReady;
  const { apps: entries, objects } = await prepareProjectImport(input, {
    core, derive: kappa,
    admit: (derived, expected) => core.run({ op: "restore", derived, expected }).decision === "Accept",
  }, { takenNames: (await all("apps")).map(a => a.name) });
  let appId = await nextId("apps"), chatId = await nextId("chats");
  const writes = [], apps = [];
  for (const entry of entries) {
    const a = entry.app, name = a.name;
    const id = appId++, at = new Date().toISOString();
    writes.push(["apps", id, { ...nulls, ...a, id, name, path: name, createdAt: a.createdAt || at, updatedAt: a.updatedAt || at }]);
    const files = Object.entries(entry.files);
    for (const [path, text] of files) writes.push(["files", id + ":" + path, text]);
    if (Array.isArray(entry.versions)) writes.push(["versions", id, entry.versions]);
    if (typeof entry.main === "string" && entry.main) writes.push(["refs", id + ":main", entry.main]);
    const chats = [];
    for (const c of Array.isArray(entry.chats) ? entry.chats : []) {
      if (!c || typeof c !== "object") continue;
      const cid = chatId++;
      writes.push(["chats", cid, { ...c, id: cid, appId: id, messages: Array.isArray(c.messages) ? c.messages : [], referencedApps: [] }]);
      chats.push(cid);
    }
    apps.push({ id, name, from: entry.from, chats, files: files.length });
  }
  for (const [k, bytes] of objects) writes.push(["objects", k, bytes]);
  const db = await dbReady;
  await new Promise((res, rej) => {
    let t;
    const refuse = () => rej(Object.assign(new Error("Project storage failed"), { code: "projectStorageFailed" }));
    try {
      t = db.transaction(["apps", "chats", "files", "objects", "refs", "versions"], "readwrite");
      t.oncomplete = res; t.onerror = refuse; t.onabort = refuse;
      const marks = new Map();
      const commit = () => {
        if (marks.size !== 2) return;
        try {
          for (const store of ["apps", "chats"]) {
            const high = marks.get(store), assigned = writes.filter(row => row[0] === store).map(row => row[1]);
            if (!Number.isSafeInteger(high) || high < 0 || assigned.some(id => !Number.isSafeInteger(id) || id <= high)) throw Error("stale allocation");
            if (assigned.length) t.objectStore("refs").put(Math.max(high, ...assigned), `allocation:${store}`);
          }
          for (const [store, key, value] of writes) { if (store === "apps" || store === "chats") t.objectStore(store).add(value, key); else t.objectStore(store).put(value, key); }
        } catch { try { t.abort(); } catch {} refuse(); }
      };
      for (const store of ["apps", "chats"]) { const q = t.objectStore("refs").get(`allocation:${store}`); q.onsuccess = () => { marks.set(store, q.result === undefined ? 0 : q.result); commit(); }; }
    } catch { if (t) { try { t.abort(); } catch {} } refuse(); }
  });
  return { apps, objects: objects.size, refused: 0 };
});

// ---- versions: a branch is a name pointing at an address; rollback restores the object at an
// address only when its bytes re-derive to it (the core's restore rule).
handlers.set("list-versions", async input => listStoredVersions(await dbReady,input));
handlers.set("get-current-branch", async () => ({ branch: "main" }));
async function versionHistoryBoundary(){await coreReady;return {readObject:key=>get("objects",key),derive:kappa,admit:async(derived,expected)=>core.run({op:"restore",derived,expected}).decision==="Accept"};}
handlers.set("get-version-changes", async input => verifiedVersionChanges(await dbReady,input,await versionHistoryBoundary()));
handlers.set("set-version-favorite", async input => setStoredVersionMetadata(await dbReady,input,"isFavorite"));
handlers.set("set-version-note", async input => setStoredVersionMetadata(await dbReady,input,"note"));
async function restore(appId, oid, revert = false) {
  oid = await resolveStoredVersion(await dbReady, appId, oid);
  await coreReady;
  return restoreStoredProject(await dbReady, appId, oid, {
    core, onCommit: hash => emit("version-created", { appId, oid: hash }),
    readObject: key => get("objects", key),
    derive: kappa,
    admit: async (derived, expected) => {
      const { decision } = core.run({ op: "restore", derived, expected });
      await put("audit", Date.now() + ":" + oid, { ts: Date.now(), operation: "snapshot.restore", resource: oid, outcome: decision });
      return decision === "Accept";
    },
  }, revert);
}
handlers.set("checkout-version", async ({ appId, purpose, versionId, branch }) => {
  if(purpose==="return"){if(branch!=="main")throw previewFailure("previewUnsupportedBranch");return versionPreview.dispatch(appId,{type:"CLOSE",operationId:crypto.randomUUID()});}
  if(purpose!=="preview")throw previewFailure("invalidPreviewCommand");if(!(await versionPreview.acquire(appId)).acquired)throw previewFailure("previewWindowBusy");return versionPreview.dispatch(appId,{type:"SELECT_VERSION",versionId,operationId:crypto.randomUUID()});
});
handlers.set("revert-version", async ({appId,previousVersionId,expectedHeadOid,currentChatMessageId,targetBranchName}) => {
  if(targetBranchName!==undefined&&targetBranchName!=="main")throw previewFailure("previewUnsupportedBranch");if(!(await versionPreview.acquire(appId)).acquired)throw previewFailure("previewWindowBusy");return versionPreview.dispatch(appId,{type:"RESTORE",versionId:previousVersionId,...(currentChatMessageId?{currentChatMessageId}:{}),...(expectedHeadOid?{expectedHeadOid}:{}),operationId:crypto.randomUUID()});
});

// ---- chats as records; the stream is the next slice
handlers.set("get-chats", async (appId) => (await all("chats")).filter((c) => appId == null || c.appId === appId).map((c) => chatRow(c)));
handlers.set("get-chat", async (id) => { const c = await get("chats", id); if (!c) throw new Error("no chat " + id); return { ...c, chatMode: c.chatMode ?? null, referencedApps: c.referencedApps || [], messages: (c.messages || []).map((x) => ({ ...x, createdAt: x.createdAt || now() })) }; });
const chatRow = (c) => ({ id: c.id, appId: c.appId, title: c.title, createdAt: new Date(c.createdAt || Date.now()), chatMode: c.chatMode ?? null, isFavorite: !!c.isFavorite });
handlers.set("get-chat-metadata", async (id) => { const c = await get("chats", id); return c ? chatRow(c) : null; });
handlers.set("create-chat", async (input) => {
  const { appId, initialChatMode, firstPromptCreationOperationId } = typeof input === "number" ? { appId: input } : (input || {});
  const capturedMode = browserChatMode(initialChatMode, await get("settings", "user"));
  const chatId = await createStoredChat(await dbReady, appId, now(), capturedMode);
  if (typeof firstPromptCreationOperationId === "string" && firstPromptCreationOperationId) {
    pendingCreations.set(firstPromptCreationOperationId, { kind: "chat", appId, chatId });
  }
  return chatId;
});
handlers.set("update-chat", input => updateChatPreferences(dbReady,input));
handlers.set("delete-chat", async (id) => { await deleteStoredChat(await dbReady, id); queueChannel?.postMessage({chatId:id}); await chatQueue.refresh(id).catch(()=>{}); });
// The sidebar's search: a chat matches by title or by the text of one of its messages, and the line
// that matched is what the renderer shows under it.
handlers.set("search-chats", async (q) => {
  const needle = String(q || "").toLowerCase();
  const out = [];
  for (const c of await all("chats")) {
    const hit = (c.messages || []).find((m) => String(m.content || "").toLowerCase().includes(needle));
    if (!needle || String(c.title || "").toLowerCase().includes(needle) || hit) {
      out.push({ id: c.id, appId: c.appId, title: c.title, createdAt: new Date(c.createdAt || Date.now()), matchedMessageContent: hit ? String(hit.content) : null });
    }
  }
  return out;
});
// Counted where it matters: the codebase and the prompt are what a build turn actually sends, and the
// window is the chosen model's own, so the renderer's context meter tells the truth.
handlers.set("chat:count-tokens", async ({ chatId, input }) => {
  const chat = chatId != null ? await get("chats", chatId) : null;
  const settings = (await get("settings", "user")) || DEFAULT_SETTINGS;
  const route = chatModelRoute(chat?.modelSelection,settings.selectedModel);
  const availableModels = await getUorR4Models();
  const contextWindow = (availableModels.find((m) => m.apiName === route.model) || availableModels[0]).contextWindow;
  const tokens = (text) => Math.ceil(String(text || "").length / 4);
  const captured = chat && chat.appId != null ? await captureProjectSource(await dbReady,chat.appId) : null;
  const moduleNow = captured ? activeApplicationModule(captured, lexlean.canonicalSource) : null;
  const codebaseTokens = tokens(moduleNow && appShape.isAppShape(moduleNow[1]) ? prism.moduleContext(moduleNow[1]) : prism.specContext(captured ? new Map(captured.files).get("prism/spec.json") : null));
  const systemPromptTokens = tokens(chatExecutionPrompt(chatExecutionPolicy(null,chat?.chatMode),prism.SPEC_RULES) + "\n\n" + (chat && chat.appId != null ? await themeLibrary("resolve",{appId:chat.appId}) : ""));
  const history = chat ? await messagesOf(chatId) : [];
  const messageHistoryTokens = history.reduce((n,m)=>n+tokens(m.content),0);
  const inputTokens = tokens(input);
  return { estimatedTotalTokens: codebaseTokens + systemPromptTokens + messageHistoryTokens + inputTokens, actualMaxTokens: null, messageHistoryTokens, codebaseTokens, mentionedAppsTokens: 0, inputTokens, systemPromptTokens, contextWindow };
});

// ---- the app run machine. the renderer talks to app running as a remote machine: it subscribes
// to a key and dispatches intents; the host owns the state and publishes snapshots. Here START builds
// the project in the tab (esbuild-wasm, dependencies through an import map) and serves it through
// the worker under the model's previewPath of the head address; PROXY_READY is the snapshot with
// phase ready and that url. No process is spawned anywhere.
const machines = new Map(); // "app_run:<appId>" -> { revision, state }
const caps = (phase) => ({ canStart: phase === "idle" || phase === "stopped" || phase === "errored", canRestart: phase === "ready", canRebuild: phase === "ready", canStop: phase === "ready" || phase === "starting", canReload: phase === "ready" });
function machineOf(appId) {
  const key = "app_run:" + appId;
  if (!machines.has(key)) machines.set(key, { revision: 0, state: { appId, revision: 0, previewReloadEpoch: 0, phase: "idle", operation: null, startedAt: null, url: null, operationError: null, exit: null, capabilities: caps("idle"), invocationRef: null, lastSettlement: null } });
  return machines.get(key);
}
// ---- build telemetry (shell/telemetry, studio/telemetry/src): the turn's moments are marked here, in the
// live frame's messages and in publish, kept in this browser's IndexedDB, and never sent. Each mark is four
// numbers into a preallocated ring (well under a microsecond); the record is written once, at idle, after
// the turn. Where a record may go is the kernel's decision (`telemetry EFFECT ASKED DEVICE`, FK-25 to FK-27).
// The live frame's paint is timed when its message arrives here: the frame posts it on the animation frame
// that paints, so the reading is the paint plus one message hop.
const probe = createProbe();
const tm = { turn: null, model: null, token: false, declaration: false, frame: false, complete: false, repaired: false, editOpen: false };
const tmMark = (kind, value = 0) => probe.mark(K[kind], value);
const tmOnce = (kind) => { if (tm.turn && !tm[kind]) { tm[kind] = true; tmMark(kind); } };
function tmStart(chatId) {
  probe.drain();
  Object.assign(tm, { turn: chatId, model: null, token: false, declaration: false, frame: false, complete: false, repaired: false });
  tmMark("enter");
}
let telemetryReady = null;
const telemetryOf = () => telemetryReady ??= (async () => {
  const [{ createTelemetry }, { fromData }, { indexedDbBackend }] = await Promise.all([import("./telemetry/telemetry.mjs"), import("./telemetry/model.mjs"), import("./telemetry/store.mjs")]);
  const read = async (name) => (await fetch(new URL("./telemetry/data/" + name + ".json", import.meta.url))).json();
  const names = await read("modules");
  const model = fromData(Object.fromEntries(await Promise.all(names.map(async (n) => [n, await read(n)]))));
  return { model, telemetry: createTelemetry({ model, backend: indexedDbBackend(), ask: (line) => window.__forgeKernel.ask(line) }) };
})();
const tmIdle = (fn) => (self.requestIdleCallback ? requestIdleCallback(fn, { timeout: 5000 }) : setTimeout(fn, 500));
// An approval or a publish is not a build turn: its Enter is forgotten.
const tmCancel = (chatId) => { if (tm.turn === chatId) { tm.turn = null; probe.drain(); } };
// The turn's events become one record at idle; a refusal or a failure costs the turn nothing.
function tmFlush() {
  const { events, dropped } = probe.drain(), modelName = tm.model || "unknown";
  tm.turn = null;
  if (!events.length) return;
  tmIdle(() => telemetryOf().then(({ telemetry }) => telemetry.record({ events, dropped, model: modelName })).catch((e) => console.warn("[telemetry]", (e && e.code) || e)));
}
// What each app was let reach: the worker hands over the kernel's decisions as it serves an app
// (sw.template.js, noteGate); they are recorded at idle like a turn. A preview's address changes with every
// build, so the address is named here with the app it belongs to, and its builds count as one app.
const tmApps = new Map();
const tmNameApp = (address, appId, name) => tmApps.set(address, { key: "app:" + appId, name: name || null });
if (navigator.serviceWorker) navigator.serviceWorker.addEventListener("message", (e) => {
  const d = e.data;
  if (!d || d.type !== "telemetry/gate" || typeof d.id !== "string" || !Array.isArray(d.decisions)) return;
  const named = tmApps.get(d.id) || {};
  tmIdle(() => telemetryOf().then(({ telemetry }) => telemetry.noteApp({ id: d.id, key: named.key ?? null, name: named.name ?? null, installed: d.installed === true, decisions: d.decisions })).catch((err) => console.warn("[telemetry]", (err && err.code) || err)));
});
async function openTelemetry() {
  const [{ openPanel }, t] = await Promise.all([import("./telemetry/panel.mjs"), telemetryOf()]);
  openPanel(t);
}
addEventListener("keydown", (e) => { if (e.ctrlKey && e.altKey && (e.key === "t" || e.key === "T")) { e.preventDefault(); openTelemetry().catch(() => {}); } });
const tmHash = () => { if (location.hash === "#telemetry") openTelemetry().catch(() => {}); };
addEventListener("hashchange", tmHash);
addEventListener("load", tmHash, { once: true });

function publish(appId, patch) {
  const m = machineOf(appId);
  if (tm.turn && patch.previewReloadEpoch > (m.state.previewReloadEpoch || 0)) tmMark("reload");
  m.revision += 1;
  m.state = { ...m.state, ...patch, revision: m.revision, capabilities: caps(patch.phase || m.state.phase) };
  emit("distributed-machine:snapshot", { protocolVersion: 1, machineId: "app_run", encodedKey: { appId }, actorInstanceId: "web-app_run-" + appId, revision: m.revision, encodedState: m.state });
  if (m.state.phase === "ready" && m.state.url && m.state.url.appUrl) scheduleCover(appId, m.state.url.appUrl);
  return m;
}
// ---- app covers (covers.mjs). Every preview carries the capture responder; when an app's preview is
// ready and its version has no cover yet, the preview is loaded once more, hidden, at cover size, and its
// picture is stored with the version it shows. The card's words are asked of the visitor's own model
// once per app, and stored only when they meet the rules in covers.mjs.
let coverLibraryReady = null;
const coverLibrary = () => (coverLibraryReady ??= fetch(new URL("assets/html-to-image.js", BASE)).then((r) => (r.ok ? r.text() : null)).catch(() => null));
async function withCoverResponder(html) {
  const library = await coverLibrary();
  return library ? covers.withResponder(html, library) : html;
}
const coverTimers = new Map();
const coverSources = new Map();
let coverQueue = Promise.resolve();
// A turn writes in batches and the preview is announced ready after each; the picture waits for the last.
function scheduleCover(appId, appUrl) {
  if (!String(appUrl).startsWith(BASE.href)) return;
  // Capture when the preview is announced, before debounce, capture or model work.
  const built = coverSources.get(appId);
  const live = appUrl === liveFrameUrl + "#" + encodeURIComponent(appId);
  if (!live && built?.appUrl !== appUrl) return;
  const source = live ? dbReady.then(db => captureProjectSource(db, appId)) : Promise.resolve(structuredClone(built.source));
  void source.catch(() => {});
  clearTimeout(coverTimers.get(appId));
  coverTimers.set(appId, setTimeout(() => {
    coverTimers.delete(appId);
    coverQueue = coverQueue.then(async () => coverApp(appId, appUrl, await source)).catch(e => console.warn("cover", appId, e?.code || "galleryStorageFailed"));
  }, 4000));
}
async function galleryChanged(appId) {
  dispatchEvent(new CustomEvent(covers.EVENT, { detail: { appId } }));
  await windowCoordination.invalidate([{family:"apps"}]).catch(() => {});
}
async function coverApp(appId, appUrl, source) {
  const version = source.reference + " " + appUrl;
  const current = await readGallery(dbReady, appId);
  if (current?.version === version) return;
  const checkProjection = () => {
    if (appUrl === liveFrameUrl + "#" + encodeURIComponent(appId)) {
      const json = new Map(source.files).get("prism/spec.json");
      if (!json || JSON.stringify(JSON.parse(json)) !== JSON.stringify(liveLatest.get(appId)?.raw)) throw galleryFailure("gallerySourceChanged");
    } else if (coverSources.get(appId)?.appUrl !== appUrl) throw galleryFailure("gallerySourceChanged");
  };
  checkProjection();
  const shot = await covers.capture(appUrl);
  if (!shot) return;
  checkProjection();
  await publishGallery(dbReady, appId, source, {picture:await covers.encodeCover(shot.png),version});
  await galleryChanged(appId);
}
const esbuild = () => loadBrowserCompiler(BASE);
// Generated-code previews disclose network destinations before compilation. The grant is
// installed only after the captured source and prior permission record still match.
// Canonical live projection keeps its separate network-free serving grant.
const PREVIEW_ENDPOINTS = ["https://esm.sh", "https://cdn.tailwindcss.com"];
const previewId = (pathname) => pathname.split("/").filter(Boolean).pop();
async function grantPreview(id, endpoints, permission = null, expected) {
  if (permission) return commitPreviewPermission(await dbReady, id, permission, expected);
  const record = { serve: true, network: { endpoints: [...endpoints], denied: [] } };
  try { await put("refs", "capabilities:" + id, record); }
  catch { throw previewPermissionFailure("previewPermissionStorageFailed"); }
}
mountPreviewPermissions({
  list: async () => {
    const rows = [];
    for (const key of await keys("refs")) {
      if (typeof key !== "string" || !key.startsWith("capabilities:")) continue;
      const record = await get("refs", key);
      if (record?.serve === true && record.previewConsent?.profile === "foundry/preview-consent/1") rows.push({ id: key.slice(13), source: record.previewConsent.source, endpoints: [...record.network.endpoints] });
    }
    return rows;
  },
  revoke: async id => {
    try { await put("refs", "capabilities:" + id, { serve: false, network: { endpoints: [], denied: [] } }); }
    catch { throw previewPermissionFailure("previewPermissionStorageFailed"); }
    const prefix = new URL(`p/${id}/`, BASE).href;
    for (const frame of document.querySelectorAll("iframe")) if (frame.src.startsWith(prefix)) frame.src = "about:blank";
  },
});

// The Hologram brand kit on every app the model writes as code: the kit's tokens in the scaffold's own
// variable names (shell/scaffold-brand.css, generated by tools/brand_kit.py) and its web fonts as data
// URLs, since a preview loads nothing from this origin. Added unlayered after the app's CSS, so the
// kit wins over the scaffold's stock theme, which sits in `@layer base`.
const BRAND_FONTS = [["Geist", "Geist-Regular", 400], ["Geist", "Geist-Medium", 500], ["Geist", "Geist-SemiBold", 600], ["Archivo", "Archivo-SemiBold", 600]];
let brandHeadReady = null;
function brandHead() {
  return (brandHeadReady ??= (async () => {
    const r = await fetch(new URL("scaffold-brand.css", BASE));
    if (!r.ok) throw new Error("scaffold-brand.css answered " + r.status);
    let faces = "";
    for (const [family, file, weight] of BRAND_FONTS) {
      const f = await fetch(new URL("fonts/" + file + ".woff2", BASE));
      if (f.ok) faces += `@font-face { font-family: "${family}"; src: url("data:font/woff2;base64,${b64(new Uint8Array(await f.arrayBuffer()))}") format("woff2"); font-weight: ${weight}; font-display: swap; }\n`;
    }
    return `<style>${faces}${(await r.text()).replace(/<\/style/gi, "<\\/style")}</style>`;
  })().catch((error) => { brandHeadReady = null; throw error; }));
}

async function verifiedBuildSource(appId) {
  return captureBuildSource(await dbReady, appId, {
    readObject: key => get("objects", key), derive: kappa,
    admit: async (derived, expected) => core.run({ op: "restore", derived, expected }).decision === "Accept",
  });
}
async function buildPreview(appId) {
  await coreReady;
  const source = await verifiedBuildSource(appId);
  if (applicationShape(source) === 'stateful') throw shapeFailure('statefulBuildUnsupported');
  const { path } = core.run({ op: "preview", kappa: source.reference });
  const appUrl = new URL(path.slice(1), BASE);
  const owner = await get("apps", appId);
  const endpoints = Array.isArray(owner && owner.grants) ? owner.grants : PREVIEW_ENDPOINTS;
  tmNameApp("p:" + previewId(appUrl.pathname), appId, owner && owner.name);
  const previousPermission = await get("refs", "capabilities:" + previewId(appUrl.pathname));
  const permission = endpoints.length ? await authorizePreview({ id: previewId(appUrl.pathname), source: source.reference, endpoints }) : null;
  const approvedEndpoints = permission ? permission.disclosure.endpoints : [];
  const files = new Map(source.files);
  const pkg = await (await fetch(new URL("scaffold/package.json", BASE))).json();
  const pkgs = Object.fromEntries(Object.entries(pkg.dependencies).map(([n, v]) => [n, v.replace(/^[\^~]/, "")]));
  // Packages the model added with <prism-add-dependency>: name or name@version, resolved by esm.sh.
  for (const d of source.dependencies) { const at = d.lastIndexOf("@"); if (at > 0) pkgs[d.slice(0, at)] = d.slice(at + 1); else pkgs[d] = ""; }
  const react = pkgs.react, dom = pkgs["react-dom"];
  const imports = { react: `https://esm.sh/react@${react}`, "react/": `https://esm.sh/react@${react}/`, "react-dom": `https://esm.sh/react-dom@${dom}?external=react`, "react-dom/": `https://esm.sh/react-dom@${dom}&external=react/` };
  for (const [n, v] of Object.entries(pkgs)) { if (n === "react" || n === "react-dom") continue; const at = v ? "@" + v : ""; imports[n] = `https://esm.sh/${n}${at}?external=react,react-dom`; imports[n + "/"] = `https://esm.sh/${n}${at}&external=react,react-dom/`; }
  const tw = await (await fetch(new URL("scaffold/tailwind.config.ts", BASE))).text();
  const m = tw.match(/theme:\s*(\{[\s\S]*\n  \}),\n  plugins/);
  const tailwindConfig = m ? "{darkMode:['class'],theme:" + m[1] + "}" : "{}";
  const exts = ["", ".tsx", ".ts", ".jsx", ".js", "/index.tsx", "/index.ts"];
  const resolveFile = (p) => { for (const e of exts) if (files.has(p + e)) return p + e; return null; };
  const plugin = { name: "project", setup(b) {
    b.onResolve({ filter: /.*/ }, (args) => {
      if (args.kind === "entry-point") { const r = resolveFile(args.path); if (r) return { path: r, namespace: "project" }; }
      if (args.path.startsWith("@/")) { const r = resolveFile("src/" + args.path.slice(2)); if (r) return { path: r, namespace: "project" }; }
      if (/^\.\.?\//.test(args.path) || args.path.startsWith("/")) {
        const base = args.importer ? args.importer.split("/").slice(0, -1).join("/") : "src";
        const norm = new URL(args.path, "file:///" + base + "/").pathname.slice(1);
        const r = resolveFile(norm); if (r) return { path: r, namespace: "project" };
        return { errors: [{ text: "not in project: " + norm }] };
      }
      return { path: args.path, external: true };
    });
    b.onLoad({ filter: /.*/, namespace: "project" }, (args) => {
      const src = files.get(args.path);
      if (args.path.endsWith(".css")) return { contents: src.replace(/@tailwind[^;]*;/g, ""), loader: "css" };
      return { contents: src, loader: args.path.endsWith(".tsx") ? "tsx" : args.path.endsWith(".ts") ? "ts" : "jsx" };
    });
  } };
  const es = await esbuild();
  const r = await es.build({ entryPoints: ["src/main.tsx"], bundle: true, write: false, format: "esm", target: "es2022", jsx: "automatic", plugins: [plugin], outdir: "out", logLevel: "silent",
    define: { "import.meta.env.DEV": "true", "import.meta.env.MODE": '"development"', "import.meta.env.BASE_URL": JSON.stringify(appUrl.pathname), "process.env.NODE_ENV": '"development"' } });
  const js = r.outputFiles.find((f) => f.path.endsWith(".js")).text;
  const css = (r.outputFiles.find((f) => f.path.endsWith(".css")) || { text: "" }).text;
  const html = `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<script type="importmap">${JSON.stringify({ imports })}<\/script>
<script>tailwind = { config: ${tailwindConfig} };<\/script>
<script src="https://cdn.tailwindcss.com"><\/script>
<style>${css}</style>${await brandHead()}</head><body><div id="root"></div>
<script type="module">${js}<\/script></body></html>`;
  const shown = await withCoverResponder(html);
  await assertBuildSourceCurrent(await dbReady, appId, source);
  const cache = await caches.open("previews");
  await cache.put(appUrl.pathname + "index.html", new Response(shown, { headers: { "content-type": "text/html; charset=utf-8" } }));
  try { await grantPreview(previewId(appUrl.pathname), approvedEndpoints, permission, previousPermission); }
  catch (error) { await cache.delete(appUrl.pathname + "index.html"); throw error; }
  coverSources.set(appId, {appUrl:appUrl.href,source:structuredClone(source)});
  return { appUrl: appUrl.href, bytes: js.length };
}
// ---- the live preview. An app with rules is not built: it is projected. One fixed frame on shadcn/ui
// (shell/live/, compiled once by tools/live_components.mjs), addressed by the kappa of its own bytes and served once under the preview route with
// no network grant at all, draws any spec it is sent. The host sends it the projection of the spec as
// it stands (mid-stream, checked, proved) and the frame patches only the nodes whose kappa changed,
// so a change to the spec reaches the screen without a build or a reload.
let liveFrameReady = null;
function liveFrame() {
  return (liveFrameReady ??= (async () => {
    const text = async (p) => { const r = await fetch(new URL(p, BASE)); if (!r.ok) throw new Error(p + " answered " + r.status); return r.text(); };
    const fonts = {};
    for (const p of FONTS) { const r = await fetch(new URL(p, BASE)); if (r.ok) fonts[p] = b64(new Uint8Array(await r.arrayBuffer())); }
    const html = await withCoverResponder(frameHtml({ brandCss: await text("brand.css"), componentsCss: await text("live/components.css"), componentsJs: await text("live/components.js"), interpreter: prism.INTERPRETER, fonts }));
    // The worker serves the frame: on a first visit the frame waits for it, or it would load nothing.
    await coreReady; await workerActive;
    const { path } = core.run({ op: "preview", kappa: await kappa(enc.encode(html)) });
    const url = new URL(path.slice(1), BASE);
    await grantPreview(previewId(url.pathname), []);
    await (await caches.open("previews")).put(url.pathname + "index.html", new Response(html, { headers: { "content-type": "text/html; charset=utf-8" } }));
    return url.href;
  })().catch((error) => { liveFrameReady = null; throw error; }));
}
const liveWindows = new Map(); // the frame's window -> the app it shows
const liveLatest = new Map();  // appId -> { raw, trust }
const livePending = new Set();
const liveShown = new Set();   // apps whose preview a streaming turn has pointed at the live frame
const liveTimes = [];          // patch to pixel, as each frame reports it
const themeOf = () => (document.documentElement.classList.contains("dark") ? "dark" : "light");
// A frame is trusted to receive an app's spec only when it is an iframe of this page showing the live frame.
const isLiveFrame = (source) => !!liveFrameUrl && [...document.querySelectorAll("iframe")].some((f) => f.contentWindow === source && f.src.startsWith(liveFrameUrl));
let liveFrameUrl = null, workerActive = null;
function sendLive(appId) {
  const latest = liveLatest.get(appId);
  if (!latest) return;
  const p = live.project(latest.raw);
  const message = { type: "live/projection", app: String(appId), root: p.root, logic: p.logic, logicKappa: p.logicKappa, nodes: p.nodes, trust: latest.trust, theme: themeOf(), edit: !!latest.edit };
  for (const [win, app] of liveWindows) { if (win.closed) liveWindows.delete(win); else if (app === appId) win.postMessage(message, "*"); }
}
// Coalesced to one projection per animation frame: a model streams far faster than a screen draws.
// A stateful app (shell/live/app.mjs): its module is run as it is written, and each frame goes to the
// same live frame a decision's projection goes to. The library model is fetched on the first such app.
const appApps = new Set();     // apps whose preview is a running module, not a projection
const postApp = (appId, message) => { const m = { ...message, theme: themeOf() }; for (const [win, app] of liveWindows) { if (win.closed) liveWindows.delete(win); else if (app === appId) win.postMessage(m, "*"); } };
let appFrames = null;
let appModelLoad = null;
const appModel = () => appModelLoad || (appModelLoad = fetch(new URL("live/v2/Model.json", BASE)).then((r) => { if (!r.ok) throw new Error("live/v2/Model.json answered " + r.status); return r.json(); }).catch((error) => { appModelLoad = null; throw error; }));
const appFramesReady = () => appFrames || (appFrames = appModel().then((model) => appShape.appLive({ post: postApp, library: { "Foundation.View.Browser.V1.Model": model } })).catch(error => { appFrames = null; throw error; }));
// The checker for a stateful app runs in this tab (shell/live/app.mjs checkApp): it reads, it runs, its theorems can be stated.
const checkAppModule = async (module) => appShape.checkApp(module, { "Foundation.View.Browser.V1.Model": await appModel() });
function pushApp(appId, module, trust = "draft", options = {}) {
  assertGenerationActive(options.signal);
  appApps.add(appId);
  appFramesReady().then((frames) => { assertGenerationActive(options.signal); return frames.push(appId, module, trust, options); }).catch((e) => { if (!options.signal?.aborted) console.error(e); });
}
async function settleAppFrames(appId, signal) {
  assertGenerationActive(signal);
  if (appFrames && appApps.has(appId)) await (await appFrames).settled(appId);
  assertGenerationActive(signal);
}
function forgetApplicationFrames(appId) {
  liveLatest.delete(appId); livePending.delete(appId); liveEditors.delete(appId); livePointed.delete(appId);
  appApps.delete(appId);
  if (appFrames) appFrames.then(frames => frames.forget(appId)).catch(() => {});
}
function pushLive(appId, raw, trust, edit = false) {
  liveLatest.set(appId, { raw, trust, edit });
  // An edit is one change a person just made: it goes now, not at the next frame, so the frame can paint
  // it on its own next frame. Only a model's stream, many changes a frame, is coalesced.
  if (edit) { livePending.delete(appId); sendLive(appId); return; }
  if (livePending.has(appId)) return;
  livePending.add(appId);
  requestAnimationFrame(() => { livePending.delete(appId); sendLive(appId); });
}
window.addEventListener("message", (e) => {
  const m = e.data;
  if (!m || typeof m.type !== "string" || !m.type.startsWith("live/") || !isLiveFrame(e.source)) return;
  // The frame reads its app from the address (a string); the host keys apps as the store does (a number).
  if (m.type === "live/hello") {
    // The hidden warm-up frame has no app. Only a real project identity may read source.
    const app = typeof m.app === "number" ? m.app : typeof m.app === "string" && /^[1-9][0-9]*$/.test(m.app) ? Number(m.app) : 0;
    if (!Number.isSafeInteger(app) || app < 1) return;
    liveWindows.set(e.source, app);
    Promise.resolve().then(async () => {
      if (appApps.has(app)) { const frames = await appFramesReady(); return frames.has(app) ? frames.resend(app) : refreshPreview(app); }
      if (liveLatest.has(app)) return sendLive(app);
      return refreshPreview(app);
    }).catch(() => refuse(app, null, "The app preview could not be refreshed. Reopen it to try again."));
    return;
  }
  if (m.type === "live/painted") { if (m.changed) { liveTimes.push(m.ms); if (liveTimes.length > 200) liveTimes.shift(); } return; }
  // Everything else acts on an app: the one this frame was shown, never the one the message names.
  const appId = liveWindows.get(e.source);
  if (!appId) return;
  if (m.type === "live/intent") { if (appApps.has(appId)) appFramesReady().then((frames) => frames.intent(appId, m)).catch(() => {}); return; }
  if (m.type === "live/patch" && Array.isArray(m.ops)) patchSpec(appId, m.ops, { author: "user", key: m.key || null }).catch(() => {});
  else if (m.type === "live/undo" || m.type === "live/redo") stepSpec(appId, m.type === "live/undo" ? "undo" : "redo").catch(() => {});
  else if (m.type === "live/point") { if (livePointed.get(appId)?.key === m.key) livePointed.delete(appId); else livePointed.set(appId, { key: m.key, kind: m.kind, edit: m.edit || {}, text: String(m.text || "").slice(0, 200) }); }
});
new MutationObserver(() => { for (const appId of liveLatest.keys()) pushLive(appId, liveLatest.get(appId).raw, liveLatest.get(appId).trust); }).observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
// Build telemetry watches the same frame messages through its own listener, so the live handler above stays
// exactly the production one its tests lift out; it applies the same trust checks and only marks moments.
window.addEventListener("message", (e) => {
  const m = e.data;
  if (!m || typeof m.type !== "string" || !m.type.startsWith("live/") || !isLiveFrame(e.source)) return;
  if (m.type === "live/hello") { if (tm.turn) tmOnce("frame"); return; }
  if (m.type === "live/painted") {
    if (!m.changed) return;
    if (tm.turn) { tmMark("update", Math.round((m.work || 0) * 1000)); tmMark("pixel"); }
    if (tm.editOpen) { tm.editOpen = false; tmMark("editScreen"); if (!tm.turn) tmFlush(); }
    return;
  }
  if (m.type === "live/patch" && Array.isArray(m.ops) && liveWindows.get(e.source)) { tm.editOpen = true; tmMark("edit"); }
});
// The rules as they stand: the checked spec at its kernel phase, else what the model wrote, as a draft.
// A stateful app is its module: it is run again from the stored source. Resolves whether this app is one.
async function refreshApp(appId, options = {}) {
  const app = await appStateOf(appId);
  if (!app) return false;
  pushApp(appId, app.source, app.state?.phase || 'draft', options);
  return true;
}
// What the preview shows for an app as it stands: a running module, or a decision's projection.
async function refreshPreview(appId, options = {}) { if (!(await refreshApp(appId, options))) await refreshLive(appId); }
async function refreshLive(appId) {
  const current = await captureProjectSource(await dbReady, appId);
  if (applicationShape(current) === 'stateful') return;
  const json = await get("files", appId + ":prism/spec.json");
  if (!json) return;
  const state = await rulesOf(appId);
  // The spec as written is drawn, not the checker's normalised copy: every edit path on the screen is a
  // path in this spec, so a click patches exactly what the model or the user wrote.
  try {
    const bound = state?.spec && state.specKappa === await kappa(enc.encode(json));
    const trust = bound ? state.phase === "published" && state.publicationCurrent === false ? "proved" : state.phase : "draft";
    const spec = JSON.parse(json);
    pushLive(appId, spec, trust);
    // The edit history is ready before the first click, so even the first edit is drawn optimistically,
    // before any storage read, instead of waiting for the history to be built.
    if (liveEditors.has(appId)) liveEditors.get(appId).sync(spec); else liveEditors.set(appId, createEditor(spec, checkForEdit));
    if (liveShown.has(appId) && liveFrameUrl) scheduleCover(appId, liveFrameUrl + "#" + encodeURIComponent(appId));
  } catch (e) {}
}

// ---- the one write path. A click on the screen and a patch the agent writes arrive here alike, as
// spec patches (shell/live/edit.mjs). Each is drawn at once (optimistic), then checked by Forge's
// checker in milliseconds; a patch that does not check is refused, the last good spec is drawn again,
// and the reason is shown on the node that was edited. A patch that checks is saved through the same
// path as a chat turn (rulesAfterTurn: the kernel's phase, the rule module) and sealed as a version.
// Patches to one app run one at a time, in order, so the user and the agent never race.
const liveEditors = new Map(); // appId -> its edit history
const livePointed = new Map(); // appId -> the node the user points the agent at
const liveQueue = new Map();
const checkForEdit = (spec) => { try { const c = prism.checkSpec(spec); return { ok: !!c.ok, issues: c.issues || [] }; } catch (e) { return { ok: false, issues: [String((e && e.message) || e)] }; } };
async function editorOf(appId, source) {
  const json = source ? new Map(source.files).get("prism/spec.json") : await get("files", appId + ":prism/spec.json");
  if (!json) return null;
  const spec = JSON.parse(json);
  let ed = liveEditors.get(appId);
  if (!ed) { ed = createEditor(spec, checkForEdit); liveEditors.set(appId, ed); } else ed.sync(spec);
  return ed;
}
const liveBusy = new Map(); // appId -> jobs queued or running
function inOrder(appId, job) {
  liveBusy.set(appId, (liveBusy.get(appId) || 0) + 1);
  const run = () => Promise.resolve().then(job).finally(() => { const n = liveBusy.get(appId) - 1; if (n) liveBusy.set(appId, n); else liveBusy.delete(appId); });
  const next = (liveQueue.get(appId) || Promise.resolve()).then(run, run);
  liveQueue.set(appId, next.catch(() => {}));
  return next;
}
function refuse(appId, key, message) {
  for (const [win, app] of liveWindows) if (app === appId && !win.closed) win.postMessage({ type: "live/refused", app: String(appId), key, message }, "*");
}
async function saveSpec(appId, spec, options = {}) {
  const result = await rulesAfterTurn(appId, "<prism-spec>" + JSON.stringify(spec).replaceAll("<", "\\u003c") + "</prism-spec>", [], { onCommit: (oid) => emit("version-created", { appId, oid }), ...options });
  await refreshLive(appId);
  return result;
}
async function patchSpec(appId, ops, { author = "user", key = null, ...options } = {}) {
  ops = structuredClone(ops);
  // Optimistic, before anything is awaited: with the app's history already in memory and nothing ahead of
  // it in the queue, the patched spec is drawn now, then checked.
  const cached = liveEditors.get(appId);
  let drawn = false;
  if (cached && !liveBusy.has(appId)) { try { pushLive(appId, applyOps(cached.spec, ops), "draft", true); drawn = true; } catch (e) { if (!(e instanceof PatchError)) throw e; } }
  return inOrder(appId, async () => {
    assertGenerationActive(options.signal);
    const source = options.source || await captureProjectSource(await dbReady, appId);
    const ed = await editorOf(appId, source);
    if (!ed) return { ok: false, issues: ["This app has no rules yet."] };
    const before = ed.spec;
    try { if (!drawn) pushLive(appId, applyOps(before, ops), "draft", true); }
    catch (e) { if (!(e instanceof PatchError)) throw e; refuse(appId, key, e.message); return { ok: false, issues: [e.message] }; }
    // The check samples up to 200,000 requests on this thread, which the live frame shares: it waits
    // until the frame has painted the edit, so the person sees their change before it is judged.
    await new Promise((r) => (typeof requestAnimationFrame === "function" && typeof setTimeout === "function" ? requestAnimationFrame(() => setTimeout(r, 0)) : r()));
    const result = ed.apply(ops);
    if (!result.ok) { await refreshLive(appId); refuse(appId, key, result.issues[0]); return result; }
    try {
      const saved = await saveSpec(appId, result.spec, { ...options, source });
      return { ...result, ...saved };
    } catch (e) {
      liveEditors.delete(appId);
      await refreshLive(appId);
      refuse(appId, key, e.message);
      throw e;
    }
  });
}
async function stepSpec(appId, way) {
  return inOrder(appId, async () => {
    const source = await captureProjectSource(await dbReady, appId);
    const ed = await editorOf(appId, source);
    if (!ed || !(way === "undo" ? ed.canUndo : ed.canRedo)) return;
    const spec = way === "undo" ? ed.undo() : ed.redo();
    pushLive(appId, spec, "draft", true);
    try { await saveSpec(appId, spec, { source }); }
    catch (e) { liveEditors.delete(appId); await refreshLive(appId); refuse(appId, null, e.message); throw e; }
  });
}
// The built page takes the preview back from the live frame (an app's first turn opened the frame, then
// wrote code, not rules): announced as a restart, so the renderer forgets the live frame's route.
async function showBuilt(appId) {
  const { appUrl } = await buildPreview(appId);
  if (liveShown.has(appId)) return;
  const m = machineOf(appId);
  publish(appId, { phase: "starting", operation: "restart", invocationRef: { kind: "app-run", entityKey: appId, operationId: "built-" + appId + "-" + Date.now() }, startedAt: Date.now(), operationError: null, exit: null });
  publish(appId, { phase: "ready", operation: null, invocationRef: null, startedAt: Date.now(), operationError: null, exit: null, url: { appUrl, originalUrl: appUrl, mode: "cloud" }, previewReloadEpoch: m.state.previewReloadEpoch + 1 });
}
// Point the app's preview at the live frame. The iframe loads once: after this, the address it shows
// never changes for this app, so the preview never reloads.
async function showLive(appId) {
  liveShown.add(appId);
  liveFrameUrl = await liveFrame();
  const appUrl = liveFrameUrl + "#" + encodeURIComponent(appId);
  const m = machineOf(appId);
  if (m.state.phase === "ready" && m.state.url && m.state.url.appUrl === appUrl) return appUrl;
  // The renderer keeps the route the preview was on when the new address shares its origin (it expects
  // a dev server per app), and every preview here shares this origin; so a preview already showing
  // another page is first announced as restarting, the renderer's own signal to forget that route.
  if (m.state.url) publish(appId, { phase: "starting", operation: "restart", invocationRef: { kind: "app-run", entityKey: appId, operationId: "live-" + appId + "-" + Date.now() }, startedAt: Date.now(), operationError: null, exit: null });
  publish(appId, { phase: "ready", operation: null, invocationRef: null, startedAt: Date.now(), operationError: null, exit: null, url: { appUrl, originalUrl: appUrl, mode: "cloud" }, previewReloadEpoch: m.state.previewReloadEpoch + 1 });
  return appUrl;
}
// An app whose preview is the live frame: one with rules, or one that is a running module.
const hasRules = async (appId) => !!(await get("files", appId + ":prism/spec.json")) || !!(await get("files", appId + ":" + appShape.SOURCE_PATH));
// An app whose code is still the scaffold it was made from. Code a model writes is never applied, so an
// app made from an idea stays one until its rules arrive: its preview is the live frame, which Forge draws
// from the rules on this origin, and nothing is built or fetched from another site for it.
async function scaffoldOnly(appId) {
  const made = await loadScaffold(), files = (await captureProjectSource(await dbReady, appId)).files;
  return files.every(([path, text]) => !(path.startsWith("src/") || Object.hasOwn(made, path)) || made[path] === text);
}
const projected = async (appId) => (await hasRules(appId)) || (await scaffoldOnly(appId));
// Warm before the first spec: the frame is built (about 90 ms of fetching, encoding and hashing) and
// loaded once, hidden, while the page is idle, so the worker has it cached and its kernel has verified
// itself and answered the gate before a model writes a word. The hidden copy goes once it says hello.
function warmLive() {
  liveFrame().then((url) => {
    liveFrameUrl = url;
    const f = document.createElement("iframe");
    f.src = url + "#";
    f.setAttribute("aria-hidden", "true"); f.tabIndex = -1;
    f.style.cssText = "position:fixed;width:0;height:0;border:0;opacity:0;pointer-events:none";
    const done = (e) => { if (e.source === f.contentWindow && e.data && e.data.type === "live/hello") { window.removeEventListener("message", done); f.remove(); } };
    window.addEventListener("message", done);
    setTimeout(() => { window.removeEventListener("message", done); f.remove(); }, 15000);
    document.body.append(f);
  }).catch(() => {});
}
(window.requestIdleCallback || ((fn) => setTimeout(fn, 1500)))(warmLive, { timeout: 4000 });
window.__live = { liveFrame, showLive, pushLive, pushApp, appFrames: appFramesReady, refreshLive: refreshPreview, patchSpec, stepSpec, pointed: livePointed, times: liveTimes, project: live.project };

handlers.set("distributed-machine:subscribe", async ({ machineId, encodedKey }) => {
  if (machineId === "version_preview") return previewEnvelope(encodedKey.appId);
  if (machineId === "chat_stream") return chatSubscribe({ machineId, encodedKey });
  if (machineId !== "app_run") throw new Error("no browser machine " + machineId);
  const m = machineOf(encodedKey.appId);
  return { protocolVersion: 1, machineId, encodedKey, actorInstanceId: "web-app_run-" + encodedKey.appId, revision: m.revision, encodedState: m.state };
});
handlers.set("distributed-machine:unsubscribe", () => undefined);
handlers.set("distributed-machine:dispatch", async (envelope) => {
  const { machineId, encodedKey, messageId, encodedEvent } = envelope;
  if (machineId === "version_preview") return previewDispatch(envelope);
  if (machineId === "chat_stream") return chatDispatch(envelope);
  if (machineId !== "app_run") throw new Error("no browser machine " + machineId);
  const appId = encodedKey.appId, type = encodedEvent && encodedEvent.type;
  const receipt = (m) => ({ kind: "applied", actorInstanceId: "web-app_run-" + appId, revision: m.revision, transactionSequence: m.revision, messageId });
  if (type === "START" || type === "RESTART" || type === "MANUAL_RELOAD") {
    const op = type === "START" ? "run" : type === "RESTART" ? (encodedEvent.operation === "rebuild" ? "rebuild" : "restart") : "reload";
    // An app with rules is projected, not built: the live frame already shows it, so a start, restart
    // or reload keeps the frame (no epoch bump, no reload) and resends the rules.
    if(await get("files",appId+":prism/interaction.json")!==undefined){
      try{const opened=await openInteractionApplication(appId,{run:input=>handlers.get("forge:interaction-run")(input),probe:input=>executeInteraction(kernelReady,input)});if(!opened.current)throw interactionFailure("interactionApplicationMismatch");}
      catch(error){return receipt(publish(appId,{phase:"errored",operation:null,operationError:{message:error.message}}));}
      return receipt(publish(appId,{phase:"stopped",operation:null,url:null,lastSettlement:{operationId:encodedEvent.operationId||messageId,kind:"run",outcome:"succeeded"}}));
    }
    // The first build may have opened the frame before its first module was complete.
    if (liveShown.has(appId) || await projected(appId)) {
      try { await showLive(appId); if (await hasRules(appId)) await refreshPreview(appId); }
      catch (e) { const m = publish(appId, { phase: "errored", operation: null, operationError: { message: String((e && e.message) || e) } }); return receipt(m); }
      const m = publish(appId, { lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "run", outcome: "succeeded" } });
      return receipt(m);
    }
    const m = publish(appId, { phase: type === "MANUAL_RELOAD" ? "reloading" : "starting", operation: op, startedAt: Date.now(), operationError: null, exit: null });
    m.pending = (async () => {
      try {
        const { appUrl } = await buildPreview(appId);
        // A build that finishes after the app's rules took the preview (a model began writing a spec
        // while it ran) is about a page the app no longer shows: the live frame stays, and the start it
        // answered still settles, or the renderer keeps waiting on it.
        if (liveShown.has(appId)) { await showLive(appId).catch(() => {}); publish(appId, { phase: "ready", operation: null, lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "run", outcome: "succeeded" } }); return; }
        const cur = machineOf(appId);
        publish(appId, { phase: "ready", operation: null, url: { appUrl, originalUrl: appUrl, mode: "cloud" }, previewReloadEpoch: cur.state.previewReloadEpoch + 1, lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "run", outcome: "succeeded" } });
      } catch (e) {
        publish(appId, { phase: "errored", operation: null, operationError: { message: String((e && e.message) || e) }, lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "run", outcome: "failed", error: { message: String((e && e.message) || e) } } });
      }
    })();
    return receipt(m);
  }
  if (type === "STOP_REQUESTED") {
    const m = publish(appId, { phase: "stopped", operation: null, url: null, exit: { exitCode: 0, timestamp: Date.now() }, lastSettlement: { operationId: encodedEvent.operationId || messageId, kind: "stop", outcome: "succeeded" } });
    return receipt(m);
  }
  return { kind: "ignored", actorInstanceId: "web-app_run-" + appId, revision: machineOf(appId).revision, messageId, reason: "unknown intent " + type };
});
handlers.set("connection-flow:get-states", async () => ({
  github: await githubClient.getFlowState(),
  supabase: { status: "disconnected", revision: 0 },
  neon: { status: "disconnected", revision: 0 },
}));
handlers.set("window-infrastructure:attach-interest", input => windowCoordination.attach(input));
handlers.set("window-infrastructure:detach-interest", input => windowCoordination.detach(input));
const previewEpoch=crypto.randomUUID();
const previewActorId=appId=>`web-version_preview-${appId}-${previewEpoch}`;
const previewEnvelope=appId=>({protocolVersion:1,machineId:"version_preview",encodedKey:{appId},actorInstanceId:previewActorId(appId),revision:versionPreview.snapshot(appId).revision,encodedState:VersionPreviewRemoteSnapshotSchema.parse(versionPreview.snapshot(appId))});
async function previewModelBoundary(){await coreReady;return {guest:kernelReady,core,derive:kappa,readObject:key=>get("objects",key),admit:async(derived,expected)=>core.run({op:"restore",derived,expected}).decision==="Accept"};}
const versionPreview=createVersionPreviewController({
  locks:navigator.locks,
  store:async appId=>{await coreReady;return previewProjectStore(await dbReady,appId,{guest:kernelReady,core,derive:kappa,readObject:key=>get("objects",key),admit:async(derived,expected)=>core.run({op:"restore",derived,expected}).decision==="Accept"});},
  mintOwner:()=>{let owner=0;while(!owner)owner=crypto.getRandomValues(new Uint32Array(1))[0];return owner;},
  prepareMessageCommand:async(appId,event)=>{
    const db=await dbReady,model=await previewModelBoundary(),fork=event.type==="RESTORE_TO_MESSAGE",chatId=fork?event.chatId:event.currentChatMessageId.chatId,messageId=fork?event.messageId:event.currentChatMessageId.messageId;
    const prepared=await prepareMessageRestoration(db,{appId,chatId,messageId,mode:fork?"fork":"truncate",restoreCodebase:fork?event.restoreCodebase:true,...(!fork?{versionId:event.versionId}:{})},{resolve:reference=>resolveStoredVersion(db,appId,reference),verify:reference=>prepareSnapshot(reference,model)});
    const effectIdentity=await kappa(new TextEncoder().encode(JSON.stringify(event)));
    const request={kind:prepared.versionId?4:3,...(prepared.versionId?{versionId:prepared.versionId}:{}),effectIdentity,...(event.expectedHeadOid?{expectedHeadOid:versionAddress(event.expectedHeadOid)}:{})};
    return {request,boundary:previewProjectStore(db,appId,model,{chatUpdate:prepared.chatUpdate})};
  },
  validateEvent:value=>VersionPreviewIntentEventSchema.safeParse(value),
  publish:(appId,snapshot)=>emit("distributed-machine:snapshot",{protocolVersion:1,machineId:"version_preview",encodedKey:{appId},actorInstanceId:previewActorId(appId),revision:snapshot.revision,encodedState:VersionPreviewRemoteSnapshotSchema.parse(snapshot)}),
  created:(appId,oid)=>emit("version-created",{appId,oid}),
  restart:async appId=>{const operationId=crypto.randomUUID();await handlers.get("distributed-machine:dispatch")({protocolVersion:1,machineId:"app_run",encodedKey:{appId},messageId:operationId,encodedEvent:{type:"RESTART",operationId}});await machineOf(appId).pending;const m=machineOf(appId);if(m.state.lastSettlement?.operationId!==operationId||m.state.lastSettlement.outcome!=="succeeded")throw previewFailure("previewUnavailable");},
});
async function previewDispatch(raw){
  const {encodedKey,messageId,encodedEvent,expectedActorInstanceId}=structuredClone(raw),appId=encodedKey?.appId;
  if(!Number.isSafeInteger(appId)||appId<1||typeof messageId!=="string"||!messageId||messageId.length>256)throw previewFailure("invalidPreviewCommand");
  const actorInstanceId=previewActorId(appId);if(expectedActorInstanceId&&expectedActorInstanceId!==actorInstanceId)return {kind:"rejected",messageId,reason:"stale-actor"};
  let result;try{result=await versionPreview.dispatch(appId,encodedEvent);}catch(error){emit("version-preview:result",{appId,operationId:encodedEvent.operationId,notification:{kind:"error",message:error.message},affectedChatId:null,createdChatId:null});throw error;}if(result)emit("version-preview:result",{appId,operationId:encodedEvent.operationId,notification:result.notification,affectedChatId:result.affectedChatId,createdChatId:result.createdChatId});const revision=versionPreview.snapshot(appId).revision;
  return {kind:"applied",actorInstanceId,revision,transactionSequence:revision,messageId};
}
handlers.set("version-preview:acquire-window-interest", ({appId}) => versionPreview.acquire(appId));
handlers.set("version-preview:restore-window-interest", ({appId}) => versionPreview.acquire(appId));
handlers.set("version-preview:release-window-interest", ({appId,operationId}) => versionPreview.release(appId,operationId));
window.addEventListener("pagehide",()=>versionPreview.dispose(),{once:true});
handlers.set("is-capacitor", () => false);
handlers.set("get-proposal", () => null);
handlers.set("select-app-for-preview", () => undefined);
handlers.set("git:get-uncommitted-files", async input => readWorkingTree(await dbReady,input,await versionHistoryBoundary()));
handlers.set("git:get-uncommitted-file-diff", async input => readWorkingTree(await dbReady,input,await versionHistoryBoundary()));
handlers.set("github:list-local-branches", async (appId) => {
  const id = typeof appId === "object" ? appId?.appId : appId;
  const app = id ? await get("apps", id) : null;
  const current = app?.githubBranch || "main";
  return { branches: [current], current };
});
handlers.set("reload-env-path", () => undefined);
handlers.set("get-cloud-sandbox-status", () => null);
// The chat's context: which of the project's files a turn carries. the renderer keeps the globs on the app
// record and shows each one with what it matches, so the counts are measured here over the app's own
// files -- no glob library, the two wildcards the renderer's dialog writes are enough.
const globMatch = (glob, path) => new RegExp("^" + String(glob).split("**").map((s) => s.split("*").map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("[^/]*")).join(".*") + "$").test(path);
async function contextCounts(appId, globs) {
  const paths = await filesOf(appId);
  const out = [];
  for (const { globPath } of globs || []) {
    let files = 0, chars = 0;
    for (const p of paths) if (globMatch(globPath, p)) { files += 1; chars += String((await get("files", appId + ":" + p)) || "").length; }
    out.push({ globPath, files, tokens: Math.ceil(chars / 4) });
  }
  return out;
}
handlers.set("get-context-paths", async ({ appId }) => {
  const c = ((await get("apps", appId)) || {}).chatContext || {};
  return {
    contextPaths: await contextCounts(appId, c.contextPaths),
    smartContextAutoIncludes: await contextCounts(appId, c.smartContextAutoIncludes),
    excludePaths: await contextCounts(appId, c.excludePaths),
  };
});
handlers.set("set-context-paths", async ({ appId, chatContext }) => { const a = await get("apps", appId); if (!a) throw new Error("no app " + appId); a.chatContext = chatContext; await put("apps", appId, a); });
// Unimplemented imported capabilities have no handler. The typed dispatch
// boundary reports unsupportedChannel, preserving the renderer error state.
window.__preview = buildPreview;

// ---- the renderer's tags: what the model writes becomes files. The grammar is the renderer's: a block per
// file, attributes path and description, content verbatim between the tags; delete and rename are
// single tags; add-dependency names packages the import map must carry.
function parsePrismTags(text) {
  const writes = [], deletes = [], renames = [], deps = [];
  const attr = (s, name) => { const m = s.match(new RegExp(name + '="([^"]*)"')); return m ? m[1] : null; };
  for (const m of text.matchAll(/<prism-write\s+([^>]*)>([\s\S]*?)<\/prism-write>/g)) {
    let content = m[2]; if (content.startsWith("\n")) content = content.slice(1); if (content.endsWith("\n")) content = content.slice(0, -1);
    const fence = content.match(/^```[\w-]*\n([\s\S]*?)\n```\s*$/); if (fence) content = fence[1];
    writes.push({ path: attr(m[1], "path"), description: attr(m[1], "description"), content });
  }
  for (const m of text.matchAll(/<prism-delete\s+([^>]*?)\s*\/?>/g)) deletes.push({ path: attr(m[1], "path") });
  for (const m of text.matchAll(/<prism-rename\s+([^>]*?)\s*\/?>/g)) renames.push({ from: attr(m[1], "from"), to: attr(m[1], "to") });
  for (const m of text.matchAll(/<prism-add-dependency\s+([^>]*?)\s*\/?>/g)) { const p = attr(m[1], "packages") || attr(m[1], "package"); if (p) deps.push(...p.split(/\s+/).filter(Boolean)); }
  return { writes, deletes, renames, deps };
}
async function applyPrismTags(appId, tags, options = {}) {
  const copied = copyGenerationInput(tags, options.source), { signal, onCommit } = options;
  await coreReady;
  assertGenerationActive(signal);
  const source = copied.source || await captureProjectSource(await dbReady, appId);
  assertCanonicalGeneration(source, copied.tags, lexlean);
  return applyStoredProjectPatch(await dbReady, appId, copied.tags, { core, derive: kappa, source, signal, onCommit });
}
window.__chat = { parsePrismTags, applyPrismTags };

let standardsCache = null;
async function loadStandardsIndex() {
  if (!standardsCache) {
    const resp = await fetch(new URL("standards-index.json", BASE), { cache: "no-store" });
    if (!resp.ok) throw new Error(`standards-index.json HTTP ${resp.status}`);
    standardsCache = await resp.json();
  }
  return standardsCache;
}
async function lookupStandards(query, limit = 5) {
  const index = await loadStandardsIndex();
  const sections = Array.isArray(index?.sections) ? index.sections : [];
  const q = String(query || "").trim().toLowerCase();
  const terms = q.split(/\s+/).filter(Boolean);
  if (!terms.length) return { ok: true, count: sections.length, matches: sections.slice(0, limit) };
  const scored = sections
    .map((sec) => {
      const id = String(sec.id || "").toLowerCase();
      const title = String(sec.title || "").toLowerCase();
      const tags = Array.isArray(sec.tags) ? sec.tags.join(" ") : "";
      const hay = `${id} ${title} ${sec.topic || ""} ${tags} ${sec.source || ""} ${sec.summary || ""} ${sec.content || sec.body || ""}`.toLowerCase();
      let score = 0;
      for (const term of terms) {
        if (id.includes(term)) score += 6;
        if (title.includes(term)) score += 4;
        if (tags.toLowerCase().includes(term)) score += 4;
        if (hay.includes(term)) score += 2;
      }
      return { sec, score };
    })
    .filter((x) => x.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit);
  return { ok: true, query, matches: scored.map((x) => x.sec) };
}
function cleanWorkspacePath(raw, allowEmpty = false) {
  let s = String(raw ?? "").trim().replace(/\\/g, "/");
  s = s.replace(/^\/workspace\/?/, "").replace(/^\.\//, "").replace(/^\/+/, "").replace(/\/+$/, "");
  if (allowEmpty && (!s || s === "root" || s === ".")) return "";
  if (!s || s.split("/").some((p) => !p || p === "." || p === "..") || /[\x00-\x1f:]/.test(s)) {
    throw new Error(`Invalid workspace path: ${raw}`);
  }
  return s;
}
function createFoundryTools({ appId = null, chatMode = "build", signal, onCommit }) {
  return {
    workspace_list: async ({ dir } = {}) => {
      if (!appId) {
        return { ok: true, dir: "root", files: [], note: 'Home Chat has no open Studio app yet. Click "Build this in Studio" to create one.' };
      }
      const prefix = cleanWorkspacePath(dir ?? "root", true);
      const allFiles = await filesOf(appId);
      const files = prefix ? allFiles.filter((f) => f === prefix || f.startsWith(prefix + "/")) : allFiles;
      return { ok: true, appId, dir: prefix || "root", files };
    },
    workspace_read: async ({ path: rawPath, start_line, end_line } = {}) => {
      if (!appId) return { ok: false, error: "Home Chat has no open Studio app." };
      const rel = cleanWorkspacePath(rawPath, false);
      const text = await get("files", `${appId}:${rel}`);
      if (text === undefined) return { ok: false, error: `File not found: ${rel}` };
      const lines = String(text).split("\n");
      const start = Number.isInteger(start_line) && start_line > 0 ? start_line - 1 : 0;
      const end = Number.isInteger(end_line) && end_line >= start + 1 ? end_line : lines.length;
      return { ok: true, path: rel, total_lines: lines.length, start_line: start + 1, end_line: Math.min(end, lines.length), content: lines.slice(start, end).join("\n") };
    },
    workspace_write: async ({ path: rawPath, content } = {}) => {
      if (!appId || (chatMode !== "build" && chatMode !== "local-agent")) {
        return { ok: false, error: "Read-only mode; switch to Build or promote to Studio to write files." };
      }
      const rel = cleanWorkspacePath(rawPath, false);
      const text = String(content ?? "");
      await applyPrismTags(appId, { writes: [{ path: rel, description: "workspace_write", content: text }], deletes: [], renames: [], deps: [] }, { signal, onCommit });
      return { ok: true, path: rel, bytes: text.length, updatedFiles: true };
    },
    linux_exec: async ({ command, cwd } = {}) => {
      const cmd = String(command ?? "").trim();
      if (!cmd) return { ok: false, error: "command is required" };
      const res = await linuxVmHost.execCommand(appId ?? null, cmd, { echoToSession: Boolean(appId), cwd });
      return { ok: res.exit_code === 0, runtime: "linux-6.18.7-v86", kernel: "6.18.7", ...res };
    },
    mvm_exec: async ({ command } = {}) => {
      const cmd = String(command ?? "").trim();
      if (!cmd) return { ok: false, error: "command is required" };
      const res = await mvmHost.execCommand(appId ?? null, cmd, { echoToSession: Boolean(appId) });
      return { ok: res.exit_code === 0, ...res };
    },
    github_status: async () => ({ ok: true, ...(await githubClient.getStatus()) }),
    github_list_repos: async ({ per_page } = {}) => {
      const repos = await githubClient.listRepos({ perPage: per_page || 30 });
      return { ok: true, count: repos.length, repos };
    },
    github_clone: async ({ url, owner, repo, repository, branch, app_name, appName } = {}) => {
      return githubClient.cloneRepoFromUrl({
        url,
        owner,
        repo: repo ?? repository,
        branch,
        appName: app_name ?? appName,
      });
    },
    github_pull: async ({ branch } = {}) => {
      if (!appId) return { ok: false, error: "Open a Studio app before pulling from GitHub." };
      return githubClient.pull({ appId, branch });
    },
    github_push: async ({ message, branch } = {}) => {
      if (!appId) return { ok: false, error: "Open a Studio app before pushing to GitHub." };
      return githubClient.push({ appId, message, branch });
    },
    prismpm_verify: async ({ target, action } = {}) => {
      await coreReady;
      if (!appId) return { ok: true, target: target || "registry", view: core.run({ op: "view" }) };
      const source = await captureProjectSource(await dbReady, appId);
      const shape = applicationShape(source);
      const rules = await rulesOf(appId);
      if (action === "publish") {
        const pub = await handlers.get("holo:publish")({ appId });
        return { ok: Boolean(pub?.kappa), target, shape, rules, publication: pub };
      }
      return { ok: true, target: target || shape, shape, rules, applicationState: source.applicationState || null };
    },
    lexlean_check: async ({ path: rawPath } = {}) => {
      if (!appId) return { ok: false, error: "Open a Studio app to check a LexLean module." };
      const rel = cleanWorkspacePath(rawPath || "prism/Application.lex.tex", false);
      const text = await get("files", `${appId}:${rel}`);
      if (text === undefined) return { ok: false, error: `Module not found: ${rel}` };
      if (appShape.isAppShape(text)) {
        const verdict = await checkAppModule(text, { signal });
        return { ok: Boolean(verdict.ok), path: rel, shape: "stateful", issues: verdict.issues || [] };
      }
      const parsed = lexlean.specFromTurn(text);
      return { ok: Boolean(parsed && !parsed.error), path: rel, shape: "rules", error: parsed?.error || null };
    },
    lexlean_verify: async ({ path: rawPath, profile } = {}) => {
      if (!appId) return { ok: false, error: "Open a Studio app to verify LexLean proofs." };
      const rel = cleanWorkspacePath(rawPath || "prism/spec.json", false);
      if (rel === "prism/spec.json") {
        const rawSpec = await get("files", `${appId}:prism/spec.json`);
        if (!rawSpec) return { ok: false, error: "prism/spec.json not found in app" };
        const checked = await prism.checkSpec(JSON.parse(rawSpec));
        return { ok: Boolean(checked?.ok), path: rel, profile: profile || "core", issues: checked?.issues || [] };
      }
      const text = await get("files", `${appId}:${rel}`);
      if (text === undefined) return { ok: false, error: `Module not found: ${rel}` };
      const verdict = await checkAppModule(text, { signal });
      return { ok: Boolean(verdict?.ok), path: rel, profile: profile || "core", issues: verdict?.issues || [] };
    },
    standards_lookup: async ({ query } = {}) => lookupStandards(query, 5),
    stitch_fetch: async ({ url, method } = {}) => {
      const target = String(url ?? "").trim();
      if (!target) return { ok: false, error: "url is required" };
      if (target.startsWith("stitch://")) {
        return stitchNetwork.fetchPeer(target, { method: method || "GET" });
      }
      const safeMethod = String(method || "GET").trim().toUpperCase();
      if (!["GET", "HEAD"].includes(safeMethod)) {
        return { ok: false, error: "stitch_fetch supports GET or HEAD methods only" };
      }
      let parsed;
      try {
        parsed = new URL(target, location.href);
      } catch {
        return { ok: false, error: "Invalid URL" };
      }
      const isSameOrigin = parsed.origin === location.origin;
      const defaultAllowedHosts = new Set([
        "raw.githubusercontent.com",
        "api.github.com",
        "github.com",
        "huggingface.co",
        "cdn-lfs.huggingface.co",
        "uor-foundation.github.io",
        "casey-allard.github.io",
        "registry.npmjs.org",
      ]);
      let customAllowed = [];
      try {
        customAllowed = JSON.parse(localStorage.getItem("uor.stitch.allowedOrigins") || "[]");
      } catch {}
      const originAllowed =
        isSameOrigin ||
        (parsed.protocol === "https:" &&
          (defaultAllowedHosts.has(parsed.hostname) ||
            (Array.isArray(customAllowed) && customAllowed.includes(parsed.origin))));
      if (!originAllowed) {
        return {
          ok: false,
          error: `stitch_fetch blocked cross-origin URL (${parsed.origin}); only same-origin or explicitly allowlisted HTTPS hosts are permitted.`,
        };
      }
      const resp = await fetch(parsed.href, {
        method: safeMethod,
        credentials: "omit",
        redirect: "error",
        signal,
      });
      const MAX_PREVIEW_BYTES = 8192;
      let text = "";
      if (safeMethod !== "HEAD" && resp.body?.getReader) {
        const reader = resp.body.getReader();
        const chunks = [];
        let total = 0;
        try {
          for (;;) {
            const { done, value } = await reader.read();
            if (done || !value) break;
            const take = Math.min(value.byteLength, MAX_PREVIEW_BYTES - total);
            if (take > 0) chunks.push(value.subarray(0, take));
            total += take;
            if (total >= MAX_PREVIEW_BYTES) {
              void reader.cancel().catch(() => {});
              break;
            }
          }
        } finally {
          reader.releaseLock();
        }
        const merged = new Uint8Array(total);
        let offset = 0;
        for (const chunk of chunks) {
          merged.set(chunk, offset);
          offset += chunk.byteLength;
        }
        text = dec.decode(merged);
      } else if (safeMethod !== "HEAD") {
        text = (await resp.text()).slice(0, MAX_PREVIEW_BYTES);
      }
      return { ok: resp.ok, status: resp.status, url: parsed.href, body_preview: text };
    },
    stitch_peer_status: async ({ peer_id } = {}) => ({
      ok: true,
      peer_id: peer_id || null,
      v13_status: uorR4.getStatus(),
      xor_2_of_3_ready: true,
      stitch: await stitchNetwork.getStatus(),
    }),
  };
}

// ---- the chat stream machine. the renderer submits a turn only here: SUBMIT on the chat_stream
// machine, then it watches the snapshot (admitting, streaming, finalizing, idle) and applies the
// chunk events to the message it renders. The model is OpenRouter with the visitor's own key; the
// prompt is the renderer's text tag build prompt, so the answer carries <prism-write> blocks the host applies
// to the project, seals as a version, and the renderer then reloads the preview by itself.
const chatActorEpoch=crypto.randomUUID();
const chatActorId=chatId=>"web-chat_stream-"+chatId+"-"+chatActorEpoch;
const chatMachines = new Map(); // chatId -> { revision, state, abort }
const idleChat = (chatId) => ({ schemaVersion: 1, chatId, revision: 0, phase: "idle", invocationRef: null, error: null, queueRevision: 0, queue: [], queuePaused: false, queuePauseReason: null, stopPolicyVersion: 0, capabilities: { canSubmit: true, canCancel: false, canPauseQueue: true, canResumeQueue: false }, lastAcceptance: null, lastCompletion: null, lastQueueMutation: null });
function chatMachineOf(chatId) { if (!chatMachines.has(chatId)) chatMachines.set(chatId, { revision: 0, state: idleChat(chatId), abort: null }); return chatMachines.get(chatId); }
function publishChat(chatId, patch) {
  const m = chatMachineOf(chatId);
  m.revision += 1;
  const phase = patch.phase || m.state.phase;
  const queuePaused = patch.queuePaused === undefined ? m.state.queuePaused : patch.queuePaused;
  m.state = { ...m.state, ...patch, revision: m.revision, capabilities: { canSubmit: true, canCancel: phase === "admitting" || phase === "streaming", canPauseQueue: !queuePaused, canResumeQueue: queuePaused } };
  emit("distributed-machine:snapshot", { protocolVersion: 1, machineId: "chat_stream", encodedKey: { chatId }, actorInstanceId: chatActorId(chatId), revision: m.revision, encodedState: m.state });
  // shellUpdate is declared below and read here before it exists while the host starts: var, and checked.
  if (shellUpdate && shellUpdate.waiting && !turnRunning()) setTimeout(reloadIfIdle, 1500);
  return m;
}
// ---- never cut a turn short. A turn (a build, a proof, a publish) runs in this page; reloading the page
// ends it here while a cloud model may go on generating its answer, holding the shared plan's one slot
// for minutes. So: a newer shell waits until no turn runs (the worker announces it, see sw.template.js);
// leaving the page while one runs asks first; and a turn a reload did cut is marked as cut, not left as a
// silent half message.
function turnRunning() { return [...chatMachines.values()].some((m) => ["admitting", "streaming", "finalizing", "cancelling"].includes(m.state.phase)) || Boolean(homeChatAbort); }
var shellUpdate = { waiting: false };
function reloadIfIdle() { if (shellUpdate.waiting && !turnRunning()) location.reload(); }
if (navigator.serviceWorker) navigator.serviceWorker.addEventListener("message", (e) => {
  if (e.data && e.data.type === "shell-updated") { shellUpdate.waiting = true; reloadIfIdle(); }
});
addEventListener("beforeunload", (e) => { if (turnRunning()) { e.preventDefault(); e.returnValue = ""; } });
// A turn a reload cut: the last message of a chat left as an open reasoning block or empty. Nothing runs
// when the page starts, so such a message is never a live turn.
const CUT = "\n\n<prism-output type=\"warning\" message=\"This answer was interrupted: the page reloaded before it finished. Ask again.\"></prism-output>";
async function markInterruptedTurns() {
  for (const chat of await all("chats")) {
    const last = chat && chat.messages && chat.messages[chat.messages.length - 1];
    if (!last || last.role !== "assistant" || last.content.includes("<prism-output")) continue;
    const open = /^<think>/.test(last.content) && !last.content.includes("</think>");
    if (!open && last.content.trim()) continue;
    last.content = (open ? last.content + "</think>" : last.content) + CUT;
    await updateStoredChat(await dbReady, chat.id, { messages: chat.messages });
  }
}
markInterruptedTurns().catch((e) => console.warn("[host] marking interrupted turns", e));
const djb2 = (str) => { let h = 5381; for (let i = 0; i < str.length; i++) h = (((h << 5) + h) ^ str.charCodeAt(i)) >>> 0; return h; };
const now = () => new Date().toISOString();
async function messagesOf(chatId) { const c = await get("chats", chatId); return c ? c.messages : []; }
async function saveMessages(chatId, messages) { return updateStoredChat(await dbReady, chatId, { messages }); }
// ---- proved rules. An app that decides something carries prism/spec.json, a Prism spec. Its build
// phase (drafting, checked, proving, proved, published) is the Forge kernel's answer, never this
// file's: the kernel is the Prism app whose state machine Lean proves (FK-01 to FK-04), so the chat
// can say proved only after Lean attested the current rules, and publish only proved rules. The
// state lives at refs["prism:<appId>"].
const kernelReady = import(new URL("kernel.js", BASE).href).then(() => self.ForgeKernel.load((p) => fetch(new URL(p, BASE))));
// The renderer's screens ask the kernel too, for the decisions it holds about them (the build screen's layout,
// what counts as this week): the answer is the kernel's, the screen only draws it. Nothing is asked before the
// guest has passed its pin and every acceptance vector.
window.__forgeKernel = Object.freeze({ ask: (line) => kernelReady.then((k) => k.ask(line)) });
// The build screen draws whichever layout the kernel last answered for this window's width, then asks again:
// the answer is asked here as soon as the kernel is ready, and on every resize, so the build screen starts in
// the kernel's layout instead of drawing the other one and remounting.
const askLayout = () => kernelReady.then((k) => k.ask("layout " + Math.max(0, Math.round(innerWidth)))).then((a) => { if (a === "tabs" || a === "split") { try { localStorage.setItem("holo.layout.narrow", a === "tabs" ? "1" : "0"); } catch (e) {} } }).catch(() => {});
askLayout();
addEventListener("resize", askLayout);
async function representationCommand(operation, input, reconstruct = false) {
  if (!input || typeof input !== "object" || Array.isArray(input) || Object.keys(input).sort().join(",") !== (reconstruct ? "basis,bytes,identity" : "basis,bytes")) return { ok: false, code: "invalidRepresentation" };
  try { return { ok: true, value: await (reconstruct ? operation(input.bytes, input.identity, input.basis) : operation(input.bytes, input.basis)) }; }
  catch (error) { if (error instanceof RepresentationError) return { ok: false, code: error.code }; throw error; }
}
handlers.set("forge:pack-bytes", input => representationCommand(packBytes, input));
handlers.set("forge:unpack-bytes", input => representationCommand(unpackBytes, input, true));
handlers.set("forge:compose", async (input) => {
  if (!input || typeof input !== "object" || Array.isArray(input) || Object.keys(input).sort().join(",") !== "features,policy") return { ok: false, code: "malformedComposition", error: "Composition requires policy and features" };
  return admitWithGuest(await kernelReady, input.policy, input.features);
});
handlers.set("forge:work", async input => executeWorkWithGuest(await kernelReady, input));
handlers.set("forge:event", async input => admitEventWithGuest(await kernelReady, input));
handlers.set("forge:queue", input=>queueWithGuest(kernelReady,input));
handlers.set("forge:version-preview", input => previewWithGuest(kernelReady,input));
handlers.set("forge:route", input => routeWithGuest(kernelReady,input));
handlers.set("forge:interact", input => executeInteraction(kernelReady, input));
let interactionModelSource;
async function loadInteractionModelSource(){
  if(interactionModelSource!==undefined)return interactionModelSource;
  try{const response=await fetch(new URL("interaction-model.lex.tex",BASE),{credentials:"omit",redirect:"error"});if(!response.ok)throw Error();const text=await response.text();if(text.length>131072||!text.startsWith("\\begin{lexlean}{InteractionModel}"))throw Error();interactionModelSource=text;return text;}catch{throw interactionFailure("interactionApplicationMismatch");}
}
handlers.set("forge:interaction-create", async raw => {
  const input=structuredClone(raw);
  if(!input || Object.keys(input).sort().join()!=="kind,name" || typeof input.name!=="string" || !input.name.trim() || input.name.length>120)throw interactionFailure("invalidInteractionApplication");
  const document=await validateInteractionDocument(JSON.stringify(interactionPreset(input.kind)),kernelReady);
  await coreReady;
  const modelSource=await loadInteractionModelSource();
  const files={"prism/interaction.json":JSON.stringify(document,null,2)+"\n","prism/Application.lex.tex":interactionModule(document.graph),"prism/InteractionModel.lex.tex":modelSource,"README.md":"# "+document.title+"\nOpen this project using Foundry Interactive apps. Edit prism/interaction.json to author its canonical profile.\n"};
  const prepared=await prepareNewProject({name:input.name,files},{core,derive:kappa});
  const created=await createStoredProject(await dbReady,prepared,nulls);
  emit("version-created",{appId:created.app.id,oid:created.oid});return {app:withDates(created.app),chatId:created.chatId};
});
handlers.set("forge:interaction-run", async input => {const copied=structuredClone(input);await coreReady;return runStoredInteraction(await dbReady,kernelReady,kappa,copied,await loadInteractionModelSource());});

async function eventDeliveryCommand(operation, raw) {
  const input=copyEventCommand(operation,raw);
  const db=await dbReady;
  if(operation==="pending")return readEventJournal(db,input.appId);
  return updateEventJournal(db,await kernelReady,input.appId,operation,input.context ?? input.envelope ?? input.receipt);
}
handlers.set("forge:event-configure", input => eventDeliveryCommand("configure", input));
handlers.set("forge:event-deliver", input => eventDeliveryCommand("deliver", input));
handlers.set("forge:event-pending", input => eventDeliveryCommand("pending", input));
handlers.set("forge:event-acknowledge", input => eventDeliveryCommand("acknowledge", input));
const PHASES = ["drafting", "checked", "proving", "proved", "published"];
const discovery=createDiscovery({guest:kernelReady,derive:kappa,create:input=>handlers.get("forge:interaction-create")(input)});
handlers.set("forge:discover", input=>discovery.discover(input));
handlers.set("forge:discovery-create", input=>discovery.accept(input));
async function nextPhase(phase, event) {
  const answer = await (await kernelReady).ask(phase + " " + event);
  if (!PHASES.includes(answer)) throw new Error("the kernel answered " + answer + " to " + phase + " " + event);
  return answer;
}
const rulesOf = async (appId) => readBoundRules(await dbReady, appId, kappa);
const rulesSnapshotOf = async (appId) => readBoundRulesSnapshot(await dbReady, appId, kappa);
const saveRules = async (appId, state, expected, signal) => saveBoundRules(await dbReady, appId, state, kappa, expected, signal);
// After a turn: the spec the model wrote (if any) becomes prism/spec.json; new rules drop the proof
// (the kernel's edit), are checked (its check), and on a pass the app's rule module is written.
// Returns the files changed and the card that replaces the spec in the chat.
// The end of a turn that wrote a stateful app: its module is kept as its source and checked. Resolves
// null when the turn wrote none, and the turn is then a decision's (rulesAfterTurn).
async function appAfterTurn(appId, text, options = {}) {
  const appModule = options.moduleText === undefined ? appShape.moduleFromTurn(text, { final: true }) : {text:options.moduleText};
  // Lines that still do not read change nothing; the chat says what the reader said.
  if (appModule && appModule.notation && !appModule.text) {
    assertGenerationActive(options.signal);
    const source = options.source || await captureProjectSource(await dbReady, appId);
    assertGenerationActive(options.signal);
    return { changed: 0, source, card: appShape.notationCard(appModule.issues) };
  }
  if (!appModule || !appShape.isAppShape(appModule.text)) return null;
  assertGenerationActive(options.signal);
  await coreReady;
  const source = options.source || await captureProjectSource(await dbReady, appId);
  const files = new Map(source.files), writes = [], deletes = retiredApplicationPaths(source, 'stateful', appShape.SOURCE_PATH, lexlean.canonicalSource).map(path => ({path}));
  const stage = (path, content) => { if (files.get(path) !== content) { files.set(path, content); writes.push({ path, content }); } };
  const appSource = appModule.text.trim() + "\n";
  stage(appShape.SOURCE_PATH, appSource);
  if (appModule.notation) stage(appShape.NOTATION_PATH, appModule.notation);
  // The phase is the kernel's answer: an edit sends the app back to drafting, a check that passes
  // moves it to checked. A check that could not be made leaves it drafting and says why.
  const known = new Map(source.files).has('prism/spec.json') ? null : await boundApplicationState(source, kappa);
  const sourceKappa = await kappa(enc.encode(appSource));
  let appState;
  if (known && known.sourceKappa === sourceKappa && ["checked", "proving", "proved", "published"].includes(known.phase)) appState = known;
  else {
    let phase = known && known.phase ? await nextPhase(known.phase, "edit") : "drafting";
    // A reading is not sent anywhere to be checked as it is shown: it is checked when the person approves it.
    const verdict = options.check === false ? { ok: false, issues: [] } : await checkAppModule(appSource, { signal: options.signal, session: options.session }).catch((e) => ({ ok: false, issues: [], unchecked: String((e && e.message) || e).slice(0, 200) }));
    assertGenerationActive(options.signal);
    if (verdict.ok) phase = await nextPhase(phase, "check");
    appState = { phase, sourceKappa, issues: verdict.issues || [], ...(verdict.unchecked ? { unchecked: verdict.unchecked } : {}), updated: now() };
    stage(appShape.STATE_PATH, JSON.stringify(appState, null, 2) + "\n");
  }
  stage(appShape.STATE_PATH, JSON.stringify(appState, null, 2) + '\n');
  const priorRules = await rulesSnapshotOf(appId);
  const kept = writes.length || deletes.length || priorRules.expected !== undefined || JSON.stringify(source.applicationState) !== JSON.stringify(appState)
    ? await applyStoredProjectPatch(await dbReady, appId, { writes, deletes, renames: [], deps: [] }, { core, derive: kappa, ...options, source, rulesUpdate:{expected:priorRules.expected,state:null}, appUpdate:{expected:source.applicationState,state:appState} })
    : {changed:0,source};
  assertGenerationActive(options.signal);
  if (kept.changed && (new Map(source.files).has('prism/spec.json') || !appShape.isAppShape(new Map(source.files).get(appShape.SOURCE_PATH)))) forgetApplicationFrames(appId);
  return { changed: kept.changed, source: kept.source, card: appShape.card(appState) };
}
const afterTurn = async (appId, text, written = [], options = {}) => (await appAfterTurn(appId, text, options)) || rulesAfterTurn(appId, text, written, options);
async function rulesAfterTurn(appId, text, written = [], options = {}) {
  assertGenerationActive(options.signal);
  await coreReady;
  const source = options.source || await captureProjectSource(await dbReady, appId);
  const files = new Map(source.files), writes = [], deletes = [];
  let previousCanonicalPath = null;
  try { previousCanonicalPath = `prism/${lexlean.canonicalSource(JSON.parse(files.get("prism/spec.json"))).name}.lex.tex`; } catch {}
  const stage = (path, content) => { if (files.get(path) !== content) { files.set(path, content); writes.push({ path, content }); } };
  // The rest of the app's prism/prompt/1, when the turn wrote it: the idea routed onto every facet
  // (prism/builder/route.mjs). It lands with the rules in the one source transaction; a turn that
  // writes none keeps the parts the app has.
  const promptTag = prism.parsePromptTag(text);
  if (promptTag && promptTag.parts) stage("prism/prompt.json", JSON.stringify(promptTag.parts, null, 2) + "\n");
  // A LexLean module, when the model wrote one, carries the rules: the spec is read from it, keeping the
  // screen's words only when their checked interface still belongs to the new module.
  const lex = lexlean.specFromTurn(text);
  let presentationReset = false;
  if (lex && lex.error) return { changed: 0, source, card: `**Rules** · not read\n\n- ${lex.error}\n\nAsk me to fix the module.` };
  if (lex) {
    let previous = null;
    try { previous = JSON.parse(files.get("prism/spec.json") || "null"); } catch { previous = {}; }
    const presentation = modulePresentation(previous, lex.spec);
    if (presentation.code) return { changed: 0, source, card: `**Rules** · not read\n\n- ${presentation.error}\n\nAsk me to fix the module.` };
    presentationReset = presentation.reset;
    text = "<prism-spec>" + JSON.stringify(presentation.spec).replaceAll("<", "\\u003c") + "</prism-spec>";
  }
  const tag = prism.parseSpecTag(text);
  if (!tag?.spec && appShape.isAppShape(files.get(appShape.SOURCE_PATH))) return {changed:0,source,card:null};
  if (tag && tag.spec) stage("prism/spec.json", JSON.stringify(tag.spec, null, 2) + "\n");
  // Every app's rules have a canonical LexLean source, whatever wrote them (a module, the on-device
  // model's JSON, a click or an agent's patch): the same meaning is always the same module bytes. It is
  // staged with the spec, so both land in the one source transaction.
  if (files.get("prism/spec.json")) {
    try {
      const c = lexlean.canonicalSource(JSON.parse(files.get("prism/spec.json"))), path = `prism/${c.name}.lex.tex`;
      for (const oldPath of retiredApplicationPaths(source, 'decision', path, lexlean.canonicalSource)) {
        if (files.has(oldPath)) {files.delete(oldPath); deletes.push({path:oldPath});}
      }
      stage(path, c.source);
      if (previousCanonicalPath && previousCanonicalPath !== path && files.has(previousCanonicalPath)) { files.delete(previousCanonicalPath); deletes.push({ path: previousCanonicalPath }); }
    } catch (e) {}
  }
  const current = files.get("prism/spec.json"), snapshot = await rulesSnapshotOf(appId), before = snapshot.state;
  if (!current) return { changed: 0, source, card: tag && tag.error ? `**Rules** · not checked\n\n- ${tag.error}\n\nAsk me to fix the rules.` : null };
  const specKappa = await kappa(enc.encode(current));
  const proofKey = spec => live.kappaOfValue({ name: spec.name, logic: live.logicOf(spec) });
  if (before && before.specKappa === specKappa && writes.length === 0 && deletes.length === 0 && source.applicationState === undefined) return { changed: 0, source, card: tag ? prism.card(before) : null };
  let phase = before ? await nextPhase(before.phase, "edit") : "drafting";
  let logicKappa = null, retained = false;
  let checked;
  try { checked = prism.checkSpec(JSON.parse(current)); } catch (e) { checked = { ok: false, issues: ["The rules are not valid JSON: " + e.message] }; }
  if (!checked.ok && appShape.isAppShape(new Map(source.files).get(appShape.SOURCE_PATH))) return {changed:0,source,card:'**Rules** · not checked\n\nThe stateful module was retained because the replacement decision is invalid.'};
  if (checked.ok) {
    logicKappa = proofKey(checked.spec);
    if (before?.spec && before.logicKappa === logicKappa && proofKey(before.spec) === logicKappa) {
      const reword = await rewordPhase(before.phase);
      if (reword) { phase = reword; retained = true; }
    }
    if (!retained) phase = await nextPhase(phase, "check");
    stage("src/lib/prism.ts", checked.module);
    const page = files.get("src/pages/Index.tsx") || "";
    if (!page.includes("@/lib/prism") && !written.some((p) => p && p.startsWith("src/pages/"))) stage("src/pages/Index.tsx", prism.screenSource());
  }
  const notes = [...(presentationReset ? ["The application interface changed or its previous presentation was invalid; the view and descriptions were reset to the new module."] : []), ...(tag && tag.repaired ? ["Forge repaired the JSON syntax the model wrote. Read the rules before you approve them."] : []), ...(checked.notes || [])];
  // The full model: the builder projects every facet from the parts and the checked spec, or says why not.
  let parts = null;
  try { parts = JSON.parse(files.get("prism/prompt.json") || "null"); } catch {}
  if (promptTag && promptTag.error) notes.push(promptTag.error);
  const facets = checked.ok ? await prism.facetsOf(parts, checked.spec) : null;
  const state = { ...(retained ? before : {}), phase, specKappa, logicKappa, spec: checked.spec || null, prompt: parts, facets, issues: checked.issues, notes, vectors: checked.vectors || 0, proof: retained ? before.proof : null, updated: now() };
  // A semantic proof survives wording changes; a release still identifies the older bytes.
  if (retained && before.phase === "published" && before.specKappa !== specKappa) state.publicationCurrent = false;
  // Compare the captured lifecycle and install its successor in the source transaction.
  // A completed proof wins over this stale edit; no deleted-status interval is exposed.
  const applied = await applyStoredProjectPatch(await dbReady, appId, { writes, deletes, renames: [], deps: [] }, { core, derive: kappa, ...options, source, rulesUpdate: { expected: snapshot.expected, state }, appUpdate:{expected:source.applicationState,state:null} });
  assertGenerationActive(options.signal);
  if (applied.changed && appShape.isAppShape(new Map(source.files).get(appShape.SOURCE_PATH))) forgetApplicationFrames(appId);
  return { changed: applied.changed, source: applied.source, card: prism.card(state) };
}
// The kernel's answer to a reword. A kernel guest older than the event (KERNEL_REV pins the guest the page
// runs) answers invalid; the edit is then decided as an edit, by the kernel's own answer to that, and the
// proof is dropped as before. Nothing here decides a phase.
async function rewordPhase(phase) {
  try { return await nextPhase(phase, "reword"); } catch (e) { return null; }
}
// Lean's kernel proves the rules in this tab (prism.prove), and the kernel moves the phase on the Forge kernel
// guest's verdict. A proof whose logic changed meanwhile is dropped: it is about other rules. A reword meanwhile
// keeps it, since the logic kappa is the same. Resolves the prover's result, or null when the logic changed under it.
async function proveRules(appId, logicKey, onWait, options = {}) {
  const state = await rulesOf(appId);
  const reservation = options.expectedState || state;
  if (!sameRuleReservation(state, reservation) || (state.logicKappa || state.specKappa) !== logicKey) return null;
  let submitted = false;
  const result = await prism.prove(reservation.spec, { guest: kernelReady, beforeSubmit: () => { options.beforeSubmit?.(); submitted = true; } }).catch((e) => {
    if (options.beforeSubmit && !submitted) throw e;
    return { ok: false, error: String((e && e.message) || e) };
  });
  const phase = await nextPhase("proving", result.ok ? "attest" : "fail");
  let attempted;
  for (;;) {
    const latest = await rulesOf(appId);
    if (!sameRuleReservation(latest, reservation)) return null;
    const predecessor = JSON.stringify(latest);
    if (predecessor === attempted) return null;
    attempted = predecessor;
    // Reword can commit between this read and CAS. Retry the completed result,
    // not the prover, only while a new current predecessor has matching semantics.
    if (await saveRules(appId, { ...latest, phase, proof: result.ok ? result.record : null, axioms: result.ok ? result.axioms || {} : null, proofError: result.ok ? null : result.error, updated: now() }, latest)) break;
  }
  await put("audit", Date.now() + ":prism:" + appId, { timestamp_millis: Date.now(), principal: "app:" + appId, operation: "prism.rules.prove", resource: logicKey, outcome: result.ok ? "proved" : "refused" });
  await refreshLive(appId);
  return result;
}
// "approve" is the user's yes to the guarantees on the card: no model is asked. The answer is a
// message, and, when Lean was started, the proof to wait for.
// A stateful app's standing (shell/live/app.mjs, STATE_PATH), only while it is about the stored module.
async function appStateOf(appId) {
  const captured = await captureProjectSource(await dbReady, appId);
  const files = new Map(captured.files), source = files.get(appShape.SOURCE_PATH);
  applicationShape(captured);
  if (typeof source !== "string" || !appShape.isAppShape(source)) return null;
  return {source, captured, state:await boundApplicationState(captured,kappa), expectedState:captured.applicationState};
}
// Approval and proof completion compare the snapshot they examined, never a fresh replacement.
const saveAppFiles = async (appId, writes, source, appUpdate) => applyStoredProjectPatch(await dbReady, appId, { writes, deletes: [], renames: [], deps: [] }, { core, derive: kappa, source, appUpdate });
// Approval of a stateful app: the kernel moves it from checked to proving, and Lean is asked.
async function approveApp(appId, app, options = {}) {
  const { source, state } = app;
  if (options.signal) assertGenerationActive(options.signal);
  if (!state) return { text: "This app's module is not checked yet, so there is nothing to approve. Ask me to fix it." };
  if (state.phase === "proving") return { text: "Lean is already proving this app's properties." };
  if (state.phase === "proved" || state.phase === "published") return { text: "This app's properties are already proved by Lean." };
  if (state.phase !== "checked") return { text: "This app's module is not checked yet, so there is nothing to approve. Ask me to fix it." };
  const count = appShape.theoremsOf(source).length;
  if (!count) return { text: "This app's module states no properties, so there is nothing for Lean to prove. Ask me to add its promises." };
  const phase = await nextPhase("checked", "approve");
  const reservation = { ...state, phase, approved: now(), approvalId: options.approvalId || crypto.randomUUID() };
  let saved, submitted = false, abandoned = false, rollback;
  const cancel = () => {
    if (submitted) return Promise.resolve(false);
    abandoned = true;
    return (rollback ||= (async () => {
      if (!saved) return false;
      const attempted = new Set();
      let conflict;
      for (;;) {
        const latest = await appStateOf(appId);
        if (!latest?.state || latest.source !== source || JSON.stringify(latest.expectedState) !== JSON.stringify(reservation)) return false;
        // Unrelated edits may advance the project while this exact proof reservation survives.
        // Restore only its lifecycle, using each new current source once; the CAS preserves all
        // unrelated bytes and refuses a replacement approval or an edit to the approved module.
        const predecessor = JSON.stringify(latest.captured);
        if (attempted.has(predecessor)) throw conflict;
        attempted.add(predecessor);
        try {
          await saveAppFiles(appId, [{ path: appShape.STATE_PATH, content: JSON.stringify(state, null, 2) + "\n" }], latest.captured, { expected: reservation, state });
          await refreshApp(appId).catch(() => {});
          return true;
        } catch (error) {
          if (!["projectEditConflict", "applicationStateConflict"].includes(error?.code)) throw error;
          conflict = error;
        }
      }
    })());
  };
  try {
    if (options.signal) assertGenerationActive(options.signal);
    saved = await saveAppFiles(appId, [{ path: appShape.STATE_PATH, content: JSON.stringify(reservation, null, 2) + "\n" }], app.captured, { expected: app.expectedState, state: reservation });
    if (options.signal) assertGenerationActive(options.signal);
    await refreshApp(appId);
    if (options.signal) assertGenerationActive(options.signal);
  } catch (error) {
    await cancel();
    if (["projectEditConflict", "applicationStateConflict"].includes(error?.code)) return { text: "The app changed before approval was recorded. Check the current module again." };
    throw error;
  }
  return {
    text: appShape.provingMessage(count), message: appShape.proofMessage, cancel, submitted: () => submitted,
    prove: async (onWait) => {
      try {
        return await proveApp(appId, state.sourceKappa, source, onWait, { expectedState: reservation, beforeSubmit: () => {
          if (options.signal) assertGenerationActive(options.signal);
          if (abandoned) throw Object.assign(new Error("Generation cancelled"), { code: "generationCancelled" });
          submitted = true;
        } });
      } finally { if (!submitted) await cancel(); }
    },
  };
}
// Lean proves the module with its kernel in this tab (prism.js proveModule); the Forge kernel moves the phase on its
// verdict. A proof whose module changed meanwhile is dropped: it is about other bytes.
async function proveApp(appId, sourceKappa, source, onWait, options = {}) {
  const current = await appStateOf(appId);
  if (!current?.state || current.state.sourceKappa !== sourceKappa || current.state.phase !== "proving" || current.source !== source || (options.expectedState && JSON.stringify(current.expectedState) !== JSON.stringify(options.expectedState))) return null;
  const model = await appModel(), proved = source;
  let submitted = false;
  const result = await prism.proveModule(source, appShape.semanticData(source, { theorems: true }), model.declarations, { guest: kernelReady, beforeSubmit: () => { options.beforeSubmit?.(); submitted = true; } }).catch((e) => {
    if (options.beforeSubmit && !submitted) throw e;
    return { ok: false, error: String((e && e.message) || e) };
  });
  const attempted = new Set();
  let theorems, promises;
  for (;;) {
    const latest = await appStateOf(appId);
    if (!latest?.state || latest.source !== source || latest.state.sourceKappa !== sourceKappa || latest.state.phase !== "proving" || latest.state.approvalId !== current.state.approvalId) return null;
    const predecessor = JSON.stringify(latest.captured);
    if (attempted.has(predecessor)) return null;
    attempted.add(predecessor);
    const phase = await nextPhase("proving", result.ok ? "attest" : "fail");
    const kept = result.ok ? proved.trim() + "\n" : latest.source;
    theorems = appShape.theoremsOf(kept); promises = appShape.promisesOf(kept);
    const state = { ...latest.state, phase, sourceKappa: await kappa(enc.encode(kept)), proof: result.ok ? result.record : null, proofError: result.ok ? null : result.error, theorems: result.ok ? theorems : [], promises: result.ok ? promises : [], updated: now() };
    try {
      await saveAppFiles(appId, [...(kept !== latest.source ? [{ path: appShape.SOURCE_PATH, content: kept }] : []), { path: appShape.STATE_PATH, content: JSON.stringify(state, null, 2) + "\n" }], latest.captured, { expected: latest.expectedState, state });
      break;
    } catch (error) {
      if (!["projectEditConflict", "applicationStateConflict"].includes(error?.code)) throw error;
      // An unrelated edit can race this CAS. Retry only the completed result against a new
      // capture of the same module and approval, never the already submitted proof request.
    }
  }
  await put("audit", Date.now() + ":prism:" + appId, { timestamp_millis: Date.now(), principal: "app:" + appId, operation: "prism.app.prove", resource: sourceKappa, outcome: result.ok ? "proved" : "refused" });
  await refreshApp(appId);
  return { ...result, theorems, promises };
}
const approve = async (appId, options = {}) => { const app = await appStateOf(appId); return app ? approveApp(appId, app, options) : approveRules(appId, options); };
async function approveRules(appId, options = {}) {
  if (options.signal) assertGenerationActive(options.signal);
  const state = await rulesOf(appId);
  if (!state) return { text: "This app has no rules to prove yet. Tell me what it decides and I will draft them." };
  if (state.phase === "proving") return { text: "Lean is already proving these rules." };
  if (state.phase === "proved" || state.phase === "published") return { text: "These rules are already proved by Lean." };
  if (state.phase !== "checked") return { text: "The rules are not checked yet, so there is nothing to approve. Ask me to fix them." };
  const phase = await nextPhase("checked", "approve"), reservation = { ...state, phase, approved: now(), approvalId: options.approvalId || crypto.randomUUID() };
  if (options.signal) assertGenerationActive(options.signal);
  if (!await saveRules(appId, reservation, state)) return { text: "The rules changed before approval was recorded. Check the current rules again." };
  let submitted = false, abandoned = false, rollback;
  const cancel = () => {
    if (submitted) return Promise.resolve(false);
    abandoned = true;
    return (rollback ||= (async () => {
      const attempted = new Set();
      for (;;) {
        const latest = await rulesOf(appId);
        if (!sameRuleReservation(latest, reservation)) return false;
        const predecessor = JSON.stringify(latest);
        if (attempted.has(predecessor)) return false;
        attempted.add(predecessor);
        const restored = { ...latest, phase: state.phase };
        for (const key of ["approvalId", "approved"]) {
          if (Object.hasOwn(state, key)) restored[key] = state[key]; else delete restored[key];
        }
        if (!await saveRules(appId, restored, latest)) continue;
        await refreshLive(appId).catch(() => {});
        return true;
      }
    })());
  };
  try { if (options.signal) assertGenerationActive(options.signal); await refreshLive(appId); if (options.signal) assertGenerationActive(options.signal); }
  catch (error) { await cancel(); throw error; }
  return {
    text: prism.provingMessage(state.spec), spec: state.spec, cancel, submitted: () => submitted,
    prove: async (onWait) => {
      try {
        return await proveRules(appId, state.logicKappa || state.specKappa, onWait, { expectedState: reservation, beforeSubmit: () => {
          if (options.signal) assertGenerationActive(options.signal);
          if (abandoned) throw Object.assign(new Error("Generation cancelled"), { code: "generationCancelled" });
          submitted = true;
        } });
      } finally { if (!submitted) await cancel(); }
    },
  };
}
// The approve turn stays open while Lean works, as a build turn stays open while the model writes:
// the renderer shows it working, the message counts the seconds, and the result completes it.
async function approveTurn(chatId, appId, messages, userId, ref, intentId) {
  const m = chatMachineOf(chatId), abort = new AbortController(), cancel = () => abort.abort();
  m.abort = abort; m.queueSignal?.addEventListener("abort", cancel, { once: true });
  if (m.queueSignal?.aborted) cancel();
  messages.push({ id: userId + 1, role: "assistant", content: "", approvalState: "approved", commitHash: null, createdAt: now() });
  const show = async (content) => {
    messages[messages.length - 1] = { ...messages[messages.length - 1], content };
    await saveMessages(chatId, messages);
    emit("chat:response:chunk", { chatId, invocationRef: ref, messages });
  };
  let started, observing = true, stopListener, error = null;
  try {
    assertGenerationActive(abort.signal);
    started = await approve(appId, { signal: abort.signal, approvalId: intentId });
    assertGenerationActive(abort.signal);
    await show(started.text);
    assertGenerationActive(abort.signal);
    if (started.prove) {
      const stopped = new Promise(resolve => {
        stopListener = () => resolve("stopped"); abort.signal.addEventListener("abort", stopListener, { once: true });
        if (abort.signal.aborted) stopListener();
      });
      const proof = started.prove(s => { if (observing && !abort.signal.aborted) void show(started.text + `\n\nLean is working, ${s} s.`).catch(() => {}); });
      proof.catch(() => {});
      const result = await Promise.race([proof, stopped]);
      observing = false;
      if (result === "stopped") {
        await started.cancel?.();
        await show(started.submitted?.()
          ? started.text + "\n\nStopped watching. Lean keeps working; the result is kept with the app's rules."
          : "Stopped before proof submission. No proof request was sent.");
      } else {
        await show(started.text + "\n\n" + (result ? (started.message ? started.message(result) : prism.proofMessage(started.spec, result)) : "The rules changed while Lean worked, so this proof is about other rules and was dropped."));
      }
    }
  } catch (failure) {
    observing = false;
    try { await started?.cancel?.(); } catch (rollbackFailure) { failure = rollbackFailure; }
    if (!abort.signal.aborted || failure?.code !== "generationCancelled") error = String((failure && failure.message) || failure);
    try { await show(error ? `<prism-output type="error" message="${error.replace(/"/g, "'")}"></prism-output>` : started?.submitted?.()
      ? "Stopped watching. Lean keeps working; the result is kept with the app's rules."
      : "Stopped before proof submission. No proof request was sent."); }
    catch (storageFailure) { error = String((storageFailure && storageFailure.message) || storageFailure); }
  } finally {
    observing = false;
    if (stopListener) abort.signal.removeEventListener("abort", stopListener);
    m.queueSignal?.removeEventListener("abort", cancel);
    if (m.abort === abort) m.abort = null;
    publishChat(chatId, { phase: "finalizing", error, lastCompletion: { intentId, invocationRef: ref, outcome: error ? "errored" : "completed", updatedFiles: false, suppressAutoReview: true, targetAppId: appId } });
    publishChat(chatId, { phase: "idle", invocationRef: null });
    emit("chat:stream:end", { chatId });
  }
}
handlers.set("prism:status", async ({ appId }) => (await rulesOf(appId)) || null);

// "publish": a proved app with rules becomes a holo on this device, by its κ, with no account and no server.
// Its page is Forge's kernel page (app/, whose verified app.holo carries RuleJudge, FA-90) with the app's
// own card (shell/live/card-files.mjs); the card asks the kernel "rule " with the spec's words and the
// request's, so the answer shown is the kernel guest's, after every acceptance vector replayed in this tab.
// A rule that compares two fields has no rule words yet, so such an app is refused, never guessed.
function ruleWords(spec) {
  const hex = (n) => { if (!Number.isInteger(n) || n < 0 || n > 0xffffffff) throw new Error("a bound outside 32 bits"); return [0, 8, 16, 24].map((k) => ((n >>> k) & 255).toString(16).padStart(2, "0")).join(""); };
  const w = prism.ruleWordsOf(spec);
  return { words: w.words.map(hex).join(""), outcomes: w.outcomes, choices: w.choices };
}
const START = (w) => `import { load } from "./vendor/prism-code/src/browser.mjs";import start from "./card.mjs";const W=${JSON.stringify(w)},e=new TextEncoder(),d=new TextDecoder(),h=(n)=>[0,8,16,24].map((k)=>((n>>>k)&255).toString(16).padStart(2,"0")).join("");
const ask=(k)=>async(b)=>{const r=d.decode(await k.dispatch(e.encode("rule "+W.words+d.decode(b).split(" ").map((v,i)=>h(W.choices[i]?W.choices[i].indexOf(v):Number(v))).join(""))));if(!r.startsWith("ok "))throw new Error(r);return e.encode(W.outcomes[parseInt(r.slice(11,19).match(/../g).reverse().join(""),16)]);};
let app=null;try{const k=await load("./app.holo"),a=await k.acceptance();if(k.identities.verified&&a.failed.length===0)app={dispatch:ask(k)};}catch(x){console.error(x);}start({app,show:(b)=>d.decode(b),$:(id)=>document.getElementById(id)});\n`;
async function rulePage(spec) {
  const w = ruleWords(spec), at = new URL("app/", BASE), bytes = async (u) => { const r = await fetch(u, { cache: "no-store" }); if (!r.ok) throw new Error(u + " answered " + r.status); return new Uint8Array(await r.arrayBuffer()); };
  const keep = (await (await fetch(new URL("precache.json", at), { cache: "no-store" })).json()).files.map((f) => f.path.slice(2)).filter((p) => /^(app\.holo|brand\/|fonts\/|prism-page\.css|favicon\.svg|guest-worker\.mjs|vendor\/|worker-runtime\/)/.test(p));
  const { cardFiles } = await import("./live/card-files.mjs");
  const card = new Map((await cardFiles(spec, { load: async (p) => dec.decode(await bytes(new URL(p, BASE))) })).files.map((f) => [f.path.slice(5), f.content]));
  const html = `<!doctype html><html lang="en" class="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${String(spec.title || spec.name).replace(/[<&]/g, (c) => (c === "<" ? "&lt;" : "&amp;"))}</title><link rel="icon" href="favicon.svg"><link rel="stylesheet" href="brand/hologram-warm.css"><link rel="stylesheet" href="brand/fonts.css"><link rel="stylesheet" href="prism-page.css"><style>${card.get("card.css")}</style></head><body class="has-card"><script type="application/json" id="app-data">${JSON.stringify({ inputLabel: spec.title || spec.name }).replace(/</g, "\\u003c")}</script>${card.get("card.html")}<output id="out"></output><ol id="hist"></ol><script type="module" src="start.mjs"></script></body></html>\n`;
  const files = [{ path: "index.html", bytes: enc.encode(html) }, { path: "card.mjs", bytes: enc.encode(card.get("card.mjs")) }, { path: "start.mjs", bytes: enc.encode(START(w)) }];
  for (const p of keep) files.push({ path: p, bytes: await bytes(new URL(p, at)) });
  // FA-05: the kernel this page carries is the one its page build named, and every vector it carries passes here.
  const kernel = await (await import("./prism-code/src/index.mjs")).loadArchive(files.find((f) => f.path === "app.holo").bytes, {});
  const admitted = admitPublication((await (await fetch(new URL("app.json", at), { cache: "no-store" })).json()).attestations?.build?.id, kernel.identities?.archiveFingerprint, await kernel.acceptance());
  if (!admitted.ok) throw new Error(admitted.error);
  return files.sort((x, y) => (x.path < y.path ? -1 : 1));
}
async function publishTurn(chatId, appId, messages, userId, ref, intentId) {
  const m = chatMachineOf(chatId);
  messages.push({ id: userId + 1, role: "assistant", content: "", approvalState: "approved", commitHash: null, createdAt: now() });
  const show = async (content) => {
    messages[messages.length - 1] = { ...messages[messages.length - 1], content };
    await saveMessages(chatId, messages);
    emit("chat:response:chunk", { chatId, invocationRef: ref, messages });
  };
  const finish = () => {
    publishChat(chatId, { phase: "finalizing", error: null, lastCompletion: { intentId, invocationRef: ref, outcome: "completed", updatedFiles: false, suppressAutoReview: true, targetAppId: appId } });
    publishChat(chatId, { phase: "idle", invocationRef: null });
    emit("chat:stream:end", { chatId });
  };
  const rules = await rulesOf(appId);
  const refusal = !rules || !rules.spec ? "This app has no rules to build yet. Tell me what it decides, approve the rules, then say publish."
    : rules.phase !== "proved" && rules.phase !== "published" ? (rules.phase === "checked" ? "Reply approve first: Lean proves the rules, then they are published on this device." : `The rules are ${rules.phase === "proving" ? "being proved by Lean; say publish when it is done" : "not checked yet; ask me to fix them"}.`)
    : rules.facets && !rules.facets.ok ? `Publishing writes the whole model, and its facets are not projected yet: ${rules.facets.issue} Ask me to fix the app's prompt, then say publish.`
    : null;
  if (refusal) { await show(refusal); return finish(); }
  const row = (outcome, resource) => put("audit", Date.now() + ":publish:" + appId, { timestamp_millis: Date.now(), principal: "app:" + appId, operation: "prism.application.publish", resource, outcome });
  const head = `**Publishing ${rules.spec.title}** on this device, as a holo named by its contents. Nothing leaves this browser.`;
  await show(`${head}\n\nComposing the page and replaying the kernel's acceptance vectors.`);
  const out = await handlers.get("holo:publish")({ appId }).catch((e) => ({ decision: "Refuse", word: String((e && e.message) || e) }));
  if (!out.kappa) { await row("refused", rules.specKappa || ""); await show(`${head}\n\nNot published: ${out.word}`); return finish(); }
  const next = await nextPhase(rules.phase, "publish");
  const current = await saveRules(appId, { ...rules, phase: next === "published" ? next : rules.phase, publishedAs: out.url, publicationCurrent: true, build: { kappa: out.kappa } }, rules);
  if (!current) { await row("stale", out.kappa); await show(`${head}\n\nThe holo was made from earlier rules, so the current rules are not marked published. Approve them, then say publish.`); return finish(); }
  await row("granted", out.kappa);
  const run = machineOf(appId);
  publish(appId, { phase: "ready", operation: null, url: { appUrl: out.url, originalUrl: out.url, mode: "local" }, previewReloadEpoch: run.state.previewReloadEpoch + 1 });
  await show(`${head}\n\n**Published:** [open it](${out.url}) · holo \`${out.kappa}\`, ${Math.round(out.byteLength / 1024)} KB, in ${Math.max(1, Math.round(out.ms / 1000))} s.\n\nShare the file from Publish, under Download; anyone opens it in any browser with holo.html, which checks it byte for byte before it runs.`);
  finish();
}
window.__prism = { rulesAfterTurn: afterTurn, approveRules: approve, rulesOf, appStateOf, nextPhase, check: prism.checkSpec };

async function chatTurn(chatId, intent) {
  const m = chatMachineOf(chatId);
  assertGenerationActive(m.queueSignal);
  const chat = await get("chats", chatId);
  if(!chat)throw chatQueueFailure("chatQueueMissing");
  const execution = chatExecutionPolicy(intent.requestedChatMode,chat.chatMode);
  const appId = chat.appId;
  const ref = intent.invocationRef;
  const settings = (await get("settings", "user")) || DEFAULT_SETTINGS;
  const V = await viewReady();
  const route = chatModelRoute(chat.modelSelection,settings.selectedModel);
  const {model,provider} = route;
  if (tm.turn === chatId) tm.model = model;
  // T2: the user message, accepted
  const messages = await messagesOf(chatId);
  const userId = messages.length ? Math.max(...messages.map((x) => x.id)) + 1 : 1;
  messages.push({ id: userId, role: "user", content: intent.prompt, approvalState: null, commitHash: null, createdAt: now() });
  await saveMessages(chatId, messages);
  publishChat(chatId, { phase: "streaming", lastAcceptance: { intentId: intent.intentId, acceptance: "message-accepted", acceptedMessageId: userId } });
  emit("chat:stream:start", { chatId, invocationRef: ref });
  if (provider !== UOR_R4_PROVIDER && execution.sourceEffects && (prism.isApproval(intent.prompt) || prism.isPublish(intent.prompt))) tmCancel(chatId);
  if (provider !== UOR_R4_PROVIDER && execution.sourceEffects && prism.isApproval(intent.prompt)) return approveTurn(chatId, appId, messages, userId, ref, intent.intentId);
  if (provider !== UOR_R4_PROVIDER && execution.sourceEffects && prism.isPublish(intent.prompt)) return publishTurn(chatId, appId, messages, userId, ref, intent.intentId);
  const speculativeLive = provider !== UOR_R4_PROVIDER && execution.sourceEffects && !(await hasRules(appId)) && messages.filter((x) => x.role === "user").length === 1;
  if (speculativeLive) showLive(appId).catch(() => liveShown.delete(appId));
  // T3: the assistant placeholder
  const projectSource = await captureProjectSource(await dbReady, appId);
  const head = projectSource.reference;
  const asstId = userId + 1;
  messages.push({ id: asstId, role: "assistant", content: "", approvalState: "approved", commitHash: null, sourceCommitHash: head, model, createdAt: now() });
  await saveMessages(chatId, messages);
  emit("chat:response:chunk", { chatId, invocationRef: ref, messages });
  let full = "", lastSent = "", updatedFiles = false, summary, error = null;
  const abort = new AbortController(); m.abort = abort; if(m.queueSignal?.aborted){abort.abort();assertGenerationActive(abort.signal);}
  let provenance = null;
  const committed = [];
  const onCommit = (oid) => { committed.push(oid); updatedFiles = true; emit("version-created", { appId, oid }); };
  try {
    let uorR4StatusBefore = null;
    if (provider === UOR_R4_PROVIDER) {
      await uorR4.ensureModelLoaded(model, { signal: abort.signal });
      uorR4StatusBefore = uorR4.getStatus();
    }
    // R4 decides the route before the request is made: a route to any model outside Foundry is
    // ungranted, so it is refused here and no byte of the request leaves the tab.
    const before = r4Request({
      provider,
      engine: "match",
      weights: provider === UOR_R4_PROVIDER ? (uorR4StatusBefore?.weightsMatched ? "match" : "mismatch") : "none",
    });
    const allowed = await r4Decide(before);
    if (allowed.outcome !== "answer") throw new Error(r4Refusal(allowed, before));
    let lastSave = 0;
    const onText = async (t) => {
      assertGenerationActive(abort.signal);
      full = t;
      if (tm.turn === chatId) {
        if (!tm.token && t && !t.startsWith("<think>")) tmOnce("token");
        if (!tm.complete && (t.includes("\\end{semanticmodule}") || t.includes("</prism-lexlean>") || t.includes("</prism-spec>"))) tmOnce("complete");
      }
      const shown = provider === UOR_R4_PROVIDER
        ? prism.withoutThinking(full)
        : (execution.sourceEffects ? prism.shownText(full) : readOnlyChatText(prism.withoutThinking(full)));
      let lcp = 0; while (lcp < lastSent.length && lcp < shown.length && lastSent[lcp] === shown[lcp]) lcp++;
      if (lcp < lastSent.length) { messages[messages.length - 1].content = shown; emit("chat:response:chunk", { chatId, invocationRef: ref, messages }); }
      else emit("chat:response:chunk", { chatId, invocationRef: ref, streamingMessageId: asstId, streamingPatch: { offset: lcp, content: shown.slice(lcp), ...(lcp > 0 ? { prefixHash: djb2(shown.slice(0, lcp)) } : {}) } });
      lastSent = shown;
      // The rules as far as the model has written them go to the live frame now, not after the turn.
      if (provider !== UOR_R4_PROVIDER && execution.sourceEffects && full.includes("<prism-spec>")) {
        const raw = live.partialSpec(full);
        if (raw) { if (!liveShown.has(appId)) { liveShown.add(appId); showLive(appId).catch(() => liveShown.delete(appId)); } tmOnce("declaration"); pushLive(appId, raw, "draft"); }
      }
      // LexLean a model writes is not projected: the frame shows the spec, and nothing else is a source.
      if (Date.now() - lastSave > 150) { messages[messages.length - 1].content = shown; await saveMessages(chatId, messages); lastSave = Date.now(); }
    };
    // Each text is shown in its turn, after the one before it has been saved.
    let textWork = Promise.resolve();
    const enqueueText = (text) => {
      textWork = textWork.then(() => onText(text));
      void textWork.catch(() => {});
      return textWork;
    };
    if (provider === UOR_R4_PROVIDER) {
      await enqueueText("");
      const tools = createFoundryTools({ appId, chatMode: execution.mode, signal: abort.signal, onCommit });
      const uorTurn = await runUorR4AgentTurn({
        appId,
        chatId,
        modelId: model,
        messages: messages.slice(0, -1),
        userPrompt: intent.prompt,
        chatMode: execution.mode,
        signal: abort.signal,
        uorR4Runtime: uorR4,
        mvmHost,
        tools,
        onText: enqueueText,
      });
      await textWork;
      const made = r4Request({
        provider,
        engine: "match",
        weights: uorTurn.weightsMatched ? "match" : "mismatch",
      });
      const shown = await r4Decide(made);
      if (shown.outcome !== "answer") throw new Error(r4Refusal(shown, made));
      const rawTurn = uorTurn.fullText || full;
      const sm = rawTurn.match(/<prism-chat-summary>([\s\S]*?)<\/prism-chat-summary>/);
      summary = sm ? sm[1].trim() : summary;
      if (summary && (!chat.title || chat.title === "New chat")) {
        await updateStoredChat(await dbReady, chatId, { title: summary });
      }
      let card = null;
      if (execution.sourceEffects) {
        await settleAppFrames(appId, abort.signal);
        assertGenerationActive(abort.signal);
        const wrote = parsePrismTags(rawTurn);
        let changed = 0;
        if (wrote.writes.length || wrote.deletes.length || wrote.renames.length || wrote.deps.length) {
          const applied = await applyPrismTags(appId, wrote, { signal: abort.signal, onCommit });
          changed += applied.changed || 0;
        }
        const specResult = await afterTurn(appId, rawTurn, [], { signal: abort.signal, onCommit }).catch(() => null);
        if (specResult) {
          changed += specResult.changed || 0;
          card = specResult.card || null;
        }
        updatedFiles = updatedFiles || Boolean(uorTurn.updatedFiles) || changed > 0;
        if (await hasRules(appId)) {
          await showLive(appId).catch(() => {});
          await refreshPreview(appId, { signal: abort.signal });
        } else if (updatedFiles) {
          if (speculativeLive && liveShown.has(appId)) liveShown.delete(appId);
          await showBuilt(appId).catch(() => {});
          await refreshPreview(appId, { signal: abort.signal });
        }
      }
      const commit = committed.at(-1) || null;
      const rendered = execution.sourceEffects
        ? (prism.shownText(rawTurn) || prism.withoutThinking(rawTurn))
        : readOnlyChatText(prism.withoutThinking(rawTurn));
      messages[messages.length - 1] = {
        ...messages[messages.length - 1],
        content: prism.withCard(rendered, card) || prism.withoutThinking(rawTurn),
        commitHash: commit,
        approvalState: execution.sourceEffects ? "approved" : null,
        requestId: null,
        model,
      };
    } else {
      // Foundry's own language model reads the idea: IdeaReader, compiled with Prism into the kernel guest, reads
      // every request so far into the app notation (closed lexicon, fixed grammar, no weights). A phrase it cannot
      // read is an open claim the chat shows, never a guess. R4 judges the answer as a compiled one.
      const asks = messages.filter((x) => x.role === "user" && x.content && !prism.isApproval(x.content)).map((x) => x.content);
      await enqueueText("");
      // A stop ends the turn at once, as it did a model's stream: the reading is then left unread.
      const lines = (await Promise.race([(await kernelReady).ask("read " + [...new Set(asks)].join(", ")), new Promise((_, no) => abort.signal.addEventListener("abort", () => no(abort.signal.reason), { once: true }))])).split("\n");
      assertGenerationActive(abort.signal);
      const made = r4Request({ provider });
      const shown = await r4Decide(made);
      if (shown.outcome !== "answer") throw new Error(r4Refusal(shown, made));
      // "= " lines are the idea's goal and measure as read; "? " lines are open questions, shown and never guessed.
      const said = (m) => lines.filter((l) => l.startsWith(m)).map((l) => "- " + l.slice(2)).join("\n"), notationLines = lines.filter((l) => l && !/^[?=] /.test(l));
      await enqueueText((notationLines.length ? "<prism-lexlean>\n" + notationLines.join("\n") + "\n</prism-lexlean>\n\n" : "") + [said("= "), said("? ") && "Open questions:\n" + said("? ")].filter(Boolean).join("\n\n"));
      messages[messages.length - 1].model = LOCAL_MODEL;
      // T6: completed batches have their own committed snapshots.
      // A streamed parser still owns the pre-turn snapshot until it settles. Finish it before our
      // own transaction changes that snapshot, so ownership checks cannot mistake our commit for an edit.
      if (execution.sourceEffects) await settleAppFrames(appId, abort.signal);
      assertGenerationActive(abort.signal);
      const sm = full.match(/<prism-chat-summary>([\s\S]*?)<\/prism-chat-summary>/); summary = sm ? sm[1].trim() : undefined;
      if (summary && (!chat.title || chat.title === "New chat")) { await updateStoredChat(await dbReady, chatId, { title: summary }); }
      if (execution.sourceEffects) {
        // Code a model wrote is never applied: the app's code and screen are projections of its module.
        const wrote = parsePrismTags(full);
        const refusedCode = wrote.writes.length + wrote.deletes.length + wrote.renames.length + wrote.deps.length;
        const tags = { ...wrote, writes: [], deletes: [], renames: [], deps: [] };
        const applied = await applyPrismTags(appId, tags, { signal: abort.signal, source: projectSource, onCommit });
        let changed = applied.changed;
        const turn = full;
        const spec = (await appAfterTurn(appId, turn, { signal: abort.signal, source: applied.source, onCommit })) || await rulesAfterTurn(appId, turn, [], { signal: abort.signal, source: applied.source, onCommit });
        const patchRefusals = [];
        for (const match of full.matchAll(/<prism-spec-patch>([\s\S]*?)<\/prism-spec-patch>/g)) {
          assertGenerationActive(abort.signal);
          let ops; try { ops = JSON.parse(match[1]); } catch {}
          const list = ops ? (Array.isArray(ops) ? ops : [ops]) : [];
          const result = ops ? await patchSpec(appId, list, { author: "agent", signal: abort.signal, source: spec.source, onCommit }) : { ok: false, issues: ["The patch is not JSON."] };
          if (result.ok) { spec.changed += result.changed; spec.source = result.source; spec.card = result.card; continue; }
          const refused = list.length > 1 ? [] : [result.issues[0]];
          if (list.length > 1) for (const op of list) {
            assertGenerationActive(abort.signal);
            const one = await patchSpec(appId, [op], { author: "agent", signal: abort.signal, source: spec.source, onCommit });
            if (one.ok) { spec.changed += one.changed; spec.source = one.source; spec.card = one.card; } else { const label=Array.isArray(op?.path)&&op.path.every(p=>typeof p==="string"||Number.isSafeInteger(p))?op.path.join("."):"patch";refused.push(`${label || "patch"}: ${one.issues[0]}`); }
          }
          patchRefusals.push(...refused);
        }
        if (patchRefusals.length) spec.card = (spec.card ? spec.card + "\n\n" : "") + "**Patch refused.** " + patchRefusals.join(" ");
        if (refusedCode) spec.card = (spec.card ? spec.card + "\n\n" : "") + prism.CODE_REFUSED;
        if (tm.turn === chatId && tm.complete) {
          let phase = null;
          try { phase = JSON.parse(await get("files", appId + ":" + appShape.STATE_PATH) || "null")?.phase ?? (await rulesOf(appId))?.phase ?? null; } catch (e) {}
          tmMark("rechecked");
          if (phase && phase !== "drafting") { tmMark("read", tm.repaired ? 2 : 1); tmMark("checked"); } else tmMark("refused");
        }
        changed += spec.changed;
        if (await hasRules(appId)) { await showLive(appId).catch(() => {}); await refreshPreview(appId, { signal: abort.signal }); }
        else if (speculativeLive && liveShown.has(appId) && !(await scaffoldOnly(appId))) { liveShown.delete(appId); showBuilt(appId).catch(() => {}); }
        updatedFiles = updatedFiles || changed > 0;
        const commit = committed.at(-1) || null;
        messages[messages.length - 1] = { ...messages[messages.length - 1], content: prism.withCard(full, spec.card), commitHash: commit, approvalState: "approved", requestId: provenance && provenance.receipt || null };
      } else {
        messages[messages.length - 1] = {...messages[messages.length - 1],content:readOnlyChatText(prism.withoutThinking(full)),commitHash:null,approvalState:null,requestId:provenance?.receipt||null};
      }
    }
    await saveMessages(chatId, messages);
    emit("chat:response:chunk", { chatId, invocationRef: ref, messages });
  } catch (e) {
    error = abort.signal.aborted ? null : String((e && e.message) || e);
    messages[messages.length - 1] = { ...messages[messages.length - 1], commitHash: committed.at(-1) || null, content: (execution.sourceEffects ? prism.shownText(full) : readOnlyChatText(prism.withoutThinking(full))) + (updatedFiles ? "\n\nCommitted project changes were retained." : "") + (abort.signal.aborted ? "\n\n<prism-output type=\"warning\" message=\"Response cancelled by user\"></prism-output>" : error ? `\n\n<prism-output type="error" message="${error.replace(/"/g, "'")}"></prism-output>` : "") };
    await saveMessages(chatId, messages);
    emit("chat:response:chunk", { chatId, invocationRef: ref, messages });
  } finally {
    try { if (execution.sourceEffects && !abort.signal.aborted) await settleAppFrames(appId, abort.signal); }
    catch (e) { if (!abort.signal.aborted) error ||= String((e && e.message) || e); }
  }
  m.abort = null;
  const outcome = abort.signal.aborted ? "cancelled" : error ? "errored" : "completed";
  const completion = { intentId: intent.intentId, invocationRef: ref, outcome, updatedFiles, suppressAutoReview: true, targetAppId: appId, ...(summary ? { chatSummary: summary } : {}), ...(error ? { error } : {}), ...(provenance && provenance.cut ? { warningMessages: ["The model ran out of room before it finished; an unfinished file was not written."] } : {}), ...(provenance && provenance.receipt ? { warningMessages: [(provenance.served ? V.servedLabel : V.sealedLabel) + " · " + provenance.receipt.replace(/^blake3:/, "").slice(0, 12) + "…"] } : {}) };
  publishChat(chatId, { phase: "finalizing", error, lastCompletion: completion });
  publishChat(chatId, { phase: outcome === "errored" ? "errored" : "idle", invocationRef: null });
  emit("chat:stream:end", { chatId });
  if (tm.turn === chatId) tmFlush();
}
const queueChannel=typeof BroadcastChannel==="function"?new BroadcastChannel("foundry:chat-queues:v1"):null;
const queueBoundary={validate:value=>SerializableChatTurnIntentSchema.safeParse(value),validateEvent:value=>ChatStreamIntentEventSchema.safeParse(value),serialize:serializeImmutableChatTurnPayload,derive:kappa};
const chatQueue=createChatQueueController({
  guest:kernelReady,boundary:async()=>queueBoundary,locks:queueChannel?navigator.locks:null,
  store:{read:async chatId=>readStoredChatQueue(await dbReady,chatId),commit:async(expected,journal)=>commitStoredChatQueue(await dbReady,expected,journal)},
  mintOwner:()=>{let owner=0;while(!owner)owner=crypto.getRandomValues(new Uint32Array(1))[0];return owner;},
  notify:message=>queueChannel?.postMessage(message),
  publish:(chatId,snapshot,extra)=>{
    const m=chatMachineOf(chatId),patch={...(snapshot||{})};
    if(extra.journal&&!m.queueRunning){const active=extra.journal.entries.find(e=>e.id===extra.journal.state.active);patch.phase=active?"streaming":"idle";patch.invocationRef=active?.intent.invocationRef||null;}
    if(extra.acceptance)patch.lastAcceptance=extra.acceptance;if(extra.error)patch.error=extra.error;if(extra.reportedError)patch.error=extra.reportedError;publishChat(chatId,patch);
  },
  run:async(chatId,intent,signal)=>{
    const m=chatMachineOf(chatId);m.queueRunning=true;m.queueSignal=signal;const cancel=()=>m.abort?.abort();signal.addEventListener("abort",cancel);
    tmStart(chatId);
    publishChat(chatId,{phase:"admitting",invocationRef:intent.invocationRef,error:null});
    try{assertGenerationActive(signal);await chatTurn(chatId,intent);return m.state.lastCompletion?.intentId===intent.intentId?m.state.lastCompletion:{outcome:"errored"};}
    catch(error){if(["invalidChatModelSelection","unsupportedChatProvider"].includes(error?.code))publishChat(chatId,{error:error.code==="unsupportedChatProvider"?"The chat provider is unsupported":"The chat model selection is invalid"});throw error;}
    finally{signal.removeEventListener("abort",cancel);m.queueRunning=false;m.queueSignal=null;m.abort=null;if(m.state.lastCompletion?.intentId!==intent.intentId){publishChat(chatId,{phase:"finalizing",lastCompletion:{intentId:intent.intentId,invocationRef:intent.invocationRef,outcome:signal.aborted?"cancelled":"errored",targetAppId:intent.appId??null,updatedFiles:false,suppressAutoReview:true}});emit("chat:stream:end",{chatId});}}
  },
});
queueChannel?.addEventListener("message",event=>{const chatId=event.data?.chatId;if(Number.isSafeInteger(chatId)&&chatId>0)void chatQueue.refresh(chatId).catch(()=>{});});
window.addEventListener("pagehide",()=>{chatQueue.dispose();queueChannel?.close();},{once:true});
window.addEventListener("pageshow",event=>{if(event.persisted)location.reload();});
const chatSubscribe=async({machineId,encodedKey})=>{await chatQueue.snapshot(encodedKey.chatId);const m=chatMachineOf(encodedKey.chatId);return {protocolVersion:1,machineId,encodedKey,actorInstanceId:chatActorId(encodedKey.chatId),revision:m.revision,encodedState:m.state};};
async function chatDispatch(raw){
  const {encodedKey,messageId,expectedActorInstanceId,encodedEvent}=structuredClone(raw),chatId=encodedKey?.chatId;
  if(!Number.isSafeInteger(chatId)||chatId<1||typeof messageId!=="string"||!messageId||messageId.length>256)throw chatQueueFailure("invalidChatQueueCommand");
  const aid=chatActorId(chatId);if(expectedActorInstanceId&&expectedActorInstanceId!==aid)return {kind:"rejected",messageId,reason:"stale-actor"};
  await chatQueue.dispatch(chatId,encodedEvent);const m=chatMachineOf(chatId);return {kind:"applied",actorInstanceId:aid,revision:m.revision,transactionSequence:m.revision,messageId};
}
handlers.set("chat:observe-submission-stop-policy",async chatId=>(await readStoredChatQueue(await dbReady,chatId)).journal.state.stopPolicy);
handlers.set("chat:cancel",chatId=>chatQueue.cancelCurrent(chatId));

// ---- the typed host boundary (FoundrySpecification.hostResponseBoundary)
function dispatch(channel, args) {
  seen.set(channel, (seen.get(channel) || 0) + 1);
  const output = window.__contracts && window.__contracts[channel];
  const input = window.__inputContracts && window.__inputContracts[channel];
  let captured;try{captured=structuredClone(args);}catch{captured=null;}
  return invokeBoundCommand(commandRegistry, channel, args, handlers.get(channel), (name, values, handler) => invokeHost(name, values, handler, output, output || input ? { schema: input } : null)).then(async envelope=>{
    const binding=commandRegistry.entries.find(c=>c.channel===channel),feature=commandRegistry.features.find(f=>f.id===binding?.feature);
    if(envelope.ok&&feature?.effects.includes("storage")&&!/^([a-z-]+:)?(get|list|read|count|check)(-|$)|^(window-infrastructure:|distributed-machine:(subscribe|unsubscribe))/.test(channel)){
      const value=captured?.[0],appId=value?.appId??value?.encodedKey?.appId??envelope.value?.app?.id,chatId=value?.chatId??value?.encodedKey?.chatId??envelope.value?.chatId;
      const scopes=windowRecoveryScopes();if(Number.isSafeInteger(appId)&&appId>0)scopes.push({family:"app",appId});if(Number.isSafeInteger(chatId)&&chatId>0)scopes.push({family:"chat",chatId});await windowCoordination.invalidate(scopes).catch(()=>{});
    }return envelope;
  });
}




// ---- publish: a version becomes a Hologram application. The View is the project's built page with
// every dependency bundled in (a portable View has no network); the guest is this very core; the model
// document and the source manifest are the core's bytes; PrismPM's own archive code, vendored verbatim
// into the core, composes the .holo v4 and validates it; the archive is kept at its address, unpacked
// under holoPath(κ) for the worker, and offered as one link and one download. The core decides first:
// the shell closure of this page must be the closure the lane recorded beside the core, and the
// lane's attestation must be present; otherwise the View's refusal word and an audit row.
async function buildPortable(appId, captured) {
  await coreReady;
  const source = captured || await verifiedBuildSource(appId);
  if (applicationShape(source) === 'stateful') throw shapeFailure('statefulBuildUnsupported');
  const environment=(await (await fetch(new URL("manifest.json",BASE),{cache:"no-store"})).json()).closure;
  return runBuildWork({appId,source:source.reference,environment},()=>withPortableBuildPermission({id:"portable-"+crypto.randomUUID(),source:source.reference}, async lease => {
  const t0 = performance.now();
  const files = new Map(source.files);
  const pkg = await (await fetch(new URL("scaffold/package.json", BASE))).json();
  const pkgs = Object.fromEntries(Object.entries(pkg.dependencies).map(([n, v]) => [n, v.replace(/^[\^~]/, "")]));
  for (const d of source.dependencies) { const at = d.lastIndexOf("@"); if (at > 0) pkgs[d.slice(0, at)] = d.slice(at + 1); else pkgs[d] = ""; }
  // Every package resolves to one esm.sh URL and is fetched once; react and react-dom are external
  // to every other package so one React is bundled; the output is one script with no imports left.
  const urlOf = (name) => {
    const bare = name.startsWith("@") ? name.split("/").slice(0, 2).join("/") : name.split("/")[0];
    const sub = name.slice(bare.length), v = pkgs[bare] ? "@" + pkgs[bare] : "";
    if (bare === "react") return `https://esm.sh/react${v}${sub}?target=es2022`;
    if (bare === "react-dom") return `https://esm.sh/react-dom${v}${sub}?target=es2022&external=react`;
    return `https://esm.sh/${bare}${v}${sub}?target=es2022&external=react,react-dom`;
  };
  return withBuildModules(async remote => {
  const tw = await (await fetch(new URL("scaffold/tailwind.config.ts", BASE))).text();
  const m = tw.match(/theme:\s*(\{[\s\S]*\n  \}),\n  plugins/);
  const tailwindConfig = m ? "{darkMode:['class'],theme:" + m[1] + "}" : "{}";
  const exts = ["", ".tsx", ".ts", ".jsx", ".js", "/index.tsx", "/index.ts"];
  const resolveFile = (p) => { for (const e of exts) if (files.has(p + e)) return p + e; return null; };
  const plugin = { name: "portable", setup(b) {
    b.onResolve({ filter: /.*/ }, (args) => {
      if (args.namespace === "http") {
        if (/^https?:\/\//.test(args.path) || args.path.startsWith("/") || /^\.\.?\//.test(args.path)) return { path: new URL(args.path, args.importer).href, namespace: "http" };
        return { path: urlOf(args.path), namespace: "http" };
      }
      if (args.kind === "entry-point") { const r = resolveFile(args.path); if (r) return { path: r, namespace: "project" }; }
      if (args.path.startsWith("@/")) { const r = resolveFile("src/" + args.path.slice(2)); if (r) return { path: r, namespace: "project" }; }
      if (/^\.\.?\//.test(args.path) || args.path.startsWith("/")) {
        const base = args.importer ? args.importer.split("/").slice(0, -1).join("/") : "src";
        const norm = new URL(args.path, "file:///" + base + "/").pathname.slice(1);
        const r = resolveFile(norm); if (r) return { path: r, namespace: "project" };
        return { errors: [{ text: "not in project: " + norm }] };
      }
      if (/^https?:\/\//.test(args.path)) return { path: args.path, namespace: "http" };
      return { path: urlOf(args.path), namespace: "http" };
    });
    b.onLoad({ filter: /.*/, namespace: "http" }, async (args) => {
      const text = await remote.fetchText(args.path);
      return { contents: text, loader: /\.css(\?|$)/.test(args.path) ? "css" : "js" };
    });
    b.onLoad({ filter: /.*/, namespace: "project" }, (args) => {
      const src = files.get(args.path);
      if (args.path.endsWith(".css")) return { contents: src.replace(/@tailwind[^;]*;/g, ""), loader: "css" };
      return { contents: src, loader: args.path.endsWith(".tsx") ? "tsx" : args.path.endsWith(".ts") ? "ts" : "jsx" };
    });
  } };
  const es = await esbuild();
  const r = await es.build({ entryPoints: ["src/main.tsx"], bundle: true, write: false, format: "esm", target: "es2022", jsx: "automatic", plugins: [plugin], outdir: "out", logLevel: "silent", minify: true,
    define: { "import.meta.env.DEV": "false", "import.meta.env.MODE": '"production"', "import.meta.env.BASE_URL": "__HOLO_BASE__", "process.env.NODE_ENV": '"production"' } });
  const js = r.outputFiles.find((f) => f.path.endsWith(".js")).text;
  const css = (r.outputFiles.find((f) => f.path.endsWith(".css")) || { text: "" }).text;
  // Tailwind, compiled once at publish: the play build cannot be fetched (its CDN sends no CORS
  // header), so the page is loaded once, hidden, under a preview address with the play build as a
  // script tag; the build scans the page (the bundle is inline, so every class it names is seen) and
  // the stylesheet it writes is taken as static CSS. The published page then needs no network and no
  // compiler to style itself.
  const inline = (s) => s.replace(/<\/script/g, "<\\/script");
  const body = `<style>${css}</style>${await brandHead()}</head><body><div id="root"></div>\n<script type="module">${inline(js)}<\/script></body></html>`;
  const head = `<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">\n<script>var __HOLO_BASE__ = location.pathname.replace(/index\\.html$/, "");`;
  // Every class like token of the bundle, the project's CSS and its HTML, listed once in a hidden
  // element, so the compiler's first scan already sees every candidate whether or not the page has
  // rendered it yet; the compile is then judged by the rendered page: no class in use without a rule.
  const candidates = [...new Set((js + " " + css).match(/[A-Za-z][A-Za-z0-9_\-:\/\.\[\]#%!]*/g) || [])].filter((t) => t.length < 120).join(" ");
  const compilePage = head + ` tailwind = { config: ${tailwindConfig} };<\/script>\n<script src="https://cdn.tailwindcss.com"><\/script>\n<div hidden id="tw-candidates" class="${candidates.replace(/"/g, "")}"></div>\n` + body;
  const tailwindCss = await compileTailwind(compilePage, lease);
  const html = head + `<\/script>\n<style>${tailwindCss}</style>\n` + body;
  await assertBuildSourceCurrent(await dbReady, appId, source);
  return { html, sourceReference: source.reference, modules: remote.count, jsBytes: js.length, cssBytes: css.length, tailwindBytes: tailwindCss.length, ms: Math.round(performance.now() - t0) };
  }, {signal:lease.signal,fetch:async(url,init)=>{await lease.check();return fetch(url,init);}});
  }, {
    assertSource:async()=>assertBuildSourceCurrent(await dbReady, appId, source),
    grant:async(id,permission)=>grantPreview(id,permission.disclosure.endpoints,permission,undefined),
    readGrant:id=>get("refs","capabilities:"+id),
    revoke:id=>del("refs","capabilities:"+id),
  }), {guest:await kernelReady,derive:kappa,...buildWorkStore(await dbReady,appId,source)});
}
// Runs inside the compile frame. The frame is a preview address, so the worker serves it in a CSP
// sandbox with an opaque origin and the host cannot read its document: the frame reports instead,
// every 250 ms, the compiled sheet, whether the app rendered, and the classes in use that no rule
// covers. A report is the app's own word about its own styles; a false one can only spoil that app.
function tailwindReporter() {
  const escapeClass = (c) => c.replace(/[^a-zA-Z0-9_-]/g, (m) => "\\" + m);
  setInterval(() => {
    const sheets = [...document.querySelectorAll("style")].map((s) => s.textContent);
    const root = document.getElementById("root");
    const all = sheets.join("\n");
    const used = new Set([...document.querySelectorAll("[class]")].filter((e) => e.id !== "tw-candidates").flatMap((e) => String(e.className.baseVal ?? e.className).split(/\s+/).filter(Boolean)));
    parent.postMessage({ tailwindReport: true, sheet: sheets.find((t) => t.includes("--tw-")) || "", rendered: !!(root && (root.childElementCount || root.textContent.trim())), missing: [...used].filter((c) => !all.includes("." + escapeClass(c))) }, "*");
  }, 250);
}

async function compileTailwind(page, lease) {
  await lease.check();
  const id = lease.id;
  const path = new URL("p/" + id + "/index.html", BASE).pathname;
  const at = page.lastIndexOf("</body>");
  const reporting = page.slice(0, at) + `<script>(${tailwindReporter})();<\/script>` + page.slice(at);
  await lease.check();
  const cache = await caches.open("previews");
  await cache.put(path, new Response(reporting, { headers: { "content-type": "text/html; charset=utf-8" } }));
  const frame = document.createElement("iframe");
  frame.style.cssText = "position:fixed;width:1280px;height:800px;left:-2000px;top:0;visibility:hidden";
  let report = null;
  const listen = (e) => { if (e.source === frame.contentWindow && e.data && e.data.tailwindReport) report = e.data; };
  window.addEventListener("message", listen);
  frame.src = path;
  const removeFrame = () => frame.remove();
  lease.signal.addEventListener("abort", removeFrame, {once:true});
  try {
    await lease.check();
    document.body.append(frame);
    const t0 = performance.now();
    let last = "", stable = 0;
    while (performance.now() - t0 < 30000) {
      await new Promise((r) => setTimeout(r, 250));
      await lease.check();
      if (!report || !report.sheet || !report.rendered) continue;
      // Converged when the rendered page uses no class the compiled sheet (or the project's own CSS) lacks.
      // The sheet is inlined into a <style> element of the published page, so it may not close that element.
      const sheet = String(report.sheet).replace(/<\/style/gi, "<\\/style");
      const missing = Array.isArray(report.missing) ? report.missing.map(String) : [];
      const key = sheet.length + ":" + missing.join(",");
      if (key === last) { stable += 1; if (stable >= 3 && missing.length === 0) return sheet; if (stable >= 12) throw portablePermissionFailure("portableBuildStyleIncomplete"); } else { last = key; stable = 0; }
    }
    if (last) throw new Error("tailwind did not converge on the page");
    throw new Error("tailwind did not compile the page");
  } finally {
    window.removeEventListener("message", listen);
    lease.signal.removeEventListener("abort", removeFrame);
    frame.remove();
    await cache.delete(path);
  }
}
window.__portable = buildPortable;
const PROVENANCE_FIELDS = ["source_id", "semantic_id", "compiler_semantics_id", "snapshot_id", "stdlib_semantics_id", "prism_stdlib_crate_sha256", "lexlean_commit", "lexlean_package_sha256", "lean4_prod_commit", "hologram_live_commit", "uor_hologram_commit", "target_profile_id", "lean_manifest_sha256", "lcnf_manifest_sha256", "generated_core_sha256", "cargo_name", "cargo_version", "cargo_crate_sha256", "view_model_id", "browser_projection_sha256"];
let lastPublish = null;
const holoUrl = (k) => new URL(core.run({ op: "holo-path", kappa: k }).path.slice(1), BASE).href;
handlers.set("holo:view", async () => { await coreReady; return core.run({ op: "view" }); });
handlers.set("holo:list", async (appId) => { await coreReady; return (await installed()).filter((r) => r.appId === appId).map((r) => ({ ...r, url: holoUrl(r.kappa) })); });
handlers.set("holo:publish", async ({ appId }) => {
  await coreReady;
  const t0 = performance.now();
  const words = core.run({ op: "view" });
  const [manifest, provenance] = await Promise.all([
    fetch(new URL("manifest.json", BASE), { cache: "no-store" }).then((r) => r.json()),
    fetch(new URL("provenance.json", BASE), { cache: "no-store" }).then((r) => (r.ok ? r.json() : null)).catch(() => null),
  ]);
  const closure = manifest.closure, built = provenance ? provenance.browser_projection_sha256 || "" : "";
  const attestation = provenance && provenance.attestation_id ? provenance.attestation_id : null;
  const { decision } = core.run({ op: "publish-decision", closure, built, attestation });
  const captured = await verifiedBuildSource(appId);
  const app = { name: captured.name }, version = captured.reference;
  const row = (outcome, resource) => put("audit", Date.now() + ":publish:" + appId, { timestamp_millis: Date.now(), principal: "app:" + appId, operation: "holo.application.publish", resource, outcome });
  if (decision !== "Accept") { await row("refused", version || ""); return { decision, word: words.publishRefusedLabel, closure, built, attestation }; }
  // Cached proof status can be absent after an edit; source determines the publication route.
  // An app with rules is published only once Lean proved them; its page asks the kernel guest (rulePage).
  const ruled = requiresRulePublication(captured), rules = ruled ? await rulesOf(appId) : null;
  if (ruled && !(rules && rules.spec && (rules.phase === "proved" || rules.phase === "published"))) { await row("refused", version); return { decision: "Chat", word: "This app has rules: approve them in the chat, then say publish.", closure, built, attestation }; }
  const page = ruled ? { files: await rulePage(rules.spec), ms: 0 } : await buildPortable(appId, captured);
  const view = encodeView(ruled ? page.files : [{ path: "index.html", bytes: enc.encode(page.html) }, { path: "build/modules.json", bytes: enc.encode(JSON.stringify(page.moduleClosure, null, 2) + "\n") }, ...page.moduleFiles]);
  const guest = new Uint8Array(await (await fetch(new URL(ruled ? "kernel/guest.wasm" : "core.wasm", BASE), { cache: "no-store" })).arrayBuffer());
  const model = core.run({ op: "publish-preimage", application: app.name, version, closure, attestation }).bytes;
  const modelBytes = enc.encode(model);
  const modelId = await sha256(modelBytes);
  const source = core.run({ op: "source-manifest", leanManifest: provenance.lean_manifest_sha256, coverage: provenance.coverage_sha256, kernel: provenance.kernel_ir_sha256, modelId, semanticId: provenance.semantic_id, sourceId: provenance.source_id }).bytes;
  const sourceBytes = enc.encode(source);
  const prov = Object.fromEntries(PROVENANCE_FIELDS.map((k) => [k, provenance[k]]));
  const t1 = performance.now();
  const composed = core.run({ op: "compose", application: app.name, guest: b64(guest), view: b64(view), model: b64(modelBytes), source: b64(sourceBytes), provenance: prov });
  const composeMs = Math.round(performance.now() - t1);
  const bytes = unb64(composed.bytes);
  const address = await kappa(bytes);
  if (address !== composed.identities.archive_kappa) throw new Error("the page's address of the archive differs from PrismPM's: " + address);
  await assertBuildSourceCurrent(await dbReady, appId, captured);
  const done = await install(bytes, { appId, application: app.name, version });
  tmNameApp("holo:" + done.path.split("/").filter(Boolean).pop(), appId, app.name);
  await row("granted", address);
  const record = { application: app.name, version, closure, attestation, provenance: prov, identities: composed.identities, directory: composed.directory, byteLength: bytes.length, modelId, viewBytes: view.length, guestBytes: guest.length };
  lastPublish = { record, bytes, guest, view, model: modelBytes, source: sourceBytes, page };
  return { decision, kappa: address, url: done.url, byteLength: bytes.length, viewBytes: view.length, guestBytes: guest.length, ms: Math.round(performance.now() - t0), buildMs: page.ms, composeMs, installMs: done.ms, modules: page.modules, jsBytes: page.jsBytes, cssBytes: page.cssBytes, tailwindBytes: page.tailwindBytes, identities: composed.identities, record };
});

window.electron = {
  ipcRenderer: {
    invokeEnvelope: (channel, ...args) => dispatch(channel, args),
    invoke: (channel, ...args) => dispatch(channel, args).then(unwrapHost),
    send: (channel, ...args) => {
      if (channel === "first-prompt:commit-creation") {
        const opId = args[0]?.operationId;
        if (opId) pendingCreations.delete(opId);
        return;
      }
      if (channel === "first-prompt:cancel-creation") {
        const opId = args[0]?.operationId;
        const pending = opId ? pendingCreations.get(opId) : null;
        if (pending) {
          pendingCreations.delete(opId);
          void (pending.kind === "app"
            ? handlers.get("delete-app")({ appId: pending.appId })
            : dbReady.then((db) => deleteStoredChat(db, pending.chatId))
          ).catch(() => {});
        }
        return;
      }
      const n = (seen.get(channel) || 0) + 1; seen.set(channel, n); if (n === 1) console.warn("[host] send:", channel);
    },
    on: (channel, fn) => { if (!listeners.has(channel)) listeners.set(channel, new Set()); listeners.get(channel).add(fn); return () => listeners.get(channel).delete(fn); },
    removeListener: (channel, fn) => { const set = listeners.get(channel); if (set) set.delete(fn); },
    removeAllListeners: (channel) => { listeners.delete(channel); },
  },
};
// The landing page keeps the visitor's first prompt; on arrival the builder creates the app from
// the scaffold, opens its chat and runs the turn, so the visitor lands in the renderer's UI mid build.
(async () => {
  let first = null;
  try { first = JSON.parse(sessionStorage.getItem("holo.first-prompt.v1") || "null"); sessionStorage.removeItem("holo.first-prompt.v1"); } catch (e) {}
  if (!first || !first.prompt) return;
  const name = ("app-" + first.prompt.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")).slice(0, 40) || "app";
  const created = await handlers.get("create-app")({ name });
  const intentId="first-"+crypto.randomUUID();
  const intent=await bindQueuePayload({schemaVersion:1,intentId,appId:created.app.id,chatId:created.chatId,invocationRef:{kind:"chat-stream",entityKey:created.chatId,operationId:intentId},prompt:first.prompt,redo:false,selectedComponents:[],requestedChatMode:"build"},serializeImmutableChatTurnPayload);
  await chatDispatch({encodedKey:{chatId:created.chatId},messageId:intentId,encodedEvent:{type:"SUBMIT",intent,observedStopPolicyVersion:0}});
  location.hash = "#/chat?id=" + created.chatId + "&appId=" + created.app.id;
})();
const mvmHost = createMvmHost({
  emit,
  getAppName: async (appId) => (appId != null ? (await get("apps", appId))?.name : null),
});

let linuxVmHost = null;
let stitchNetwork = null;
let syncFromLinuxActive = false;

const githubVault = createGitHubVault({
  coreReady: coreReady.then(() => core),
  prismFrame: inference.prismFrame,
  getRef: (k) => get("refs", k),
  putRef: (k, v) => put("refs", k, v),
  deleteRef: (k) => del("refs", k),
});

const githubClient = createGitHubClient({
  vault: githubVault,
  getApp: (appId) => get("apps", appId),
  listApps: () => all("apps"),
  putApp: (appId, app) => put("apps", appId, app),
  listAppFiles: (appId) => filesOf(appId),
  readAppFile: (appId, rel) => get("files", `${appId}:${rel}`),
  getRef: (k) => get("refs", k),
  putRef: (k, v) => put("refs", k, v),
  deleteRef: (k) => del("refs", k),
  createClonedProject: async ({ name, files, appDefaults }) => {
    await coreReady;
    const prepared = await prepareNewProject({ name, initialChatMode: "build", files }, { core, derive: kappa });
    const created = await createStoredProject(await dbReady, prepared, { ...nulls, ...appDefaults }, themesData);
    void windowCoordination.invalidate([{ family: "apps" }]).catch(() => {});
    return created;
  },
  applyWorkspacePatch: async (appId, patch) => {
    await applyPrismTags(appId, patch, {});
    void windowCoordination.invalidate([{ family: "apps" }, { family: "versions", appId }]).catch(() => {});
  },
  onWorkspaceSynced: async (appId) => {
    const st = linuxVmHost?.getStatus?.();
    if (st && st.state === "online" && st.activeAppId === appId) {
      await linuxVmHost.syncWorkspaceToVm(appId).catch(() => {});
    }
  },
  emit,
});

stitchNetwork = createStitchNetwork({
  emit,
  resolveLocalResource: async (resourcePath) => {
    const clean = String(resourcePath || "").replace(/^\/+/, "").split("?")[0];
    if (!clean || clean === "app.json" || clean === "standards-index.json" || clean === "models.json") {
      const url = new URL(clean || "app.json", BASE);
      const r = await fetch(url, { credentials: "omit", redirect: "error" });
      if (!r.ok) return { ok: false, status: r.status, error: `HTTP ${r.status}` };
      return { ok: true, status: 200, body_preview: (await r.text()).slice(0, 8192) };
    }
    const statusMatch = clean.match(/^apps\/(\d+)\/(.+)$/);
    if (statusMatch) {
      const content = await get("files", `${Number(statusMatch[1])}:${statusMatch[2]}`);
      if (typeof content === "string") {
        return { ok: true, status: 200, body_preview: content.slice(0, 8192) };
      }
    }
    return { ok: false, status: 404, error: `Unknown local resource: ${clean}` };
  },
  onEthFrame: (frame) => linuxVmHost?.injectEthFrame?.(frame),
});

linuxVmHost = createLinuxVmHost({
  emit,
  getAppName: async (appId) => (appId != null ? (await get("apps", appId))?.name : null),
  listAppFiles: (appId) => filesOf(appId),
  readAppFile: (appId, rel) => get("files", `${appId}:${rel}`),
  applyWorkspaceChanges: async (appId, patch) => {
    syncFromLinuxActive = true;
    try {
      await applyPrismTags(appId, patch, {});
      void windowCoordination.invalidate([{ family: "apps" }, { family: "versions", appId }]).catch(() => {});
    } finally {
      syncFromLinuxActive = false;
    }
  },
  getEthBridge: () => stitchNetwork,
});

function getTerminalRuntime() {
  try {
    return localStorage.getItem("uor.terminal.runtime") === "mvm" ? "mvm" : "linux";
  } catch {
    return "linux";
  }
}

function setTerminalRuntime(mode) {
  const next = mode === "mvm" ? "mvm" : "linux";
  try {
    localStorage.setItem("uor.terminal.runtime", next);
  } catch {}
  emit("terminal:runtime-changed", { mode: next });
  return { ok: true, mode: next };
}

function terminalHostForSession(sessionId) {
  return String(sessionId || "").startsWith("mvm-") ? mvmHost : linuxVmHost;
}

function activeTerminalHost() {
  return getTerminalRuntime() === "mvm" ? mvmHost : linuxVmHost;
}

handlers.set("terminal:get-runtime", () => ({ mode: getTerminalRuntime(), linux: linuxVmHost.getStatus() }));
handlers.set("terminal:set-runtime", ({ mode } = {}) => setTerminalRuntime(mode));
handlers.set("terminal:open", (input) => activeTerminalHost().openSession(input));
handlers.set("terminal:close", (input) => terminalHostForSession(input?.sessionId).closeSession(input));
handlers.set("terminal:kill", (input) => terminalHostForSession(input?.sessionId).killSession(input));
handlers.set("terminal:write", (input) => terminalHostForSession(input?.sessionId).writeInput(input));
handlers.set("terminal:resize", (input) => terminalHostForSession(input?.sessionId).resizeSession(input));
handlers.set("terminal:serialize", (input) => terminalHostForSession(input?.sessionId).serializeSession(input));
handlers.set("linux:status", () => linuxVmHost.getStatus());
handlers.set("linux:sync-workspace", async ({ appId, direction = "both" } = {}) => {
  const toVm = direction === "to-vm" || direction === "both" ? await linuxVmHost.syncWorkspaceToVm(appId) : null;
  const fromVm = direction === "from-vm" || direction === "both" ? await linuxVmHost.syncVmToWorkspace(appId) : null;
  return { ok: true, toVm, fromVm };
});
handlers.set("mvm:exec", async ({ appId, command }) => mvmHost.execCommand(appId ?? null, command));
handlers.set("mvm:identity", async ({ appId } = {}) => mvmHost.getIdentity(appId ?? null));
handlers.set("stitch:status", () => stitchNetwork.getStatus());
handlers.set("stitch:connect", (opts = {}) => stitchNetwork.joinChannel(opts));
handlers.set("stitch:disconnect", () => stitchNetwork.disconnect());

handlers.set("github:status", () => githubClient.getStatus());
handlers.set("github:connect-token", (input = {}) => githubClient.connect(input));
handlers.set("github:disconnect", () => githubClient.disconnect());
handlers.set("github:list-repos", () => githubClient.listRepos());
handlers.set("github:get-repo-branches", (input) => githubClient.getRepoBranches(input));
handlers.set("github:is-repo-available", (input) => githubClient.isRepoAvailable(input));
handlers.set("github:create-repo", (input) => githubClient.createRepo(input));
handlers.set("github:link-repo", async (input) => {
  const res = await githubClient.linkRepo(input);
  void windowCoordination.invalidate([{ family: "apps" }]).catch(() => {});
  return res;
});
handlers.set("github:unlink-repo", async (input) => {
  const res = await githubClient.unlinkRepo(input);
  void windowCoordination.invalidate([{ family: "apps" }]).catch(() => {});
  return res;
});
handlers.set("github:diff", (input) => githubClient.diff(input));
handlers.set("github:pull", (input) => githubClient.pull(input));
handlers.set("github:push", (input) => githubClient.push(input));
handlers.set("github:list-remote-branches", async ({ appId } = {}) => {
  const app = appId ? await get("apps", appId) : null;
  if (!app?.githubOrg || !app?.githubRepo) return [];
  const branches = await githubClient.getRepoBranches({ owner: app.githubOrg, repo: app.githubRepo });
  return branches.map((b) => b.name);
});
handlers.set("github:list-collaborators", async ({ appId } = {}) => {
  try {
    const status = await githubClient.getStatus();
    if (!status.connected || !appId) return [];
    const app = await get("apps", appId);
    if (!app?.githubOrg || !app?.githubRepo) return [];
    return [];
  } catch {
    return [];
  }
});
handlers.set("github:clone-repo-from-url", async (input) => {
  try {
    const res = await githubClient.cloneRepoFromUrl(input);
    const fullApp = await handlers.get("get-app")(res.app.id);
    return { app: fullApp, hasAiRules: Boolean(res.hasAiRules) };
  } catch (err) {
    return { error: String(err?.message || err) };
  }
});

let vmSyncTimer = null;
inference.onWorkspaceWrite((store, key) => {
  if (syncFromLinuxActive || store !== "files") return;
  const parsedAppId = Number(String(key || "").split(":")[0]);
  if (!Number.isSafeInteger(parsedAppId) || parsedAppId < 1) return;
  const st = linuxVmHost.getStatus();
  if (st.state !== "online" || st.activeAppId !== parsedAppId) return;
  clearTimeout(vmSyncTimer);
  vmSyncTimer = setTimeout(() => {
    void linuxVmHost.syncWorkspaceToVm(parsedAppId).catch(() => {});
  }, 250);
});
uorR4.onProgress((progress) => emit("uor-r4:progress", progress));
handlers.set("standards:lookup", ({ query, limit } = {}) => lookupStandards(query, limit));
handlers.set("uor-r4:status", () => uorR4.getCatalog({ refresh: true }));
handlers.set("uor-r4:test-stitch-recover", ({ groupIndex = 0, dropped = 2 } = {}) => uorR4.testStitchRecover({ groupIndex, dropped }));
handlers.set("uor-r4:select-model", ({ modelId }) => uorR4.ensureModelLoaded(modelId));
handlers.set("uor-r4:load-local-file", async ({ fileName, bytes, bytesBase64, tokenizerJson }) => {
  let rawBytes;
  if (bytes instanceof Uint8Array) {
    rawBytes = bytes;
  } else if (Array.isArray(bytes)) {
    rawBytes = new Uint8Array(bytes);
  } else if (typeof bytesBase64 === "string") {
    rawBytes = unb64(bytesBase64);
  } else {
    throw new Error("bytes or bytesBase64 is required");
  }
  const tokenizerBytes = typeof tokenizerJson === "string" && tokenizerJson ? enc.encode(tokenizerJson) : null;
  return uorR4.loadLocalArtifact({ fileName, bytes: rawBytes, tokenizerBytes });
});
handlers.set("uor-r4:predict-discrete", ({ modelId, prompt, tokenIds }) => uorR4.predictDiscrete({ modelId, prompt, tokenIds }));

let homeChatAbort = null;
let homeChatEpoch = 0;
handlers.set("home-chat:history", async () => {
  const stored = (await get("refs", "home-chat:history")) || [];
  return { messages: Array.isArray(stored) ? stored : [] };
});
handlers.set("home-chat:clear", async () => {
  homeChatEpoch += 1;
  if (homeChatAbort) {
    homeChatAbort.abort(new Error("Cleared"));
    homeChatAbort = null;
  }
  await put("refs", "home-chat:history", []);
  emit("home-chat:chunk", { messages: [] });
  emit("home-chat:end", { outcome: "cancelled", messages: [] });
  reloadIfIdle();
  return { messages: [] };
});
handlers.set("home-chat:cancel", async () => {
  if (homeChatAbort) {
    homeChatAbort.abort(new Error("Cancelled"));
  }
  return { cancelled: true };
});
handlers.set("home-chat:send", async ({ prompt, modelId } = {}) => {
  const text = String(prompt ?? "").trim();
  if (!text) throw new Error("Prompt is required");
  homeChatEpoch += 1;
  const myEpoch = homeChatEpoch;
  if (homeChatAbort) {
    homeChatAbort.abort(new Error("Superseded"));
    homeChatAbort = null;
  }
  const abort = new AbortController();
  homeChatAbort = abort;

  const settings = (await get("settings", "user")) || DEFAULT_SETTINGS;
  const fallbackModel =
    !settings.selectedModel ||
    settings.selectedModel.provider !== UOR_R4_PROVIDER ||
    settings.selectedModel.name === "uor-r4:synthetic-seed-5" ||
    settings.selectedModel.name === "synthetic-seed-5"
      ? { provider: UOR_R4_PROVIDER, name: UOR_R4_DEFAULT_MODEL }
      : settings.selectedModel;
  const requested = modelId ? { provider: UOR_R4_PROVIDER, name: modelId } : null;
  const homeRoute = chatModelRoute(requested, fallbackModel);
  const activeModel = homeRoute.model;
  if (myEpoch !== homeChatEpoch) {
    return { outcome: "cancelled", messages: [] };
  }
  const prior = ((await get("refs", "home-chat:history")) || []).filter(Boolean);
  if (myEpoch !== homeChatEpoch) {
    return { outcome: "cancelled", messages: [] };
  }
  const nextId = prior.length ? Math.max(...prior.map((x) => Number(x.id) || 0)) + 1 : 1;
  const userMsg = { id: nextId, role: "user", content: text, createdAt: now() };
  const asstMsg = { id: nextId + 1, role: "assistant", content: "", model: activeModel, createdAt: now() };
  const messages = [...prior, userMsg, asstMsg];
  await put("refs", "home-chat:history", messages);
  if (myEpoch !== homeChatEpoch) {
    return { outcome: "cancelled", messages: [] };
  }
  emit("home-chat:chunk", { messages });

  let error = null;
  let latest = "";
  let lastSave = 0;
  let homeTextWork = Promise.resolve();
  const enqueueHomeText = (partial) => {
    homeTextWork = homeTextWork.then(async () => {
      if (myEpoch !== homeChatEpoch) return;
      latest = prism.withoutThinking(partial);
      messages[messages.length - 1] = { ...messages[messages.length - 1], content: latest };
      emit("home-chat:chunk", { messages: [...messages] });
      if (Date.now() - lastSave > 200) {
        lastSave = Date.now();
        await put("refs", "home-chat:history", messages).catch(() => {});
      }
    });
    void homeTextWork.catch(() => {});
    return homeTextWork;
  };

  try {
    await uorR4.ensureModelLoaded(activeModel, { signal: abort.signal });
    const status = uorR4.getStatus();
    const before = r4Request({
      provider: homeRoute.provider,
      engine: "match",
      weights: status?.weightsMatched ? "match" : "mismatch",
    });
    const allowed = await r4Decide(before);
    if (allowed.outcome !== "answer") throw new Error(r4Refusal(allowed, before));
    const result = await runUorR4AgentTurn({
      appId: null,
      chatId: "home-chat",
      modelId: activeModel,
      messages: prior,
      userPrompt: text,
      chatMode: "chat",
      signal: abort.signal,
      uorR4Runtime: uorR4,
      mvmHost,
      tools: createFoundryTools({ appId: null, chatMode: "chat", signal: abort.signal }),
      onText: enqueueHomeText,
    });
    await homeTextWork;
    if (myEpoch === homeChatEpoch) {
      latest = prism.withoutThinking(result.fullText || latest);
      const made = r4Request({
        provider: homeRoute.provider,
        engine: "match",
        weights: result.weightsMatched ? "match" : "mismatch",
      });
      const shown = await r4Decide(made);
      if (shown.outcome !== "answer") throw new Error(r4Refusal(shown, made));
      messages[messages.length - 1] = {
        ...messages[messages.length - 1],
        content: latest,
        toolExecutions: result.toolExecutions || 0,
        model: activeModel,
      };
    }
  } catch (e) {
    await homeTextWork.catch(() => {});
    if (myEpoch === homeChatEpoch) {
      error = abort.signal.aborted ? null : String(e?.message || e);
      if (abort.signal.aborted) {
        messages[messages.length - 1] = {
          ...messages[messages.length - 1],
          content: latest + (latest ? "\n\n_Cancelled._" : "_Cancelled._"),
        };
      } else if (error) {
        messages[messages.length - 1] = {
          ...messages[messages.length - 1],
          content: (latest ? latest + "\n\n" : "") + `**Error:** ${error}`,
        };
      }
    }
  } finally {
    if (homeChatAbort === abort) homeChatAbort = null;
    reloadIfIdle();
  }
  if (myEpoch !== homeChatEpoch) {
    return { outcome: "cancelled", messages: [] };
  }
  await put("refs", "home-chat:history", messages);
  const outcome = abort.signal.aborted ? "cancelled" : error ? "errored" : "completed";
  emit("home-chat:chunk", { messages });
  emit("home-chat:end", { outcome, messages, ...(error ? { error } : {}) });
  return { outcome, messages, ...(error ? { error } : {}) };
});
handlers.set("home-chat:promote-to-studio", async ({ name, prompt, autoBuild = true } = {}) => {
  const history = ((await get("refs", "home-chat:history")) || []).filter(Boolean);
  const firstUser = history.find((m) => m.role === "user")?.content || prompt || "studio app";
  const derivedName =
    String(name || "").trim() ||
    ("app-" + firstUser.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")).slice(0, 40) ||
    "app";
  const created = await handlers.get("create-app")({ name: derivedName, initialChatMode: "build" });
  const appId = created.app.id;
  const chatId = created.chatId;
  const transcriptText = history.length
    ? history.map((m) => `${m.role === "user" ? "User" : "Assistant"}: ${String(m.content || "").trim()}`).join("\n\n").slice(0, 3500)
    : String(prompt || "");
  const buildPrompt = String(prompt || "").trim() || `Build the application discussed in Home Chat:\n\n${transcriptText}`;
  if (autoBuild) {
    const intentId = "promote-" + crypto.randomUUID();
    const intent = await bindQueuePayload(
      {
        schemaVersion: 1,
        intentId,
        appId,
        chatId,
        invocationRef: { kind: "chat-stream", entityKey: chatId, operationId: intentId },
        prompt: buildPrompt,
        redo: false,
        selectedComponents: [],
        requestedChatMode: "build",
      },
      serializeImmutableChatTurnPayload,
    );
    await chatDispatch({
      encodedKey: { chatId },
      messageId: intentId,
      encodedEvent: { type: "SUBMIT", intent, observedStopPolicyVersion: 0 },
    });
  }
  return { appId, appName: created.app.name, chatId, buildPrompt };
});

window.__gallery = Object.freeze({read: appId => readGallery(dbReady, appId)});
window.__host = { handlers, emit, seen, get, put, all, kappa, core: () => core, coreReady, view: () => coreReady.then((c) => c.run({ op: "view" })), publish: () => lastPublish };
window.__uorR4 = Object.freeze({
  getCatalog: uorR4.getCatalog,
  getStatus: uorR4.getStatus,
  ensureModelLoaded: uorR4.ensureModelLoaded,
  loadLocalArtifact: uorR4.loadLocalArtifact,
  predictDiscrete: uorR4.predictDiscrete,
  mvmExec: (cmd, appId = null) => mvmHost.execCommand(appId, cmd),
  mvmIdentity: (appId = null) => mvmHost.getIdentity(appId),
  linuxExec: (cmd, appId = null, opts = {}) => linuxVmHost.execCommand(appId, cmd, opts),
  linuxStatus: () => linuxVmHost.getStatus(),
  linuxSyncWorkspace: (appId, direction = 'both') => handlers.get('linux:sync-workspace')({ appId, direction }),
  stitchStatus: () => stitchNetwork.getStatus(),
  stitchConnect: (opts = {}) => stitchNetwork.joinChannel(opts),
  stitchDisconnect: () => stitchNetwork.disconnect(),
  githubStatus: () => githubClient.getStatus(),
  submitPrompt: async (chatId, appId, prompt) => {
    const intentId = 'uor-' + crypto.randomUUID();
    const intent = await bindQueuePayload(
      {
        schemaVersion: 1,
        intentId,
        appId,
        chatId,
        invocationRef: { kind: 'chat-stream', entityKey: chatId, operationId: intentId },
        prompt,
        redo: false,
        selectedComponents: [],
        requestedChatMode: 'build',
      },
      serializeImmutableChatTurnPayload,
    );
    return chatDispatch({
      encodedKey: { chatId },
      messageId: intentId,
      encodedEvent: { type: 'SUBMIT', intent, observedStopPolicyVersion: 0 },
    });
  },
});

// ---- afflom/me ?embed=me postMessage bridge (strict origin allowlist; active only when embedded)
if (new URLSearchParams(location.search).get('embed') === 'me' && window.parent && window.parent !== window) {
  const EMBED_PROTOCOL = 'uor-workspace-bridge/1';
  const defaultAllowedParents = new Set(['https://afflom.github.io', 'https://uor-foundation.github.io', location.origin]);
  const isLoopbackOrigin = (origin) => {
    try {
      const u = new URL(origin);
      return u.protocol === 'http:' && (u.hostname === '127.0.0.1' || u.hostname === 'localhost');
    } catch {
      return false;
    }
  };
  const isAllowedParentOrigin = (origin) => {
    if (!origin || origin === 'null') return false;
    if (defaultAllowedParents.has(origin)) return true;
    if (isLoopbackOrigin(location.origin) && isLoopbackOrigin(origin)) return true;
    try {
      const custom = JSON.parse(localStorage.getItem('uor.me.allowedParentOrigins') || '[]');
      if (Array.isArray(custom) && custom.includes(origin)) return true;
    } catch {}
    return false;
  };
  window.addEventListener('message', async (event) => {
    if (!isAllowedParentOrigin(event.origin)) return;
    const msg = event.data;
    if (!msg || typeof msg !== 'object' || msg.protocol !== EMBED_PROTOCOL || typeof msg.action !== 'string') return;
    const reply = (payload) => {
      try {
        event.source?.postMessage?.({ protocol: EMBED_PROTOCOL, id: msg.id ?? null, ...payload }, event.origin);
      } catch {}
    };
    try {
      if (msg.action === 'ping') {
        reply({ ok: true, result: { studio: 'uor-r4-foundry', ready: true, runtime: await handlers.get('uor-r4:status')() } });
      } else if (msg.action === 'get-status') {
        reply({
          ok: true,
          result: {
            runtime: getTerminalRuntime(),
            linux: linuxVmHost.getStatus(),
            stitch: await stitchNetwork.getStatus(),
            github: await githubClient.getStatus(),
          },
        });
      } else if (msg.action === 'list-apps') {
        const { apps } = await handlers.get('list-apps')();
        reply({
          ok: true,
          result: apps.map((a) => ({ id: a.id, name: a.name, githubOrg: a.githubOrg, githubRepo: a.githubRepo, githubBranch: a.githubBranch })),
        });
      } else if (msg.action === 'open-app' && Number.isSafeInteger(msg.payload?.appId)) {
        const chats = (await all('chats')).filter((c) => c.appId === msg.payload.appId);
        const chatId = chats[0]?.id;
        location.hash = chatId ? `#/chat?id=${chatId}&appId=${msg.payload.appId}` : `#/apps/${msg.payload.appId}`;
        reply({ ok: true, result: { appId: msg.payload.appId, chatId: chatId || null } });
      } else if (msg.action === 'sync-workspace') {
        reply({ ok: true, result: await handlers.get('linux:sync-workspace')(msg.payload || {}) });
      } else {
        reply({ ok: false, error: `Unsupported action: ${msg.action}` });
      }
    } catch (err) {
      reply({ ok: false, error: String(err?.message || err) });
    }
  });
  let targetParentOrigin = null;
  try {
    const refOrigin = document.referrer ? new URL(document.referrer).origin : location.origin;
    if (isAllowedParentOrigin(refOrigin)) targetParentOrigin = refOrigin;
  } catch {}
  if (targetParentOrigin) {
    try {
      window.parent.postMessage({ protocol: EMBED_PROTOCOL, event: 'ready', origin: location.origin }, targetParentOrigin);
    } catch {}
  }
}

window.addEventListener('pagehide', () => {
  void linuxVmHost.destroy().catch(() => {});
  void stitchNetwork.disconnect().catch(() => {});
}, { once: true });
if ("serviceWorker" in navigator) {
  // A page an older worker drew reloads once when a newer worker takes control, so no visitor keeps running
  // a build Foundry has replaced (a removed screen stays removed). The worker's script is always fetched
  // fresh, and checked on every load.
  const drawnByWorker = !!navigator.serviceWorker.controller;
  let reloading = false;
  navigator.serviceWorker.addEventListener("controllerchange", () => { if (drawnByWorker && !reloading) { reloading = true; location.reload(); } });
  // A first visit has no worker running yet; previews wait until it runs, or until its install fails.
  workerActive = navigator.serviceWorker.register(new URL("sw.js", BASE), { updateViaCache: "none" }).then((r) => { r.update().catch(() => {}); const w = r.installing || r.waiting; return r.active || (w && new Promise((ok) => w.addEventListener("statechange", () => /activated|redundant/.test(w.state) && ok()))); }).catch(() => {});
}
// The worker installs only what a first paint needs; once this page has loaded and is idle it is asked to
// fill the rest of the closure (the engine, the chat app, the photos), so offline use follows soon after.
if ("serviceWorker" in navigator) {
  const idle = (fn) => (self.requestIdleCallback ? requestIdleCallback(fn, { timeout: 10000 }) : setTimeout(fn, 3000));
  addEventListener("load", () => idle(() => navigator.serviceWorker.ready.then((r) => r.active && r.active.postMessage("fill")).catch(() => {})), { once: true });
}
// After a deployment a page can run an entry that names a chunk the origin no longer serves: the worker has
// already revalidated the entry, so one reload runs the new build whole.
addEventListener("vite:preloadError", (event) => {
  const key = "holo.chunk.reloaded";
  if (sessionStorage.getItem(key)) return;
  sessionStorage.setItem(key, "1");
  event.preventDefault();
  location.reload();
});
addEventListener("load", () => setTimeout(() => sessionStorage.removeItem("holo.chunk.reloaded"), 10000), { once: true });
// Applications installed before the kernel's gate held no grant record, and the gate refuses to
// serve without one: each gets the grant install now writes, served with no network.
(async () => {
  await coreReady;
  for (const key of await keys("refs")) {
    if(typeof key!=="string"||!key.startsWith("capabilities:portable-"))continue;
    const record=await get("refs",key);
    if(record?.previewConsent?.purpose==="portable-build"&&(!Number.isFinite(record.previewConsent.expires)||Date.now()>=record.previewConsent.expires)){
      const id=key.slice(13);await del("refs",key);await (await caches.open("previews")).delete(new URL("p/"+id+"/index.html",BASE).pathname);
    }
  }
  for (const ref of await installed()) {
    const id = core.run({ op: "holo-path", kappa: ref.kappa }).path.split("/").filter(Boolean).pop();
    if (!(await get("refs", "capabilities:" + id))) await put("refs", "capabilities:" + id, { serve: true, network: { endpoints: [], denied: [] } });
  }
})().catch(() => {});
// One closure per page. A document can be served by a worker that installed an older shell while a
// newer one is published; then its scripts and the page disagree and a screen breaks for reasons that
// are not in the code. The worker names its cache after the closure it installed, and the origin serves
// the closure that is published, so the two are compared here and the page reloads itself once.
(async () => {
  const key = "holo.closure.reloaded";
  try {
    const names = await caches.keys();
    const running = (names.find((n) => n.startsWith("shell-")) || "").slice("shell-".length);
    const published = (await (await fetch(new URL("manifest.json", BASE), { cache: "no-store" })).json()).closure;
    if (!running || !published || running === published) { sessionStorage.removeItem(key); return; }
    if (sessionStorage.getItem(key) === published) { console.warn("[host] this page runs shell", running.slice(0, 12), "while", published.slice(0, 12), "is published; the worker could not take over"); return; }
    sessionStorage.setItem(key, published);
    console.warn("[host] a newer shell is published; reloading once", running.slice(0, 12), "->", published.slice(0, 12));
    location.reload();
  } catch (e) {}
})();

mountInteractionApplications({
  create:input=>handlers.get("forge:interaction-create")(input),
  list:async()=>{const apps=[];for(const app of await all("apps"))if(await get("files",app.id+":prism/interaction.json")!==undefined)apps.push(app);return apps;},
  run:input=>handlers.get("forge:interaction-run")(input),
  probe:input=>executeInteraction(kernelReady,input),
});

mountApplicationDiscovery({discover:input=>handlers.get("forge:discover")(input),accept:input=>handlers.get("forge:discovery-create")(input),open:appId=>openInteractionApplication(appId,{run:input=>handlers.get("forge:interaction-run")(input),probe:input=>executeInteraction(kernelReady,input)})});

// ---- your work on your other devices (FA-95, verification/private-workspace.md). Foundry's pages on every
// device of the visitor's Prism ID meet on the frame's sealed app channel (UOR-Foundation/template FR-05) and
// connect directly (WebRTC; a public STUN server learns their addresses, never the work). They exchange, app by
// app, its origin and when it last changed, and Foundry's Prism kernel decides each (Workspace kind 5, FA-94):
// the newer copy is taken, and a tie goes one way on both devices. A taken app travels as a one-app export
// through Import, which checks every byte before anything is replaced; a window showing that app keeps it until
// the next exchange. A deleted app is deleted. The visitor hears one sentence when work arrives.
const SYNC_KEY = "workspace:sync:", STUN = [{ urls: "stun:stun.cloudflare.com:3478" }], PART = 60000, ORIGIN = /^[0-9a-f]{32}$/;
const APP_REFS = /^(\d+):main$|^(?:prism|gallery|stateful|interaction):(\d+)$/;
const syncMarks = new Map(), syncOrigins = new Map(), chatApps = new Map(), syncChannels = new Set();
let syncApplying = false, syncTimer = 0, syncArrived = false;
const syncDirty = new Set(), syncMe = crypto.randomUUID();
// Every tab of one browser shares the work, so sync runs between devices, never between tabs: each signal carries
// this browser's id, kept where all its tabs read it.
const syncDevice = (() => { try { const id = localStorage.getItem("foundry:sync-device") ?? crypto.randomUUID(); localStorage.setItem("foundry:sync-device", id); return id; } catch { return syncMe; } })();
const newOrigin = () => Array.from(crypto.getRandomValues(new Uint8Array(16)), (b) => b.toString(16).padStart(2, "0")).join("");
// The app a record belongs to, if any.
function appOf(store, key, value) {
  if (store === "apps" || store === "versions") return key;
  if (store === "files") return Number(String(key).split(":")[0]);
  if (store === "chats") return value?.appId ?? chatApps.get(key);
  const m = store === "refs" && typeof key === "string" && key.match(APP_REFS);
  return m ? Number(m[1] ?? m[2]) : undefined;
}
inference.onWorkspaceWrite((store, key, value, deleted) => {
  if (store === "chats" && !deleted && Number.isSafeInteger(value?.appId)) chatApps.set(key, value.appId);
  const appId = appOf(store, key, value);
  if (syncApplying || !Number.isSafeInteger(appId) || appId < 1) return;
  const origin = syncOrigins.get(appId) ?? newOrigin();
  syncOrigins.set(appId, origin);
  markSync(origin, { at: Date.now(), gone: store === "apps" ? Boolean(deleted) : Boolean(deleted && syncMarks.get(origin)?.gone), appId });
});
function markSync(origin, mark) {
  syncMarks.set(origin, mark);
  syncDirty.add(origin);
  clearTimeout(syncTimer);
  syncTimer = setTimeout(async () => {
    const db = await dbReady, dirty = [...syncDirty];
    syncDirty.clear();
    await new Promise((done) => { const t = db.transaction("refs", "readwrite"); for (const o of dirty) t.objectStore("refs").put(syncMarks.get(o), SYNC_KEY + o); t.oncomplete = t.onerror = t.onabort = done; });
    for (const channel of syncChannels) offerSync(channel);
  }, 500);
}
const syncBase64 = (bytes) => { let text = ""; for (let i = 0; i < bytes.length; i += 32768) text += String.fromCharCode(...bytes.subarray(i, i + 32768)); return btoa(text); };
// One app as Import reads it: its files, chats, versions and head, and every object those name.
async function exportApp(appId) {
  const app = await get("apps", appId), paths = await tx("files", "readonly", (s) => s.getAllKeys(appRange(appId)));
  const files = Object.fromEntries(await Promise.all(paths.map(async (k) => [k.slice(String(appId).length + 1), await get("files", k)])));
  const chats = (await all("chats")).filter((c) => c.appId === appId), versions = (await get("versions", appId)) || [], main = await get("refs", appId + ":main");
  const objects = {}, walk = async (k) => {
    if (typeof k !== "string" || k in objects) return;
    const bytes = await get("objects", k);
    if (!(bytes instanceof Uint8Array)) return;
    objects[k] = syncBase64(bytes);
    try { for (const a of JSON.stringify(JSON.parse(new TextDecoder().decode(bytes))).match(/blake3:[0-9a-f]{64}/g) ?? []) await walk(a); } catch {}
  };
  for (const k of [main, ...versions.map((v) => v.oid), ...chats.flatMap((c) => [c.initialCommitHash, ...(c.messages ?? []).flatMap((m) => [m.commitHash, m.sourceCommitHash])])]) await walk(k);
  return JSON.stringify({ schema: EXPORT_SCHEMA, apps: [{ app: { name: app.name, createdAt: app.createdAt, updatedAt: app.updatedAt, extraDeps: app.extraDeps ?? [] }, files, main, versions, chats }], objects });
}
// The kernel's answer for a copy another device holds: take it, keep ours, or a refusal by name.
async function syncDecision(origin, at, gone, device, authentic, bound, within = 1) {
  const local = syncMarks.get(origin), word = (t) => [Math.floor(t / 4294967296), t % 4294967296];
  return (await kernelReady).ask(inference.workspaceRequest(5, 1, 1, authentic, bound, within, local ? 1 : 0, gone ? 1 : 0, ...word(at), ...word(local?.at ?? 0), device > syncMe ? 1 : 0));
}
async function sendSync(channel, value) {
  const text = JSON.stringify(value), id = newOrigin(), n = Math.ceil(text.length / PART);
  for (let i = 0; i < n; i++) {
    while (channel.bufferedAmount > 1 << 22) await new Promise((r) => setTimeout(r, 50));
    channel.send(n === 1 ? text : JSON.stringify({ t: "part", id, i, n, data: text.slice(i * PART, (i + 1) * PART) }));
  }
}
const offerSync = (channel) => channel.readyState === "open" && sendSync(channel, { t: "have", device: syncMe, list: [...syncMarks].map(([o, m]) => [o, m.at, m.gone]) }).catch(() => {});
const valid = (origin, at) => ORIGIN.test(origin) && Number.isSafeInteger(at) && at >= 0;
async function onSync(channel, m) {
  if (m.t === "have" && typeof m.device === "string" && Array.isArray(m.list)) {
    const want = [];
    for (const [origin, at, gone] of m.list) if (valid(origin, at) && (await syncDecision(origin, at, gone, m.device, 1, 1)) === "take") want.push(origin);
    channel.peer = m.device;
    channel.wanted = new Set(want);
    if (want.length) await sendSync(channel, { t: "want", origins: want });
  } else if (m.t === "want" && Array.isArray(m.origins)) {
    for (const origin of m.origins) {
      const mark = syncMarks.get(origin);
      if (mark) await sendSync(channel, { t: "app", origin, at: mark.at, gone: mark.gone, doc: mark.gone ? null : await exportApp(mark.appId) });
    }
  } else if (m.t === "app" && valid(m.origin, m.at)) await takeApp(channel, m);
}
async function takeApp(channel, m) {
  const bound = channel.wanted?.delete(m.origin) ? 1 : 0, local = syncMarks.get(m.origin), estimate = await navigator.storage?.estimate?.().catch(() => null);
  const within = !estimate || estimate.usage + (m.doc?.length ?? 0) < estimate.quota ? 1 : 0;
  let answer = await syncDecision(m.origin, m.at, m.gone, channel.peer, 1, bound, within);
  if (answer === "take" && !m.gone) {
    // Every byte is checked before the local copy goes: a copy that fails is refused, and nothing changes.
    try { await coreReady; await prepareProjectImport(m.doc, { core, derive: kappa, admit: (derived, expected) => core.run({ op: "restore", derived, expected }).decision === "Accept" }, { takenNames: (await all("apps")).filter((a) => a.id !== local?.appId).map((a) => a.name) }); }
    catch { answer = await syncDecision(m.origin, m.at, m.gone, channel.peer, 0, bound, within); }
  }
  if (answer !== "take") return answer === "keep" || inference.tell(answer);
  if (local?.appId && !local.gone && await shownInAWindow(local.appId)) return;
  syncApplying = true;
  try {
    if (local?.appId && !local.gone && (await get("apps", local.appId))) await handlers.get("delete-app")({ appId: local.appId });
    const appId = m.gone ? null : (await handlers.get("forge:import")(m.doc)).apps[0].id;
    syncOrigins.delete(local?.appId);
    if (appId) syncOrigins.set(appId, m.origin);
    markSync(m.origin, { at: m.at, gone: Boolean(m.gone), appId });
  } finally { syncApplying = false; }
  void windowCoordination.invalidate([{ family: "apps" }]).catch(() => {});
  if (!syncArrived && !m.gone) (syncArrived = true), inference.tell("workspaceArrived");
}
async function shownInAWindow(appId) {
  try { return readWindowJournal(await (await windowsStore).read()).windows.some((w) => w.visible.some((v) => v.kind === "app" && v.id === appId)); } catch { return true; }
}
const iceDone = (pc) => new Promise((done) => { if (pc.iceGatheringState === "complete") return done(); pc.addEventListener("icegatheringstatechange", () => pc.iceGatheringState === "complete" && done()); setTimeout(done, 3000); });
void (async () => {
  const db = await dbReady;
  await new Promise((done) => {
    const t = db.transaction("refs"), q = t.objectStore("refs").openCursor(IDBKeyRange.bound(SYNC_KEY, SYNC_KEY + "￿"));
    q.onsuccess = () => { const c = q.result; if (!c) return; const mark = c.value, origin = c.key.slice(SYNC_KEY.length); if (mark) { syncMarks.set(origin, mark); if (!mark.gone) syncOrigins.set(mark.appId, origin); } c.continue(); };
    t.oncomplete = t.onerror = done;
  });
  for (const chat of await all("chats")) chatApps.set(chat.id, chat.appId);
  for (const app of await all("apps")) if (!syncOrigins.has(app.id)) { const origin = newOrigin(); syncOrigins.set(app.id, origin); markSync(origin, { at: Date.parse(app.updatedAt) || Date.now(), gone: false, appId: app.id }); }
  const peers = new Map(), frame = await inference.prismFrame();
  let signals = null, joined = frame.account?.id;
  // A signal the relay refuses is dropped: the peers meet on the next exchange.
  const signal = (m) => signals?.send({ ...m, device: syncDevice })?.catch?.(() => {});
  const hear = async (m) => {
    if (!m || typeof m.from !== "string" || m.from === syncMe || m.device === syncDevice || (m.to && m.to !== syncMe)) return;
    if (m.t === "here") { if (!m.to) signal({ t: "here", from: syncMe, to: m.from }); if (syncMe < m.from && !peers.has(m.from)) connect(m.from, true); }
    else if (m.t === "offer" && !peers.has(m.from)) { const pc = connect(m.from, false); await pc.setRemoteDescription(m.sdp); await pc.setLocalDescription(); await iceDone(pc); signal({ t: "answer", from: syncMe, to: m.from, sdp: pc.localDescription.toJSON() }); }
    else if (m.t === "answer") await peers.get(m.from)?.setRemoteDescription(m.sdp);
  };
  // Do not initiate external relay connections on page boot unless explicitly opted in.
  const syncOptIn = () => { try { return localStorage.getItem("uor.studio.enablePrismSync") === "1"; } catch { return false; } };
  const listen = async () => { if (!syncOptIn()) return; signals?.stop(); signals = await frame.appChannel(hear); signal({ t: "here", from: syncMe }); };
  frame.addEventListener("accountchange", () => { if (frame.account?.id === joined) return; joined = frame.account?.id; for (const pc of peers.values()) pc.close(); peers.clear(); if (syncOptIn()) void listen().catch(() => {}); });
  function connect(id, offering) {
    const pc = new RTCPeerConnection({ iceServers: STUN }), parts = new Map();
    peers.set(id, pc);
    const wire = (channel) => {
      channel.onopen = () => { syncChannels.add(channel); offerSync(channel); };
      channel.onclose = () => syncChannels.delete(channel);
      channel.onmessage = ({ data }) => {
        let m = JSON.parse(data);
        if (m.t === "part") { const got = parts.get(m.id) ?? []; got[m.i] = m.data; parts.set(m.id, got); if (got.filter((x) => x !== undefined).length < m.n) return; parts.delete(m.id); m = JSON.parse(got.join("")); }
        onSync(channel, m).catch(() => {});
      };
    };
    if (offering) { wire(pc.createDataChannel("workspace")); pc.setLocalDescription().then(() => iceDone(pc)).then(() => signal({ t: "offer", from: syncMe, to: id, sdp: pc.localDescription.toJSON() })); }
    else pc.ondatachannel = ({ channel }) => wire(channel);
    const end = () => { pc.close(); peers.delete(id); };
    pc.onconnectionstatechange = () => ["failed", "closed", "disconnected"].includes(pc.connectionState) && end();
    setTimeout(() => pc.connectionState !== "connected" && end(), 20000);
    return pc;
  }
  if (syncOptIn()) await listen();
  setInterval(() => syncChannels.forEach(offerSync), 30000);
})().catch(() => {});
