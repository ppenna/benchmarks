#!/usr/bin/env sh 
apk add openrc
apk add util-linux
apk add openssh

# Change default password to root
echo "root:root" | chpasswd

ln -s agetty /etc/init.d/agetty.ttyS0
echo ttyS0 > /etc/securetty
rc-update add agetty.ttyS0 default

# Make sure special file systems are mounted on boot:
rc-update add devfs boot
rc-update add procfs boot
rc-update add sysfs boot

# Configure basic networking
## Configure basic network in /etc/network/interfaces
cat > /etc/network/interfaces <<EOF
auto lo
iface lo inet loopback

auto eth0
EOF
## Configure dns
echo 'nameserver 8.8.8.8' > /etc/resolv.conf

## Copy executables
# Copy http executable 
cp /script/target/x86_64-unknown-linux-musl/release/rust-http-echo /usr/bin/rust-http-echo
cp /script/guest_scripts/start_rust_http_echo.sh /usr/bin/start_rust_http_echo.sh
chmod +x /usr/bin/rust-http-echo
chmod +x /usr/bin/start_rust_http_echo.sh

# Copy ip config script to /usr/bin
cp /script/guest_scripts/configure_vm_ip.sh /usr/bin/configure_vm_ip.sh
chmod +x /usr/bin/configure_vm_ip.sh

## configure SSH
# Allow pub ssh key
mkdir -p /root/.ssh
chmod 700 /root/.ssh
cp -v /script/output/id_rsa.pub /root/.ssh/authorized_keys
chmod 600 /root/.ssh/authorized_keys

## Configure services
# Copy service file for network setup
cp /script/guest_scripts/configure-network /etc/init.d/configure-network
chmod +x /etc/init.d/configure-network
# Copy service file for http
cp /script/guest_scripts/rust-http-echo /etc/init.d/rust-http-echo
chmod +x /etc/init.d/rust-http-echo

# Mark ssh to be started on boot
rc-update add sshd default
# Add both services to the default runlevel
rc-update add configure-network default
rc-update add rust-http-echo default

# Then, copy the newly configured system to the rootfs image:
for d in bin etc lib root sbin usr; do tar c "/$d" | tar x -C /my-rootfs; done
   
for dir in dev proc run sys var; do mkdir /my-rootfs/${dir}; done

# Create the logging directory for the service
mkdir -p /my-rootfs/var/log

# Copy the executable to the rootfs

exit