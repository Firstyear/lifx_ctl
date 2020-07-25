#!/bin/sh
wasm-pack build --target web && \
    rollup ./main.js --format iife --file ./pkg/bundle.js && \
    cp ./pkg/bundle.js ../pkg/ && \
    cp ./pkg/lifx_wasm_bg.wasm ../pkg/

