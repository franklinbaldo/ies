#!/usr/bin/env python3
"""Recusa artefatos binários no índice Git, inclusive se o worktree divergir."""

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


def index_violations(cwd: pathlib.Path | None = None) -> list[str]:
    # Inspeciona blobs efetivamente preparados para commit, não o worktree.
    output = subprocess.check_output(
        ["git", "ls-files", "--stage", "-z"], cwd=cwd
    )
    violations = []
    for record in output.split(b"\x00"):
        if not record:
            continue
        metadata, raw_path = record.split(b"\t", 1)
        mode, sha, stage = metadata.split()
        path = raw_path.decode("utf-8", "surrogateescape")
        if stage != b"0":
            violations.append(path)  # índice em conflito não pode passar
            continue
        if mode == b"160000":  # gitlink, não é blob
            continue
        content = subprocess.check_output(
            ["git", "cat-file", "blob", sha.decode("ascii")], cwd=cwd
        )
        if is_binary_artifact(pathlib.PurePosixPath(path), content):
            violations.append(path)
    return violations


def main() -> int:
    violations = index_violations()
    if violations:
        print("ERRO: binários/artefatos compilados no índice Git:", file=sys.stderr)
        for path in violations:
            print(f"  - {path}", file=sys.stderr)
        print("Remova do índice com git rm --cached <arquivo>; artifacts ficam no CI.", file=sys.stderr)
        return 1
    print("OK: nenhum binário/artefato compilado no índice Git")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
