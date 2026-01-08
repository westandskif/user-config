#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
runtime_dir="${DNS_ROUTE_RUNTIME_DIR:-$SCRIPT_DIR/run}"
DNS_BACKUP_FILE="$runtime_dir/dns-backup"
PID_FILE="$runtime_dir/dns-route.pid"

# Kill dns-route daemon
if [[ -f "$PID_FILE" ]]; then
    if kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
        kill "$(cat "$PID_FILE")" || true
        echo "Stopped dns-route daemon"
    fi
    rm -f "$PID_FILE"
fi

# Restore original DNS settings
NETWORKSETUP=/usr/sbin/networksetup

if [[ -f "$DNS_BACKUP_FILE" ]]; then
    IFS='|' read -r service dns < "$DNS_BACKUP_FILE"
    if [[ -n "$service" ]]; then
        if [[ "$dns" == "There aren't any DNS Servers set on"* ]] || [[ -z "$dns" ]]; then
            $NETWORKSETUP -setdnsservers "$service" "Empty"
            echo "Cleared DNS for $service (was not set)"
        else
            # shellcheck disable=SC2086
            $NETWORKSETUP -setdnsservers "$service" $dns
            echo "Restored DNS for $service: $dns"
        fi
    fi
    rm -f "$DNS_BACKUP_FILE"
fi
