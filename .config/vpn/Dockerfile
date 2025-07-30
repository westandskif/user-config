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
      dante-server

ADD my_expressvpn_usa_-_washington_dc_udp.ovpn \
    up.txt \
    /mnt/vpn

ADD danted.conf /etc/danted.conf
ADD entrypoint.sh down.sh route-up.sh vpn-config.sh /mnt/vpn/
RUN chmod +x /mnt/vpn/*.sh

WORKDIR /mnt/vpn

EXPOSE 1080

ENTRYPOINT ["/bin/bash", "/mnt/vpn/entrypoint.sh"]
