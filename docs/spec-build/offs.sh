#!/bin/bash
# usage: offs.sh page  -> prints heading offsets of docs/spec-build/out/page.html
# Writes one temporary folder inside docs/spec-build/out and removes only that folder on exit.
. "$(dirname "${BASH_SOURCE[0]}")/paths.sh"
case "$1" in ""|*/*) echo "usage: offs.sh page (a name, no folder)" >&2; exit 2 ;; esac
[ -f "$PAGES/$1.html" ] || { echo "No page: $PAGES/$1.html. Run build.mjs first." >&2; exit 1; }
inside_root "$PAGES"
TMPDIR_=$(mktemp -d "$REAL/tmp-XXXXXX") || exit 1
trap 'rm -rf "$TMPDIR_"' EXIT
TMP=$TMPDIR_/page.html
sed 's#</body>#<script>window.addEventListener("load",function(){var o={};document.querySelectorAll("h2,h3,h4").forEach(function(h){o[h.textContent.slice(0,28)]=Math.round(h.getBoundingClientRect().top+scrollY)});o.END=document.documentElement.scrollHeight;o.SW=document.documentElement.scrollWidth;var p=document.createElement("pre");p.id="OFFS";p.textContent=JSON.stringify(o);document.body.appendChild(p)})</script></body>#' "$PAGES/$1.html" > "$TMP"
"$CHROME" --headless=new --disable-gpu --window-size=1280,1000 --virtual-time-budget=6000 --dump-dom "file://$TMP" 2>/dev/null | grep -o '<pre id="OFFS">[^<]*' | sed 's/<pre id="OFFS">//'
