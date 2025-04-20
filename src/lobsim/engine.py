"""Python side of the Rust exchange: runs the command line binary and parses its output."""

from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "engine" / "target" / "release" / "lobsim"


def available() -> bool:
    return BINARY.exists()


def _value(text: str):
    try:
        return float(text)
    except ValueError:
        return text


def call(command: str, **params) -> dict:
    """`call("execute", schedule="twap", size=200)` -> {"mean_shortfall": ..., "rows": [[...], ...]}"""
    args = [f"{k}={v}" for k, v in params.items()]
    out = subprocess.run([str(BINARY), command, *args], capture_output=True, text=True, check=True).stdout
    res: dict = {"rows": []}
    for line in out.splitlines():
        f = line.split()
        if f[0] == "row":
            res["rows"].append([_value(x) for x in f[1:]])
        else:
            res[f[0]] = _value(f[1])
    return res
