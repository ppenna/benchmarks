#!/usr/bin/env sh

# Extract IP from kernel command line
IP_WITH_MASK=$(cat /proc/cmdline | grep -oe 'ip_with_mask=[0-9./]*')
IP_WITH_MASK=${IP_WITH_MASK#*=}
# Extract route from kernel command line
ROUTE=$(cat /proc/cmdline | grep -oe 'route=[0-9.]*')
ROUTE=${ROUTE#*=}

ip addr add ${IP_WITH_MASK} dev eth0
ip link set eth0 up
ip route add default via ${ROUTE} dev eth0