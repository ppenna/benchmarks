#!/usr/bin/env bash
TAP_DEV=${1:-"tap0"}
HOST_IFACE="eth0"

echo "Cleaning up network setup for device $TAP_DEV and host interface $HOST_IFACE"

# Delete tap device
sudo ip link del "$TAP_DEV" 2> /dev/null || true

# Delete iptables rules
sudo iptables -t nat -D POSTROUTING -o "$HOST_IFACE" -j MASQUERADE || true
sudo iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT || true
sudo iptables -D FORWARD -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT || truen