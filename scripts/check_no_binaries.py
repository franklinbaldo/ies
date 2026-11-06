#!/usr/bin/env python3
"""Recusa artefatos binários rastreados pelo Git (não afeta artifacts do CI)."""

from __future__ import annotations

import pathlib
import subprocess
import sys

ARTIFACT_DIRS = frozenset({"target", "build", "dist", "__pycache__", ".venv"})
ARTIFACT_SUFFIXES = frozenset({
    ".exe", ".dll", ".so", ".dylib", ".a", ".o", ".obj", ".lib",
    ".pdb", ".wasm", ".class", ".pyc", ".pyo", ".pyd",
})
BINARY_SIGNATURES = (
    b"\x7fELF", b"MZ", b"\x00asm", b"!<arch>\n",
    b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe",
    b"\xfe\xed\xfa\xcf", b"\xcf\xfa\xed\xfe",
    b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca",
)


def is_binary_artifact(path: pathlib.PurePath, content: bytes) -> bool:
    if any(part in ARTIFACT_DIRS for part in path.parts[:-1]):
        return True
    if path.suffix.lower() in ARTIFACT_SUFFIXES:
        return True
    if path.name.lower() in {"ies.exe", "ies.dll"}:
        return True
    if any(content.startswith(signature) for signature in BINARY_SIGNATURES):
        return True
    if b"\x00" in content:
        return True
    try:
        content.decode("utf-8")
    except UnicodeDecodeError:
        return True
    return False


def tracked_paths() -> list[pathlib.Path]:
    output = subprocess.check_output(["git", "ls-files", "--cached", "-z"])
    return [pathlib.Path(name.decode("utf-8", "surrogateescape"))
            for name in output.split(b"\x00") if name]


def main() -> int:
    violations = []
    for path in tracked_paths():
        if not path.is_file():  # arquivo removido no working tree
            continue
        if is_binary_artifact(path, path.read_bytes()):
            violations.append(str(path))
    if violations:
        print("ERRO: binários/artefatos compilados rastreados pelo Git:", file=sys.stderr)
        for path in violations:
            print(f"  - {path}", file=sys.stderr)
        print("Remova do índice com git rm --cached <arquivo>; preserve artifacts no CI.", file=sys.stderr)
        return 1
    print("OK: nenhum binário/artefato compilado rastreado pelo Git")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
