#!/usr/bin/env python3
"""Prepare the pinned Reactor dependency. Requires Python 3.12+ and Git."""

import hashlib
import io
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
VERSION = "0.100.0"
SHA256 = "5e3f0c06995fb2b791584562e32c76d4fed2241d0460a82c3fcdf30a32beadc3"
ARCHIVE = f"windows-reactor-{VERSION}.crate"
URL = f"https://static.crates.io/crates/windows-reactor/{ARCHIVE}"
PATCH = ROOT / "patches/windows-reactor/query-submitted.patch"
DESTINATION = ROOT / "vendor/windows-reactor"


def setup():
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    cached = sorted((cargo_home / "registry/cache").glob(f"*/{ARCHIVE}"))
    if cached:
        archive = cached[0].read_bytes()
    else:
        print(f"Downloading {URL}", flush=True)
        with urllib.request.urlopen(URL, timeout=60) as response:
            archive = response.read()
    if hashlib.sha256(archive).hexdigest() != SHA256:
        raise RuntimeError("Reactor archive checksum mismatch; no files were installed")

    # Stage outside the checkout so Git applies the patch without repository prefixes.
    with tempfile.TemporaryDirectory(prefix="anilist-reactor-") as temporary:
        staging = Path(temporary)
        with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as package:
            package.extractall(staging, filter="data")
        source = staging / f"windows-reactor-{VERSION}"
        subprocess.run(["git", "apply", "--check", str(PATCH)], cwd=source, check=True)
        subprocess.run(["git", "apply", "--whitespace=nowarn", str(PATCH)], cwd=source, check=True)

        if DESTINATION.exists():
            # Preserve local edits; never silently replace an existing dependency.
            for expected in source.rglob("*"):
                if not expected.is_file() or expected.name.startswith(".cargo"):
                    continue
                actual = DESTINATION / expected.relative_to(source)
                if not actual.is_file() or actual.read_bytes() != expected.read_bytes():
                    raise RuntimeError(
                        f"Existing dependency differs: {actual}. "
                        "Move vendor/windows-reactor aside and rerun setup."
                    )
            print(f"Reactor {VERSION} with QuerySubmitted is already prepared")
            return

        DESTINATION.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(source), str(DESTINATION))
        print(f"Prepared Reactor {VERSION} with QuerySubmitted at {DESTINATION}")


if __name__ == "__main__":
    setup()
