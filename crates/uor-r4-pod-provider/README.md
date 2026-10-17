# Rust Vast adapter for the admitted pod runner

`uor-r4-pod-provider` implements only the Vast command shapes consumed by
`scripts/pod/uor-pod`: `show instances --raw`, `show instance ID --raw`,
`search offers QUERY --storage GB -o FIELD --raw`, `create instance OFFER`
with image/disk/SSH/direct/label/env/onstart flags, and `destroy instance ID`.
An optional leading `vast` is accepted. Output is JSON; unsupported flags refuse.

This is Rust tooling with curl providing TLS transport. It executes no Python.
Authentication uses `VAST_API_KEY`, then `$XDG_CONFIG_HOME/vastai/vast_api_key`
(default `~/.config/vastai/vast_api_key`), then legacy `~/.vast_api_key`.
Credential contents and create bodies are sent to curl on stdin, never argv.
Response output projects only the fields needed by the admitted runner and
removes credential/environment payloads. The API origin is pinned to
`https://console.vast.ai`; overrides and redirects are not accepted.

Instance inventory follows the installed Vast CLI's v1 pagination, returning
every page or an error. Missing, malformed, negative or nonfinite instance
hourly prices and invalid GPU counts refuse the entire inventory, preventing
the runner's defaults from reducing admission counts or price totals. Unknown
statuses remain conservatively RUNNING in the runner. Offer queries preserve its current default filters,
`cuda_vers` alias, order, allocated storage, and optional limit. Create sends
exactly one mutation with no automatic retry. Transport/HTTP/malformed-response
failures require the runner's existing label reconciliation before another
create; successful create output retains only success and contract identity.
Destroy sends one request without an interactive prompt; the caller owns its
authorization and subsequent disappearance check.

The admitted runner still owns spending caps, resource admission, lease/job
ownership, duplicate-create reconciliation, bootstrap, cleanup and training
holds. This adapter grants no permission to rent or train. Tests use request
and response fixtures only, with no network, credentials or live mutation.

Port provenance: installed Vast `api/{client,instances,offers,query}.py`,
`cli/commands/{instances,offers}.py` and `utils.py` were inspected read-only.
No Python command or secret file was executed or opened for the port.

Local qualification: Rust 1.97.1 built the adapter successfully. Five fixture
tests passed: query aliases/order/storage/limits; create flag preservation and
response secret filtering; complete pagination and partial-inventory refusal;
single-instance/destroy REST mapping and malformed replies; stdin config and
argument refusal. Test compilation took 3.95 s, fixture execution 0.00 s, and
the normal binary build 1.12 s. These are tooling measurements; no API request,
pod rental, model computation or training occurred during this qualification.

Admission-field repair: all six fixture tests passed, including a malformed
second page that must refuse previously collected valid rows, and malformed
single-instance cost/count replies. This final test compilation took 1.18 s,
fixture execution 0.00 s, and the rebuilt normal binary 1.04 s. The five-test
receipt above is the initial implementation receipt, superseded by these six
passing controls. No live request or rental was made by the implementation agent.

```sh
CARGO_TARGET_DIR=local/pod-provider-target /Users/casey.allard/.cargo/bin/cargo test -p uor-r4-pod-provider --offline
CARGO_TARGET_DIR=local/pod-provider-target /Users/casey.allard/.cargo/bin/cargo build -p uor-r4-pod-provider --offline
./local/pod-provider-target/debug/uor-r4-pod-provider --version
```

The explicit Cargo path above avoids the unrelated `cargo-ai` executable that
currently shadows Rust Cargo on this host's default PATH.

## Optional exact-driver placement

`search offers` accepts `driver_version=MAJOR.MINOR.PATCH` (or `==`) as an exact
string predicate. Three nonempty decimal components are required; zero padding
is preserved. Ordering, ranges, wildcards and lists are refused for this field.
Safe offer output additionally retains `driver_version`, `machine_id` and
`host_id` when present. Missing metadata is not synthesized.

For one `uor-pod up` invocation, `UOR_POD_VAST_DRIVER_VERSION` optionally supplies
that exact predicate. There is no default project-wide driver requirement.
The runner also filters missing/nonstring/mismatched driver metadata locally
before price sorting, then rechecks at create. No eligible offer means no
allocation. An active selector cannot select Runpod or automatically fall back
to it under `--wait`; unset behavior, placement constraints and caps remain.
`--wait` may still wait for a qualifying Vast offer until its declared deadline.

Before each selected create attempt the existing ledger records
`vast-offer-selected` with offer ID, requested/advertised driver, machine/host
IDs (null if the provider omitted them), raw quote, rounded admission price,
query-start and selection UTC timestamps. The successful `up` record and pod
cache retain that same selection object. These are provider metadata receipts,
not a guarantee that the host cannot change after search. Keep the remote
actual-driver check before model work. This option releases no rental, timing,
training or model compute and does not alter any experiment's allowlist.

Tests use the existing Rust request/response fixtures and the runner's fake
provider account; no real credentials, network or provider mutation is needed.
