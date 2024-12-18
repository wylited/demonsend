#!/usr/bin/env python3

import socket

# Multicast configuration
MULTICAST_GROUP = "224.0.0.167"
PORT = 53317

def send_multicast_message():
    # Create a UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)

    # Set the TTL (time-to-live) for multicast messages
    # TTL > 1 allows the message to cross multiple networks
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 2)

    message = "Hello, Multicast! This is a test message."
    sock.sendto(message.encode(), (MULTICAST_GROUP, PORT))
    print(f"Sent message: {message}")

if __name__ == "__main__":
    send_multicast_message()
