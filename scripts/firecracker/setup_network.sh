TAP_DEV=${1:-"tap0"}
TAP_IP=${2:-"172.16.0.1"}
VM_IP=${3:-"172.16.0.2"}
MASK_SHORT="/16"
HOST_IFACE="eth0"

echo "Setting up network for device $TAP_DEV with TAP IP $TAP_IP, VM_IP ${VM_IP} and host interface $HOST_IFACE"

# Setup network interface
sudo ip tuntap add dev "$TAP_DEV" mode tap
sudo ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
sudo ip link set dev "$TAP_DEV" up

# Set up microVM internet access
# sudo iptables -t nat -A POSTROUTING -o "$HOST_IFACE" -s "$VM_IP" -j MASQUERADE
# sudo iptables -I FORWARD 1 -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT

sudo nft add rule firecracker postrouting ip saddr ${VM_IP} oifname ${HOST_IFACE} counter masquerade
sudo nft add rule firecracker filter iifname ${TAP_DEV} oifname ${HOST_IFACE} accept