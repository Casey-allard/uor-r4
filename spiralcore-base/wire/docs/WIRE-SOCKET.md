# Wire socket: the adapter framing as a socket-layer protocol

The adapter has two deployment shapes that share one wire protocol:

    proxy pair     app --plain--> edge adapter ==compressed==> gateway adapter --plain--> service
    socket layer   app uses WireSocket / WireListener directly (no proxy process)

The socket layer adapts bytes **before they enter TCP/IP** on the sending side and reverses it on the
receiving side, while the proxy pair adapts unmodified applications in front of their TCP traffic. A
WireSocket client can also talk straight to an existing adapter gateway, and an adapter edge can front a
WireListener service - the framing is the same.

## Wire format (shared with tools/wire_adapter.py)

    handshake  MAGIC (UORADP01), u8 route-name length, route name, MAGIC reply
    stream     u32 header + payload, per direction; bit 31 of the header marks a raw frame
               zero-length frame = end of stream
    mode       first chunk of each direction picks raw (>=4 KiB and incompressible) or streaming zlib

So a socket-layer peer is wire-compatible with the proxy pair; no translation daemon is needed.

## API

Client:

    from wire_socket import WireSocket
    sock = WireSocket.connect(host + ':' + port, route='')
    sock.sendall(b'hello')
    sock.shutdown()
    data = sock.recv(65536)
    print(sock.stats())   # {'plain': ..., 'wire': ..., 'ratio': ...}

Server:

    from wire_socket import WireListener
    listener = WireListener.bind('0.0.0.0:7042')
    conn = listener.accept()          # speaks the wire protocol; conn.recv/sendall are sockets-like
    data = conn.recv(65536)
    conn.sendall(data)

## Measured (tools/wire_socket_smoke.py)

    peer-to-peer text payload      ratio 0.0041
    peer-to-peer random payload    ratio 1.0001 (raw fallback)
    WireSocket -> adapter gateway  ratio 0.0041 (no edge process needed)
    adapter edge -> WireListener   exact round-trip (reverse interop)
    WIRE_SOCKET_PASS

Evidence: evidence/artifacts/wire-socket/run.log.

## When to use which

- unmodified app / any TCP service: use the **proxy pair** (edge + gateway adapters); no code changes.
- app you control that can swap its socket object: use the **socket layer** (WireSocket/WireListener);
  no daemon, one fewer hop, same wire format, and the peer can be a proxy gateway or another socket peer.
