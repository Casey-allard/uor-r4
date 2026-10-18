#!/usr/bin/env python3
"""Minimal TCP echo server for adapter smoke tests: reads, echoes, supports half-close."""
import argparse
import socket
import threading


def handle(conn):
    try:
        while True:
            data = conn.recv(65536)
            if not data:
                break
            conn.sendall(data)
    except OSError:
        pass
    try:
        conn.shutdown(socket.SHUT_WR)
    except OSError:
        pass
    try:
        conn.close()
    except OSError:
        pass


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--listen', default='127.0.0.1:0')
    args = parser.parse_args()
    host, port = args.listen.rsplit(':', 1)
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((host, int(port)))
    sock.listen(16)
    print('READY %s:%d' % sock.getsockname()[:2], flush=True)
    while True:
        conn, _ = sock.accept()
        threading.Thread(target=handle, args=(conn,), daemon=True).start()


if __name__ == '__main__':
    main()
