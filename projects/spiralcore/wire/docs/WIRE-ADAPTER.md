# Wire adapter (paired compression endpoints)

A small adapter that sits **in front of arbitrary TCP traffic on both ends**, like a pair of routing
endpoints: the edge adapter accepts plaintext locally, the gateway adapter delivers plaintext to the
local service, and the segment between the two adapters is compressed. It is transport-agnostic and
container-friendly; it does not touch the UOR protocol or its frames.

```
local app --plain--> edge adapter ==compressed frames==> gateway adapter --plain--> local service
```

## Usage

    # gateway side (next to the service; the service still speaks plaintext on 7049):
    python3 tools/wire_adapter.py gateway --listen 0.0.0.0:7042 --target 127.0.0.1:7049

    # edge side (clients connect here as if to the service):
    python3 tools/wire_adapter.py client --listen 127.0.0.1:7041 --remote gateway-host:7042

    # any client now uses edge:7041, e.g.
    objpeer pull --store stores/c1 --addr 127.0.0.1:7041 --select-sha256 SHA --out artifact.bin

Every accepted connection is relayed with two independent zlib streams (one per direction): a magic
handshake, then `u32 length + chunk` frames, with a zero-length frame as end-of-stream. Chunks are
flushed with Z_SYNC_FLUSH for latency; `--stats-file` writes connection/byte counters and the
wire/plain ratio (SIGTERM writes it too).

## Container image

`tools/Containerfile.wire_adapter` builds a `python:3.13-alpine` image:

    docker build -f tools/Containerfile.wire_adapter -t uor-wire-adapter .
    docker run --rm --network host uor-wire-adapter gateway --listen 0.0.0.0:7042 --target 127.0.0.1:7049

The host used here has no container runtime (docker/podman absent; nested Vast blocks OCI), so the
recipe is provided and locally tested outside the container; any host with a runtime can run it.

## Measurements (local, tools/wire_adapter_smoke.sh)

| Traffic | Plain bytes | Wire ratio |
|---|---|---|
| Repetitive JSON-like payload (~1.34 MiB one-way; ~2.69 MiB round-trip) | 1.34 MiB | 0.0043 (232x smaller) |
| Random payload (96 KiB) | 96 KiB | **1.0001** (raw fallback, framing only) |
| Cold objpeer pull, 2 MiB compressible artifact | cold | **0.6797** (~32% smaller end to end) |

The objpeer number is the realistic one: the protocol carries random content ids that cannot compress,
so payload compression saves about a third even on very compressible data; random/incompressible
traffic stays within 0.1% of plain size.

## Limits

- Point-to-point pair with hub routing and per-route compression policy (no multi-hop mesh or global routing yet); transport security is external (SSH measured over WAN; WireGuard/stunnel recipes).
- No authentication or encryption between adapters; run TLS/WireGuard underneath or bind the gateway
  to a private network or SSH tunnel.
- Small or incompressible messages pay frame/flush overhead; `--level 0` is the escape hatch when a
  deployment needs exact byte parity.
- CPU is spent on zlib; `--level` trades ratio for speed (default 6).

## Evidence

`evidence/artifacts/wire-adapter/run.log` and `summary.txt`; reproduce with
`OBJPEER=... bash tools/wire_adapter_smoke.sh`.


## WAN validation (pod gateway, laptop edge)

An 8 MiB letter-like artifact (393,210 store records) pulled from a Vast pod to the laptop over the
public internet:

| Path | Wall | Wire |
|---|---|---|
| Direct | 37.93 s | 22,938,942 B |
| Through the adapter pair | **32.08 s** | **20,737,367 B (ratio 0.904)** |

End-to-end payload identical. The saving is modest here because the UOR protocol carries random content
ids (about a third of the bytes); on text/JSON traffic the same adapter compresses far more (0.0043).
Objective: put a compression stage on any network path without touching the protocol or the application.


## Hub / routing mode

A gateway can front several services with a named route table:

    python3 tools/wire_adapter.py gateway --listen 0.0.0.0:7042 \
        --route model=127.0.0.1:7049 --route store=127.0.0.1:7050
    python3 tools/wire_adapter.py client --listen 127.0.0.1:7041 --remote gateway-host:7042 --route model

The client names a route in the handshake (magic + u8 length + name); the gateway resolves it to a local
target (falling back to --target when no name is given, or rejecting unknown names when there is no
default). `--stats-file` now carries a `routes` object with per-route connections/bytes/ratio.

