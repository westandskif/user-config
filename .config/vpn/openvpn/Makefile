start:
	pushd dns-route && cargo build --release && popd
	sudo openvpn \
	    --config openvpn_WA_0.ovpn \
	    --script-security 2 \
	    --route-up route-up.sh \
	    --down down.sh \
	    --mute-replay-warnings \
	    --auth-user-pass up.txt
