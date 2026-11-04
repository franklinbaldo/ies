#!/usr/bin/env python3
"""Mede compilação de base e candidata em cópias descartáveis, sem rede SEI."""

import argparse
import json
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

COMANDO = ["cargo", "build", "--locked", "-p", "ies", "--bin", "ies", "--timings"]
ARQUIVO_ALTERADO = "src/processo_obter/fluxo.rs"
PARSER_ORIGINAL = "src/html_atributos/tokenizador.rs"
PARSER_EXTRAIDO = "crates/ies-web/src/html_atributos/tokenizador.rs"


def executar(comando, pasta):
    inicio = time.perf_counter()
    resultado = subprocess.run(
        comando, cwd=pasta, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, check=False, timeout=900
    )
    segundos = round(time.perf_counter() - inicio, 3)
    if resultado.returncode:
        raise RuntimeError(
            f"Falhou ({resultado.returncode}): {' '.join(comando)}\n"
            f"STDOUT: {resultado.stdout[-3000:]}\nSTDERR: {resultado.stderr[-3000:]}"
        )
    return segundos, resultado.stdout


def grafo_do_workspace(pasta):
    _, saida = executar(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        pasta,
    )
    metadata = json.loads(saida)
    membros = set(metadata["workspace_members"])
    pacotes = [p for p in metadata["packages"] if p["id"] in membros]
    nomes = {p["name"] for p in pacotes}
    arestas = sorted(
        [p["name"], d["name"]]
        for p in pacotes for d in p["dependencies"] if d["name"] in nomes
    )
    return {"pacotes": sorted(nomes), "dependencias": arestas}


def medir(workspace, saida):
    workspace = workspace.resolve()
    if not (workspace / "Cargo.lock").is_file():
        raise ValueError("Workspace sem Cargo.lock")
    with tempfile.TemporaryDirectory(prefix="ies-build-audit-") as temporario:
        copia = Path(temporario) / "workspace"
        shutil.copytree(
            workspace, copia,
            ignore=shutil.ignore_patterns(".git", "target", "__pycache__", ".venv"),
        )
        grafo = grafo_do_workspace(copia)
        limpo, _ = executar(COMANDO, copia)
        sem_mudanca, _ = executar(COMANDO, copia)
        alterado = copia / ARQUIVO_ALTERADO
        if not alterado.is_file():
            raise ValueError(f"Arquivo de referência ausente: {ARQUIVO_ALTERADO}")
        with alterado.open("a", encoding="utf-8") as codigo:
            codigo.write("\n// Auditoria do build: comentário sintético e descartável.\n")
        incremental, _ = executar(COMANDO, copia)
        parser = copia / (PARSER_EXTRAIDO if (copia / PARSER_EXTRAIDO).exists() else PARSER_ORIGINAL)
        if not parser.is_file():
            raise ValueError(f"Parser de atributos ausente: {parser}")
        with parser.open("a", encoding="utf-8") as codigo:
            codigo.write("\n// Auditoria do parser: comentário sintético e descartável.\n")
        parser_incremental, _ = executar(COMANDO, copia)
    resultado = {
        "cenario": ARQUIVO_ALTERADO,
        "cenario_parser": PARSER_EXTRAIDO if (workspace / PARSER_EXTRAIDO).exists() else PARSER_ORIGINAL,
        "comando": COMANDO,
        "tempos_segundos": {
            "limpo": limpo, "sem_mudanca": sem_mudanca,
            "operacao_alterada": incremental, "parser_alterado": parser_incremental,
        },
        "grafo": grafo,
        "observacao": "Mesmo executor; cópias e targets separados; sem SEI; sem baseline de release.",
    }
    saida.write_text(json.dumps(resultado, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(resultado, ensure_ascii=False))


def diferenca_percentual(antes, depois):
    return round((depois / antes - 1) * 100, 1) if antes else None


def comparar(base, proposta, sumario):
    anterior = json.loads(base.read_text(encoding="utf-8"))
    nova = json.loads(proposta.read_text(encoding="utf-8"))
    linhas = [
        "## Auditoria comparativa de build — IES",
        "",
        "| Cenário | Base (s) | Proposta (s) | Variação |",
        "|---|---:|---:|---:|",
    ]
    for chave, rotulo in [
        ("limpo", "Build limpo"),
        ("sem_mudanca", "Build sem alterações"),
        ("operacao_alterada", "Mudança em operação"),
        ("parser_alterado", "Mudança em parser HTML"),
    ]:
        antes = anterior["tempos_segundos"][chave]
        depois = nova["tempos_segundos"][chave]
        delta = diferenca_percentual(antes, depois)
        texto = "indefinido" if delta is None else f"{delta:+.1f}%"
        linhas.append(f"| {rotulo} | {antes:.3f} | {depois:.3f} | {texto} |")
    linhas += [
        "",
        "Medição exploratória, em ordem base → proposta; não demonstra ganho estatístico.",
        "Cold build recompila dependências nos dois diretórios; efeitos de cache do Cargo home podem variar.",
        "Nenhum benchmark executa requisições ao SEI.",
    ]
    relatorio = "\n".join(linhas) + "\n"
    print(relatorio)
    if sumario:
        with sumario.open("a", encoding="utf-8") as arquivo:
            arquivo.write(relatorio)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="acao", required=True)
    medir_cmd = sub.add_parser("medir")
    medir_cmd.add_argument("--workspace", type=Path, required=True)
    medir_cmd.add_argument("--saida", type=Path, required=True)
    comparar_cmd = sub.add_parser("comparar")
    comparar_cmd.add_argument("--base", type=Path, required=True)
    comparar_cmd.add_argument("--proposta", type=Path, required=True)
    comparar_cmd.add_argument("--sumario", type=Path)
    argumentos = parser.parse_args()
    if argumentos.acao == "medir":
        medir(argumentos.workspace, argumentos.saida)
    else:
        comparar(argumentos.base, argumentos.proposta, argumentos.sumario)


if __name__ == "__main__":
    main()
