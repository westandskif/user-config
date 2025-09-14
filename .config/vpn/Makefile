.PHONY: tst

image_name := myvpn:latest

build:
	docker build -t $(image_name) .

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
		--restart unless-stopped \
		--privileged \
		$(image_name)
