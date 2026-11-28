#!/usr/bin/env bash
# Signs the three files a window updates from with the update key, checks each signature against the
# public key the app is built with, and writes the update manifest dist/latest.json.
# The Installers workflow runs it in a release, in the one job that holds the update key.
#
#   tools/sign-updates.sh <tag>
#   tools/sign-updates.sh v1.2.0
#
# dist/ must hold, for the tag:
#   Mortise-<tag>-macos-arm64.app.tar.gz   the signed and notarized app, packed
#   Mortise-<tag>-windows-x64-setup.exe
#   Mortise-<tag>-linux-x86_64.AppImage
#
# Environment: CARGO_PACKAGER_SIGN_PRIVATE_KEY (the private key, as `cargo packager signer generate`
# wrote it) and CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD (its password, empty if it has none).
# Needs cargo-packager, minisign and jq.
#
# A window installs a download only when its signature verifies with the public key in
# crates/mortise-app/src/update/mod.rs. A release signed with another key would be refused by every
# window, so this script fails before anything is published.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <tag>" >&2
  exit 2
fi
tag=$1
if ! [[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "::error::Tag '$tag' is not vX.Y.Z."
  exit 1
fi

if [ -z "$(printf '%s' "${CARGO_PACKAGER_SIGN_PRIVATE_KEY:-}" | tr -d '[:space:]')" ]; then
  echo "::error::A release must sign its updates, and the secret CARGO_PACKAGER_SIGN_PRIVATE_KEY is empty or not set. Add it as an environment secret of the release environment, as the update key section of CONTRIBUTING.md says."
  exit 1
fi

source=crates/mortise-app/src/update/mod.rs
public_key=$(sed -n 's/^const PUBLIC_KEY: &str = "\(.*\)";$/\1/p' "$source")
if [ -z "$public_key" ]; then
  echo "::error::$source holds no public update key on a const PUBLIC_KEY line."
  exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf '%s' "$public_key" | base64 -d > "$work/update.pub"

# The key of the manifest is <system>-<architecture>, as cargo-packager-updater names them.
platforms=(
  "macos-aarch64 app Mortise-$tag-macos-arm64.app.tar.gz"
  "windows-x86_64 nsis Mortise-$tag-windows-x64-setup.exe"
  "linux-x86_64 appimage Mortise-$tag-linux-x86_64.AppImage"
)

manifest=$(jq -n --arg version "${tag#v}" --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{version: $version, pub_date: $date, platforms: {}}')
for platform in "${platforms[@]}"; do
  read -r target format file <<<"$platform"
  if [ ! -f "dist/$file" ]; then
    echo "::error::dist/$file is missing."
    exit 1
  fi
  cargo packager signer sign "dist/$file" > /dev/null
  signature=$(cat "dist/$file.sig")
  rm "dist/$file.sig"
  printf '%s' "$signature" | base64 -d > "$work/signature"
  if ! minisign -V -q -m "dist/$file" -p "$work/update.pub" -x "$work/signature"; then
    echo "::error::The signature of $file does not verify with the public key in $source. The update key of the release environment is not the key the app is built with."
    exit 1
  fi
  manifest=$(jq --arg target "$target" --arg format "$format" --arg signature "$signature" \
    --arg url "https://github.com/mortise-dev/mortise/releases/download/$tag/$file" \
    '.platforms[$target] = {url: $url, signature: $signature, format: $format}' <<<"$manifest")
  echo "$file: signed for $target."
done

printf '%s\n' "$manifest" > dist/latest.json
echo "dist/latest.json"
