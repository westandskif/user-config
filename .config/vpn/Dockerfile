FROM ubuntu:latest

RUN mkdir -p /mnt/vpn
RUN apt update \
    && apt install -qy \
      openvpn \
      vim \
      less \
      iputils-ping \
      curl \
      net-tools \
      iproute2 \
      iptables \
      dante-server \
      privoxy

# expects *.ovpn file to be at /mnt/vpn/config.ovpn
# *.ovpn file should contain the below:
# >>> script-security 2
# >>> route-noexec
# >>> route-up /mnt/vpn/route-up.sh
# >>> down /mnt/vpn/down.sh
# >>> mute-replay-warnings
# ADD config_NJ.ovpn /mnt/vpn/config.ovpn

ADD danted.conf /etc/danted.conf
ADD privoxy.conf /etc/privoxy/config
ADD up.txt entrypoint.sh down.sh route-up.sh vpn-config.sh /mnt/vpn/
RUN chmod +x /mnt/vpn/*.sh

WORKDIR /mnt/vpn

EXPOSE 1080 3128

ENTRYPOINT ["/bin/bash", "/mnt/vpn/entrypoint.sh"]
