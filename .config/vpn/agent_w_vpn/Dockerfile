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

RUN mkdir -p /mnt/app \
    && mkdir -p /root/.codex \
    && mkdir -p /root/.claude
ADD entrypoint.sh /mnt/
RUN chmod +x /mnt/*.sh

WORKDIR /mnt/app

ENTRYPOINT ["/bin/bash", "/mnt/entrypoint.sh"]
