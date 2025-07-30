1. add the below to your ovpn file:

```
script-security 2
route-noexec
route-up /mnt/vpn/route-up.sh
down /mnt/vpn/down.sh
mute-replay-warnings
```

1. add `up.txt` file with 2 lines (username 1st, password 2nd)

1. run `make start`

1. use localhost:1080 as SOCKS5 in Firefox
