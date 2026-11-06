"""Formas curtas do codemod `migrar_erro_html`.

- `?` e `return Err(..)` usam o `From<ErroHtml>` derivado da variante
  `Html`, sem escrevê-la, quando a função devolve o próprio `enum`.
- O caminho `crate::erro_html::ErroHtml` vira `ErroHtml`, com um `use`.
"""

import re

from migrar_erro_html_plano import TIPO, atributos_antes, mesmo, nos, texto


def em_retorno(no):
    """O valor de `no` é o da função: cauda de bloco, braço, `if`, até o corpo."""
    while True:
        p = no.parent
        if p is None:
            return False
        if p.type == 'block':
            if not mesmo(p.named_children[-1], no):
                return False
            if p.parent is not None and p.parent.type == 'function_item':
                return True
        elif p.type == 'match_arm':
            if not mesmo(p.child_by_field_name('value'), no):
                return False
        elif p.type == 'match_block':
            pass
        elif p.type == 'expression_statement':
            # `match`/`if` na cauda: o tree-sitter o embrulha, sem `;`.
            if p.children[-1].type == ';':
                return False
        elif p.type == 'match_expression':
            if mesmo(p.child_by_field_name('value'), no):
                return False
        elif p.type in ('if_expression', 'else_clause'):
            if p.type == 'if_expression' and mesmo(p.child_by_field_name('condition'), no):
                return False
        elif p.type == 'return_expression':
            return True
        else:
            return False
        no = p


def forma_curta(plano, a, f, s, nome):
    """'try' em `.ok_or(X)?`/`.map_err(|_| X)?`, 'into' em `return Err(X)`,
    quando a função devolve o próprio `enum` e o `From<ErroHtml>` existe."""
    if nome in plano.some or any(t.split('::')[-1] in plano.some for t in plano.enums[nome].internas.values()):
        return None
    p = s.parent
    ancora, forma = None, None
    if p.type == 'closure_expression' and mesmo(p.child_by_field_name('body'), s):
        args = p.parent
        if args is not None and args.type == 'arguments':
            p, s = args, p
    if p.type == 'arguments' and p.parent is not None and p.parent.type == 'call_expression':
        chamada = p.parent
        fn = chamada.child_by_field_name('function')
        avo = chamada.parent
        if fn.type == 'field_expression' and texto(fn.child_by_field_name('field'), f) in ('ok_or', 'map_err') \
                and avo is not None and avo.type == 'try_expression' and len(p.named_children) == 1:
            ancora, forma = avo, 'try'
        elif fn.type == 'identifier' and texto(fn, f) == 'Err' and avo is not None and (
                avo.type == 'return_expression' or em_retorno(chamada)):
            ancora, forma = avo, 'into'
    if ancora is None:
        return None
    q = ancora.parent
    while q is not None and q.type != 'function_item':
        if q.type == 'closure_expression':
            return None
        q = q.parent
    if q is None:
        return None
    retorno = q.child_by_field_name('return_type')
    if retorno is None or retorno.type != 'generic_type' or texto(retorno.child_by_field_name('type'), f) != 'Result':
        return None
    args = retorno.child_by_field_name('type_arguments').named_children
    if len(args) != 2 or plano.res.nome(a, q, texto(args[1], f)) != nome:
        return None
    return forma


def faixas_de_teste(f, arv):
    saida = []
    for m in nos(arv.root_node, 'mod_item'):
        if m.child_by_field_name('body') is None:
            continue
        if any(texto(x, f) == '#[cfg(test)]' for x in atributos_antes(m)):
            saida.append((m.start_byte, m.end_byte))
    return saida


def nome_curto(plano, edicoes):
    """`ErroHtml` no lugar do caminho inteiro, com um `use` no topo do arquivo.

    Arquivo que não é de teste e só usa o nome em `mod tests` em linha fica
    com o caminho inteiro: o `use` seria morto fora dos testes.
    """
    por_arquivo = {}
    for e in edicoes:
        por_arquivo.setdefault(e[0], []).append(e)
    saida = []
    for a, lista in por_arquivo.items():
        f, arv = plano.fontes[a]
        usa = [e for e in lista if TIPO in e[3] and 'use ' not in e[3]]
        teste = a.stem.startswith(('test', 'acervo')) or 'tests' in a.parts
        faixas = faixas_de_teste(f, arv)
        fora = [e for e in usa if not any(i <= e[1] < j for i, j in faixas)]
        if not usa or not (teste or fora) or re.search(rb'\bErroHtml\b', f):
            saida.extend(lista)
            continue
        for e in lista:
            saida.append((e[0], e[1], e[2], e[3].replace(TIPO, 'ErroHtml')) if TIPO in e[3] and 'use ' not in e[3]
                         else e)
        primeiro = next((u for u in arv.root_node.named_children if u.type == 'use_declaration'), None)
        if primeiro is not None:
            ini = f.rfind(b'\n', 0, primeiro.start_byte) + 1
            saida.append((a, ini, ini, f'use {TIPO};\n'))
        else:
            fim = 0
            for c in arv.root_node.children:
                if c.type == 'line_comment' and texto(c, f).startswith('//!'):
                    fim = c.end_byte
                else:
                    break
            saida.append((a, fim, fim, (f'\nuse {TIPO};\n' if fim else f'use {TIPO};\n')))
    return saida
