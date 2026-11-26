#!/usr/bin/env bash
# Checks the command line tool built for one target, then packs it into dist/<archive>, with LICENSE
# and THIRD-PARTY.md beside the binary at the top level. The check runs the binary, so the job must
# run on the platform of the target: it prints the version of Cargo.toml, exits 1 on the fixture
# shop-api, and on Linux is statically linked. The Installers workflow runs it after
# `cargo build --release -p mortise-cli --target <target>`, and tools/sign-macos-cli.sh after signing.
#
#   tools/pack-cli.sh <target> <archive>
#   tools/pack-cli.sh x86_64-unknown-linux-musl mortise-v0.3.0-linux-x86_64.tar.gz
#
# The archive name ends in .tar.gz or .zip. A zip is made with zip, or with 7z where zip is missing
# (the Windows runner).
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <target> <archive name ending in .tar.gz or .zip>" >&2
  exit 2
fi
target=$1
archive=$2

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

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp "$built" "$stage/$binary"
cp LICENSE "$stage/LICENSE"
cp licenses/THIRD-PARTY.md "$stage/THIRD-PARTY.md"

mkdir -p dist
out="$PWD/dist/$archive"
rm -f "$out"
case "$archive" in
  *.tar.gz)
    tar -czf "$out" -C "$stage" "$binary" LICENSE THIRD-PARTY.md
    ;;
  *.zip)
    if command -v zip > /dev/null; then
      (cd "$stage" && zip -q -X "$out" "$binary" LICENSE THIRD-PARTY.md)
    else
      (cd "$stage" && 7z a -tzip -bso0 -bsp0 "$out" "$binary" LICENSE THIRD-PARTY.md)
    fi
    ;;
  *)
    echo "::error::$archive must end in .tar.gz or .zip."
    exit 2
    ;;
esac
echo "dist/$archive"
