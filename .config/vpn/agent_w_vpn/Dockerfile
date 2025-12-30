FROM ubuntu:latest

ARG MY_PROXY_HOST

RUN apt update \
    && apt install -qy \
      git \
      iptables \
      less \
      curl \
      unzip \
      iproute2 \
      git \
      ripgrep \
      tar \
      ca-certificates

ENV TUN2SOCKS_NAME=tun2socks-linux-arm64
ENV SOCKS_HOST=$MY_PROXY_HOST

RUN curl -L -O "https://github.com/xjasonlyu/tun2socks/releases/download/v2.6.0/${TUN2SOCKS_NAME}.zip" \
    && unzip "${TUN2SOCKS_NAME}.zip" -d /usr/local/bin \
    && chmod +x "/usr/local/bin/${TUN2SOCKS_NAME}"

RUN set -eux; \
    arch="$(uname -m)"; \
    case "$arch" in \
      x86_64)  asset="codex-x86_64-unknown-linux-musl.tar.gz" ;; \
      aarch64|arm64) asset="codex-aarch64-unknown-linux-musl.tar.gz" ;; \
      *) echo "Unsupported arch: $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/openai/codex/releases/latest/download/${asset}" -o /tmp/codex.tgz; \
    tar -xzf /tmp/codex.tgz -C /usr/local/bin; \
    rm -f /tmp/codex.tgz; \
    mv /usr/local/bin/codex-* /usr/local/bin/codex; \
    chmod +x /usr/local/bin/codex

RUN set -eux; \
  curl -fsSL https://claude.ai/install.sh | bash
RUN echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

RUN mkdir -p /mnt/app \
    && mkdir -p /root/.codex \
    && mkdir -p /root/.claude
ADD entrypoint.sh /mnt/
RUN chmod +x /mnt/*.sh

WORKDIR /mnt/app

ENTRYPOINT ["/bin/bash", "/mnt/entrypoint.sh"]
