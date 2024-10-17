#!/usr/bin/env bash

# Delete all the current wasms first
rm -rf ./artifacts/*.wasm

if [[ $(arch) == "arm64" ]]; then
  image="cosmwasm/optimizer-arm64:0.16.1"
else
  image="cosmwasm/optimizer:0.16.1"
fi

# Optimized builds
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  ${image}
