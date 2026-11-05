#!/usr/bin/env python3
"""Inventário léxico conservador de subcódigos Rust e tabelas normativas."""
import json
from pathlib import Path
import re
import sys

CODIGO = re.compile(r'[a-z][a-z0-9]*(?:_[a-z0-9]+)*\Z')


def string_rust(texto, inicio):
    """Lê string normal, decodificando escapes Rust (não escapes Python)."""
    i, partes = inicio + 1, []
    simples = {'n': '\n', 'r': '\r', 't': '\t', '0': '\0', '\\': '\\', '"': '"', "'": "'"}
    while i < len(texto):
        c = texto[i]
        if c == '"':
            return ''.join(partes), i + 1
        if c != '\\':
            partes.append(c)
            i += 1
            continue
        i += 1
        if i == len(texto):
            break
        c = texto[i]
        if c in simples:
            partes.append(simples[c])
            i += 1
        elif c == 'x' and re.fullmatch('[0-9a-fA-F]{2}', texto[i + 1:i + 3]):
            partes.append(chr(int(texto[i + 1:i + 3], 16)))
            i += 3
        elif c == 'u' and texto[i + 1:i + 2] == '{':
            fim = texto.find('}', i + 2)
            if fim < 0:
                raise ValueError('escape Unicode aberto')
            partes.append(chr(int(texto[i + 2:fim].replace('_', ''), 16)))
            i = fim + 1
        elif c in '\r\n':
            while i < len(texto) and texto[i].isspace():
                i += 1
        else:
            raise ValueError('escape de string não reconhecido')
    raise ValueError('string aberta')


def tokens_rust(texto):
    """Tokeniza comentários aninhados, strings, chars, lifetimes e delimitadores."""
    tokens, i = [], 0
    while i < len(texto):
        if texto[i].isspace():
            i += 1
        elif texto.startswith('//', i):
            fim = texto.find('\n', i)
            i = len(texto) if fim < 0 else fim + 1
        elif texto.startswith('/*', i):
            nivel, i = 1, i + 2
            while i < len(texto) and nivel:
                if texto.startswith('/*', i):
                    nivel, i = nivel + 1, i + 2
                elif texto.startswith('*/', i):
                    nivel, i = nivel - 1, i + 2
                else:
                    i += 1
            if nivel:
                raise ValueError('comentário aberto')
        else:
            raw = re.match(r'(?:br|cr|r)(#*)"', texto[i:])
            char = re.match(r"(?:b)?'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", texto[i:])
            if raw:
                inicio = i + raw.end()
                fim_marca = '"' + raw[1]
                fim = texto.find(fim_marca, inicio)
                if fim < 0:
                    raise ValueError('raw string aberta')
                tipo = 'str' if texto[i] == 'r' else 'bytes'
                tokens.append((tipo, texto[inicio:fim]))
                i = fim + len(fim_marca)
            elif char:
                tokens.append(('char', ''))
                i += char.end()
            elif texto[i] == '"' or texto[i:i + 2] in ('b"', 'c"'):
                tipo = 'str' if texto[i] == '"' else 'bytes'
                valor, i = string_rust(texto, i if tipo == 'str' else i + 1)
                tokens.append((tipo, valor))
            elif texto[i].isalpha() or texto[i] == '_':
                fim = i + 1
                while fim < len(texto) and (texto[fim].isalnum() or texto[fim] == '_'):
                    fim += 1
                tokens.append(('id', texto[i:fim]))
                i = fim
            else:
                tokens.append(('punct', texto[i]))
                i += 1
    return tokens


def pares(tokens):
    pilha, resultado = [], {}
    for i, (tipo, valor) in enumerate(tokens):
        if tipo != 'punct':
            continue
        if valor in '([{':
            pilha.append((i, valor))
        elif valor in ')]}':
            if not pilha or pilha[-1][1] != {')': '(', ']': '[', '}': '{'}[valor]:
                raise ValueError('delimitadores Rust incompatíveis')
            inicio, _ = pilha.pop()
            resultado[inicio] = i
    if pilha:
        raise ValueError('delimitador Rust aberto')
    return resultado


def extrair_rust(texto):
    tokens = tokens_rust(texto)
    fechamentos = pares(tokens)
    codigos, i = set(), 0
    while i < len(tokens):
        # Os módulos de teste inline não constituem emissão pública.
        if tokens[i:i + 6] == [('punct', '#'), ('punct', '['), ('id', 'cfg'),
                               ('punct', '('), ('id', 'test'), ('punct', ')')]:
            fim = fechamentos[i + 1] + 1
            while fim < len(tokens) and tokens[fim][1] not in ('{', ';'):
                fim += 1
            if fim < len(tokens):
                i = fechamentos.get(fim, fim) + 1
                continue
        if tokens[i] == ('id', 'fn') and i + 1 < len(tokens):
            nome = tokens[i + 1][1]
            if nome == 'subcodigo' or nome.startswith('subcodigo_'):
                j = i + 2
                while j < len(tokens) and tokens[j][1] not in ('{', ';'):
                    if j in fechamentos:
                        j = fechamentos[j]
                    j += 1
                if j < len(tokens) and tokens[j] == ('punct', '{'):
                    fim = fechamentos[j]
                    for tipo, valor in tokens[j + 1:fim]:
                        if tipo == 'str':
                            if not CODIGO.fullmatch(valor):
                                raise ValueError(f'literal não classificável em {nome}')
                            codigos.add(valor)
                    i = fim + 1
                    continue
        # Complemento para emissões JSON literais, inclusive em falha aninhada.
        if (tokens[i:i + 2] == [('str', 'subcodigo'), ('punct', ':')]
                and i + 2 < len(tokens) and tokens[i + 2][0] == 'str'):
            valor = tokens[i + 2][1]
            if not CODIGO.fullmatch(valor):
                raise ValueError('subcódigo JSON não classificável')
            codigos.add(valor)
        i += 1
    return codigos


