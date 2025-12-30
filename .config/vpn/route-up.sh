#!/bin/bash

source "`dirname $0`/vpn-config.sh"

logger "OpenVPN route-up.sh: bridge if: $VPNBRIDGE_IF addr: $VPNBRIDGE_ADDR table: $VPNBRIDGE_TBL"

ip route add default via $route_vpn_gateway dev $dev table $VPNBRIDGE_TBL
ip rule add from $ifconfig_local lookup $VPNBRIDGE_TBL
