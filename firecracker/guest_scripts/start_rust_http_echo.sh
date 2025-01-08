#!/usr/bin/env sh
IP_WITH_MASK=$(cat /proc/cmdline | grep -oe 'ip_with_mask=[0-9./]*')
IP_WITH_MASK=${IP_WITH_MASK#*=}
IP=$(echo $IP_WITH_MASK | cut -d'/' -f1)

/usr/bin/rust-http-echo -listen $IP:8080