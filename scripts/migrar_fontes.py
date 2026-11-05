"""Codemod: `.map_err(|_| E::X)` que descarta um erro do projeto vira
`E::X(causa)` com `#[causa(fonte)]`: o fato continua sendo `X` (mesmo nome
público) e a causa tipada sobe junto.

Uso (ferramenta de desenvolvimento, fora do gate; precisa de cargo):

    uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \
        python scripts/migrar_fontes.py [--aplicar]

1. Descobre o tipo da origem de cada `map_err(|_| ...)` pelo compilador
   (troca temporária por `|_: ()|` e leitura do erro E0631).
2. Elegível: variante unitária, origem que deriva `Causa`, e toda
   construção da variante é um desses `map_err` com a mesma origem.
3. Aplica, compila e corrige o que o compilador apontar (padrões viram
   `X(_)`, `assert_eq!(.., Err(E::X))` vira `matches!`). Enum que não
   fecha volta ao estado original e sai da rodada.
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsr

RAIZ = Path(__file__).resolve().parents[1]
PARSER = ts.Parser(ts.Language(tsr.language()))
ENV = dict(os.environ, CARGO_TARGET_DIR=os.environ.get('CARGO_TARGET_DIR', str(Path.home() / '.cargo-target-ies')))


def texto(no, fonte):
    return fonte[no.start_byte:no.end_byte].decode()


def nos(raiz, tipo):
    pilha = [raiz]
    while pilha:
        no = pilha.pop()
        if no.type == tipo:
            yield no
        pilha.extend(no.named_children)


def arquivos():
    return [a for a in sorted((RAIZ / 'src').rglob('*.rs'))]


def cargo(*args):
    r = subprocess.run(['cargo', *args, '-q', '--offline', '--message-format=json'], cwd=RAIZ, env=ENV,
                       capture_output=True, text=True, check=False)
    mensagens = []
    for linha in r.stdout.splitlines():
        try:
            m = json.loads(linha)
        except ValueError:
            continue
        if m.get('reason') == 'compiler-message' and m['message']['level'] == 'error':
            mensagens.append(m['message'])
    return mensagens


def tipos_das_origens():
    """{(arquivo, linha, coluna do `|`): tipo} pelo compilador."""
    originais = {}
    for a in arquivos():
        s = a.read_text(encoding='utf-8')
        if 'map_err(|_|' in s:
            originais[a] = s
            a.write_text(s.replace('map_err(|_|', 'map_err(|_: ()|'), encoding='utf-8')
    try:
        mensagens = cargo('build') + cargo('test', '--no-run')
    finally:
        for a, s in originais.items():
            a.write_text(s, encoding='utf-8')
    tipos = {}
    for m in mensagens:
        r = re.search(r'expected closure signature `fn\(([^)]*)\) ->', m['rendered'])
        if not r:
            continue
        for sp in m['spans']:
            if 'map_err' in (sp.get('text') or [{}])[0].get('text', '') or sp['is_primary']:
                chave = (str(RAIZ / sp['file_name']), sp['line_start'])
                tipos.setdefault(chave, r.group(1).strip())
    return tipos


def apelidos(arvore, fonte):
    mapa = {}
    for u in nos(arvore.root_node, 'use_as_clause'):
        mapa[texto(u.child_by_field_name('alias'), fonte)] = texto(u.child_by_field_name('path'), fonte).split('::')[-1]
    return mapa


def enums_com_causa(fontes):
    """{nome: (arquivo, nó do enum, {variante: nó})} dos que derivam Causa."""
    saida = {}
    for a, (f, arv) in fontes.items():
        for e in nos(arv.root_node, 'enum_item'):
            irmao, deriva = e.prev_named_sibling, False
            while irmao is not None and irmao.type == 'attribute_item':
                t = texto(irmao, f)
                deriva = deriva or (t.startswith('#[derive(') and 'Causa' in t)
                irmao = irmao.prev_named_sibling
            if not deriva:
                continue
            variantes = {texto(v.child_by_field_name('name'), f): v
                         for v in e.child_by_field_name('body').named_children if v.type == 'enum_variant'}
            saida.setdefault(texto(e.child_by_field_name('name'), f), []).append((a, e, variantes))
    return saida


def planejar(fontes, tipos, excluidos):
    causas = enums_com_causa(fontes)
    nomes_causa = set(causas)
    sitios = {}      # (enum, variante) -> [(arquivo, nó do arg, tipo)]
    construcoes = {}  # (enum, variante) -> qtd de usos em expressão
    for a, (f, arv) in fontes.items():
        mapa = apelidos(arv, f)
        for c in nos(arv.root_node, 'call_expression'):
            fn = c.child_by_field_name('function')
            if fn.type != 'field_expression' or texto(fn.child_by_field_name('field'), f) != 'map_err':
                continue
            args = c.child_by_field_name('arguments').named_children
            if len(args) != 1 or args[0].type != 'closure_expression':
                continue
            clo = args[0]
            params = texto(clo.child_by_field_name('parameters'), f)
            corpo = clo.child_by_field_name('body')
            if params != '|_|' or corpo.type != 'scoped_identifier':
                continue
            partes = texto(corpo, f).split('::')
            if len(partes) != 2:
                continue
            enum = mapa.get(partes[0], partes[0])
            tipo = tipos.get((str(a), c.start_point[0] + 1)) or tipos.get((str(a), clo.start_point[0] + 1))
            if tipo is None:
                continue
            sitios.setdefault((enum, partes[1]), []).append((a, clo, tipo))
        for s in nos(arv.root_node, 'scoped_identifier'):
            partes = texto(s, f).split('::')
            if len(partes) < 2:
                continue
            enum = mapa.get(partes[-2], partes[-2])
            pai = s.parent
            if pai is not None and pai.type == 'closure_expression':
                continue  # o próprio sítio
            if any(t in (pai.type if pai else '') for t in ('pattern',)):
                continue
            construcoes[(enum, partes[-1])] = construcoes.get((enum, partes[-1]), 0) + 1
    escolhidos = {}
    for (enum, variante), lista in sitios.items():
        if enum in excluidos or enum not in causas or len(causas[enum]) != 1:
            continue
        _, _, variantes = causas[enum][0]
        v = variantes.get(variante)
        if v is None or v.child_by_field_name('body') is not None:
            continue
        tipos_origem = {t for _, _, t in lista}
        if len(tipos_origem) != 1:
            continue
        tipo = tipos_origem.pop()
        if tipo.split('::')[-1].split('<')[0] not in nomes_causa:
            continue
        if construcoes.get((enum, variante), 0) > 0:
            continue
        escolhidos[(enum, variante)] = (tipo, lista)
    return escolhidos, causas


def aplicar(fontes, escolhidos, causas):
    edicoes = {}
    for (enum, variante), (tipo, lista) in escolhidos.items():
        a, _, variantes = causas[enum][0]
        v = variantes[variante]
        f = fontes[a][0]
        nome = v.child_by_field_name('name')
        caminho = tipo if tipo.startswith('crate::') else f'crate::{tipo}' if '::' in tipo else tipo
        linha = f.rfind(b'\n', 0, v.start_byte) + 1
        recuo = f[linha:v.start_byte].decode()
        edicoes.setdefault(a, []).append((linha, linha, f'{recuo}#[causa(fonte)]\n'))
        edicoes.setdefault(a, []).append((nome.end_byte, nome.end_byte, f'({caminho})'))
        for arq, clo, _ in lista:
            corpo = clo.child_by_field_name('body')
            edicoes.setdefault(arq, []).append((clo.start_byte, clo.end_byte, texto(corpo, fontes[arq][0])))
    for a, lista in edicoes.items():
        f = bytearray(fontes[a][0])
        for ini, fim, novo in sorted(set(lista), key=lambda x: (x[0], x[1]), reverse=True):
            f[ini:fim] = novo.encode()
        a.write_bytes(bytes(f))
    return edicoes


def corrigir(escolhidos):
    """Ajusta padrões e asserts que o compilador apontar; devolve erros restantes."""
    variantes = {v for _, v in escolhidos}
    for _ in range(6):
        erros = cargo('test', '--no-run') + cargo('build')
        if not erros:
            return []
        mexeu = False
        for m in erros:
            for sp in m['spans']:
                if not sp['is_primary']:
                    continue
                arq = RAIZ / sp['file_name']
                if not sp['file_name'].startswith('src/') or not arq.exists():
                    continue
                linhas = arq.read_text(encoding='utf-8').split('\n')
                i = sp['line_start'] - 1
                antes = linhas[i]
                depois = antes
                for v in variantes:
                    # padrão `E::X` sem campo -> `E::X(_)`
                    depois = re.sub(rf'(\b[A-Z]\w*::{v})\b(?!\s*[({{])', r'\1(_)', depois)
                depois = re.sub(r'assert_eq!\((.*),\s*Err\(([\w:]+\(_\))\)\)', r'assert!(matches!(\1, Err(\2)))', depois)
                if depois != antes:
                    linhas[i] = depois
                    arq.write_text('\n'.join(linhas), encoding='utf-8')
                    mexeu = True
        if not mexeu:
            return erros
    return cargo('test', '--no-run') + cargo('build')


def main():
    excluidos = set()
    print('lendo tipos pelo compilador...', flush=True)
    tipos = tipos_das_origens()
    print(f'{len(tipos)} sítios com tipo', flush=True)
    for rodada in range(4):
        fontes = {a: (a.read_bytes(), None) for a in arquivos()}
        fontes = {a: (f, PARSER.parse(f)) for a, (f, _) in fontes.items()}
        escolhidos, causas = planejar(fontes, tipos, excluidos)
        sitios = sum(len(l) for _, l in escolhidos.values())
        print(f'rodada {rodada}: {len(escolhidos)} variantes, {sitios} map_err', flush=True)
        if '--aplicar' not in sys.argv or not escolhidos:
            for (e, v), (t, l) in sorted(escolhidos.items()):
                print(f'  {e}::{v} <- {t} ({len(l)})')
            return
        aplicar(fontes, escolhidos, causas)
        erros = corrigir(escolhidos)
        if not erros:
            print('compila; feito')
            return
        culpados = set()
        for m in erros:
            for (e, v) in escolhidos:
                if v in m['rendered'] or e in m['rendered']:
                    culpados.add(e)
        print(f'  {len(erros)} erros; excluindo {sorted(culpados) or "tudo"}')
        subprocess.run(['git', 'checkout', '-q', '--', 'src'], cwd=RAIZ, check=True)
        if not culpados:
            return
        excluidos |= culpados


if __name__ == '__main__':
    main()
