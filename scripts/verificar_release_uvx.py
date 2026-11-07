"""Valida uma tag de publicacao Rust/uvx antes de distribuir wheels (#346)."""

import re
import sys
import tomllib
from pathlib import Path


def conferir_tag(tag: str, raiz: Path) -> str:
    if re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag) is None:
        raise ValueError("tag de release deve ter forma vMAJOR.MINOR.PATCH")

    with (raiz / "Cargo.toml").open("rb") as arquivo:
        cargo = tomllib.load(arquivo)
    with (raiz / "pyproject.toml").open("rb") as arquivo:
        projeto = tomllib.load(arquivo)

    with (raiz / "Cargo.lock").open("rb") as arquivo:
        lock = tomllib.load(arquivo)

    versao_rust = cargo["package"]["version"]
    versao_python = projeto["project"]["version"]
    raiz_ies = [p["version"] for p in lock["package"] if p["name"] == "ies"]
    if len(raiz_ies) != 1:
        raise ValueError("Cargo.lock deve conter um pacote ies")
    if raiz_ies[0] != versao_rust:
        raise ValueError("Cargo.lock diverge da versao do IES")
    if projeto["project"]["name"] != "ies-sei":
        raise ValueError("nome de distribuicao Python diferente de ies-sei")
    if versao_rust != versao_python or tag != f"v{versao_rust}":
        raise ValueError(
            f"tag {tag!r} difere das versoes Cargo ({versao_rust}) "
            f"e pyproject ({versao_python})"
        )
    return versao_rust


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print("uso: verificar_release_uvx.py TAG [RAIZ]", file=sys.stderr)
        return 2
    raiz = Path(sys.argv[2]) if len(sys.argv) == 3 else Path(__file__).resolve().parents[1]
    try:
        versao = conferir_tag(sys.argv[1], raiz)
    except (ValueError, OSError, KeyError, tomllib.TOMLDecodeError) as erro:
        print(f"release recusada: {erro}", file=sys.stderr)
        return 1
    print(f"Versao conferida: {versao}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
