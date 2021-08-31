#!/bin/bash
# In order to use this script run `. ./devsetup/setup.sh` from the project directory

# set this to whichever latest version of the optimizer is
OPTIMIZER_VERSION="0.11.4"

alias rust-optimizer='docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:${OPTIMIZER_VERSION}'

alias workspace-optimizer='docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:${OPTIMIZER_VERSION}'

alias terrad='../LocalTerra/terracore/terrad'