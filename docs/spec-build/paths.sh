# Sourced by shoot.sh and offs.sh. Sets ROOT (the repository root) and PAGES (where build.mjs writes).
# The root comes from this file's own path, never from the current folder. Python resolves ROOT and
# every checked path, so both use one convention, also under Git Bash with a Windows Python.
# Each value is written with no line end: a Windows Python ends a line with CR LF, and Git Bash keeps the CR.
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && python3 -c 'import os, sys; sys.stdout.write(os.path.realpath(os.getcwd()))') || exit 1
PAGES=$ROOT/docs/spec-build/out

# inside_root PATH: stop with status 1 unless PATH, with `..` and links resolved, is inside ROOT.
# PATH does not need to exist yet. On success REAL holds the resolved path: use it, not PATH,
# to make folders and files, so no step walks the original path.
inside_root() {
  REAL=$(python3 - "$ROOT" "$1" <<'PY'
import os, sys
root, real = os.path.realpath(sys.argv[1]), os.path.realpath(sys.argv[2])
try:
    inside = os.path.commonpath([root, real]) == root
except ValueError:  # another drive on Windows
    inside = False
if not inside:
    sys.exit(f"Write inside the repository: {sys.argv[2]} is outside {root}.")
sys.stdout.write(real)
PY
) || exit 1
}

# file_url PATH: the file URL of PATH, percent-encoded, so a `#` or `?` in a folder name stays in the path.
file_url() {
  python3 -c 'import pathlib, sys; sys.stdout.write(pathlib.Path(sys.argv[1]).resolve().as_uri())' "$1"
}

# The Chrome path is the macOS one. Set CHROME to use another browser on Linux or Windows.
CHROME=${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
