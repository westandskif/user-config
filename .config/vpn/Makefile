.PHONY: tst codex

IMAGE_NAME := myvpn:latest
CONTAINER_NAME := my-socks5-over-vpn
NETWORK_NAME := my-socks5-over-vpn
CURRENT_DIR = $(shell pwd)


build:
	docker build -t $(IMAGE_NAME) .
	docker rm -f $(CONTAINER_NAME) > /dev/null 2>&1 || true
	docker network inspect $(NETWORK_NAME) > /dev/null 2>&1 || docker network create $(NETWORK_NAME)

bash:
	docker run --rm -it --privileged --entrypoint=bash $(IMAGE_NAME) 

start: build
	docker run \
		--name $(CONTAINER_NAME) \
		-d \
		--network $(NETWORK_NAME) \
		-p 127.0.0.1:1080:1080 \
		-p 127.0.0.1:3128:3128 \
		-v $(CURRENT_DIR)/openvpn_WA_0.ovpn:/mnt/vpn/config.ovpn \
		--restart unless-stopped \
		--privileged \
		$(IMAGE_NAME)


# INPUTS
WORKDIR := $(or $(WORKDIR),$(CURRENT_DIR))
# ----

agents:
	docker build \
		-f agent_w_vpn/Dockerfile \
		-t agent_w_vpn:latest \
		--build-arg MY_PROXY_HOST=$(CONTAINER_NAME) \
		agent_w_vpn
	docker run \
		--rm \
		-it \
	    -v agents-config:/root \
		-v $$(realpath $(WORKDIR)):/mnt/app \
		--network $(NETWORK_NAME) \
		--privileged \
		--device=/dev/net/tun:/dev/net/tun \
		--cap-add=NET_ADMIN \
		agent_w_vpn:latest \
		bash


# make agent NAME=claude WORKDIR=~/.config/nvim
# curl -fsSL https://claude.ai/install.sh | bash
# echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
# export PATH="$HOME/.local/bin:$PATH"
