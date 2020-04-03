#!/usr/bin/env bash

[ -d ./tools ] || (echo "Run this script for the repo root" >&2; exit 1)

cargo readme -r ibuilder > README.md
cargo readme -r ibuilder_derive > ibuilder_derive/README.md
