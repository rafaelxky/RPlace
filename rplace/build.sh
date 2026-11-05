#!/usr/bin/env bash
set -euo pipefail

cargo build --workspace --release
sudo install -Dm755 target/release/rplace /usr/local/bin/rplace

# addons
addons_dir="${XDG_DATA_HOME:-$HOME/.local/share}/rplace/addons"
# lua
install -Dm755 target/release/librplace_lua.so "$addons_dir/librplace_lua.so"
# regex
install -Dm755 target/release/librplace_regex.so "$addons_dir/librplace_regex.so"
# fuzzy syntax
install -Dm755 target/release/librplace_fuzzy_sintax.so "$addons_dir/librplace_fuzzy_sintax.so"

echo "Release build successful."