.PHONY: tst

image_name := myvpn:latest

current_dir = $(shell pwd)

build:
	docker build -t $(image_name) .
	docker rm -f my-socks5-over-vpn || true

bash:
	docker run --rm -it --privileged --entrypoint=bash $(image_name) 

run:
	docker run --rm -it \
		-p 127.0.0.1:1080:1080 \
		-p 127.0.0.1:3128:3128 \
		--privileged $(image_name)

start:
	docker run \
		--name my-socks5-over-vpn \
		-d \
		-p 127.0.0.1:1080:1080 \
		-p 127.0.0.1:3128:3128 \
		-v $(current_dir)/config_NJ.ovpn:/mnt/vpn/config.ovpn \
		--restart unless-stopped \
		--privileged \
		$(image_name)
