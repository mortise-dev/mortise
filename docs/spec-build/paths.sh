# Sourced by shoot.sh and offs.sh. Sets ROOT (the repository root) and PAGES (where build.mjs writes).
# The root comes from this file's own path, never from the current folder.
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
PAGES=$ROOT/docs/spec-build/out

# inside_root PATH: stop with status 1 unless PATH, with `..` and links resolved, is inside ROOT.
# PATH does not need to exist yet.
inside_root() {
  local real
  real=$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$1") || exit 1
  case "$real/" in
    "$ROOT"/*) ;;
    *) echo "Write inside the repository: $1 is outside $ROOT." >&2; exit 1 ;;
  esac
}

# The Chrome path is the macOS one. Set CHROME to use another browser on Linux or Windows.
CHROME=${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
