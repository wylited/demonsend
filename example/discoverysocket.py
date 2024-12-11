#!/usr/bin/env python3

import socket
import json
import threading
import time
from datetime import datetime

MCAST_GRP = '224.0.0.167'
MCAST_PORT = 53318

def create_multicast_socket():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('', MCAST_PORT))

    mreq = socket.inet_aton(MCAST_GRP) + socket.inet_aton('0.0.0.0')
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
    return sock

def listener():
    sock = create_multicast_socket()
    print(f"Listening for discovery packets on {MCAST_GRP}:{MCAST_PORT}")

    while True:
        data, addr = sock.recvfrom(1024)
        try:
            decoded = json.loads(data.decode())
            print(f"\nReceived from {addr} at {datetime.now()}")
            print(f"Device Info: {json.dumps(decoded, indent=2)}")
        except json.JSONDecodeError:
            print(f"\nReceived non-JSON data from {addr}: {data.decode()}")

def announcer():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 32)

    test_device = {
        "alias": "TestDevice",
        "deviceModel": "Test",
        "deviceType": "desktop",
        "fingerprint": "test123",
        "port": 53317,
        "protocol": "https",
        "version": "2.0",
        "download": "/download",
        "announce": True
    }

    while True:
        print("\nSending announcement...")
        sock.sendto(json.dumps(test_device).encode(), (MCAST_GRP, MCAST_PORT))
        time.sleep(5)

def main():
    listener_thread = threading.Thread(target=listener)
    announcer_thread = threading.Thread(target=announcer)

    listener_thread.daemon = True
    announcer_thread.daemon = True

    listener_thread.start()
    announcer_thread.start()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\nShutting down...")

if __name__ == "__main__":
    main()
