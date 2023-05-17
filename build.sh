#! /bin/sh
cargo build -r
cargo build -r --target=x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/whatson ~/git/dev-tools/whatson