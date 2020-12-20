#!/bin/bash
set -e

(cd twin-stick-shooter-client && ./build.sh "$@")
cargo run --bin twin-stick-shooter-server -- \
    --http-listen-addr 0.0.0.0:8080 \
    --webrtc-listen-addr 0.0.0.0:8081 \
    --webrtc-public-addr 10.0.1.103:8081 \
    --static-content-path twin-stick-shooter-client/www
