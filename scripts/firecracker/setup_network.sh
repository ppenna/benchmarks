TAP_DEV=${1:-"tap0"}
TAP_IP=${2:-"172.16.0.1"}
MASK_SHORT="/24"
HOST_IFACE="eth0"

echo "Setting up network for device $TAP_DEV with IP $TAP_IP and host interface $HOST_IFACE"

# Setup network interface
sudo ip tuntap add dev "$TAP_DEV" mode tap
sudo ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
sudo ip link set dev "$TAP_DEV" up

# Enable ip forwarding
sudo sh -c "echo 1 > /proc/sys/net/ipv4/ip_forward"

# Set up microVM internet access
sudo iptables -t nat -A POSTROUTING -o "$HOST_IFACE" -j MASQUERADE
sudo iptables -I FORWARD 1 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
sudo iptables -I FORWARD 1 -i "$TAP_DEV" -o "$HOST_IFACE" -j ACCEPT