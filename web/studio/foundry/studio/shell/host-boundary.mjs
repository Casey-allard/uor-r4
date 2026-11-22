// Typed adapter for FoundrySpecification.hostResponseBoundary. The schema belongs
// to the imported renderer contract; this adapter never invents a successful value.
const marker = "prism-ipc-envelope-v1";
const success = (value) => ({ __prismIpcEnvelope: marker, ok: true, value });
const refusal = (code, channel, kind, message) => ({
  __prismIpcEnvelope: marker,
  ok: false,
  error: { name: "PrismError", kind, code, message: `${message}: ${channel}` },
});

export function refuseCommandBinding(channel, code) {
  if (code === "unsupportedChannel") return refusal(code, channel, "not_found", "This capability is unavailable in the browser");
  const known = ["commandRegistryInvalid", "commandBindingMissing", "commandBindingMismatch"].includes(code) ? code : "commandRegistryInvalid";
  return refusal(known, channel, "internal", "The command differs from its canonical ownership binding");
}

export async function invokeHost(channel, args, handler, schema, inputContract = null) {
  if (typeof handler !== "function") {
    return refusal("unsupportedChannel", channel, "not_found", "This capability is unavailable in the browser");
  }
  if (inputContract !== null) {
    let inputSchema;
    try {
      inputSchema = inputContract.schema;
      if (!inputSchema || typeof inputSchema.safeParse !== "function") {
        return refusal("inputContractUnavailable", channel, "internal", "The browser input contract is unavailable");
      }
    } catch { return refusal("inputValidatorFailed", channel, "internal", "The browser input validator failed"); }
    if (!Array.isArray(args) || args.length > 1) {
      return refusal("invalidInput", channel, "validation", "The command arguments do not match their contract");
    }
    let input;
    try { input = structuredClone(args[0]); }
    catch { return refusal("invalidInput", channel, "validation", "The command cannot be transferred"); }
    try {
      const parsed = inputSchema.safeParse(input);
      if (parsed?.success !== true) {
        return refusal("invalidInput", channel, "validation", "The command arguments do not match their contract");
      }
      if (!Object.hasOwn(parsed, "data")) {
        return refusal("inputValidatorFailed", channel, "internal", "The browser input validator returned no value");
      }
      args = [parsed.data];
    } catch { return refusal("inputValidatorFailed", channel, "internal", "The browser input validator failed"); }
  }
  let value;
  try {
    value = await handler(...args);
  } catch (error) {
    if (['applicationStateReadOnly','applicationShapeConflict','statefulBuildUnsupported'].includes(error?.code)) return refusal(error.code, channel, 'validation', 'Application source authority refused this operation');
    if(["invalidChatModelSelection","unsupportedChatProvider"].includes(error?.code)) return refusal(error.code, channel, "validation", "Chat model routing refused");
    if(error?.code === "invalidChatExecutionMode") return refusal(error.code, channel, "validation", "Invalid chat execution mode");
    if (["invalidChatPreferences","chatPreferencesMissing","chatPreferencesStateInvalid","chatPreferencesStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", "Chat preference operation refused");
    if (["projectScaffoldFailed","projectPreparationFailed","projectStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "internal", "Project creation refused");
    if (["invalidThemeCommand","themeLibraryInvalid","themeMissing","themeAppMissing","themeLibraryLimit","themeStorageFailed","themeCatalogMismatch"].includes(error?.code)) return refusal(error.code, channel, "validation", "Theme library operation refused");
    if (["invalidSettingsCommand","invalidStoredSettings","settingsStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", "Browser settings operation refused");
    if (["invalidPromptCommand","promptLibraryInvalid","promptMissing","promptSlugConflict","promptLibraryLimit","promptLibraryStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", "Prompt library operation refused");
    if (error?.code === "projectPreviewActive") return refusal(error.code, channel, "validation", "Close version preview before changing project source");
    if (["windowCoordinationUnavailable","invalidWindowCoordination","windowCoordinationStorageFailed","windowCoordinationConflict","windowCoordinationLimitExceeded","windowTabOwnershipConflict"].includes(error?.code)) return refusal(error.code, channel, "validation", "Window coordination refused");
    if (["invalidMessageRestore","messageRestoreMissing","messageRestoreBusy","messageRestoreConflict","messageRestoreVersionMissing","messageRestoreStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", error.code==="messageRestoreBusy"?"Stop and clear queued chats before restoring message history":"Message restoration refused");
    if (["invalidStoredPreview","previewUnavailable","previewStorageFailed","previewSourceChanged","previewMissing","previewWindowBusy","previewInterestMissing","previewOperationConflict","previewUnsupportedBranch","projectPreviewActive","invalidPreviewState","invalidPreviewCommand","previewRevisionConflict","previewRevisionExhausted","previewAlreadyOwned","previewInactive","previewWrongOwner","malformedPreviewRequest","previewRequestTooLarge","previewRuntimeFailed","invalidPreviewResponse"].includes(error?.code)) return refusal(error.code, channel, "validation", "Version preview refused");
    if (["invalidWorkingTreeRequest","workingTreeStateInvalid","workingTreeComparisonLimit"].includes(error?.code)) return refusal(error.code, channel, "validation", "Working-tree query refused");
    if (["invalidVersionHistory","versionHistoryMissing","versionHistoryStorageFailed","versionHistoryStateInvalid"].includes(error?.code)) return refusal(error.code, channel, "validation", "Version history command refused");
    if (["invalidChatQueueCommand", "chatQueueUnavailable", "chatQueueJournalInvalid", "chatQueueConflict", "chatQueueMissing", "chatQueueStorageFailed", "chatQueueLimitExceeded", "chatQueueIntentConflict", "invalidQueueState", "invalidQueueCommand", "queueRevisionConflict", "queueFull", "queueItemMissing", "queueItemDuplicate", "queueActiveConflict", "queuePaused", "queueStopConflict", "queueOwnerConflict", "queueRevisionExhausted", "malformedQueueRequest", "queueRequestTooLarge", "queueRuntimeFailed", "invalidQueueResponse"].includes(error?.code)) return refusal(error.code, channel, "validation", "Chat queue was refused");
    if (["invalidDiscoveryIntent", "discoveryRefused", "discoveryReviewChanged", "discoveryUnavailable"].includes(error?.code)) return refusal(error.code, channel, "validation", "Application discovery was refused");
    if (["invalidState", "wrongOwner", "staleRevision", "invalidPhase", "invalidAmount", "budgetExceeded", "reservationMissing", "reservationExceeded", "obligationExceeded", "checkpointNotAdvanced", "unfinishedWork", "invalidOwner", "invalidWorkRequest", "workCounterExhausted", "workRuntimeFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", "The work guest refused this operation");
    if (["invalidStoredProviderCredential", "unsupportedCredentialProvider", "providerCredentialArtifact"].includes(error?.code)) return refusal(error.code, channel, "validation", "Provider credential access was refused");
    if (["invalidBuildWork", "buildWorkBusy", "buildWorkUnavailable", "buildWorkConflict", "buildWorkSourceChanged", "buildWorkArtifactInvalid", "buildWorkLimitExceeded"].includes(error?.code)) return refusal(error.code, channel, "validation", "Durable build work was refused");
    if (["invalidPublicationPermission", "publicationPermissionMissing", "publicationPermissionDenied", "publicationPermissionExpired", "publicationPermissionOffline", "publicationPermissionUnavailable", "publicationSourceChanged"].includes(error?.code)) return refusal(error.code, channel, "validation", "Public publication permission was refused");
    if (["portableBuildStyleIncomplete", "invalidPortableBuildPermission", "portableBuildPermissionDenied", "portableBuildPermissionExpired", "portableBuildPermissionRevoked", "portableBuildPermissionOffline", "portableBuildPermissionUnavailable"].includes(error?.code)) return refusal(error.code, channel, "validation", "Portable build permission was refused");
    if (error?.code === "unsupportedChannel") return refusal("unsupportedChannel", channel, "not_found", "This capability is unavailable in the browser");
    if (["commandRegistryInvalid", "commandBindingMissing", "commandBindingMismatch"].includes(error?.code)) return refusal(error.code, channel, "internal", "The command differs from its canonical ownership binding");
    if (["invalidInteractionModel", "invalidInteractionState", "unknownInteractionEvent", "malformedInteraction", "interactionTooLarge", "interactionRuntimeFailed", "invalidInteractionResponse", "invalidInteractionApplication", "interactionApplicationMismatch", "interactionStateChanged", "interactionStateStale", "interactionAudioUnavailable"].includes(error?.code)) return refusal(error.code, channel, "validation", "Interaction application was refused");
    if (["invalidEventDelivery", "eventDeliveryUnconfigured", "eventDeliveryConflict", "eventDeliverySourceChanged", "eventDeliveryPending", "eventDeliveryLimitExceeded", "eventDeliveryAcknowledgementRefused"].includes(error?.code)) return refusal(error.code, channel, "validation", "Event delivery was refused");
    if (["previewPermissionChanged", "invalidPreviewPermission", "previewPermissionDenied", "previewPermissionExpired", "previewPermissionOffline", "previewPermissionUnavailable", "previewPermissionStorageFailed"].includes(error?.code)) return refusal(error.code, channel, "validation", "Preview permission was not granted or could not be changed");
    if (error?.code === "canonicalBuildMismatch") return refusal("canonicalBuildMismatch", channel, "validation", "Build source does not match its canonical model projections");
    if (error?.code === "canonicalGenerationRefused") return refusal("canonicalGenerationRefused", channel, "validation", "Use canonical authoring commands to change model-owned files");
    if (error?.code === "projectEditConflict") return refusal("projectEditConflict", channel, "validation", "The project changed during the edit");
    if (error?.code === "invalidCanonicalEdit") return refusal("invalidCanonicalEdit", channel, "validation", "The canonical model edit is invalid or unsupported");
    if (error?.code === "canonicalProjectionReadOnly") return refusal("canonicalProjectionReadOnly", channel, "validation", "Edit the canonical model to change its generated rule runtime");
    if (error?.code === "browserCompilerUnavailable") return refusal("browserCompilerUnavailable", channel, "internal", "The browser compiler could not initialize");
    if (error?.code === "browserTelemetryUnavailable") return refusal("browserTelemetryUnavailable", channel, "validation", "Vendor telemetry is unavailable in this browser");
    return refusal("handlerFailed", channel, "internal", "The browser operation failed");
  }
  // Legacy channels without imported output contracts retain their handler result.
  // Their existence is not evidence of schema coverage or capability completeness.
  if (schema == null) return success(value);
  try {
    if (schema.safeParse(value)?.success !== true) {
      return refusal("invalidResponse", channel, "internal", "The browser response did not match its contract");
    }
  } catch {
    return refusal("validatorFailed", channel, "internal", "The browser response validator failed");
  }
  return success(value);
}

// Both IPC entrypoints preserve the same typed failure. The renderer's generated
// client already decodes the envelope; older direct callers use this adapter.
export function unwrapHost(envelope) {
  if (envelope.ok) return envelope.value;
  throw Object.assign(new Error(envelope.error.message), envelope.error);
}
