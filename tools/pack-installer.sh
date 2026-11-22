#!/usr/bin/env bash
# Moves the one installer cargo-packager built into dist/ under its release name.
# The Installers workflow runs it after `cargo packager`.
#
#   tools/pack-installer.sh '<glob in target/packages>' <release name>
#   tools/pack-installer.sh '*.dmg' Mortise-v0.1.0-macos-arm64.dmg
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 '<glob in target/packages>' <release name>" >&2
  exit 2
fi
built=$1
name=$2

# bash 3.2 on macOS has no mapfile; compgen exits 1 on no match.
files=()
while IFS= read -r f; do
  if [ -n "$f" ]; then files+=("$f"); fi
done < <(compgen -G "target/packages/$built" || true)
if [ "${#files[@]}" -ne 1 ]; then
  echo "::error::Expected one $built in target/packages, found ${#files[@]}."
  exit 1
fi
mkdir -p dist
mv "${files[0]}" "dist/$name"
echo "dist/$name"
