#!/usr/bin/env python3
"""Auditoria de vulnerabilidades das dependências crates.io do Cargo.lock no OSV.

Uma única consulta em lote (`/v1/querybatch`) com todo pacote do registro
crates.io. Falha listando pacote, versão e ids dos avisos não aceitos em
`scripts/osv_allowlist.json`. Sem rede: falha com `CI` definido; fora do CI
avisa `NOT_RUN` e passa (ver docs/ci.md). Só biblioteca padrão.
"""
import json
import os
from pathlib import Path
import sys
import tomllib
import urllib.error
import urllib.request

OSV_URL = 'https://api.osv.dev/v1/querybatch'
CRATES_IO = {
    'registry+https://github.com/rust-lang/crates.io-index',
    'sparse+https://index.crates.io/',
}
RAIZ = Path(__file__).resolve().parent.parent
LOCK = RAIZ / 'Cargo.lock'
ALLOWLIST = RAIZ / 'scripts' / 'osv_allowlist.json'
TIMEOUT_S = 30


class RedeIndisponivel(Exception):
    """A resposta do OSV não foi obtida ou não é reconhecível."""


def pacotes_do_lock(texto):
    """Pares (nome, versão) únicos do registro crates.io, ordenados."""
    dados = tomllib.loads(texto)
    pacotes = set()
    for pacote in dados.get('package', []):
        if pacote.get('source') in CRATES_IO:
            pacotes.add((pacote['name'], pacote['version']))
    return sorted(pacotes)


def carregar_allowlist(texto):
    """Mapa (pacote, id) -> justificativa; entrada sem justificativa é erro."""
    dados = json.loads(texto)
    entradas = dados.get('accepted') if isinstance(dados, dict) else None
    if not isinstance(entradas, list):
        raise ValueError('allowlist sem a lista "accepted"')
    aceitos = {}
    for indice, entrada in enumerate(entradas):
        if not isinstance(entrada, dict):
            raise ValueError(f'entrada {indice} não é objeto')
        campos = {}
        for campo in ('id', 'package', 'justification'):
            valor = entrada.get(campo)
            if not isinstance(valor, str) or not valor.strip():
                raise ValueError(f'entrada {indice} sem "{campo}" escrito')
            campos[campo] = valor.strip()
        chave = (campos['package'], campos['id'])
        if chave in aceitos:
            raise ValueError(f'entrada {indice} repete {chave[0]} {chave[1]}')
        aceitos[chave] = campos['justification']
    return aceitos


def consulta(pacotes):
    return {'queries': [
        {'package': {'ecosystem': 'crates.io', 'name': nome}, 'version': versao}
        for nome, versao in pacotes
    ]}


def ids_da_resposta(resposta, quantidade):
    """Ids por consulta e tokens de próxima página; rejeita forma inesperada."""
    resultados = resposta.get('results') if isinstance(resposta, dict) else None
    if not isinstance(resultados, list) or len(resultados) != quantidade:
        raise RedeIndisponivel('resposta do OSV sem um resultado por pacote')
    ids, tokens = [], []
    for resultado in resultados:
        if not isinstance(resultado, dict):
            raise RedeIndisponivel('resultado do OSV não é objeto')
        vulns = resultado.get('vulns', [])
        if not isinstance(vulns, list) or not all(
                isinstance(v, dict) and isinstance(v.get('id'), str) for v in vulns):
            raise RedeIndisponivel('resultado do OSV sem ids reconhecíveis')
        ids.append([v['id'] for v in vulns])
        tokens.append(resultado.get('next_page_token'))
    return ids, tokens


def postar(corpo):
    pedido = urllib.request.Request(
        OSV_URL, data=json.dumps(corpo).encode(), method='POST',
        headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(pedido, timeout=TIMEOUT_S) as resposta:
            return json.load(resposta)
    except (urllib.error.URLError, OSError, ValueError) as erro:
        raise RedeIndisponivel(f'{type(erro).__name__}: {erro}') from erro


def avisos_por_pacote(pacotes, enviar=postar):
    """Lote único; páginas extras (mais de 1.000 avisos) só para quem as tiver."""
    corpo = consulta(pacotes)
    ids, tokens = ids_da_resposta(enviar(corpo), len(pacotes))
    achados = {p: set(i) for p, i in zip(pacotes, ids)}
    pendentes = [(p, q, t) for p, q, t in zip(pacotes, corpo['queries'], tokens) if t]
    while pendentes:
        pagina = {'queries': [{**q, 'page_token': t} for _, q, t in pendentes]}
        ids, tokens = ids_da_resposta(enviar(pagina), len(pendentes))
        for (pacote, _, _), novos in zip(pendentes, ids):
            achados[pacote].update(novos)
        pendentes = [(p, q, t) for (p, q, _), t in zip(pendentes, tokens) if t]
    return {p: sorted(i) for p, i in achados.items() if i}


def filtrar(achados, aceitos):
    """Separa avisos não aceitos e lista entradas da allowlist sem uso."""
    abertos, usados = {}, set()
    for (nome, versao), ids in achados.items():
        restantes = []
        for ident in ids:
            if (nome, ident) in aceitos:
                usados.add((nome, ident))
            else:
                restantes.append(ident)
        if restantes:
            abertos[(nome, versao)] = restantes
    return abertos, sorted(set(aceitos) - usados)


def executar(texto_lock, texto_allowlist, ambiente, enviar=postar, saida=print):
    """Código de saída: 0 limpo ou NOT_RUN local; 1 avisos; 2 sem rede no CI."""
    aceitos = carregar_allowlist(texto_allowlist)
    pacotes = pacotes_do_lock(texto_lock)
    try:
        achados = avisos_por_pacote(pacotes, enviar)
    except RedeIndisponivel as erro:
        if ambiente.get('CI'):
            saida(f'FAIL: OSV indisponível no CI ({erro})')
            return 2
        saida(f'NOT_RUN: OSV indisponível ({erro}); auditoria não executada. '
              'O CI falha nesta situação.')
        return 0
    abertos, sem_uso = filtrar(achados, aceitos)
    for nome, ident in sem_uso:
        saida(f'aviso: allowlist aceita {nome} {ident}, que o OSV não relatou')
    if abertos:
        saida(f'FAIL: {len(abertos)} pacote(s) com avisos no OSV:')
        for (nome, versao), ids in sorted(abertos.items()):
            saida(f'  {nome} {versao}: {", ".join(ids)}')
        return 1
    saida(f'OK: {len(pacotes)} pacotes crates.io sem avisos abertos no OSV')
    return 0


def main():
    return executar(LOCK.read_text(encoding='utf-8'),
                    ALLOWLIST.read_text(encoding='utf-8'), os.environ)


if __name__ == '__main__':
    sys.exit(main())
