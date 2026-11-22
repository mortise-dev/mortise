#!/bin/bash
# usage: offs.sh page  -> prints heading offsets, captures full page
OUT=/Users/nicolaswidart/Sites/php/packages/LaravelModules/docs/companion-app/design-fable
D=$(pwd)/shots
sed 's#</body>#<script>window.addEventListener("load",function(){var o={};document.querySelectorAll("h2,h3,h4").forEach(function(h){o[h.textContent.slice(0,28)]=Math.round(h.getBoundingClientRect().top+scrollY)});o.END=document.documentElement.scrollHeight;o.SW=document.documentElement.scrollWidth;var p=document.createElement("pre");p.id="OFFS";p.textContent=JSON.stringify(o);document.body.appendChild(p)})</script></body>#' $OUT/$1.html > $D/tmp-$1.html
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu --window-size=1280,1000 --virtual-time-budget=6000 --dump-dom "file://$D/tmp-$1.html" 2>/dev/null | grep -o '<pre id="OFFS">[^<]*' | sed 's/<pre id="OFFS">//'
