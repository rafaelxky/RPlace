#!/usr/bin/env bash

cargo build --release
sudo cp target/release/rplace /usr/local/bin/

echo "release build successfull, you can use the program with \"rplace <source> <target>\""
