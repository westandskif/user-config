#!/bin/bash

echo "100 openvpn" >> /etc/iproute2/rt_tables

openvpn --config my_expressvpn_usa_-_washington_dc_udp.ovpn --auth-user-pass up.txt &
until ip link show vpnbridge0 2>/dev/null; do sleep 1; done
danted &

echo "SOCKS5 OVER VPN IS READY!"
while true; do sleep 1000; done
