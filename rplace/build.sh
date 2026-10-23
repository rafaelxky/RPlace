#!/usr/bin/env bash
set -euo pipefail

cargo build --workspace --release
sudo install -Dm755 target/release/rplace /usr/local/bin/rplace

# addons_dir="${XDG_DATA_HOME:-$HOME/.local/share}/rplace/addons"
# install -Dm755 target/release/librplace_lua.so "$addons_dir/librplace_lua.so"
# install -Dm755 target/release/librplace_regex.so "$addons_dir/librplace_regex.so"

echo "Release build successful."