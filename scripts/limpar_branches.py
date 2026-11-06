#!/usr/bin/env python3
"""Limpeza conservadora das branches remotas já alcançáveis pela main.

Exige gh auth login e git push autenticado para origin. Padrão: dry-run.
--apply apaga somente refs integradas, não protegidas e sem PR aberta,
com lease atômico do SHA exato (não apaga push concorrente).
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



def delete_with_lease(name: str, expected_sha: str, *, cwd=None) -> None:
    """Remove apenas o SHA conferido; recusa push concorrente na mesma ref."""
    ref = f"refs/heads/{name}"
    subprocess.run(
        ["git", "push", "--porcelain",
         f"--force-with-lease={ref}:{expected_sha}", "origin", f":{ref}"],
        check=True, text=True, capture_output=True, cwd=cwd,
    )

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
            # Uma PR pode abrir durante a varredura; releia imediatamente.
            open_heads = {
                p["head"]["ref"] for p in paginar(f"{root}/pulls?state=open")
                if p["head"]["repo"] and p["head"]["repo"]["full_name"] == args.repo
            }
            if name in open_heads:
                print(f"SKIP (PR aberta recentemente): {name}")
                continue
            # A exclusão usa lease atômico no servidor. Se houve novo push,
            # a ref permanece intacta, mesmo depois da última consulta API.
            delete_with_lease(name, current["commit"]["sha"])
            print(f"REMOVIDA: {name}")
        else:
            print(f"CANDIDATA (dry-run): {name}")
        removidas += 1
    print(f"{'Removidas' if args.apply else 'Candidatas'}: {removidas}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
