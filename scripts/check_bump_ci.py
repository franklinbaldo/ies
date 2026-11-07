"""Gate canônico de bump: versões 0.x.y consistentes e sem regressão (#346).

No PR/push para main, compara a versão atual ao primeiro pai do commit.
Um PR sem alteração de versão não precisa de bump: há fatias frequentes
em paralelo; a publicação exige tag validada separadamente.
"""

import os
from pathlib import Path
import re
import subprocess
import sys
import tomllib

# -I remove o diretorio do script do sys.path; adicionar apenas nossa pasta
# versionada, nunca o cwd ou um caminho fornecido pelo usuario.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from verificar_release_uvx import conferir_tag

SERIE_ZERO = re.compile(r"0\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")


def parsear_versao(versao: str) -> tuple[int, int]:
    match = SERIE_ZERO.fullmatch(versao)
    if match is None:
        raise ValueError("versao deve permanecer em 0.MINOR.PATCH (sem prerelease)")
    return int(match.group(1)), int(match.group(2))


def versao_no_pai(raiz: Path) -> str:
    comando = ["git", "-C", str(raiz), "show", "HEAD^1:Cargo.toml"]
    resultado = subprocess.run(
        comando, capture_output=True, text=True, check=False,
    )
    if resultado.returncode != 0:
        raise ValueError("nao foi possivel ler Cargo.toml do commit anterior")
    try:
        return tomllib.loads(resultado.stdout)["package"]["version"]
    except (tomllib.TOMLDecodeError, KeyError) as erro:
        raise ValueError("Cargo.toml do commit anterior invalido") from erro


def conferir_bump(raiz: Path, evento: str, ref: str) -> str:
    with (raiz / "Cargo.toml").open("rb") as arquivo:
        versao = tomllib.load(arquivo)["package"]["version"]
    atual = parsear_versao(versao)
    # Reusa a validacao unica de identidade da tag, Cargo, PyProject e lock.
    conferir_tag(f"v{versao}", raiz)
    if evento == "pull_request" or (evento == "push" and ref == "refs/heads/main"):
        anterior = versao_no_pai(raiz)
        if atual < parsear_versao(anterior):
            raise ValueError(f"regressao SemVer: {anterior} -> {versao}")
        return f"SemVer CI: {anterior} -> {versao} (serie 0 preservada)"
    return f"SemVer CI: {versao} (manifestos conferidos; sem comparacao Git)"


def main() -> int:
    if len(sys.argv) > 2:
        print("uso: check_bump_ci.py [RAIZ]", file=sys.stderr)
        return 2
    raiz = Path(sys.argv[1]) if len(sys.argv) == 2 else Path(__file__).resolve().parents[1]
    try:
        resultado = conferir_bump(
            raiz, os.environ.get("GITHUB_EVENT_NAME", ""),
            os.environ.get("GITHUB_REF", ""),
        )
    except (ValueError, OSError, KeyError, tomllib.TOMLDecodeError) as erro:
        print(f"CHECK BUMP FAIL: {erro}", file=sys.stderr)
        return 1
    print(resultado)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
