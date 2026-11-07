#!/usr/bin/env python3
"""Regenerate Flatpak's offline registry sources from the committed lockfile."""
import json
from pathlib import Path
import tomllib

root = Path(__file__).resolve().parent.parent
sources = []
for package in tomllib.loads((root / "Cargo.lock").read_text())["package"]:
    if "source" not in package:
        continue
    if package["source"] != "registry+https://github.com/rust-lang/crates.io-index":
        raise SystemExit(f"Unsupported Cargo source: {package['source']}")
    name, version, checksum = (package[k] for k in ("name", "version", "checksum"))
    destination = f"cargo/vendor/{name}-{version}"
    sources.extend([
        {
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": destination,
        },
        {
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": destination,
            "dest-filename": ".cargo-checksum.json",
        },
    ])
sources.append({
    "type": "inline",
    "contents": '[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "cargo/vendor"\n',
    "dest": "cargo",
    "dest-filename": "config.toml",
})
(root / "flatpak/cargo-sources.json").write_text(json.dumps(sources, indent=2) + "\n")
print(f"{len(sources) // 2} crates locked for offline Flatpak builds")
