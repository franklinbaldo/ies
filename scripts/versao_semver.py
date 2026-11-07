"""Incrementa SemVer de IES sem afetar dependencias ou publicar releases (#346).

Uso: python3 -I scripts/versao_semver.py minor [--aplicar]
Sem --aplicar, so mostra a proposta. Versoes pre-release nao sao aceitas
por esta ferramenta; requerem politica propria de empacotamento Python.
"""

import argparse
import os
from pathlib import Path
import re
import tempfile
import tomllib

VERSAO = re.compile(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)")
CABECALHO = re.compile(r"(?m)^\[{1,2}[^\]\n]+\]{1,2}[ \t]*$")
LINHA_VERSAO = re.compile(r'(?m)^(version[ \t]*=[ \t]*")([^"\n]+)(".*)$')


def incrementar(versao: str, tipo: str) -> str:
    combinacao = VERSAO.fullmatch(versao)
    if combinacao is None:
        raise ValueError("versao atual nao e MAJOR.MINOR.PATCH simples")
    major, minor, patch = map(int, combinacao.groups())
    if major != 0:
        raise ValueError("IES deve permanecer na serie 0.x.y")
    if tipo == "minor":
        return f"{major}.{minor + 1}.0"
    if tipo == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise ValueError(f"incremento invalido: {tipo}")


def substituir_versao(texto: str, secao: str, anterior: str, proxima: str,
                     nome: str | None = None) -> str:
    cabecalhos = list(CABECALHO.finditer(texto))
    encontrados = []
    for indice, cabecalho in enumerate(cabecalhos):
        if cabecalho.group().strip() != secao:
            continue
        fim = cabecalhos[indice + 1].start() if indice + 1 < len(cabecalhos) else len(texto)
        corpo = texto[cabecalho.end():fim]
        if nome is not None and re.search(
            rf'(?m)^name[ \t]*=[ \t]*"{re.escape(nome)}"[ \t]*$', corpo
        ) is None:
            continue
        encontrados.append((cabecalho.end(), fim, corpo))
    if len(encontrados) != 1:
        raise ValueError(f"secao {secao} esperada uma vez; encontradas {len(encontrados)}")
    inicio, fim, corpo = encontrados[0]

    def trocar(mo: re.Match[str]) -> str:
        if mo.group(2) != anterior:
            raise ValueError(f"{secao}: versao da fonte difere da declarada")
        return f"{mo.group(1)}{proxima}{mo.group(3)}"

    substituido, quantidade = LINHA_VERSAO.subn(trocar, corpo)
    if quantidade != 1:
        raise ValueError(f"{secao}: esperado um campo version; encontrados {quantidade}")
    return texto[:inicio] + substituido + texto[fim:]


def preparar(raiz: Path, tipo: str) -> tuple[str, str, dict[Path, str]]:
    fontes = {
        nome: (raiz / nome).read_text(encoding="utf-8")
        for nome in ("Cargo.toml", "pyproject.toml", "Cargo.lock")
    }
    cargo = tomllib.loads(fontes["Cargo.toml"])
    projeto = tomllib.loads(fontes["pyproject.toml"])
    lock = tomllib.loads(fontes["Cargo.lock"])
    if cargo["package"]["name"] != "ies" or projeto["project"]["name"] != "ies-sei":
        raise ValueError("manifestos nao descrevem IES / ies-sei")
    pacotes = [p for p in lock["package"] if p["name"] == "ies"]
    if len(pacotes) != 1:
        raise ValueError("Cargo.lock deve conter exatamente um pacote ies")
    antiga = cargo["package"]["version"]
    if projeto["project"]["version"] != antiga or pacotes[0]["version"] != antiga:
        raise ValueError("versoes divergentes entre Cargo.toml, pyproject.toml e Cargo.lock")
    nova = incrementar(antiga, tipo)
    alterados = {
        raiz / "Cargo.toml": substituir_versao(
            fontes["Cargo.toml"], "[package]", antiga, nova
        ),
        raiz / "pyproject.toml": substituir_versao(
            fontes["pyproject.toml"], "[project]", antiga, nova
        ),
        raiz / "Cargo.lock": substituir_versao(
            fontes["Cargo.lock"], "[[package]]", antiga, nova, "ies"
        ),
    }
    return antiga, nova, alterados


def main() -> int:
    parser = argparse.ArgumentParser(description="Propor incremento SemVer de IES")
    parser.add_argument("tipo", choices=("minor", "patch"))
    parser.add_argument("raiz", nargs="?", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--aplicar", action="store_true",
                        help="escrever os tres arquivos apos validar a proposta")
    args = parser.parse_args()
    try:
        anterior, proxima, alterados = preparar(args.raiz, args.tipo)
    except (ValueError, OSError, KeyError, tomllib.TOMLDecodeError) as erro:
        parser.exit(1, f"versao recusada: {erro}\n")
    print(f"IES {anterior} -> {proxima} ({args.tipo})")
    if not args.aplicar:
        print("PREVIA: nenhum arquivo alterado; use --aplicar para efetivar")
        return 0

    temporarios = []
    try:
        for destino, conteudo in alterados.items():
            with tempfile.NamedTemporaryFile(
                mode="w", encoding="utf-8", dir=destino.parent,
                prefix=f".{destino.name}.", delete=False,
            ) as arquivo:
                temporarios.append(Path(arquivo.name))
                arquivo.write(conteudo)
                os.chmod(arquivo.name, destino.stat().st_mode)
        for (destino, _), temporario in zip(alterados.items(), temporarios):
            temporario.replace(destino)
    except OSError as erro:
        parser.exit(1, f"falha ao escrever: {erro}\n")
    finally:
        for temporario in temporarios:
            temporario.unlink(missing_ok=True)
    print("APLICADO: Cargo.toml, pyproject.toml, Cargo.lock")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
