// Generated from CommandRegistry.commandRegistryJson. Do not edit.
export const commandRegistryData = {
  "profile": "foundry/command-registry/1",
  "features": [
    {
      "id": "actor",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "effects": [
        "storage",
        "network",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "application-discovery",
      "version": 1,
      "owner": "FoundrySpecification.discoveryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Finite vocabulary and explicit profile review; independent quality and release obligations remain."
    },
    {
      "id": "chat-preferences",
      "version": 1,
      "owner": "FoundrySpecification.chatPreferencesBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Atomic imported preferences; mode execution, inference permission and whole-product quality remain separately qualified."
    },
    {
      "id": "chat-queue",
      "version": 1,
      "owner": "ChatQueue.boundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Finite queue transitions; durable payload and actor integration are separately qualified."
    },
    {
      "id": "composition",
      "version": 1,
      "owner": "FoundrySpecification.compositionHostBoundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "creation",
      "version": 1,
      "owner": "FoundrySpecification.projectCreationBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "durable-chat",
      "version": 1,
      "owner": "FoundrySpecification.durableChatQueueBoundary",
      "effects": [
        "storage",
        "network",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Queue admission grants no effect permission; inference, build and publication boundaries remain mandatory."
    },
    {
      "id": "durable-version-preview",
      "version": 1,
      "owner": "FoundrySpecification.durableVersionPreviewBoundary",
      "effects": [
        "storage",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Origin window ownership and guest-admitted atomic source preview; imported version actor transport and independent recovery journeys apply."
    },
    {
      "id": "editing",
      "version": 1,
      "owner": "FoundrySpecification.canonicalEditBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "event-delivery",
      "version": 1,
      "owner": "FoundrySpecification.eventDeliveryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "events",
      "version": 1,
      "owner": "EventProtocol.boundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "effects": [
        "storage",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "In-page Home chat before project creation; independent studio promotion and execution boundaries apply."
    },
    {
      "id": "import",
      "version": 1,
      "owner": "FoundrySpecification.projectImportBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "interaction",
      "version": 1,
      "owner": "InteractionProtocol.interactionWireBoundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "interaction-journeys",
      "version": 1,
      "owner": "FoundrySpecification.interactionJourneyBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "lifecycle",
      "version": 1,
      "owner": "FoundrySpecification.projectLifecycleBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "local-write",
      "version": 1,
      "owner": "FoundrySpecification.storageCommitBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "message-restoration",
      "version": 1,
      "owner": "FoundrySpecification.messageRestorationBoundary",
      "effects": [
        "storage",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Verified source and message restoration share atomic acknowledgement; busy queues and changed history refuse."
    },
    {
      "id": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "effects": [
        "storage",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "UOR-R4 RV64IMAC MVM Micro-Linux WebAssembly terminal and OPFS CoW snapshots."
    },
    {
      "id": "privacy",
      "version": 1,
      "owner": "FoundrySpecification.browserTelemetryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "prompt-library",
      "version": 1,
      "owner": "FoundrySpecification.promptLibraryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Bounded transactional prompt library with exact imported contracts and rollback acceptance"
    },
    {
      "id": "publication",
      "version": 1,
      "owner": "FoundrySpecification.publicationIdentityBoundary",
      "effects": [
        "storage",
        "network",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "representation",
      "version": 1,
      "owner": "FoundrySpecification.representationBoundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "rules",
      "version": 1,
      "owner": "FoundrySpecification.rulesSourceBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "semantic-routing",
      "version": 1,
      "owner": "SemanticRouter.boundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent finite relevance and latency qualification; no permission authority."
    },
    {
      "id": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Atomic durable theme selection and exact generation disclosure; theme text grants no effects."
    },
    {
      "id": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "effects": [
        "storage",
        "network",
        "execution"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "In-browser UOR-R4 integer WASM model catalog, OPFS verification, and local upload."
    },
    {
      "id": "version-history",
      "version": 1,
      "owner": "FoundrySpecification.versionHistoryBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Snapshot and file identities require core admission; metadata does not change canonical source identity."
    },
    {
      "id": "version-preview-model",
      "version": 1,
      "owner": "VersionPreview.boundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Guest ownership admission does not grant source mutation permission or authenticate an owner."
    },
    {
      "id": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Committed bounded origin window ownership, interest relay and query invalidation recovery; browser platform locks are cooperative, not authentication."
    },
    {
      "id": "work",
      "version": 1,
      "owner": "WorkProtocol.boundary",
      "effects": [],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Independent journey and release obligations apply; this inventory grants no effects."
    },
    {
      "id": "working-tree",
      "version": 1,
      "owner": "FoundrySpecification.workingTreeBoundary",
      "effects": [
        "storage"
      ],
      "targets": [
        "browser"
      ],
      "assurance": "boundary-inventory",
      "quality": "Committed bounded origin window ownership, interest relay and query invalidation recovery; browser platform locks are cooperative, not authentication."
    }
  ],
  "commands": [
    {
      "channel": "add-log",
      "feature": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "add-to-favorite",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent-tool:get-tools",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent-tool:set-consent",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:cancel-subagent",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:fix-review-findings",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:followup-subagent",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:get-subagent-activities",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:get-subagent-messages",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:list-subagents",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:run-auto-review-barrier",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:send-subagent-message",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:skip-review-auto-fix",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "agent:start-review",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "analyze-component",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app-blueprint:add-visual",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app-blueprint:approve",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app-blueprint:edit-field",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app-blueprint:edit-visual",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app-blueprint:remove-visual",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app:get-current-commit-hash",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app:list-screenshots",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app:list-thumbnails",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "app:save-screenshot",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "appCollections:assignApps",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "appCollections:create",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "appCollections:delete",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "appCollections:list",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "appCollections:update",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "apply-app-template",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "apply-visual-editing-changes",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "approve-proposal",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "cancel-managed-node-install",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "change-app-location",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "chat:add-dep",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "chat:cancel",
      "feature": "durable-chat",
      "version": 1,
      "owner": "FoundrySpecification.durableChatQueueBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "chat:count-tokens",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "chat:observe-submission-stop-policy",
      "feature": "durable-chat",
      "version": 1,
      "owner": "FoundrySpecification.durableChatQueueBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "chat:response:ack",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "check-ai-rules",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "check-app-name",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "check-problems",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "checkout-version",
      "feature": "durable-version-preview",
      "version": 1,
      "owner": "FoundrySpecification.durableVersionPreviewBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "cleanup-theme-images",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "clear-logs",
      "feature": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "clear-session-data",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "connection-flow:acknowledge",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "connection-flow:cancel",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "connection-flow:get-states",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "connection-flow:start",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:accept-insecure-token",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:cancel",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:dismiss",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:get-server-key",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:inspect",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:reveal-credentials",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:run",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify-setup:snapshot",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:check-domain",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:clear-token",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:create-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:deploy",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:disconnect",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:discover",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:get-deploy-snapshot",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:get-status",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:save-connection",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "coolify:save-token",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "copy-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-app",
      "feature": "creation",
      "version": 1,
      "owner": "FoundrySpecification.projectCreationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-chat",
      "feature": "creation",
      "version": 1,
      "owner": "FoundrySpecification.projectCreationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-cloud-sandbox-share-link",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-custom-language-model",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-custom-language-model-provider",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "create-custom-theme",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-app",
      "feature": "lifecycle",
      "version": 1,
      "owner": "FoundrySpecification.projectLifecycleBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-apps",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-chat",
      "feature": "lifecycle",
      "version": 1,
      "owner": "FoundrySpecification.projectLifecycleBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-custom-language-model",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-custom-language-model-provider",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-custom-model",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-custom-theme",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-media-file",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "delete-messages",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "distributed-machine:dispatch",
      "feature": "durable-chat",
      "version": 1,
      "owner": "FoundrySpecification.durableChatQueueBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "distributed-machine:subscribe",
      "feature": "durable-chat",
      "version": 1,
      "owner": "FoundrySpecification.durableChatQueueBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "distributed-machine:unsubscribe",
      "feature": "actor",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "does-release-note-exist",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "edit-app-file",
      "feature": "editing",
      "version": 1,
      "owner": "FoundrySpecification.canonicalEditBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "edit-custom-language-model-provider",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "execute-app-upgrade",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "first-prompt:cancel-creation",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "product-boundary"
    },
    {
      "channel": "first-prompt:commit-creation",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "product-boundary"
    },
    {
      "channel": "forge:compose",
      "feature": "composition",
      "version": 1,
      "owner": "FoundrySpecification.compositionHostBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:discover",
      "feature": "application-discovery",
      "version": 1,
      "owner": "FoundrySpecification.discoveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:discovery-create",
      "feature": "application-discovery",
      "version": 1,
      "owner": "FoundrySpecification.discoveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:event",
      "feature": "events",
      "version": 1,
      "owner": "EventProtocol.boundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:event-acknowledge",
      "feature": "event-delivery",
      "version": 1,
      "owner": "FoundrySpecification.eventDeliveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:event-configure",
      "feature": "event-delivery",
      "version": 1,
      "owner": "FoundrySpecification.eventDeliveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:event-deliver",
      "feature": "event-delivery",
      "version": 1,
      "owner": "FoundrySpecification.eventDeliveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:event-pending",
      "feature": "event-delivery",
      "version": 1,
      "owner": "FoundrySpecification.eventDeliveryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:import",
      "feature": "import",
      "version": 1,
      "owner": "FoundrySpecification.projectImportBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:interact",
      "feature": "interaction",
      "version": 1,
      "owner": "InteractionProtocol.interactionWireBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:interaction-create",
      "feature": "interaction-journeys",
      "version": 1,
      "owner": "FoundrySpecification.interactionJourneyBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:interaction-run",
      "feature": "interaction-journeys",
      "version": 1,
      "owner": "FoundrySpecification.interactionJourneyBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:pack-bytes",
      "feature": "representation",
      "version": 1,
      "owner": "FoundrySpecification.representationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:queue",
      "feature": "chat-queue",
      "version": 1,
      "owner": "ChatQueue.boundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:route",
      "feature": "semantic-routing",
      "version": 1,
      "owner": "SemanticRouter.boundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:unpack-bytes",
      "feature": "representation",
      "version": 1,
      "owner": "FoundrySpecification.representationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:version-preview",
      "feature": "version-preview-model",
      "version": 1,
      "owner": "VersionPreview.boundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "forge:work",
      "feature": "work",
      "version": 1,
      "owner": "WorkProtocol.boundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "free-agent-quota:get-status",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "free-model-quota:get-status",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "generate-theme-from-url",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "generate-theme-prompt",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-app",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-app-env-vars",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-app-theme",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-app-upgrades",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-app-version",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-chat",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-chat-metadata",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-chats",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-cloud-sandbox-status",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-context-paths",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-current-branch",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-custom-apps-folder",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-custom-themes",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-env-vars",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-initial-load-telemetry-context",
      "feature": "privacy",
      "version": 1,
      "owner": "FoundrySpecification.browserTelemetryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-language-model-providers",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-language-models",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-language-models-by-providers",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-latest-security-review",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-node-path",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-or-create-security-fix-chat",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-proposal",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-session-debug-bundle",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-subscription-status",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-system-debug-info",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-system-platform",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-templates",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-theme-generation-model-options",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-themes",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-user-budget",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-user-settings",
      "feature": "privacy",
      "version": 1,
      "owner": "FoundrySpecification.browserTelemetryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "get-version-changes",
      "feature": "version-history",
      "version": 1,
      "owner": "FoundrySpecification.versionHistoryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "git:cancel-commit",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "git:commit-changes",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "git:discard-changes",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "git:get-uncommitted-file-diff",
      "feature": "working-tree",
      "version": 1,
      "owner": "FoundrySpecification.workingTreeBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "git:get-uncommitted-files",
      "feature": "working-tree",
      "version": 1,
      "owner": "FoundrySpecification.workingTreeBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:clone-repo-from-url",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:get-repo-branches",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:invite-collaborator",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:is-repo-available",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:list-collaborators",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:list-local-branches",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:list-remote-branches",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:list-repos",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "github:remove-collaborator",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "help:chat:cancel",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "help:chat:start",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "holo:list",
      "feature": "publication",
      "version": 1,
      "owner": "FoundrySpecification.publicationIdentityBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "holo:publish",
      "feature": "publication",
      "version": 1,
      "owner": "FoundrySpecification.publicationIdentityBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "holo:view",
      "feature": "publication",
      "version": 1,
      "owner": "FoundrySpecification.publicationIdentityBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "home-chat:cancel",
      "feature": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "home-chat:clear",
      "feature": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "home-chat:history",
      "feature": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "home-chat:promote-to-studio",
      "feature": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "home-chat:send",
      "feature": "home-chat",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "image-generation:wait-for-operation",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "import-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "install-managed-node",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "install-pnpm",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "is-capacitor",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "list-all-media",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "list-apps",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "list-versions",
      "feature": "version-history",
      "version": 1,
      "owner": "FoundrySpecification.versionHistoryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "local-models:list-lmstudio",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "local-models:list-ollama",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:add-from-catalog",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:create-server",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:delete-server",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:disconnect-oauth",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:get-tool-consents",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:is-oauth-storage-encrypted",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:list-catalog",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:list-servers",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:list-tools",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:probe-callback-port",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:probe-connection",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:set-tool-consent",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:start-oauth",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mcp:update-server",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "migration:migrate",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "migration:preview",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "move-media-file",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "mvm:exec",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "mvm:identity",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "native-theme:get-state",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:create-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:fake-connect",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:get-branch-env-vars",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:get-email-password-config",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:get-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:list-projects",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:set-active-branch",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:set-app-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:set-selected-database-branch-type",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:unset-app-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "neon:update-email-verification",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "nodejs-status",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-android",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-billing-action",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-external-url",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-file-path",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-ios",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "open-media-file",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "portal:migrate-create",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-app-folder-name",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-view:go-back",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-view:go-forward",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-view:hide",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-view:reload",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "preview-view:set-bounds",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "product-boundary"
    },
    {
      "channel": "preview-view:set-overlay-active",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "product-boundary"
    },
    {
      "channel": "preview-view:show",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "prism:status",
      "feature": "rules",
      "version": 1,
      "owner": "FoundrySpecification.rulesSourceBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "pro:transcribe-audio",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "prompts:create",
      "feature": "prompt-library",
      "version": 1,
      "owner": "FoundrySpecification.promptLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "prompts:delete",
      "feature": "prompt-library",
      "version": 1,
      "owner": "FoundrySpecification.promptLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "prompts:list",
      "feature": "prompt-library",
      "version": 1,
      "owner": "FoundrySpecification.promptLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "prompts:update",
      "feature": "prompt-library",
      "version": 1,
      "owner": "FoundrySpecification.promptLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "read-app-file",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "recording:discard-draft",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "recording:save-draft",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "recording:start",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "recording:stop",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "reject-proposal",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "reload-env-path",
      "feature": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "remove-chat-referenced-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "remove-managed-node",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "rename-app",
      "feature": "lifecycle",
      "version": 1,
      "owner": "FoundrySpecification.projectLifecycleBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "rename-branch",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "rename-media-file",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "renderer:error-toast-ready",
      "feature": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "reset-all",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "respond-to-app-input",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "restart-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "restart-prism",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "revert-version",
      "feature": "message-restoration",
      "version": 1,
      "owner": "FoundrySpecification.messageRestorationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "run-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "save-theme-image",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "search-app",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "search-app-files",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "search-chats",
      "feature": "local-read",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "select-app-folder",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "select-app-for-preview",
      "feature": "local-acknowledgement",
      "version": 1,
      "owner": "FoundrySpecification.browserReadBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "select-app-location",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "select-custom-apps-folder",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "select-node-folder",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-app-env-vars",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-app-theme",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-chat-favorite",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-context-paths",
      "feature": "local-write",
      "version": 1,
      "owner": "FoundrySpecification.storageCommitBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-custom-apps-folder",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-testing-enabled",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-user-settings",
      "feature": "privacy",
      "version": 1,
      "owner": "FoundrySpecification.browserTelemetryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-version-favorite",
      "feature": "version-history",
      "version": 1,
      "owner": "FoundrySpecification.versionHistoryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "set-version-note",
      "feature": "version-history",
      "version": 1,
      "owner": "FoundrySpecification.versionHistoryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "show-item-in-folder",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "standards:lookup",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "stop-app",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:create-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:delete-organization",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:detect-legacy-app-key",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:fake-connect-and-set-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:get-edge-logs",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:list-all-projects",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:list-branches",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:list-organizations",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:redeploy-all-functions",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:set-app-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:switch-app-to-publishable-key",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "supabase:unset-app-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "sync-capacitor",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "take-screenshot",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:close",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:kill",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:open",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:resize",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:serialize",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "terminal:write",
      "feature": "mvm-terminal",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:apply-assertions",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:delete",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:detect-legacy",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:discard-assertions",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:list",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:migrate-legacy",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:run",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:screenshot",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "tests:stop",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "uor-r4:load-local-file",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "uor-r4:predict-discrete",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "uor-r4:select-model",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "uor-r4:status",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "uor-r4:test-stitch-recover",
      "feature": "uor-model",
      "version": 1,
      "owner": "FoundrySpecification.currentMainIntegrationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "product-boundary",
      "output": "product-boundary"
    },
    {
      "channel": "update-app-commands",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "update-chat",
      "feature": "chat-preferences",
      "version": 1,
      "owner": "FoundrySpecification.chatPreferencesBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "update-custom-language-model",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "update-custom-theme",
      "feature": "theme-library",
      "version": 1,
      "owner": "FoundrySpecification.themeLibraryBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "upload-to-signed-url",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "user-input:get-pending",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "user-input:reject-follow-up",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "user-input:respond",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "validate-provider-api-key",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:connect-existing-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:create-project",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:disconnect",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:get-deployments",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:get-sync-preview",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:is-project-available",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:list-projects",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:remove-neon-env-vars",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:save-token",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "vercel:sync-neon-config",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "version-preview:acquire-window-interest",
      "feature": "durable-version-preview",
      "version": 1,
      "owner": "FoundrySpecification.durableVersionPreviewBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "version-preview:release-window-interest",
      "feature": "durable-version-preview",
      "version": 1,
      "owner": "FoundrySpecification.durableVersionPreviewBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "version-preview:restore-window-interest",
      "feature": "durable-version-preview",
      "version": 1,
      "owner": "FoundrySpecification.durableVersionPreviewBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:acknowledge-chat-tab-transfer",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:adopt-chat-tab-transfer",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:attach-interest",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:begin-chat-tab-transfer",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:bootstrap",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:confirm-source-chat-tab-removal",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:detach-interest",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:focus-chat",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:open-entity-in-new-window",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:reject-chat-tab-transfer",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:set-chat-tab-ownership",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:set-focused",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window-infrastructure:set-visible-entities",
      "feature": "window-coordination",
      "version": 1,
      "owner": "FoundrySpecification.windowCoordinationBoundary",
      "adapter": "studio/shell/host.js",
      "availability": "adapter",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window:close",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window:focus",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window:maximize",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    },
    {
      "channel": "window:minimize",
      "feature": "unavailable",
      "version": 1,
      "owner": "FoundrySpecification.capabilityAvailabilityBoundary",
      "adapter": "studio/shell/host-boundary.mjs",
      "availability": "unavailable",
      "input": "imported-renderer",
      "output": "imported-renderer"
    }
  ]
};
