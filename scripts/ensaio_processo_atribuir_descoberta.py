"""Descoberta manual #264 por CLI: somente leituras, sem POST de atribuição.

Uma página observada no Controle não prova completude nem permanência do
estado de uma conta compartilhada. A prévia independente segue obrigatória.
"""
from __future__ import annotations

import subprocess
import time

from ensaio_processo_atribuir import atribuicao_da_unidade, quantidade
from ensaio_processo_criar import chamar_ies, nup_possivel_publicavel

def descobrir_candidato(*, ies: str, origem: str, executar=chamar_ies,
                        pausa=time.sleep, limite: int = 3) -> tuple[int, dict]:
    """Observa uma pagina da caixa e ate tres processos; nunca confirma.

    Um candidato reflete apenas as leituras deste instante. A proxima
    previa continua obrigatoria, pois a conta de treinamento e compartilhada.
    """
    if origem != "https://sei.treinamento.unb.br/sei" or not 1 <= limite <= 3:
        return 2, {"estado": "BLOQUEADO", "motivo": "origem_ou_limite_invalidos"}
    comandos = ["unidade", "recebidos"]
    try:
        codigo, caixa = executar(ies, comandos)
    except (OSError, subprocess.TimeoutExpired, ValueError):
        return 1, {"estado": "LEITURA_FAIL", "requisicoes": [None]}
    requisicoes = [quantidade(caixa)]
    unidade = caixa.get("unidade")
    processos = caixa.get("processos")
    if (codigo != 0 or caixa.get("categoria") != "recebidos"
            or type(unidade) is not str or not unidade
            or type(processos) is not list):
        return 1, {"estado": "LEITURA_FAIL", "requisicoes": requisicoes}

    examinados: set[str] = set()
    consultas = 0
    for processo in processos:
        if type(processo) is not dict:
            continue
        nup, identificador = processo.get("numero"), processo.get("id")
        if (not nup_possivel_publicavel(nup)
                or type(identificador) is not str or not identificador
                or nup in examinados):
            continue
        examinados.add(nup)
        if consultas == limite:
            break
        pausa(30)
        argumentos = ["processo", nup, "--sem-documentos", "--sem-andamentos",
                      "--sem-marcadores", "--sem-propriedades"]
        try:
            codigo, lido = executar(ies, argumentos)
        except (OSError, subprocess.TimeoutExpired, ValueError):
            return 1, {"estado": "LEITURA_FAIL", "consultas_processos": consultas + 1,
                       "requisicoes": requisicoes + [None]}
        consultas += 1
        requisicoes.append(quantidade(lido))
        if codigo != 0 or quantidade(lido) is None:
            return 1, {"estado": "LEITURA_FAIL", "consultas_processos": consultas,
                       "requisicoes": requisicoes}
        if (lido.get("numero") == nup
                and lido.get("id_procedimento") == identificador
                and atribuicao_da_unidade(lido, unidade) == ""):
            return 0, {"estado": "CANDIDATO_OBSERVADO", "nup": nup,
                       "consultas_processos": consultas,
                       "requisicoes": requisicoes}
    return 1, {"estado": "SEM_CANDIDATO_OBSERVADO",
               "consultas_processos": consultas, "requisicoes": requisicoes}
