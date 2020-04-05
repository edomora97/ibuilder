#!/usr/bin/env bash

[ -d ./tools ] || (echo "Run this script for the repo root" >&2; exit 1)
[ $# -lt 1 ] && (echo "Usage: $0 version" >&2; exit 1)
if [[ "$1" == v* ]]; then
  echo "The version should not start with v" >&2
  exit 1
fi

version="$1"

echo "Updating version of ibuilder_derive to $version"
sed -i "s/^version =.*/version = \"${version}\"/" "ibuilder_derive/Cargo.toml"

echo "Updating version of ibuilder to $version"
sed -i "s/^version =.*/version = \"${version}\"/" "ibuilder/Cargo.toml"
sed -Ei "s/(ibuilder_derive.+version = )\"[^\"]+\"(.*)/\\1\"${version}\"\\2/" "ibuilder/Cargo.toml"

echo "Running cargo test"
cargo test --all

echo "Committing changes"
git add ibuilder/Cargo.toml ibuilder_derive/Cargo.toml
git commit -m "Version v${version}"

echo "Publishing ibuilder_derive"
(cd ibuilder_derive && cargo publish)

echo "Waiting a bit for crates.io"
found=no
for i in {1..20}; do
  echo "Attempt $i"
  actual=$(cargo search ibuilder_derive | head -n 1 | cut -d'"' -f 2)
  echo "crates.io reports version ${actual}"
  if [[ $actual == "${version}" ]]; then
    found=yes
    break
  fi
  echo "attempting again in 5s"
  sleep 5s
done
if [[ $found == no ]]; then
  echo "crates.io hasn't updated the version :c"
fi

echo "Publishing ibuilder"
(cd ibuilder && cargo publish)
