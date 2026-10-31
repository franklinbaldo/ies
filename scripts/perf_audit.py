#!/usr/bin/env python3
"""Benchmark sintético OFFLINE do IES e de inicialização do Todos.

Não usa credenciais, não consulta instalações SEI e não faz GET/POST.
O resultado de subprocessos é verificado; métricas de programas diferentes
não são tratadas como benchmark funcional equivalente.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import tempfile
import time
import zipfile


def executar(comando: list[str]) -> dict:
    inicio = time.perf_counter_ns()
    processo = subprocess.Popen(comando, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, close_fds=True)
    _, status, uso = os.wait4(processo.pid, 0)
    return {
        "ms": (time.perf_counter_ns() - inicio) / 1_000_000,
        "cpu_ms": (uso.ru_utime + uso.ru_stime) * 1000,
        "rss_kib": uso.ru_maxrss,
        "exit": os.waitstatus_to_exitcode(status),
    }


def medir(comando: list[str], n: int, esperado: int | None = 0) -> dict:
    # Aquecimento separado: mesmas condições de cache para ambos.
    for _ in range(3):
        resultado = executar(comando)
        if esperado is not None and resultado["exit"] != esperado:
            raise RuntimeError(f"comando retornou {resultado['exit']}, esperado {esperado}")
    amostras = [executar(comando) for _ in range(n)]
    if esperado is not None and any(v["exit"] != esperado for v in amostras):
        raise RuntimeError("código de saída inesperado durante medição")
    t = sorted(v["ms"] for v in amostras)
    rss = [v["rss_kib"] for v in amostras]
    cpu = [v["cpu_ms"] for v in amostras]
    return {
        "n": len(t), "ms_mediana": round(statistics.median(t), 3),
        "ms_p95": round(t[min(len(t) - 1, int(0.95 * len(t)))], 3),
        "ms_min": round(t[0], 3), "ms_max": round(t[-1], 3),
        "cpu_ms_mediana": round(statistics.median(cpu), 3),
        "rss_pico_kib_mediana": round(statistics.median(rss)),
        "rss_pico_kib_max": max(rss), "exit": amostras[0]["exit"],
    }


def fixture(raiz: Path, nome: str, operacao: str, metodo: str, html: str) -> Path:
    pasta = raiz / nome
    (pasta / "cassettes" / "unb").mkdir(parents=True)
    (pasta / ".cache" / "ies").mkdir(parents=True)
    titulo = "todos-sei-cassettes-unb.zip"
    arquivo = pasta / ".cache" / "ies" / titulo
    membro = "benchmark-unb.json"
    dados = {
        "complete": True, "schema_version": 1,
        "sei_root": "https://sei.treinamento.unb.br/",
        "interactions": [{
            "request": {"operation": operacao, "method": metodo},
            "response": {
                "status_code": 200, "encoding": "utf-8",
                "headers": {"content-type": "text/html; charset=utf-8"},
                "body_base64": base64.b64encode(html.encode("utf-8")).decode("ascii"),
            },
        }],
    }
    bruto = json.dumps(dados, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    with zipfile.ZipFile(arquivo, "w") as zipout:
        zipout.writestr(membro, bruto, compress_type=zipfile.ZIP_DEFLATED)
    texto = (
        "---\ntype: Cassette\ninstallation: unb\n"
        "source_zip: https://archive.org/download/todos-sei-cassettes-2026/"
        + titulo + "\narchive_member: " + membro + "\n"
        + "archive_sha1: " + hashlib.sha1(arquivo.read_bytes()).hexdigest() + "\n"
        + "cassette_sha256_lf: " + hashlib.sha256(bruto).hexdigest() + "\n---\n"
    )
    (pasta / "cassettes" / "unb" / (nome + "--sintetico.md")).write_text(texto)
    return pasta


def inputs(ies: str, raiz: Path) -> dict[str, list[str]]:
    casos = {
        "ies_ajuda": [ies, "--help"],
        "ies_versao": [ies, "--version"],
        "ies_esquema": [ies, "esquema", "--json"],
        "ies_stub_documento": [ies, "documento", "0011519", "--json"],
    }
    for tamanho in (1024, 64000, 220000):
        html = ("<html><head><title>SEI/UnB - 0011354 - Ofício</title></head>"
                "<body><p>" + "texto de teste. " * (tamanho // 16) + "</p></body></html>")
        pasta = fixture(raiz, f"documento-{tamanho}", "documento_imprimir_web", "GET", html)
        casos[f"ies_documento_texto_{tamanho}"] = [
            ies, "replay", f"documento-{tamanho}", "--acervo", str(pasta),
            "--documento", "--numero-documento", "0011354", "--texto-documento", "--json",
        ]
    for n in (20, 1000, 10000):
        linhas = "".join(
            "<tr><td>31/10/2026</td><td>Unidade A</td>"
            "<td>usuario</td><td>Decisão assinada de teste</td></tr>"
            for _ in range(n)
        )
        html = ("<html><head><title>SEI - Histórico do Processo 000123/2026-X</title>"
                "</head><body><table id='tblHistorico'>" + linhas
                + "</table></body></html>")
        pasta = fixture(raiz, f"historico-{n}", "procedimento_consultar_historico", "GET", html)
        casos[f"ies_historico_{n}"] = [
            ies, "replay", f"historico-{n}", "--acervo", str(pasta),
            "--numero-processo", "000123/2026-X", "--json",
        ]
    for n in (20, 1000, 20000):
        linhas = "".join(
            '<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal" '
            'href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento='
            + str(i + 10) + '">' + f'{i:06d}/2026-A'
            + '</a></td></tr>' for i in range(n)
        )
        html = '<html><table class="pesquisaResultado">' + linhas + '</table></html>'
        pasta = fixture(raiz, f"pesquisa-{n}", "protocolo_pesquisar", "POST", html)
        casos[f"ies_pesquisa_{n}"] = [
            ies, "replay", f"pesquisa-{n}", "--acervo", str(pasta),
            "--resultado-pesquisa", "--passo", "0", "--json",
        ]
    return casos


def principal() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ies", required=True)
    parser.add_argument("--todos", help="caminho opcional para Todos no mesmo runner")
    parser.add_argument("--repeticoes", type=int, default=20)
    args = parser.parse_args()
    if not 5 <= args.repeticoes <= 100:
        parser.error("repetições devem estar entre 5 e 100")
    resumo = {
        "hardware": {
            "plataforma": platform.platform(),
            "arquitetura": platform.machine(),
            "cpus_logicas": os.cpu_count(),
            "python": platform.python_version(),
        },
        "ies_binario_bytes": Path(args.ies).stat().st_size,
        "observacao": (
            "Picos RSS de subprocessos Linux via wait4. "
            "Fixture sintética, sem rede. Todos só comparado se disponível "
            "no mesmo runner de forma autorizada."
        ),
        "resultados": {},
        "erros": {},
    }
    with tempfile.TemporaryDirectory(prefix="ies-perf-offline-") as temp:
        casos = inputs(args.ies, Path(temp))
        if args.todos:
            casos.update({
                "todos_ajuda": [args.todos, "--help"],
                "todos_versao": [args.todos, "--version"],
            })
        for nome, cmd in casos.items():
            try:
                resumo["resultados"][nome] = medir(cmd, args.repeticoes,
                    esperado=None if nome == "ies_stub_documento" else 0)
                print("BENCH", nome, json.dumps(resumo["resultados"][nome]), flush=True)
            except (OSError, RuntimeError, ValueError) as erro:
                resumo["erros"][nome] = str(erro)
                print("BENCH_FAIL", nome, str(erro), flush=True)
    saida = json.dumps(resumo, sort_keys=True, ensure_ascii=False)
    print("PERF_JSON=" + saida, flush=True)
    if os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(os.environ["GITHUB_STEP_SUMMARY"], "a", encoding="utf-8") as arquivo:
            arquivo.write("## Benchmark IES e Todos — sem SEI real\n\n")
            arquivo.write("| Operação | Mediana ms | P95 ms | RSS mediana MiB |\n")
            arquivo.write("|---|---:|---:|---:|\n")
            for nome, item in resumo["resultados"].items():
                arquivo.write(f"| {nome} | {item['ms_mediana']} | {item['ms_p95']} "
                              f"| {item['rss_pico_kib_mediana']/1024:.2f} |\n")
            arquivo.write("\nMedições sintéticas; não equivalem a velocidade de acesso ao SEI.\n")
    if resumo["erros"]:
        raise SystemExit("alguns cenários falharam; números disponíveis apenas nos casos válidos")


if __name__ == "__main__":
    principal()
