#!/usr/bin/env bash

cd rplace 
cargo run -- package create-user 111 111 111
cargo run -- package login 111 111
cargo run -- package new
cargo run -- package push