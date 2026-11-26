#!/usr/bin/env bash
# Checks the command line tool built for one target by running it, so the job must run on the platform
# of the target: the binary prints the version of Cargo.toml, exits 1 on the fixture shop-api, and on
# Linux is statically linked. The Installers workflow runs it after
# `cargo build --release -p mortise-cli --target <target>`, before tools/pack-cli.sh. A release runs it
# again after signing. It never runs in a step that holds a secret.
#
#   tools/check-cli.sh <target>
#   tools/check-cli.sh x86_64-unknown-linux-musl
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <target>" >&2
  exit 2
fi
target=$1

case "$target" in
  *windows*) binary=mortise.exe ;;
  *) binary=mortise ;;
esac
built="target/$target/release/$binary"
if [ ! -f "$built" ]; then
  echo "::error::$built does not exist. Build it first: cargo build --release --locked -p mortise-cli --target $target"
  exit 1
fi

# tools/release.sh sets the version on the first `version = ` line of Cargo.toml, in [workspace.package].
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
printed=$("$built" --version)
if [ "$printed" != "mortise $version" ]; then
  echo "::error::$built --version printed '$printed', not 'mortise $version'."
  exit 1
fi
status=0
"$built" check fixtures/shop-api > /dev/null || status=$?
if [ "$status" -ne 1 ]; then
  echo "::error::$built check fixtures/shop-api exited $status, not 1."
  exit 1
fi
case "$target" in
  *linux*)
    linked=$(file -b "$built")
    if ! grep -Eq 'static(-pie)?(ally)? linked' <<<"$linked"; then
      echo "::error::$built is not statically linked: $linked"
      exit 1
    fi
    ;;
esac
echo "$built: $printed, checked."