- a container or microVM with an egress proxy: combine the socket layer with a proxied connect (the
  portable agent already supports SOCKS5 for mvm's deny-first egress).

## Limits

- Python library only for now; other languages need their own binding of the same framing.
- One compressed stream per TCP connection; no multiplexing of several logical streams yet.
- No in-library TLS/authentication: run over TLS/WireGuard/SSH or inside a trusted link (the adapter
  framing itself is not encrypted).
- Raw mode protects incompressible streams from expansion; protocol control traffic still pays framing
  bytes.

## Process interposer

tools/wire_interposer.c implements the same framing as an OS-level interposer: unmodified binaries get
their configured TCP connections redirected to a wire gateway and compressed before the bytes enter
TCP/IP. Measured on macOS (DYLD_INSERT_LIBRARIES) and Linux (LD_PRELOAD inside the mvm microVM): text
ratio 0.0031/0.0032, random 1.0001, unmodified Python clients 0.0042/0.0043, non-matching sockets pass
through untouched. See INTERPOSER.md; evidence/artifacts/interposer/.

## Routed socket-layer hub (WireGateway / wire_hub.py)

The socket layer also has a route table, like the adapter hub:

    gateway = WireGateway.bind('0.0.0.0:7042', {'model': '127.0.0.1:7049', 'store': '127.0.0.1:7050'})
    Thread(target=gateway.serve_forever, daemon=True).start()

or as a daemon:

    python3 tools/wire_hub.py --listen 0.0.0.0:7042 \
        --route model=127.0.0.1:7049 --route store=127.0.0.1:7050 --stats-file hub.json

The client's handshake route name selects the target. Unknown routes complete the handshake and are
closed without a target (fail closed; counted as unknown_requests). Stats are per route: connections,
plain_in/plain_out, wire bytes, ratio; WIRE_STATS_FILE-style JSON is written on connection close,
gateway.stop(), and SIGTERM.

Measured (tools/wire_socket_routing_smoke.py -> WIRE_SOCKET_ROUTING_PASS): two routes to two labelled
services returned the correct labels and exact payloads with ratios 0.0042/0.0042, unknown route failed
closed, and per-route wire counts were 5451/5478 bytes.

Measured (tools/wire_hub_cli_smoke.sh -> WIRE_HUB_CLI_PASS): the wire_hub.py daemon served a WireSocket
client and a stock C binary through the process interposer (WIRE_ROUTES=...=hub#echo); the echo route
recorded 2 connections, 13113 wire bytes, ratio 0.0024.

## Opt-in multiplexing (tools/wire_mux.py)

Several logical streams can share one wire connection. Mux uses a distinct handshake magic
(UORADM01), so single-stream peers fail closed instead of mis-parsing frames:

    handshake  UORADM01, u8 route length, route, UORADM01 reply
    frame      u16 stream_id, u8 type, u16 length, payload
    types      1 OPEN, 2 DATA compressed, 3 DATA raw, 4 FIN, 5 WINDOW (u32 credit), 6 RESET

Each stream has independent compression state and a 64 KiB send window advanced by WINDOW frames
(flow control); FIN is per-stream half-close. One writer lock serializes frames on the shared socket.

    from wire_mux import connect_mux, WireMuxGateway
    mux = connect_mux('host:7042', route='echo')
    stream = mux.open_stream()
    stream.sendall(b'request'); stream.shutdown()
    response = stream.recv(65536)

Server side is WireMuxGateway: the handshake route selects the target for all streams on that connection.

Measured (tools/wire_mux_smoke.py -> WIRE_MUX_PASS), 8 concurrent streams mixing 128 KiB text and 96 KiB
random payloads, two runs:

    per-stream bytes exact                      yes (all 8)
    window updates                              60 (flow control exercised)
    mux wire bytes (best)                       811,112   ratio 0.2750
    separate single-stream wire bytes (best)    807,844   ratio 0.2739  (+0.4% mux framing)
    mux wall seconds                           0.0569, 0.0262
    separate wall seconds                      0.0183, 0.0184

Mux trades loopback wall time and +0.4% framing for one connection carrying many streams; it is not a
throughput accelerator on localhost. Exact per-stream bytes and shutdown semantics are preserved.

Limits: client-opened streams only; one route/target per mux connection (all streams on it share the
target); fixed 64 KiB window; no stream priority or cancellation beyond RESET; Python only so far.

## Rust binding

crates/wire-rs implements WireSocket/WireListener over the unchanged framing (flate2 for zlib, no protocol
changes). tools/wire_rust_interop_smoke.sh -> WIRE_RUST_INTEROP_PASS: Rust client -> Python listener text
0.0034, Python client -> Rust server 0.0042, random 1.0001 both directions, Rust client -> adapter gateway
0.0034, byte equality asserted. See RUST-BINDING.md; evidence/artifacts/rust-interop/.

### Per-stream routing (one mux connection, many targets)

OPEN frames may carry a route name: `mux.open_stream('model')` selects a target for that logical stream
while the connection handshake route may be empty. The hub resolves each stream independently
(WireMuxGateway routes dict); unknown route names get RESET for that stream only, counted in
unknown_requests. An empty OPEN route falls back to the handshake/default target, so existing clients are
unchanged.

Measured (tools/wire_mux_routing_smoke.py -> WIRE_MUX_ROUTING_PASS): one mux connection opened streams to
two labelled targets (model/store), both returned the correct label and exact payloads with ratios
0.0052/0.0052; an unknown stream route failed closed (RESET).

## TLS transport (socket layer, measured)

TLS is supported at the socket layer without touching the framing:

    listener = WireListener.bind('0.0.0.0:7042', tls_cert='server.pem', tls_key='server.key')
    wire = WireSocket.connect('host:7042', route='svc', tls=True, ca='ca.pem')

- WireSocket wraps/receives TLS before the UORADP01 handshake; framing, hybrid modes and stats are unchanged.
- WireGateway (routed socket hub) accepts TLS listeners; its relay is a single-threaded select loop, so an
  SSLSocket is only ever touched by one thread. End-of-stream uses the zero frame, not SSLSocket.shutdown
  (which tears the TLS session down).
- Wrong-transport combinations fail closed (plain client to TLS listener, TLS client to plain listener).

Measured (tools/wire_socket_tls_smoke.py -> WIRE_SOCKET_TLS_PASS): peer TLS text ratio 0.0042, random
1.0001, TLS routed gateway text 0.0042, both wrong-transport cases failed closed.

Mux over TLS is not supported yet: the mux reader thread and writer thread share one socket, which is not
safe for SSLSocket; use the base socket layer or terminate TLS externally (stunnel/SSH) for mux.
