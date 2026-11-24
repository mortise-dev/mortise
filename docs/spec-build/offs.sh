#!/bin/bash
# usage: offs.sh page  -> prints heading offsets of docs/spec-build/out/page.html
# Writes one temporary file in that folder and removes it on exit.
. "$(dirname "${BASH_SOURCE[0]}")/paths.sh"
TMP=$PAGES/tmp-$1.html
inside_root "$TMP"
mkdir -p "$PAGES"
trap 'rm -f "$TMP"' EXIT
sed 's#</body>#<script>window.addEventListener("load",function(){var o={};document.querySelectorAll("h2,h3,h4").forEach(function(h){o[h.textContent.slice(0,28)]=Math.round(h.getBoundingClientRect().top+scrollY)});o.END=document.documentElement.scrollHeight;o.SW=document.documentElement.scrollWidth;var p=document.createElement("pre");p.id="OFFS";p.textContent=JSON.stringify(o);document.body.appendChild(p)})</script></body>#' "$PAGES/$1.html" > "$TMP"
"$CHROME" --headless=new --disable-gpu --window-size=1280,1000 --virtual-time-budget=6000 --dump-dom "file://$TMP" 2>/dev/null | grep -o '<pre id="OFFS">[^<]*' | sed 's/<pre id="OFFS">//'
