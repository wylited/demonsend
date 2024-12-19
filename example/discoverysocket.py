#!/usr/bin/env python3
 .#
import socket
import struct

# Multicast configuration
MULTICAST_GROUP = "224.0.0.167"
PORT = 53317

def listen_to_multicast():
    # Create a UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)

    # Allow multiple sockets to use the same port (reuse address)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)

    # Bind the socket to the port
    sock.bind(('', PORT))  # '' listens on all interfaces

    # Join the multicast group
    group = socket.inet_aton(MULTICAST_GROUP)
    mreq = struct.pack('4sL', group, socket.INADDR_ANY)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)

    print(f"Listening for multicast messages on {MULTICAST_GROUP}:{PORT}...")

    try:
        while True:
            # Receive data from the multicast group
            data, address = sock.recvfrom(1024)  # Buffer size is 1024 bytes
            print(f"Received message: {data.decode()} from {address}")
    except KeyboardInterrupt:
        print("Exiting...")
    finally:
        # Leave the multicast group
        sock.setsockopt(socket.IPPROTO_IP, socket.IP_DROP_MEMBERSHIP, mreq)
        sock.close()

if __name__ == "__main__":
    listen_to_multicast()
