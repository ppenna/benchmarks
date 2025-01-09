#!/usr/bin/env bash
SIZE_IN_MB="50"
ROOTFS_FILENAME="rootfs.ext4"
TMP_MOUNT="/tmp/my-rootfs"

SCRIPT_DIR=$(dirname $(realpath $0))

# Compile the http server
pushd $SCRIPT_DIR/rust-http-server/
cargo build --release --target x86_64-unknown-linux-musl

# Set the output dir to the directory where this script is located 
OUTPUT_DIR=$SCRIPT_DIR/output

mkdir -p $OUTPUT_DIR
pushd $OUTPUT_DIR

# Create ssh key
ssh-keygen -t rsa -f id_rsa -N ""
mv -v id_rsa ./rust-http-echo.id_rsa

dd if=/dev/zero of=${ROOTFS_FILENAME} bs=1M count=${SIZE_IN_MB}
mkfs.ext4 ${ROOTFS_FILENAME} 

mkdir ${TMP_MOUNT}
sudo mount ${ROOTFS_FILENAME} ${TMP_MOUNT} 

# Run docker container to configure the rootfs, this should mount both the filesystem and the script 
# directory, and run the configure_rootfs_in_container.sh file in the container
docker run --rm -v ${TMP_MOUNT}:/my-rootfs -v $SCRIPT_DIR:/script alpine:latest /script/guest_scripts/configure_rootfs_in_container.sh

sudo umount ${TMP_MOUNT} 

echo "Rootfs generated at ${OUTPUT_DIR}/${ROOTFS_FILENAME}"

popd
popd