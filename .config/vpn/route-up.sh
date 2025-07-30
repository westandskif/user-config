#!/bin/bash

source "`dirname $0`/vpn-config.sh" 

logger "OpenVPN route-up.sh: bridge if: $VPNBRIDGE_IF addr: $VPNBRIDGE_ADDR table: $VPNBRIDGE_TBL"

# Add the default route to our custom 'openvpn' routing table
ip route add default via $route_vpn_gateway dev $dev table $VPNBRIDGE_TBL

# Add new interface to act as a middleman between the fixed address used 
# by Squid and the dynamic address provided by OpenVPN
ip tuntap add dev $VPNBRIDGE_IF mode tun
ip addr add $VPNBRIDGE_ADDR dev $VPNBRIDGE_IF
ip link set dev $VPNBRIDGE_IF up

# All packets from Squid are passed to the custom routing table
ip rule add from $VPNBRIDGE_ADDR lookup $VPNBRIDGE_TBL

# Rewrite the source address to match the one provided by OpenVPN
iptables -t nat -A POSTROUTING -s $VPNBRIDGE_ADDR -j SNAT --to-source $ifconfig_local

logger "OpenVPN route-up.sh: local: $ifconfig_local remote: $route_vpn_gateway device: $dev"
