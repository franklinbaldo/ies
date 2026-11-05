"""Codemod: `.map_err(Enum::Caso)?` → `?` onde o `From` derivado já compõe.

Uso (ferramenta de desenvolvimento, fora do gate):

    uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \
        python scripts/migrar_interrogacao.py [--aplicar]

Só troca quando:
- `Enum` deriva `Causa` e `Caso` é `#[causa(interna)]` com tipo interno que
  nenhuma outra variante `interna` repete (o derive gera o `From`);
- o `?` não está dentro de uma closure;
- o tipo de retorno da função envolvente cita `Enum` (ou o apelido `use
  Enum as X`), para que o `From` usado seja o do mesmo `Enum`.

Também lista os `enum` com tipo interno repetido: duas etapas achatadas
num nível, a separar em tipos próprios.
"""

import re
import sys
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsr

RAIZ = Path(__file__).resolve().parents[1]
PARSER = ts.Parser(ts.Language(tsr.language()))


def texto(no, fonte):
    return fonte[no.start_byte:no.end_byte].decode()


def nos(raiz, tipo):
    pilha = [raiz]
    while pilha:
        no = pilha.pop()
        if no.type == tipo:
            yield no
        pilha.extend(no.named_children)


def compoe(fontes):
    """{Enum: {Caso}} com From derivado; e {Enum: [tipo repetido]}."""
    permitidos, repetidos = {}, {}
    for arquivo, (fonte, arvore) in fontes.items():
        for e in nos(arvore.root_node, 'enum_item'):
            derive = e.prev_named_sibling
            achou = False
            while derive is not None and derive.type == 'attribute_item':
                if 'Causa' in texto(derive, fonte) and texto(derive, fonte).startswith('#[derive('):
                    achou = True
                derive = derive.prev_named_sibling
            if not achou:
                continue
            nome = texto(e.child_by_field_name('name'), fonte)
            internas = []
            corpo = e.child_by_field_name('body')
            interna = False
            for c in corpo.named_children:
                if c.type == 'attribute_item':
                    interna = interna or re.search(r'#\[causa\([^)]*\binterna\b', texto(c, fonte)) is not None
                    continue
                if c.type != 'enum_variant':
                    continue
                if interna:
                    campo = c.child_by_field_name('body')
                    tipos = [t for t in campo.named_children if t.type != 'attribute_item'] if campo else []
                    if len(tipos) == 1:
                        internas.append((texto(c.child_by_field_name('name'), fonte), re.sub(r'\s+', '', texto(tipos[0], fonte))))
                interna = False
            contagem = {}
            for _, t in internas:
                contagem[t] = contagem.get(t, 0) + 1
            permitidos.setdefault(nome, set()).update(v for v, t in internas if contagem[t] == 1)
            dup = sorted(t for t, n in contagem.items() if n > 1)
            if dup:
                repetidos[f'{nome} ({arquivo.relative_to(RAIZ)})'] = dup
    return permitidos, repetidos


def apelidos(arvore, fonte):
    """`use X as Y;` → {Y: X} (último segmento)."""
    mapa = {}
    for u in nos(arvore.root_node, 'use_as_clause'):
        caminho = texto(u.child_by_field_name('path'), fonte).split('::')[-1]
        alias = texto(u.child_by_field_name('alias'), fonte)
        mapa[alias] = caminho
    return mapa


def trocas(arquivo, fonte, arvore, permitidos):
    mapa = apelidos(arvore, fonte)
    edicoes = []
    for t in nos(arvore.root_node, 'try_expression'):
        chamada = t.named_children[0] if t.named_children else None
        if chamada is None or chamada.type != 'call_expression':
            continue
        funcao = chamada.child_by_field_name('function')
        if funcao.type != 'field_expression' or texto(funcao.child_by_field_name('field'), fonte) != 'map_err':
            continue
        args = chamada.child_by_field_name('arguments').named_children
        if len(args) != 1 or args[0].type != 'scoped_identifier':
            continue
        partes = texto(args[0], fonte).split('::')
        if len(partes) != 2:
            continue
        prefixo, caso = partes
        enum = mapa.get(prefixo, prefixo)
        if caso not in permitidos.get(enum, set()):
            continue
        pai = t.parent
        dentro_de_closure = False
        while pai is not None and pai.type != 'function_item':
            if pai.type == 'closure_expression':
                dentro_de_closure = True
                break
            pai = pai.parent
        if dentro_de_closure or pai is None:
            continue
        retorno = pai.child_by_field_name('return_type')
        if retorno is None:
            continue
        palavras = set(re.findall(r'\w+', texto(retorno, fonte)))
        if enum not in palavras and prefixo not in palavras:
            continue
        receptor = funcao.child_by_field_name('value')
        edicoes.append((chamada.start_byte, chamada.end_byte, texto(receptor, fonte)))
    return edicoes


def main():
    fontes = {}
    for arquivo in sorted((RAIZ / 'src').rglob('*.rs')):
        fonte = arquivo.read_bytes()
        fontes[arquivo] = (fonte, PARSER.parse(fonte))
    permitidos, repetidos = compoe(fontes)
    total = 0
    alteradas = {}
    for arquivo, (fonte, arvore) in fontes.items():
        edicoes = trocas(arquivo, fonte, arvore, permitidos)
        if edicoes:
            alteradas[arquivo] = edicoes
            total += len(edicoes)
    print(f'{total} map_err trocáveis por ? em {len(alteradas)} arquivos')
    print(f'{len(repetidos)} enum com tipo interno repetido (etapas a separar):')
    for nome, tipos in sorted(repetidos.items()):
        print(f'  {nome}: {", ".join(tipos)}')
    if '--aplicar' in sys.argv:
        for arquivo, edicoes in alteradas.items():
            fonte = bytearray(fontes[arquivo][0])
            for inicio, fim, novo in sorted(edicoes, reverse=True):
                fonte[inicio:fim] = novo.encode()
            arquivo.write_bytes(bytes(fonte))


if __name__ == '__main__':
    main()
