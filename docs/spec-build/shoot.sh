#!/bin/bash
# usage: shoot.sh page.html out.png [height] [dark]
OUT=/Users/nicolaswidart/Sites/php/packages/LaravelModules/docs/companion-app/design-fable
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu --hide-scrollbars --window-size=1280,${3:-6000} --virtual-time-budget=6000 --screenshot="$2" "file://$OUT/$1" 2>/dev/null
