"""Conciliação SOMENTE LEITURA de efeito incerto de processo criar (#245).

Um NUP encontrado é POSSÍVEL resíduo, não prova causal da criação anterior.
Ausência de correspondência numa página da caixa NUNCA prova que não criou.
Toda falha interrompe a coleta; nunca há retry nem --confirmar.
"""
from __future__ import annotations

import json
from typing import Callable

ESPECIFICACAO = "Ensaio IES 245 - escrita unica em treinamento"
MAX_LEITURAS = 12


def nup_publicavel(valor: object) -> bool:
    """Só passar número seguro da lista observada ao CLI, sem inventar máscara."""
    return (isinstance(valor, str) and 4 <= len(valor) <= 72
            and all(c.isdigit() or c in "./-" for c in valor)
            and any(c.isdigit() for c in valor))


def reconciliar(
    executar: Callable[[str, list[str]], tuple[int, dict]],
    pausa: Callable[[int], None],
    intervalo: int = 35,
    limite: int = MAX_LEITURAS,
) -> dict:
    """Uma leitura da caixa e até limite processos, sem retries.

    executar(etapa,args) deve invocar ies --json uma única vez,
    reter stdout/stderr privados e devolver apenas (código, JSON).
    """
    if not 1 <= limite <= MAX_LEITURAS or intervalo < 30:
        raise ValueError("limites da conciliação não seguros")
    etapas = []
    total_http: int | None = 0

    def chamar(etapa: str, argumentos: list[str]) -> tuple[int, dict]:
        nonlocal total_http
        # Proibir desde a camada de agendamento quaisquer caminhos de escrita.
        if "--confirmar" in argumentos or argumentos[:2] == ["processo", "criar"]:
            raise ValueError("conciliação não pode confirmar")
        codigo, dado = executar(etapa, argumentos)
        if not isinstance(dado, dict):
            dado = {}
        pedidos = dado.get("requisicoes")
        if type(pedidos) is not int or pedidos < 0:
            total_http = None
        elif total_http is not None:
            total_http += pedidos
        subcodigo = dado.get("subcodigo")
        if (not isinstance(subcodigo, str) or len(subcodigo) > 64
                or not subcodigo or any(c not in "abcdefghijklmnopqrstuvwxyz0123456789_"
                                         for c in subcodigo)):
            subcodigo = None
        etapas.append({"etapa": etapa, "codigo": codigo,
                       "requisicoes": pedidos if type(pedidos) is int and pedidos >= 0 else None,
                       "subcodigo": subcodigo})
        return codigo, dado

    def saida(estado: str, **outros) -> dict:
        return {"estado": estado, "etapas": etapas,
                "requisicoes_totais_observadas": total_http,
                **outros}

    codigo, dados = chamar("gerados", ["unidade", "gerados"])
    if codigo != 0:
        return saida("LEITURA_INTERROMPIDA", motivo="gerados_falhou")
    processos = dados.get("processos")
    total = dados.get("total_declarado")
    faixa = dados.get("faixa_observada")
    if (dados.get("categoria") != "gerados" or not isinstance(processos, list)
            or type(total) is not int or total < len(processos)
            or (processos and (not isinstance(faixa, dict)
                  or faixa.get("inicio") != 1
                  or faixa.get("fim") != len(processos)))
            or (not processos and total != 0)):
        return saida("FORMATO_NAO_RECONHECIDO", motivo="lista_gerados_invalida")
    unidade = dados.get("unidade")
    if not isinstance(unidade, str) or not unidade.strip():
        return saida("FORMATO_NAO_RECONHECIDO", motivo="unidade_nao_publicada")

    numeros = []
    for item in processos:
        if not isinstance(item, dict):
            continue
        nup = item.get("numero")
        if nup_publicavel(nup) and nup not in numeros:
            numeros.append(nup)

    correspondencias = []
    examinados = 0
    for nup in numeros[:limite]:
        pausa(intervalo)
        codigo, encontrado = chamar("processo", [
            "processo", nup, "--sem-documentos", "--sem-marcadores", "--sem-andamentos"
        ])
        examinados += 1
        if codigo != 0:
            return saida("LEITURA_INTERROMPIDA", motivo="processo_falhou",
                         processos_examinados=examinados, candidatos=len(numeros),
                         total_declarado=total, correspondencias=correspondencias)
        propriedades = encontrado.get("propriedades")
        if (encontrado.get("numero") == nup
                and encontrado.get("unidade") == unidade
                and isinstance(propriedades, dict)
                and propriedades.get("especificacao") == ESPECIFICACAO
                and isinstance(encontrado.get("id_procedimento"), str)
                and 1 <= len(encontrado["id_procedimento"]) <= 32
                and encontrado["id_procedimento"].isascii()
                and encontrado["id_procedimento"].isdigit()):
            correspondencias.append({"nup": nup,
                                     "id_procedimento": encontrado["id_procedimento"]})
    # É sempre 'possível' e nunca 'confirmado': correspondência não prova
    # que o POST de efeito incerto foi a origem deste processo.
    return saida("POSSIVEL_RESIDUO" if correspondencias else "INCONCLUSIVO",
                 total_declarado=total, processos_da_pagina=len(processos),
                 candidatos=len(numeros), processos_examinados=examinados,
                 correspondencias=correspondencias,
                 cobertura_total=(total == len(processos) == examinados),
                 aviso="Não repetir a confirmação; ausência nesta amostra não prova ausência")


def executar_local(binario: str, etapa: str, argumentos: list[str]) -> tuple[int, dict]:
    """Saída segura: não repassa stdout/stderr brutos nem URLs/cookies."""
    import subprocess
    resultado = subprocess.run([binario, "--json", *argumentos],
                               capture_output=True, text=True, timeout=180, check=False)
    try:
        valor = json.loads(resultado.stdout)
    except ValueError:
        valor = {}
    return resultado.returncode, valor if isinstance(valor, dict) else {}