Smoke test (tools/wire_adapter_hub_smoke.sh): one hub with `model` and `store` routes to two different
objpeer stores; the model client pulls A exactly, the store client pulls B exactly, a wrong-route pull
fails closed, and the hub reports `model` 2 connections / `store` 1 with ratios 0.57/0.55.

## TLS finding (2026-10-17)

In-process TLS on the tunnel was prototyped and root-caused, but not shipped: Python SSLSocket is not
safe for concurrent read/write from two relay threads (it garbled the tunnel bytes), and shutting down
the TLS write side with shutdown(SHUT_WR) makes OpenSSL tear the session down, so the peer cannot deliver
its response. A single-threaded select relay fixed the concurrency but the close semantics still ended
sessions early. The reliable options are unchanged: run TLS/WireGuard/SSH outside the adapter (stunnel,
ssh -L, wg) - the adapter stays a simple plaintext-to-compressed-tunnel translator. A future version would
need a dedicated nonblocking SSL relay or a different TLS implementation.

## Per-flow policy (per-route compression level)

Gateway routes can carry their own compression level:

    python3 tools/wire_adapter.py gateway --listen 0.0.0.0:7042 \
        --route fast=127.0.0.1:7049 --route best=127.0.0.1:7050 \
        --route-level fast=1 --route-level best=9

--route-level NAME=N (0..9, NAME must be a defined route) controls the gateway->client direction for
that route; the client keeps its own --level for its direction. This lets one hub serve latency-sensitive
flows cheaply and bulk flows with maximum ratio.

Policy smoke (tools/wire_adapter_policy_smoke.sh): the same ~1.34 MiB one-way text payload through fast (level 1)
and best (level 9) both round-trip byte-identically; hub stats report fast 0.0050 / best 0.0043, with
best strictly smaller as expected.

## External encryption recipe (validated)

Keep the adapter plaintext-to-compressed-tunnel and put encryption **outside** it:

    # remote host: service on 7049, gateway adapter on 7051
    python3 tools/wire_adapter.py gateway --listen 127.0.0.1:7051 --target 127.0.0.1:7049

    # laptop: encrypted transport, then edge adapter, then local clients
    ssh -N -L 7051:127.0.0.1:7051 user@remote-host
    python3 tools/wire_adapter.py client --listen 127.0.0.1:7041 --remote 127.0.0.1:7051

Do not enable SSH compression (-C) while the adapter is active - one compressor is enough.

Measured on the same 8 MiB letter-like artifact over the public internet: direct 37.93 s; unencrypted
adapter 32.08 s (tunnel ratio 0.904); adapter inside an SSH -L tunnel 36.07 s with the same 0.904 ratio
and identical payload - SSH adds ~4 s of encryption overhead while preserving the adapter's byte win.
WireGuard or stunnel TLS work the same way (adapter connects to the local encrypted endpoint).

## No-expansion hybrid (per-stream probe)

Each direction probes its first chunk: if the chunk is at least 4 KiB and compresses to at least its
own size, that direction switches to **raw frames for the whole connection** (framing only, bit 31 of the
frame length marks raw); otherwise it uses a **streaming zlib compressor**, which keeps the best ratio
for real traffic. Small-first-chunk streams (typical request/response protocols) stay in streaming
compressed mode so a small manifest does not force the whole large response to go raw.

Measured with the hybrid: repetitive text 0.0043 (streaming, unchanged), 96 KiB random **1.0001** (raw,
was 1.0005), cold objpeer pull 0.6797, hub routes 0.5647/0.5576 (hub-run.log), policy routes fast 0.0050 / best 0.0043.

Limits: the mode choice is per direction per connection; a stream that starts small and becomes
incompressible stays compressed (minor expansion possible). Frame flag costs no extra bytes (bit 31 of
the existing length word).

## Live metrics

`--stats-interval SECONDS` (default 1.0, 0 disables) writes the stats file periodically while adapters
run, in addition to the SIGTERM snapshot. Writes are atomic (tmp + rename), so readers never see a
partial file. The metrics smoke sends 0.625 MiB one-way (1.25 MiB two-way) through a running pair
from the file without stopping anything; it reports e.g. `live connections=1 plain=1310720 ratio=0.0044`.

## Idle timeout

`--idle-timeout SECONDS` (default 0 = disabled) makes both sockets of a tunnel time out when no data
moves for that long, so abandoned flows do not leak adapter connections. The idle smoke verifies an idle
tunnel closes after 1.00 s while an active transfer through the same pair still round-trips exactly.

