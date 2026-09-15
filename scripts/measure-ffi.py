"""Measure comparable release shared libraries without cleaning the workspace.

Run: python scripts/measure-ffi.py [--offline]
Artifacts and JSON results are written under target/ffi-size/.
"""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    output = root / "target" / "ffi-size"
    output.mkdir(parents=True, exist_ok=True)
    variants = [
        ("nextjs-z", "anilist-nextjs-ffi", "anilist_nextjs_ffi", "z", []),
        (
            "parser-z",
            "anilist-ffi",
            "anilist",
            "z",
            ["--no-default-features", "--features", "youranimes"],
        ),
        (
            "parser-timezone-z",
            "anilist-ffi",
            "anilist",
            "z",
            [
                "--no-default-features",
                "--features",
                "youranimes,system-timezone",
            ],
        ),
        ("full-z", "anilist-ffi", "anilist", "z", []),
        (
            "parser-s",
            "anilist-ffi",
            "anilist",
            "s",
            ["--no-default-features", "--features", "youranimes"],
        ),
    ]
    measurements = []
    host = subprocess.check_output(["rustc", "-vV"], cwd=root, text=True).strip()
    for name, package, library, opt_level, features in variants:
        env = os.environ.copy()
        env["CARGO_PROFILE_RELEASE_OPT_LEVEL"] = opt_level
        # Keep measurements in this workspace, but honor Cargo's configured target.
        env["CARGO_TARGET_DIR"] = str(root / "target")
        command = [
            "cargo",
            "build",
            "--release",
            "--locked",
            "--message-format=json-render-diagnostics",
            "-p",
            package,
            *features,
        ]
        if args.offline:
            command.append("--offline")
        print(f"Building {name}", flush=True)
        result = subprocess.run(
            command,
            cwd=root,
            env=env,
            stdout=subprocess.PIPE,
            text=True,
            encoding="utf-8",
        )
        artifacts = []
        for line in result.stdout.splitlines():
            message = json.loads(line)
            if message.get("reason") == "compiler-message":
                print(message["message"].get("rendered", ""), end="")
            if (
                message.get("reason") == "compiler-artifact"
                and message["target"]["name"] == library
            ):
                artifacts.extend(
                    Path(path)
                    for path in message["filenames"]
                    if Path(path).suffix in {".dll", ".so", ".dylib"}
                )
        result.check_returncode()
        if len(artifacts) != 1:
            raise RuntimeError(
                f"Expected one shared-library artifact for {package}, got {artifacts}"
            )
        source = artifacts[0]
        destination = output / name / source.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        measurements.append(
            {
                "variant": name,
                "bytes": destination.stat().st_size,
                "artifact": str(destination),
                "cargo_artifact": str(source),
            }
        )
        (output / "measurements.json").write_text(
            json.dumps({"compiler": host, "measurements": measurements}, indent=2)
            + "\n",
            encoding="utf-8",
        )
    print(json.dumps(measurements, indent=2))


if __name__ == "__main__":
    main()
