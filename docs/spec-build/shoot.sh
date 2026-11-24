#!/bin/bash
# usage: shoot.sh page.html out.png [height] [dark]
# Reads page.html from docs/spec-build/out (written by build.mjs). Writes out.png only inside the repository.
. "$(dirname "${BASH_SOURCE[0]}")/paths.sh"
case "${3:-6000}" in ""|*[!0-9]*) echo "usage: shoot.sh page.html out.png [height in px] [dark]" >&2; exit 2 ;; esac
inside_root "$2"
mkdir -p "$(dirname "$REAL")"
"$CHROME" --headless=new --disable-gpu --hide-scrollbars "--window-size=1280,${3:-6000}" --virtual-time-budget=6000 --screenshot="$REAL" "$(file_url "$PAGES/$1")" 2>/dev/null
