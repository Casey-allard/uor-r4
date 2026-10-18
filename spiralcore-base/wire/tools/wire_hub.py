#!/usr/bin/env python3
"""Route-based wire hub: handshake route names -> local TCP targets (socket-layer hub)."""
import argparse
import os
import signal
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from wire_socket import WireGateway


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--listen", required=True)
    parser.add_argument("--route", action="append", required=True, help="NAME=HOST:PORT")
    parser.add_argument("--stats-file")
    args = parser.parse_args()
    routes = {}
    for entry in args.route:
        if "=" not in entry:
            raise SystemExit("--route must be NAME=HOST:PORT, got " + entry)
        name, target = entry.split("=", 1)
        routes[name] = target
    gateway = WireGateway.bind(args.listen, routes, args.stats_file)

    def finish(_signum, _frame):
        gateway.stop()
        raise SystemExit(0)

    signal.signal(signal.SIGTERM, finish)
    signal.signal(signal.SIGINT, finish)
    print("READY " + gateway.address(), flush=True)
    gateway.serve_forever()


if __name__ == "__main__":
    main()
