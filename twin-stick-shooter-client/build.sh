#!/bin/bash
set -e

wasm-pack build --target web --no-typescript "$@"
cp -v pkg/twin_stick_shooter_client{_bg.wasm,.js} www/
