# Process interposer: adapt before TCP/IP for unmodified binaries

The interposer injects the wire protocol underneath an existing application: the app calls ordinary
connect/send/recv, and the library redirects configured destinations to a wire gateway and compresses the
stream with the same framing as the adapter pair and WireSocket.

    unmodified app --socket API--> interposer --compressed wire--> adapter gateway --> service

## Build

macOS (dylib):

    cc -dynamiclib -o libwireinterposer.dylib tools/wire_interposer.c -lz
    codesign -f -s - libwireinterposer.dylib

Linux (shared object):

    cc -shared -fPIC -o libwireinterposer.so tools/wire_interposer.c -ldl -lz

## Configure and run

    WIRE_ROUTES="127.0.0.1:65077=127.0.0.1:7042#service" \
      DYLD_INSERT_LIBRARIES=$PWD/libwireinterposer.dylib ./stock-app
    # or on Linux:
    WIRE_ROUTES="127.0.0.1:65077=127.0.0.1:7042#service" \
      LD_PRELOAD=$PWD/libwireinterposer.so ./stock-app

WIRE_ROUTES is semicolon-separated dest=gateway[#route] entries (numeric IPv4 host:port only). A legacy
single mapping is available as WIRE_MATCH / WIRE_GATEWAY / WIRE_ROUTE. WIRE_STATS_FILE writes the
WIRE_INTERPOSER_STATS JSON on exit; WIRE_DEBUG=1 logs.

Non-matching sockets are passed through untouched (verified: passthrough_connections=0 in the smokes).

## Measured

| Platform | Host | text ratio | random ratio | unmodified Python | passthrough |
|---|---|---|---|---|---|
| macOS 27 arm64, DYLD | stock C probe + Homebrew Python 3.15 | 0.0031 | 1.0001 | 0.0042 | 0 connections |
| Debian13/Linux 6.12 aarch64 microVM, LD_PRELOAD | stock C probe + Debian Python 3.12 | 0.0032 | 1.0001 | 0.0043 | 0 connections |

macOS loopback wall: direct 0.0017 s vs interposed 0.0192 s for a 2 MiB payload (compression + handshake
cost; the interposer is not a throughput accelerator on loopback, it removes bytes on real links).
Evidence: evidence/artifacts/interposer/run-macos-dyld.log and run-linux-mvm.log; reproduce with
tools/wire_interposer_smoke.sh (macOS) or tools/wire_interposer_linux_smoke.sh (Linux).

## Limits

- Config is numeric IPv4 only: the interposer deliberately avoids hostname resolution so it cannot
  re-enter its own intercepted calls.
- Blocking connect only. Applications that use nonblocking-connect plus a poll (e.g. Python sockets with
  an explicit timeout) are not supported yet; the handshake happens on the completed blocking path.
- macOS interposes connect/send/recv/shutdown/close. It does not interpose read/write, so a macOS app
  that uses write()/read() on sockets is not adapted (Linux interposes both).
- Blocking I/O only; no io_uring/aio, no MSG_* flags, one stream per connection, fd table capped at 1024.
- SIP strips DYLD_INSERT_LIBRARIES for protected system binaries; use self-built/unsigned app binaries or
  the proxy pair for those.
- No TLS/authentication in the framing: run over TLS/WireGuard/SSH or a trusted link.
- macOS uses raw syscalls for the real operations (deprecated but functional) to avoid dyld/dlsym
  re-entry; this is why the implementation differs from the LD_PRELOAD path.

## Routing through a socket-layer hub

The interposer can target a routed hub by appending the route name:

    WIRE_ROUTES="127.0.0.1:65078=127.0.0.1:7042#echo" DYLD_INSERT_LIBRARIES=... ./stock-app

Measured in tools/wire_hub_cli_smoke.sh: a stock C probe reached the echo route through wire_hub.py and
round-tripped 2 MiB exactly; the hub reported 2 connections / 13113 wire bytes / ratio 0.0024.

## Fork

The interposer close never calls socket-wide shutdown, so a child process closing an inherited routed
descriptor does not disturb the parent connection. Receive uses non-blocking chunks and checks an atomic
active flag, so close can stop a blocked reader without a socket shutdown; a pthread_atfork child handler
re-initializes the per-fd mutexes so a multithreaded parent can fork without the child inheriting a
locked mutex. Limitation: parent and child must not both keep using the same inherited routed
descriptor - the per-process compression state makes that stream unsound.
