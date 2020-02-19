#!/bin/bash
set -ev

if [ "$FMT" = true ]; then
  cargo fmt --version || rustup component add rustfmt
fi

if [ "$CHECK" = true ]; then
  cargo clippy --version || rustup component add clippy
fi