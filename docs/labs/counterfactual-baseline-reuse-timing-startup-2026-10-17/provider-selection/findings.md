# Driver-aware offer selection — read-only investigation, 17 October 2026

## Finding and actual disposition

Vast exposes `driver_version` in preallocation offer metadata. The current UOR
provider and runner discard it and cannot select on it. An exact580.95.05
selector can prevent paying to discover an already-advertised mismatch. At the
observed query times there were **zero matching offers** under the existing
one-RTX5090 placement constraints. Timing remains BLOCKED; no allowlist expansion,
retry, source implementation, rental or new spending class is authorized here.
Deliver the blocked timing receipt and complete cleanup before commissioning
any successor source work.

## Retained observations and limits

The two selected-field captures and exact JSON predicates are retained alongside
this report. They are transcriptions of existing successful tool results, not
full raw API responses; no request was repeated to retain them. No credentials,
headers, create response, SSH address or unrelated provider fields are retained.
`capture-scope.json` records the exact jq projections and timestamp/null scope.

At2026-10-17T01:24:52Z the placement search returned5offers. Every raw offer row
had `driver_version`. Advertised versions were595.58.03,580.173.02,595.71.05,
615.78.08,615.78.08. Offer45028230/machine8914/host18 advertised595.71.05 at
$0.9083333333333333/h. This is the offer ID in timingrun1's retained creation
receipt, and its current driver metadata matches that attempt's actual595.71.05.
This does not prove the historical response value at creation time: no such
full raw offer response was retained.

At01:25:10Z an otherwise identical search plus
`"driver_version":{"eq":"580.95.05"}` returned0offers. The limit was30; the
unqualified search returned only5. These are point-in-time availability facts,
not a standing promise that a qualifying offer will or will not appear. Both
requests were POST search operations at /api/v0/bundles/, not instance creation.
Projected `datacenter`/`verified` nulls do not establish raw key absence versus
null, nor prove those predicates from response values; both were request filters.

The retained helper's read-only query for driver_version refused locally with
exit1 and `unsupported offer query field or missing value`. The exact command,
output and binary identity appear in `helper-rejection.txt`.

## Source explanation, pinned to repository commit

All source references below bind
c754f160cf140e87fef675698e3939d2244f6c9c; inspected working copies of these two
files were also verified byte-equivalent to origin/main for the investigation.

- [Provider query allowlist, lib.rs:174–198](https://github.com/UOR-Foundation/uor-r4/blob/c754f160cf140e87fef675698e3939d2244f6c9c/crates/uor-r4-pod-provider/src/lib.rs#L174-L198)
  excludes driver_version, so a selector cannot pass the existing Rust parser.
- [Provider safe offer projection, lib.rs:475–488](https://github.com/UOR-Foundation/uor-r4/blob/c754f160cf140e87fef675698e3939d2244f6c9c/crates/uor-r4-pod-provider/src/lib.rs#L475-L488)
  omits driver_version, machine_id and host_id from raw search rows.
- [Runner offer query/projection, uor-pod:344–351](https://github.com/UOR-Foundation/uor-r4/blob/c754f160cf140e87fef675698e3939d2244f6c9c/scripts/pod/uor-pod#L344-L351)
  requests CUDA capability>=12.8 and projects to id/dph/n/loc/rel; exact host-driver
  identity cannot reach placement.
- [Price/candidate selection, uor-pod:948–962](https://github.com/UOR-Foundation/uor-r4/blob/c754f160cf140e87fef675698e3939d2244f6c9c/scripts/pod/uor-pod#L948-L962)
  filters price and selects candidate IDs without a driver predicate.

The [official Vast CLI source](https://github.com/vast-ai/vast-cli/blob/master/vast.py)
also lists driver_version among offer-query/display fields. That external master
reference was consulted during investigation and can change; the captured API
observations above supply the concrete current field evidence. No Python CLI or
Python model code was executed.

## Minimal prospective selector, not implementation

Add a run-scoped exact driver selector, initially580.95.05 for this separately
reviewed timing scope. Whitelist exact driver equality in the Rust provider,
retain the driver string in safe output, and optionally retain machine/host IDs
for receipts. Enforce exact equality and nonmissing driver metadata locally
before price sorting/candidate creation as well as in the server-side query.
Retain the existing reliability/storage/GPU/caps/price admission; no match means
unavailable with zero allocation. Preserve the selected offer/driver/host/quote
and query timestamp before creation. Do not infer driver identity from
cuda_max_good, a former offer ID or an old host label.

Keep the remote580.95.05-only predicate before any binary invocation: advertised
metadata can become stale between search and startup. A mismatch still blocks
and preserves evidence rather than expanding the allowlist. No sequence of
paid driver-discovery retries is proposed.

Meaningful host-only checks: parser maps exact version string to an equality
predicate; projection retains it; missing/wrong values refuse before create;
cheaper mismatches cannot outrank a qualified offer; no qualified offer causes
zero create calls; existing fleet<=4 and total<=$8/h predicates remain active.
These source/tooling changes require their own ownership, protected delivery and
independent review after primary blocked-attempt delivery/cleanup. They do not
change or rebuild the retained CUDA executable and do not release timing.

## Evidence retention

SHA256SUMS covers this findings document and its six sibling evidence files.
Only local evidence files were written for retention. No new API call, peer
mutation, pod/lease operation, model execution or external post occurred.
