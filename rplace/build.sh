#!/usr/bin/env bash

cargo build --release
sudo cp target/release/rplace /usr/local/bin/
