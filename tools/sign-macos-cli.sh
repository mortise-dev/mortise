#!/usr/bin/env bash
# Signs the macOS command line tool with the Developer ID Application certificate, packs it into
# dist/<archive> with tools/pack-cli.sh, has Apple notarize that zip, and checks the signature of the
# binary inside it. The Installers workflow runs it in a release only, in the step that holds the
# Apple secrets, after the binary was built and checked by tools/check-cli.sh in earlier steps.
#
# It never runs the built binary, so no code of the build runs while the secrets are in the
# environment. Only codesign, notarytool and the system tools run here. tools/check-cli.sh runs the
# signed binary again in a later step, without the secrets.
#
#   tools/sign-macos-cli.sh <archive>
#   tools/sign-macos-cli.sh mortise-v0.3.0-macos-arm64.zip
#
# Environment: APPLE_CERTIFICATE (the .p12 in base64), APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY,
# APPLE_API_ISSUER, APPLE_API_KEY (the key ID) and APPLE_API_KEY_P8 (the contents of the .p8 file).
#
# A bare Mach-O binary cannot hold a stapled ticket, so Gatekeeper looks the ticket up online the first
# time a downloaded binary runs. The certificate lives in a temporary keychain and the API key in a
# temporary file; both are deleted when the script ends.
set -euo pipefail

if [ "$#" -ne 1 ] || [[ $1 != *.zip ]]; then
  echo "usage: $0 <archive name ending in .zip>" >&2
  exit 2
fi
archive=$1
target=aarch64-apple-darwin
binary="target/$target/release/mortise"

missing=''
for name in APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_P8; do
  if [ -z "$(printf '%s' "${!name:-}" | tr -d '[:space:]')" ]; then missing="$missing $name"; fi
done
if [ -n "$missing" ]; then
  echo "::error::A release must sign and notarize the command line tool, and these secrets are empty or not set:$missing. Add them as environment secrets of the release environment, as the macOS signing section of CONTRIBUTING.md says."
  exit 1
fi
case "$APPLE_API_KEY" in
  *[!A-Za-z0-9]*)
    echo '::error::APPLE_API_KEY must be the key ID, letters and digits only.'
    exit 1
    ;;
esac
if [ ! -f "$binary" ]; then
  echo "::error::$binary does not exist. Build it first: cargo build --release --locked -p mortise-cli --target $target"
  exit 1
fi

work=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/sign-cli.XXXXXX")
keychain="$work/signing.keychain-db"
searched=()
while IFS= read -r line; do
  line=${line#"${line%%[![:space:]]*}"}
  line=${line#\"}
  line=${line%\"}
  if [ -n "$line" ]; then searched+=("$line"); fi
done < <(security list-keychains -d user)

# A step that times out gets SIGTERM, which skips an EXIT trap in some shells, so trap it too.
cleanup() {
  if [ "${#searched[@]}" -gt 0 ]; then security list-keychains -d user -s "${searched[@]}" || true; fi
  security delete-keychain "$keychain" 2> /dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT
trap 'cleanup; exit 143' TERM
trap 'cleanup; exit 130' INT

echo "== Import the certificate into a temporary keychain"
password=$(openssl rand -hex 24)
(umask 077; printf '%s' "$APPLE_CERTIFICATE" | base64 --decode > "$work/certificate.p12")
security create-keychain -p "$password" "$keychain"
security set-keychain-settings -lut 3600 "$keychain"
security unlock-keychain -p "$password" "$keychain"
security import "$work/certificate.p12" -k "$keychain" -f pkcs12 -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign > /dev/null
rm -f "$work/certificate.p12"
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" > /dev/null
security list-keychains -d user -s "$keychain" "${searched[@]}"

echo "== Sign $binary"
codesign --force --options runtime --timestamp --keychain "$keychain" --sign "$APPLE_SIGNING_IDENTITY" "$binary"

echo "== Pack dist/$archive"
tools/pack-cli.sh "$target" "$archive"

echo "== Notarize dist/$archive"
key="$work/AuthKey_$APPLE_API_KEY.p8"
(umask 077; printf '%s' "$APPLE_API_KEY_P8" > "$key")
auth=(--key "$key" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER")
result=$(xcrun notarytool submit "dist/$archive" "${auth[@]}" --wait --timeout 45m --output-format json)
echo "$result"
id=$(jq -r '.id // empty' <<<"$result")
status=$(jq -r '.status // empty' <<<"$result")
if [ "$status" != Accepted ]; then
  if [ -n "$id" ]; then xcrun notarytool log "$id" "${auth[@]}" || true; fi
  echo "::error::Apple did not accept dist/$archive for notarization: status '$status'. The log above says why."
  exit 1
fi

echo "== Check the binary in dist/$archive"
unzip -q "dist/$archive" -d "$work/check"
checked="$work/check/mortise"
codesign --verify --strict --verbose=2 "$checked"
details=$(codesign -dv --verbose=4 "$checked" 2>&1)
echo "$details"
fail() {
  echo "::error::dist/$archive: $1"
  exit 1
}
grep -q '^Authority=Developer ID Application: ' <<<"$details" || fail "the binary is not signed with a Developer ID Application certificate."
grep -Eq '^CodeDirectory .*flags=0x[0-9a-f]+\([^)]*runtime' <<<"$details" || fail "the hardened runtime is off."
grep -q '^Timestamp=' <<<"$details" || fail "the signature has no secure timestamp."
cmp -s "$checked" "$binary" || fail "the binary in the zip is not the one signed."

echo "dist/$archive: signed, and notarized by Apple (not stapled: a bare binary cannot hold a ticket)."
