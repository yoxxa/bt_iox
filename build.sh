#!/bin/bash
docker buildx build --platform linux/arm64 -t bt_iox .
docker save -o build/rootfs.tar bt_iox
./ioxclient package ./build
