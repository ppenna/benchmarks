#!/usr/bin/env bash
TAP_DEV=${1:-"tap0"}
VM_IP=${2:-"172.16.0.2"}
HOST_IFACE="eth0"

echo "Cleaning up network setup for device $TAP_DEV, host interface $HOST_IFACE, and VM_IP ${VM_IP}"

# Delete tap device
sudo ip link del "$TAP_DEV" 2> /dev/null || true

# Delete iptables rules
# sudo iptables -t nat -D POSTROUTING -o "$HOST_IFACE" -s "$VM_IP" -j MASQUERADE || true
# sudo iptables -D FORWARD -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT || true