#!/bin/sh

version=$1

# Update the version in `Cargo.toml`.
cargo set-version "$version"

# Update the version in `README.md` assuming the following format:
# tower-surf = "0.0.0"
sd 'tower-surf = "\d+\.\d+\.\d+"' "tower-surf = \"$version\"" README.md
