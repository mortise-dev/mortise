#!/bin/sh
# Draws the README banner and the social preview from the pages in assets/brand.
#
#   sh tools/brand-images.sh
#
# Run it from the root of the repository after a page, the icon or the screenshot
# it shows changed. It needs Google Chrome; CHROME names another binary.
# It writes assets/brand/banner.png (1280 by 400 at scale 2) and
# assets/brand/social.png (1280 by 640, the size GitHub asks for).
set -eu

CHROME=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
BRAND=$(pwd)/assets/brand

draw() {
  "$CHROME" --headless=new --hide-scrollbars --disable-gpu --no-first-run \
    --allow-file-access-from-files --force-device-scale-factor="$4" \
    --window-size="$2" --screenshot="$BRAND/$3" "file://$BRAND/$1" 2>/dev/null
}

draw banner.html 1280,400 banner.png 2
draw social.html 1280,640 social.png 1