Note for a busy link: any silence longer than the timeout closes the flow, so leave it disabled (0) unless
the deployment wants automatic reaping.

### WireGuard or stunnel in front of the pair (recipe; not validated on this host)

Same shape everywhere: the adapter connects to a **local encrypted endpoint**, and encryption lives in
the tunnel technology of your choice.

WireGuard (kernel/UDP):

    # server (remote, 10.66.0.1 is its WireGuard address)
    wg genkey | tee server.key | wg pubkey > server.pub
    wg-quick up wg0        # [Interface] Address=10.66.0.1/24 PrivateKey=... / [Peer] client.pub ...
    python3 tools/wire_adapter.py gateway --listen 10.66.0.1:7051 --target 127.0.0.1:7049

    # edge host (10.66.0.2)
    wg-quick up wg0        # [Peer] PublicKey=server.pub Endpoint=server:51820 AllowedIPs=10.66.0.1/32
    python3 tools/wire_adapter.py client --listen 127.0.0.1:7041 --remote 10.66.0.1:7051

stunnel (TLS):

    # server: stunnel terminates TLS on 0.0.0.0:7051 and forwards to the gateway adapter on 7052
    [adapter]
    accept = 0.0.0.0:7051
    connect = 127.0.0.1:7052
    cert = /etc/stunnel/server.pem
    key = /etc/stunnel/server.key

    # client: stunnel listens 127.0.0.1:7053 and connects TLS to server:7051, verifying the CA
    [adapter]
    client = yes
    accept = 127.0.0.1:7053
    connect = server:7051
    CAfile = /etc/stunnel/ca.pem

    # adapters on both sides then talk to the stunnel endpoints (7052 server / 7053 client)

Neither WireGuard nor stunnel is installed on this host (and there is no container runtime), so these
recipes are deployment shapes, not locally measured claims; the SSH variant above is the measured one.

## Throughput (loopback, 8 MiB payload)

`tools/wire_adapter_bench.py` (concurrent reader, 8 MiB payload, Python 3.15, single run):

| Mode | Text MiB/s | Text ratio | Random MiB/s | Random ratio |
|---|---|---|---|---|
| Direct TCP echo | 633 | 1.0 | 892 | 1.0 |
| Adapter, level 1 | 395 | 0.0056 | 414 | 1.0001 |
| Adapter, level 6 (default) | 282 | 0.0043 | 277 | 1.0001 |
| Adapter, level 9 | 163 | 0.0043 | 336 | 1.0001 |

Loopback wall time is dominated by Python/zlib, and random streams take the raw path so their level
barely matters; single-run variance is visible (random level 9 > level 6). The adapter's product value is
wire-byte reduction on real links: WAN tunnel ratio 0.904 on protocol traffic and up to 232x on text.
Choose level 1 for latency/CPU, 6 as the default, 9 only when bandwidth dominates.

Second run (same command) for variance: direct 725/1222 MiB/s; adapter level 1 208/382, level 6 190/353,
level 9 178/273; text ratios are level-dependent (0.0056 at level 1, 0.0043 at level 6/9) and random is 1.0001.
them as order-of-magnitude topology evidence, not a fixed performance claim. The reproducible claim is the
ratio (byte reduction), which is stable across runs.

## Full protocol through the adapter

`tools/wire_adapter_protocol_smoke.sh` runs the whole objpeer CLI surface through one adapter pair: cold
pulls of two artifacts, multi-selector verify, pinned verify, replica audit (checks=2, verified=2),
one-verb repair from an empty store (repaired=2), a post-repair warm pull (0 nodes) and a fleet sync that
writes both artifacts to disk. Result: WIRE_ADAPTER_PROTOCOL_PASS with 9 edge connections and tunnel
ratio about 0.57 (0.5701 first run, 0.5757 re-run; loopback variance) - the adapter is transparent to session-oriented flows (series pulls, sync, repair), not just
single pulls.

Timings in the external-encryption sections come from the run summaries; the retained logs carry the byte counts (direct.log, edge.json, ssh-adapter-edge.json).

## Socket-layer mode

The same framing is exposed as a socket protocol (WireSocket/WireListener) so applications can adapt
before bytes enter TCP/IP without a proxy process; it interoperates with this proxy pair in both
directions. See WIRE-SOCKET.md and tools/wire_socket.py.
