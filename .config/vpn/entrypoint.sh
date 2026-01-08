#!/bin/bash

echo "100 openvpn" >> /etc/iproute2/rt_tables

openvpn \
	--config config.ovpn \
	--script-security 2 \
	--route-noexec \
	--route-up /mnt/vpn/route-up.sh \
	--down /mnt/vpn/down.sh \
	--mute-replay-warnings \
	--auth-user-pass up.txt \
	&
open_vpn_id="$!"

until ip link show tun0 2>/dev/null; do sleep 1; done
echo "OPENVPN STARTED"

danted &
echo "HTTP PROXY STARTED"

privoxy --no-daemon /etc/privoxy/config &
echo "SOCKS5 PROXY STARTED"

wait "$open_vpn_id"
