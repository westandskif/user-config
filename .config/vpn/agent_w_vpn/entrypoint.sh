#!/bin/bash

SOCKS_IP=$(getent hosts "$SOCKS_HOST" | awk '{print $1}')
GATEWAY_IP=$(ip route show default | awk '{ print $3 }')

# Create TUN device
ip tuntap add mode tun dev tun0
ip addr add 198.18.0.1/15 dev tun0
ip link set dev tun0 up

# Route all traffic through tun0, except traffic to the SOCKS proxy
ip route add "${SOCKS_IP}/32" via "${GATEWAY_IP}"
ip route replace default dev tun0

# Start tun2socks
/usr/local/bin/${TUN2SOCKS_NAME} -device tun0 -proxy "socks5://${SOCKS_IP}:1080" > /dev/null 2>&1 &

bash
# curl ipinfo.io
