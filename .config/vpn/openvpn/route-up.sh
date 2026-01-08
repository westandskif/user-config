#!/bin/bash
set -euo pipefail

exec 2>&1  # Redirect stderr to stdout for unified logging
echo "[route-up] Starting at $(date)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[route-up] SCRIPT_DIR=$SCRIPT_DIR"
BIN="$SCRIPT_DIR/dns-route/target/release/dns-route"
PATTERNS_FILE="$SCRIPT_DIR/patterns.txt"

# Runtime directory for all dns-route state files
runtime_dir="${DNS_ROUTE_RUNTIME_DIR:-$SCRIPT_DIR/run}"
mkdir -p "$runtime_dir"
DNS_BACKUP_FILE="$runtime_dir/dns-backup"

# Read patterns from file
echo "[route-up] Checking patterns file: $PATTERNS_FILE"
if [[ ! -f "$PATTERNS_FILE" ]]; then
    echo "[route-up] Patterns file not found at $PATTERNS_FILE; skipping dns-route"
    exit 0
fi

patterns=$(tr '\n' ',' < "$PATTERNS_FILE" | sed 's/,$//')
echo "[route-up] Loaded patterns: $patterns"
if [[ -z "$patterns" ]]; then
    echo "[route-up] No patterns found in $PATTERNS_FILE; skipping dns-route"
    exit 0
fi

# Get default gateways for en0 (regular internet, not VPN)
echo "[route-up] Detecting gateways..."
en0_gateway_ipv4=$(/usr/sbin/netstat -rn -f inet | grep -E '^default.*en0' | awk '{print $2}' | head -1) || true
en0_gateway_ipv6=$(/usr/sbin/netstat -rn -f inet6 | grep -E '^default.*en0' | awk '{print $2}' | head -1) || true
echo "[route-up] Detected en0_gateway_ipv4=$en0_gateway_ipv4 en0_gateway_ipv6=$en0_gateway_ipv6"

gateway_ipv4="${DNS_ROUTE_GATEWAY_IPV4:-${en0_gateway_ipv4:-}}"
gateway_ipv6="${DNS_ROUTE_GATEWAY_IPV6:-${en0_gateway_ipv6:-}}"
echo "[route-up] Using gateway_ipv4=$gateway_ipv4 gateway_ipv6=$gateway_ipv6"

if [[ -z "$gateway_ipv4" ]]; then
    echo "[route-up] ERROR: IPv4 gateway not found (expected DNS_ROUTE_GATEWAY_IPV4 or en0 default gateway)"
    exit 1
fi
echo "Using en0 IPv4 gateway: $gateway_ipv4"
if [[ -n "$gateway_ipv6" ]]; then
    echo "Using en0 IPv6 gateway: $gateway_ipv6"
fi

listen="${DNS_ROUTE_LISTEN:-127.0.0.1:53}"
listen_ip="${listen%:*}"
ttl="${DNS_ROUTE_TTL:-300}"
log_file="$runtime_dir/dns-route.log"
pid_file="$runtime_dir/dns-route.pid"
# Truncate log file on startup
: > "$log_file"

if [[ ! -x "$BIN" ]]; then
    echo "dns-route binary not found at $BIN"
    exit 1
fi

# Collect DNS servers pushed by OpenVPN (for matched requests).
echo "[route-up] Collecting VPN DNS servers from foreign_option_* variables..."
vpn_dns_servers=()
for opt in ${!foreign_option_*}; do
    val="${!opt}"
    echo "[route-up]   $opt=$val"
    case "$val" in
        *DNS*)
            dns_server="${val##* }"
            vpn_dns_servers+=("$dns_server")
            ;;
    esac
done
echo "[route-up] Found ${#vpn_dns_servers[@]} VPN DNS servers"

vpn_dns=""
if [[ ${#vpn_dns_servers[@]} -gt 0 ]]; then
    vpn_dns=$(IFS=','; echo "${vpn_dns_servers[*]}")
fi
echo "[route-up] vpn_dns=$vpn_dns"

if [[ -f "$pid_file" ]]; then
    if kill -0 "$(cat "$pid_file")" 2>/dev/null; then
        kill "$(cat "$pid_file")" || true
    fi
    rm -f "$pid_file"
fi

# Detect active network service and save current DNS
NETWORKSETUP=/usr/sbin/networksetup

active_service=$($NETWORKSETUP -listallnetworkservices | grep -v '^\*' | while read -r service; do
    if $NETWORKSETUP -getinfo "$service" 2>/dev/null | grep -q "^IP address:"; then
        echo "$service"
        break
    fi
done)

if [[ -n "$active_service" ]]; then
    current_dns=$($NETWORKSETUP -getdnsservers "$active_service" 2>/dev/null | tr '\n' ' ' | sed 's/ $//')
    echo "$active_service|$current_dns" > "$DNS_BACKUP_FILE"
    echo "Saved DNS backup for $active_service: $current_dns"

    # Set system DNS to our listener
    $NETWORKSETUP -setdnsservers "$active_service" "$listen_ip"
    echo "Set system DNS to $listen_ip for $active_service"
fi

if [[ -z "$vpn_dns" ]]; then
    echo "No VPN DNS servers found from OpenVPN; skipping dns-route"
    exit 0
fi

echo "DNS for matched (excluded from VPN): $gateway_ipv4"
echo "DNS for non-matched (via VPN): $vpn_dns"

cmd=(sudo "$BIN" \
    --patterns "$patterns" \
    --gateway-ipv4 "$gateway_ipv4" \
    --dns-for-matched "$gateway_ipv4" \
    --dns-for-non-matched "$vpn_dns" \
    --listen "$listen" \
    --route-ttl "$ttl" \
    --runtime-dir "$runtime_dir")

if [[ -n "$gateway_ipv6" ]]; then
    cmd+=(--gateway-ipv6 "$gateway_ipv6")
fi

echo "[route-up] Launching: ${cmd[*]}"
nohup "${cmd[@]}" >>"$log_file" 2>&1 &
echo "[route-up] Launched dns-route with PID $!"
echo "[route-up] Done"
