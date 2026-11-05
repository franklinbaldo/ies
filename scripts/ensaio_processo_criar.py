"""Ensaio manual de #245. Não é chamado por GitHub Actions.

A trava em arquivo é criada *antes* do único --confirmar e nunca é
removida automaticamente. Outro ensaio requer registro diferente e
conferência humana do resíduo anterior; nenhum retry é automático.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import stat
import subprocess
import time


def origens_de_treinamento(tabela: Path) -> set[str]:
    origens = set()
    for linha in tabela.read_text(encoding="utf-8").splitlines():
        celulas = [v.strip().strip("\u0060") for v in linha.strip().strip("|").split("|")]
        if len(celulas) == 7 and celulas[0] in {
            "unb", "ufrj", "go", "pen1", "pen2", "pen3", "pen4"
        } and celulas[2].startswith("https://"):
            origens.add(celulas[2])
    if len(origens) != 7:
        raise ValueError("cadastro de origens de treinamento incompleto")
    return origens


def origem_privada_permitida(permitidas: set[str]) -> str:
    arquivo = Path(os.environ["IES_ORIGEM_FILE"])
    modo = arquivo.lstat()
    if not stat.S_ISREG(modo.st_mode) or modo.st_mode & 0o077:
        raise ValueError("arquivo de origem não é arquivo privado 0600")
    origem = arquivo.read_text(encoding="utf-8").strip()
    if origem not in permitidas:
        raise ValueError("origem não consta das instalações de treinamento")
    return origem


def chamar_ies(ies: str, argumentos: list[str]) -> tuple[int, dict]:
    # Sem shell. Stdout/stderr não são impressos: podem conter dados de sessão.
    resultado = subprocess.run(
        [ies, "--json", *argumentos], capture_output=True, text=True,
        check=False, timeout=180,
    )
    try:
        documento = json.loads(resultado.stdout)
    except (json.JSONDecodeError, ValueError):
        documento = {}
    return resultado.returncode, documento if isinstance(documento, dict) else {}


def ensaiar(
    *, ies: str, registro: Path, tipo: str, especificacao: str,
    nivel: str, hipotese: str | None, origem: str,
    executar=chamar_ies, pausa=time.sleep,
) -> tuple[int, dict]:
    """Retorna (código, relatório saneado). Jamais repete o POST criador."""
    if registro.exists() or registro.is_symlink():
        return 2, {"estado": "BLOQUEADO", "motivo": "registro_de_execucao_ja_existe"}
    if not tipo or not especificacao or nivel not in {"publico", "restrito"}:
        return 2, {"estado": "BLOQUEADO", "motivo": "argumentos_invalidos"}
    if hipotese and nivel != "restrito":
        return 2, {"estado": "BLOQUEADO", "motivo": "hipotese_incompativel"}
    base = ["processo", "criar", "--tipo", tipo,
            "--especificacao", especificacao, "--nivel", nivel]
    if hipotese:
        base += ["--hipotese-legal", hipotese]
    try:
        codigo, previa = executar(ies, base)
    except (OSError, subprocess.TimeoutExpired, ValueError):
        return 1, {"estado": "PREVIA_FAIL", "requisicoes_previa": None}
    if (codigo != 0 or previa.get("tipo") != tipo
            or previa.get("nivel") != nivel
            or previa.get("confirmado") is not False
            or not isinstance(previa.get("requisicoes"), int)
            or previa["requisicoes"] <= 0):
        return 1, {"estado": "PREVIA_FAIL", "requisicoes_previa": previa.get("requisicoes")}
    relatorio = {"estado": "CONFIRMACAO_INICIADA", "origem": origem,
                 "requisicoes_previa": previa["requisicoes"],
                 "requisicoes_confirmacao": None, "requisicoes_verificacao": None,
                 "nup": None}
    # O_CREAT|O_EXCL impede confirmação duplicada com o mesmo registro.
    try:
        fd = os.open(registro, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return 2, {"estado": "BLOQUEADO", "motivo": "reserva_concorrente"}

    def gravar() -> None:
        os.ftruncate(fd, 0)
        os.lseek(fd, 0, os.SEEK_SET)
        os.write(fd, (json.dumps(relatorio, ensure_ascii=False) + "\n").encode())
        os.fsync(fd)

    try:
        gravar()  # trava durável ANTES da confirmação, inclusive em crash.
        try:
            pausa(60 if "tramita.processoeletronico.gov.br" in origem else 30)
            codigo, criado = executar(ies, base + ["--confirmar"])
            relatorio["requisicoes_confirmacao"] = criado.get("requisicoes")
            # Com os mesmos argumentos, confirmar apenas soma o POST
            # criador e as duas leituras verificadoras ao orçamento da prévia.
            orcamento_consistente = (
                type(criado.get("requisicoes")) is int
                and criado["requisicoes"] == previa["requisicoes"] + 3
            )
            nup = criado.get("nup")
            if (codigo != 0 or criado.get("confirmado") is not True
                    or not isinstance(nup, str) or not nup.strip()
                    or any(c.isspace() for c in nup)):
                relatorio["estado"] = "EFEITO_INCERTO"
                gravar()
                return 3, relatorio
            relatorio["nup"] = nup
            relatorio["estado"] = "ESCRITA_COM_NUP_VERIFICACAO_PENDENTE"
            gravar()
            pausa(60 if "tramita.processoeletronico.gov.br" in origem else 30)
            codigo, lido = executar(ies, ["processo", nup])
            relatorio["requisicoes_verificacao"] = lido.get("requisicoes")
            if (codigo != 0 or lido.get("numero") != nup
                    or not lido.get("id_procedimento")
                    or type(lido.get("requisicoes")) is not int
                    or lido["requisicoes"] <= 0):
                relatorio["estado"] = "VERIFICACAO_INDEPENDENTE_FALHOU"
                gravar()
                return 3, relatorio
            if not orcamento_consistente:
                relatorio["estado"] = "ORCAMENTO_DIVERGENTE"
                gravar()
                return 3, relatorio
            relatorio["estado"] = "PASS"
            gravar()
            return 0, relatorio
        except (OSError, subprocess.TimeoutExpired, ValueError):
            relatorio["estado"] = "EFEITO_INCERTO"
            gravar()
            return 3, relatorio
    finally:
        os.close(fd)


def main() -> int:
    parser = argparse.ArgumentParser(description="Ensaio manual #245; uma confirmação, sem retry")
    parser.add_argument("--ies", default="ies")
    parser.add_argument("--registro", type=Path, required=True,
                        help="caminho persistente novo; nunca reutilizar nem remover automaticamente")
    parser.add_argument("--tipo", required=True)
    parser.add_argument("--especificacao", required=True)
    parser.add_argument("--nivel", choices=["publico", "restrito"], default="publico")
    parser.add_argument("--hipotese-legal")
    args = parser.parse_args()
    try:
        permitidas = origens_de_treinamento(
            Path(__file__).resolve().parents[1] / "docs/interfaces/credenciais-de-treinamento.md"
        )
        origem = origem_privada_permitida(permitidas)
        codigo, relatorio = ensaiar(
            ies=args.ies, registro=args.registro, tipo=args.tipo,
            especificacao=args.especificacao, nivel=args.nivel,
            hipotese=args.hipotese_legal, origem=origem,
        )
    except (OSError, ValueError, KeyError):
        codigo, relatorio = 2, {"estado": "BLOQUEADO", "motivo": "origem_ou_credenciais_invalidas"}
    # Só o relatório saneado; stdout/stderr da CLI nunca são publicados.
    print(json.dumps(relatorio, ensure_ascii=False))
    return codigo


if __name__ == "__main__":
    raise SystemExit(main())
