#!/usr/bin/env python3
"""Limpeza conservadora das branches remotas já alcançáveis pela main.

Exige gh auth login. Padrão: apenas simula. --apply apaga branches
integradas, sem PR aberta, não protegidas e com SHA estável na releitura.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from urllib.parse import quote


def api(path: str, *, delete: bool = False):
    cmd = ["gh", "api"]
    if delete:
        cmd += ["--method", "DELETE"]
    cmd.append(path)
    output = subprocess.check_output(cmd, text=True)
    return json.loads(output) if output.strip() else None


def paginar(path: str):
    page = 1
    while True:
        items = api(f"{path}{'&' if '?' in path else '?'}per_page=100&page={page}")
        if not items:
            break
        yield from items
        if len(items) < 100:
            break
        page += 1


def elegivel(branch: dict, main_sha: str, refs_de_pr: set[str], ahead_by: int) -> bool:
    return (
        branch["name"] != "main"
        and not branch.get("protected", False)
        and branch["name"] not in refs_de_pr
        and ahead_by == 0
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default="franklinbaldo/ies")
    parser.add_argument("--apply", action="store_true", help="apagar (padrão: dry-run)")
    args = parser.parse_args()
    root = f"repos/{args.repo}"
    repo = api(root)
    base = repo["default_branch"]
    if base != "main":
        raise SystemExit(f"Recusado: branch padrão mudou para {base!r}")
    main_sha = api(f"{root}/branches/main")["commit"]["sha"]
    refs = {
        p["head"]["ref"] for p in paginar(f"{root}/pulls?state=open")
        if p["head"]["repo"] and p["head"]["repo"]["full_name"] == args.repo
    }
    removidas = 0
    for branch in paginar(f"{root}/branches"):
        name = branch["name"]
        if name == "main" or branch.get("protected") or name in refs:
            continue
        quoted = quote(name, safe="")
        comparison = api(f"{root}/compare/main...{quoted}")
        if not elegivel(branch, main_sha, refs, comparison["ahead_by"]):
            continue
        current = api(f"{root}/branches/{quoted}")
        if current["commit"]["sha"] != branch["commit"]["sha"]:
            print(f"SKIP (alterada durante conferência): {name}")
            continue
        if args.apply:
            api(f"{root}/git/refs/heads/{quoted}", delete=True)
            print(f"REMOVIDA: {name}")
        else:
            print(f"CANDIDATA (dry-run): {name}")
        removidas += 1
    print(f"{'Removidas' if args.apply else 'Candidatas'}: {removidas}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
