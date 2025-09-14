#!/bin/bash

echo "100 openvpn" >> /etc/iproute2/rt_tables

openvpn --config config.ovpn --auth-user-pass up.txt &
until ip link show tun0 2>/dev/null; do sleep 1; done
danted &
privoxy --no-daemon /etc/privoxy/config &

echo "SOCKS5 OVER VPN IS READY!"
while true; do sleep 1000; done
