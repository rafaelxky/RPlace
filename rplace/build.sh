#!/usr/bin/env bash
set -euo pipefail

cargo build --release
sudo install -Dm755 target/release/rplace /usr/local/bin/rplace

echo "Release build successful."