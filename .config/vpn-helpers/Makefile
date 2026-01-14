.PHONY: abc

default_gateway = $(shell route -n get default | grep gateway | awk '{ print $$2 }')

run:
	cargo build --release 
	sudo bash -c 'tcpdump -i any -n -l "udp port 53" 2>/dev/null | \
    ./target/release/dns-route-manager --gateway $(default_gateway) --patterns ".by,.ru,youtube,google,gstatic.com"'
