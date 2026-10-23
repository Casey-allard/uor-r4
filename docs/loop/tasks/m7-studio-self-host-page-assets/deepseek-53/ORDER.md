# ORDER: m7-studio-self-host-page-assets, deepseek-53 (runtime page-load trap)

Lab: deepseek. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md) (#3075). Base: `22720f207`. `References #3075`.

## Goal

Close the runtime half of the item's page-load claim: the done-when asks for zero external requests on load checked by an extended trap verification, and the earlier static scan cannot see requests a script computes at load time. Add a runtime trap that loads the Studio pages in a real browser, records every network request, self-tests with a planted external request, and proves the current result (index.html clean; demo page measured).

## Cancel probe (run first)

Class instrument/build (host-only, no GPU). At the base: `grep -n 'verify_studio_page_load' web/ docs/` finds nothing, and `web/studio/verify_studio_page.mjs` is documented as a static scan only. Task stands.

## Files in scope

- `web/**` (the new harness, README), `docs/products/**`, `docs/labs/m7-studio-self-host-page-assets/**`, this task folder.

## Done when

- `web/studio/verify_studio_page_load.mjs` drives headless Chrome over CDP with no npm dependency (built-in http static server + Node's global WebSocket), records `Network.requestWillBeSent` per page, proves the trap fires with a planted external request, and fails closed when Chrome is unavailable.
- It reports index.html as loaded with zero external requests, reports the demo page's external loads as the legacy residual, and `--strict-router` exits 1 on them.
- Docs record the measured numbers and the Chrome dependency; `loop report --check`, `loop pr-check --local`, `uor-harness check` pass; a counted non-author record lands it.
- No npm dependency, no Rust, no Pages change, no bar; the demo-page decision stays with the PM/owner.