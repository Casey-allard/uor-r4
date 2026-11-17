# ORDER for the permanent branch policy scope

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). References #2904. Base: `11c2901d90af122c53069aff62191bc4f147ff9a`.

## Goal and cancel probe

Casey chose "main, gh-pages, and a new archive branch; retire the preview and update its publishing workflow". The subsequent policy change must update all active writers and cleanup guards, including the two publishing entry points. At this base, the maintenance item covers the charter and rule log but omits the ten file paths below. Cancel if all ten are already present or covered.

## Scope

Append only the ten missing exact files to the scope of `docs/roadmap/items/infra-loop-harness.md`. The policy work covers the docs and cleanup workflows; Git, delivery and status protection in the harness; the protected-branch registry; the workspace manual; the Studio publishing helper, Studio and workflow regressions under the existing Rust test scope and Studio manual. This registration and its ORDER/REPORT are the only changes in this unit.

## Done when

Compare the parsed old and new scope and verify the old prefix and all other item text byte for byte. Confirm the only additions are the ten paths, rerun the roadmap check, and apply the repo-local PR policy. The subsequent implementation must repeat the scope check from a main tip containing this registration.

## Stop and decide if

Refer a new file, changed branch policy or research milestone decision back to the top session. Remote branch creation and retirement are separate owner-directed work.
