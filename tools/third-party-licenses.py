#!/usr/bin/env python3
"""Writes licenses/THIRD-PARTY.md: every crate the Mortise binary is built from,
on any of the three platforms, with its version and licence.

Run from the root of the repository after a dependency changed:

    python3 tools/third-party-licenses.py

It reads `cargo metadata`, which needs no network when the crates are in the
local registry. Crates that only tests use are left out.
"""

import json
import subprocess
import sys

OURS = {"mortise-core", "mortise-app"}


def main() -> int:
    raw = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    metadata = json.loads(raw)
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}

    # Walk the graph from the app, over normal and build dependencies only.
    start = [id for id, package in packages.items() if package["name"] == "mortise-app"]
    seen, queue = set(), list(start)
    while queue:
        id = queue.pop()
        if id in seen:
            continue
        seen.add(id)
        for dep in nodes[id]["deps"]:
            kinds = {kind["kind"] for kind in dep["dep_kinds"]}
            if kinds - {"dev"}:
                queue.append(dep["pkg"])

    rows = sorted(
        (
            package["name"],
            package["version"],
            package.get("license") or "see the crate",
            package.get("repository") or "",
        )
        for id, package in packages.items()
        if id in seen and package["name"] not in OURS
    )
    by_licence = {}
    for _, _, licence, _ in rows:
        by_licence[licence] = by_licence.get(licence, 0) + 1

    lines = [
        "# Third-party licences",
        "",
        "Mortise is built from the crates below. This list is written by",
        "`tools/third-party-licenses.py` from `Cargo.lock`; do not edit it by hand.",
        "It covers the crates of all three platforms, so a binary for one platform",
        "holds only a part of them.",
        "",
        "The fonts in `assets/fonts` are under the SIL Open Font License 1.1.",
        "Each folder there holds its `OFL.txt` with the copyright line.",
        "",
        f"{len(rows)} crates.",
        "",
        "## By licence",
        "",
        "| Licence | Crates |",
        "|---|---|",
    ]
    for licence, count in sorted(by_licence.items(), key=lambda item: (-item[1], item[0])):
        lines.append(f"| {licence} | {count} |")
    lines += ["", "## Crates", "", "| Crate | Version | Licence | Source |", "|---|---|---|---|"]
    for name, version, licence, repository in rows:
        lines.append(f"| {name} | {version} | {licence} | {repository} |")
    with open("licenses/THIRD-PARTY.md", "w", encoding="utf-8") as out:
        out.write("\n".join(lines) + "\n")
    print(f"{len(rows)} crates, {len(by_licence)} licence expressions")
    return 0


if __name__ == "__main__":
    sys.exit(main())