def celulas(linha):
    return re.split(r'(?<!\\)\|', linha.strip().strip('|'))


def extrair_documentacao(texto):
    """Somente coluna Subcódigo de tabelas; exemplos/prosa não dão cobertura."""
    codigos, coluna, cerca = set(), None, None
    for linha in texto.splitlines():
        marca = re.match(r'^\s*(`{3,}|~{3,})', linha)
        if marca:
            if cerca is None:
                cerca = marca[1][0]
            elif marca[1][0] == cerca:
                cerca = None
            coluna = None
            continue
        if cerca:
            continue
        if not linha.lstrip().startswith('|'):
            coluna = None
            continue
        partes = celulas(linha)
        cabecalhos = [p.strip().strip('`*').lower() for p in partes]
        if 'subcódigo' in cabecalhos or 'subcódigos' in cabecalhos:
            coluna = next(i for i, p in enumerate(cabecalhos) if p in ('subcódigo', 'subcódigos'))
        elif coluna is not None and coluna < len(partes):
            for codigo in re.findall(r'`([^`]+)`', partes[coluna]):
                if CODIGO.fullmatch(codigo):
                    codigos.add(codigo)
    return codigos


def carregar_allowlist(texto):
    dados = json.loads(texto)
    if not isinstance(dados, list):
        raise ValueError('allowlist deve ser uma lista')
    permitidos = {}
    for item in dados:
        if not isinstance(item, dict) or set(item) != {'subcodigo', 'justificativa'}:
            raise ValueError('entrada exige subcodigo e justificativa')
        codigo, motivo = item['subcodigo'], item['justificativa']
        if (not isinstance(codigo, str) or not CODIGO.fullmatch(codigo)
                or not isinstance(motivo, str) or not motivo.strip() or codigo in permitidos):
            raise ValueError('entrada inválida, duplicada ou sem justificativa')
        permitidos[codigo] = motivo
    return permitidos


def inventario(raiz):
    emitidos = {}
    for pasta in ('src', 'crates'):
        for arquivo in sorted((raiz / pasta).rglob('*.rs')):
            if any(Path(p).stem in ('tests', 'acervo') or p.startswith(('tests_', 'acervo_')) for p in arquivo.parts):
                continue
            try:
                codigos = extrair_rust(arquivo.read_text(encoding='utf-8'))
            except ValueError as erro:
                raise ValueError(f'{arquivo.relative_to(raiz)}: {erro}') from erro
            for codigo in codigos:
                emitidos.setdefault(codigo, []).append(str(arquivo.relative_to(raiz)))
    return emitidos


def verificar(raiz, imprimir=print):
    try:
        emitidos = inventario(raiz)
        if not emitidos:
            raise ValueError('nenhum subcódigo encontrado; verificar raiz e fontes')
        documentados = set()
        documentos = [
            raiz / 'docs/erros-tipados.md',
            *sorted((raiz / 'docs/erros-tipados').glob('*.md')),
            *sorted((raiz / 'contracts/cli').glob('*.md')),
        ]
        for arquivo in documentos:
            documentados.update(extrair_documentacao(arquivo.read_text(encoding='utf-8')))
        permitidos = carregar_allowlist((raiz / 'scripts/subcodigos_permitidos.json').read_text(encoding='utf-8'))
        faltantes = sorted(emitidos.keys() - documentados - permitidos.keys())
        for codigo in faltantes:
            imprimir(f'ERRO não documentado: {codigo} ({", ".join(emitidos[codigo])})')
        for codigo in sorted(documentados - emitidos.keys()):
            imprimir(f'AVISO documentado sem literal no inventário: {codigo}')
        for codigo in sorted(permitidos):
            imprimir(f'AVISO allowlist: {codigo}: {permitidos[codigo]}')
        imprimir(f'{len(emitidos)} subcódigos; {len(faltantes)} sem documentação; {len(permitidos)} exceções')
        return int(bool(faltantes))
    except (OSError, ValueError) as erro:
        imprimir(f'ERRO inventário: {erro}')
        return 1


if __name__ == '__main__':
    sys.exit(verificar(Path(__file__).resolve().parents[1]))
