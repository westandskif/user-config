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


ADD config_NJ.ovpn /mnt/vpn/config.ovpn

ADD danted.conf /etc/danted.conf
ADD up.txt entrypoint.sh down.sh route-up.sh vpn-config.sh /mnt/vpn/
RUN chmod +x /mnt/vpn/*.sh

WORKDIR /mnt/vpn

EXPOSE 1080

ENTRYPOINT ["/bin/bash", "/mnt/vpn/entrypoint.sh"]
