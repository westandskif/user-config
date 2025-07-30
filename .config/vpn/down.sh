#!/bin/bash

source "`dirname $0`/vpn_config.sh"

logger "OpenVPN down.sh: bridge if: $VPNBRIDGE_IF addr: $VPNBRIDGE_ADDR table: $VPNBRIDGE_TBL openvpn local: $ifconfig_local"

ip rule del from $VPNBRIDGE_ADDR lookup $VPNBRIDGE_TBL
iptables -t nat -D POSTROUTING -s $VPNBRIDGE_ADDR -j SNAT --to-source $ifconfig_local
ip tuntap del dev $VPNBRIDGE_IF mode tun
