#!/usr/bin/env bash
# Packs the command line tool built for one target into dist/<archive>, with LICENSE and
# THIRD-PARTY.md beside the binary at the top level. It never runs the binary: tools/check-cli.sh does
# that, in a step without secrets. The Installers workflow runs it after tools/check-cli.sh, and
# tools/sign-macos-cli.sh after signing.
#
#   tools/pack-cli.sh <target> <archive>
#   tools/pack-cli.sh x86_64-unknown-linux-musl mortise-v0.3.0-linux-x86_64.tar.gz
#
# The archive is a file name, with no folder, that ends in .tar.gz or .zip. A zip is made with zip, or
# with 7z where zip is missing (the Windows runner).
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <target> <archive name ending in .tar.gz or .zip>" >&2
  exit 2
fi
target=$1
archive=$2

# The name is checked before anything is written or removed, so no name reaches outside dist/.
case "$archive" in
  */* | *\\* | . | ..)
    echo "::error::The archive '$archive' must be a file name in dist/, with no / or \\."
    exit 2
    ;;
  ?*.tar.gz | ?*.zip) ;;
  *)
    echo "::error::The archive '$archive' must end in .tar.gz or .zip."
    exit 2
    ;;
esac

case "$target" in
  *windows*) binary=mortise.exe ;;
  *) binary=mortise ;;
esac
built="target/$target/release/$binary"
if [ ! -f "$built" ]; then
  echo "::error::$built does not exist. Build it first: cargo build --release --locked -p mortise-cli --target $target"
  exit 1
fi

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
esac
echo "dist/$archive"
