"""Registro e trocas dos ajustes à mão (ajustes_*.py). Cada troca exige o
trecho velho exatamente uma vez (ou o número dito)."""

import re

AJUSTES = []


def ajuste(arquivo, motivo):
    """Registra uma função que recebe o texto do arquivo e devolve o novo."""

    def registrar(funcao):
        AJUSTES.append((arquivo, motivo, funcao))
        return funcao

    return registrar


def trocar(texto, velho, novo, vezes=1):
    achados = texto.count(velho)
    if achados != vezes:
        raise ValueError(f"esperava {vezes}x, achou {achados}x: {velho[:80]!r}")
    return texto.replace(velho, novo)


def trocar_re(texto, padrao, novo, vezes=1):
    resultado, achados = re.subn(padrao, novo, texto)
    if achados != vezes:
        raise ValueError(f"esperava {vezes}x, achou {achados}x: {padrao[:80]!r}")
    return resultado


ERRO = "crate::arquivo_privado::ErroArquivoPrivado"
