#!/bin/bash

set -eux

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

export PATH="/root/.local/bin:$PATH"

# Install codex (if not already installed)
if ! command -v codex &> /dev/null; then
    arch="$(uname -m)"
    case "$arch" in
        x86_64)  asset="codex-x86_64-unknown-linux-musl.tar.gz" ;;
        aarch64|arm64) asset="codex-aarch64-unknown-linux-musl.tar.gz" ;;
        *) echo "Unsupported arch: $arch" >&2; exit 1 ;;
    esac
    echo "DOWNLOADING: ${asset}"
    curl -fSL --progress-bar "https://github.com/openai/codex/releases/latest/download/${asset}" -o /tmp/codex.tgz
    echo "DONWLOADED: ${asset}"
    tar -xzf /tmp/codex.tgz -C /tmp
    rm -f /tmp/codex.tgz
    mv /tmp/codex-* /root/.local/bin/codex
    chmod +x /root/.local/bin/codex
fi

# Install claude (if not already installed)
if ! command -v claude &> /dev/null; then
    curl -fSL --progress-bar https://claude.ai/install.sh | bash
fi

bash
# curl ipinfo.io
