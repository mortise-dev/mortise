#!/bin/bash
# usage: shoot.sh page.html out.png [height] [dark]
# Reads page.html from docs/spec-build/out (written by build.mjs). Writes out.png only inside the repository.
. "$(dirname "${BASH_SOURCE[0]}")/paths.sh"
inside_root "$2"
mkdir -p "$(dirname "$2")"
"$CHROME" --headless=new --disable-gpu --hide-scrollbars --window-size=1280,${3:-6000} --virtual-time-budget=6000 --screenshot="$2" "file://$PAGES/$1" 2>/dev/null
