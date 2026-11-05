"""Edições do codemod `migrar_erro_html`: ocorrências, `enum`, `use`,
`fn mensagem` e `map_err`, aplicadas de uma vez por arquivo.
"""

import re

from migrar_erro_html_curta import forma_curta, nome_curto
from migrar_erro_html_plano import (
    TIPO, TIPOS_DE_PADRAO, atributos_antes, em_teste, linha_inteira, mesmo, nos, texto,
)


def contexto_de_padrao(no):
    cur = no
    while cur.parent is not None:
        p = cur.parent
        if p.type in TIPOS_DE_PADRAO:
            if p.type == 'tuple_struct_pattern' and mesmo(p.child_by_field_name('type'), cur):
                return False
            cur = p
            continue
        if p.type == 'match_pattern':
            return True
        if p.type in ('let_declaration', 'let_condition', 'for_expression', 'parameter') \
                and mesmo(p.child_by_field_name('pattern'), cur):
            return True
        if p.type == 'closure_parameters':
            return True
        return False
    return False


def em_igualdade(no):
    p = no.parent
    while p is not None and p.type not in ('expression_statement', 'block', 'let_declaration', 'function_item'):
        if p.type == 'binary_expression' and p.child_by_field_name('operator').type in ('==', '!='):
            return True
        if p.type == 'call_expression':
            fn = p.child_by_field_name('function')
            if fn.type == 'field_expression' and fn.child_by_field_name('field').text.decode() in (
                    'contains', 'eq', 'ne'):
                return True
        p = p.parent
    return False


TOKEN = re.compile(r'(?<![\w:])((?:[A-Za-z_]\w*::)*[A-Za-z_]\w*)::([A-Z]\w*)\b(?!\s*(?:::|\(|\{|!))')


def ocorrencias(plano, fase_b):
    """[(arquivo, ini, fim, novo, (enum, variante), deixada)] das variantes migradas."""
    saida = []
    for a, (f, arv) in plano.fontes.items():
        res = plano.res
        for s in nos(arv.root_node, 'scoped_identifier'):
            p = s.parent
            if p is not None and p.type == 'scoped_identifier' and mesmo(p.child_by_field_name('path'), s):
                continue
            if p is not None and p.type == 'call_expression' and mesmo(p.child_by_field_name('function'), s):
                continue
            q = p
            dentro_de_use = False
            while q is not None:
                if q.type == 'use_declaration':
                    dentro_de_use = True
                    break
                q = q.parent
            if dentro_de_use:
                continue
            cam = s.child_by_field_name('path')
            if cam is None:
                continue
            prefixo = texto(cam, f)
            v = texto(s.child_by_field_name('name'), f)
            nome = res.nome(a, s, prefixo)
            if nome is None or v not in plano.migra.get(nome, {}):
                continue
            apel = res.apelido(a, s, prefixo)
            if contexto_de_padrao(s):
                novo = plano.padrao(nome, v, prefixo, apel)
                deixada = not fase_b
            else:
                novo = plano.construcao(nome, v, prefixo, apel, curta=forma_curta(plano, a, f, s, nome),
                                         teste=em_teste(s, f, a))
                deixada = not fase_b and em_igualdade(s)
            saida.append((a, s.start_byte, s.end_byte, novo, (nome, v), deixada))
        saida.extend(variantes_importadas(plano, a, f, arv, fase_b))
        for tt in nos(arv.root_node, 'token_tree'):
            if tt.parent is None or tt.parent.type != 'macro_invocation':
                continue
            macro = texto(tt.parent.child_by_field_name('macro'), f)
            corpo = texto(tt, f)
            base = tt.start_byte
            bruto = f[tt.start_byte:tt.end_byte]
            for r in TOKEN.finditer(corpo):
                prefixo, v = r.group(1), r.group(2)
                ini = base + len(corpo[:r.start()].encode())
                fim = base + len(corpo[:r.end()].encode())
                # o nó mais interno que contém a ocorrência, para escopo de `use`
                nome = res.nome(a, tt, prefixo)
                if nome is None or v not in plano.migra.get(nome, {}):
                    continue
                apel = res.apelido(a, tt, prefixo)
                if eh_padrao_de_matches(bruto, ini - base, macro):
                    novo = plano.padrao(nome, v, prefixo, apel)
                else:
                    novo = plano.construcao(nome, v, prefixo, apel, teste=em_teste(tt, f, a))
                saida.append((a, ini, fim, novo, (nome, v), not fase_b))
    return saida


APELIDOS_DE_VARIANTE = {}  # (arquivo, início do identificador) -> (enum, variante, prefixo)


def variantes_importadas(plano, a, f, arv, fase_b):
    """`use E::V as A;` numa função: cada `A` vira a construção e o `use` sai.

    Se `A` aparece num padrão ou numa macro, o `use` fica (fase A) e o
    compilador exclui a variante: sem o `use`, `A` num padrão viraria uma
    ligação que casa tudo, em silêncio.
    """
    saida = []
    for u in nos(arv.root_node, 'use_declaration'):
        arg = u.child_by_field_name('argument')
        if arg.type == 'use_as_clause':
            caminho, apelido = texto(arg.child_by_field_name('path'), f), texto(arg.child_by_field_name('alias'), f)
        elif arg.type == 'scoped_identifier':
            caminho = texto(arg, f)
            apelido = caminho.split('::')[-1]
        else:
            continue
        partes = caminho.split('::')
        if len(partes) < 2:
            continue
        prefixo, v = '::'.join(partes[:-1]), partes[-1]
        nome = plano.res.nome(a, u, prefixo)
        if nome is None or v not in plano.migra.get(nome, {}):
            continue
        apel = plano.res.apelido(a, u, prefixo)
        usos, arriscado = [], False
        for i in nos(u.parent, 'identifier'):
            if texto(i, f) != apelido or (u.start_byte <= i.start_byte < u.end_byte):
                continue
            pai = i.parent
            if pai.type == 'token_tree' or contexto_de_padrao(i) or pai.type in (
                    'match_pattern', 'or_pattern', 'let_declaration', 'let_condition'):
                arriscado = True
            usos.append(i)
        if arriscado and not fase_b:
            continue
        ini, fim = linha_inteira(f, u.start_byte, u.end_byte)
        saida.append((a, ini, fim, '', (nome, v), False))
        for i in usos:
            if contexto_de_padrao(i) or i.parent.type in ('match_pattern', 'or_pattern'):
                novo = plano.padrao(nome, v, prefixo, apel)
            else:
                novo = plano.construcao(nome, v, prefixo, apel, teste=em_teste(i, f, a))
                APELIDOS_DE_VARIANTE[(a, i.start_byte)] = (nome, v, prefixo, apel)
            saida.append((a, i.start_byte, i.end_byte, novo, (nome, v), False))
    return saida


def eh_padrao_de_matches(bruto, pos, macro):
    """Na árvore de tokens de `macro`, `pos` está no padrão de um `matches!`?"""
    pilha = []  # (início do grupo, é matches, vírgula de topo vista)
    i = 0
    texto_ = bruto.decode()
    alvo = len(bruto[:pos].decode())
    em_str = False
    while i < alvo:
        c = texto_[i]
        if em_str:
            if c == '\\':
                i += 2
                continue
            if c == '"':
                em_str = False
        elif c == '"':
            em_str = True
        elif c in '([{':
            antes = texto_[:i].rstrip()
            eh = antes.endswith('matches!') or (not pilha and macro.endswith('matches'))
            pilha.append([eh, False])
        elif c in ')]}':
            if pilha:
                pilha.pop()
        elif c == ',' and pilha:
            pilha[-1][1] = True
        i += 1
    for eh, virgula in reversed(pilha):
        if eh:
            return virgula
    return False


def edicoes_de_mensagem(plano):
    saida = []
    for nome, (a, fn, corpo, bracos) in plano.mensagens.items():
        migra = plano.migra.get(nome)
        if not migra:
            continue
        f = plano.fontes[a][0]
        if nome in plano.some:
            continue  # o impl inteiro sai
        cobertos = []
        for b, alternativas, _ in bracos:
            restam = [x for x, v in alternativas if v not in migra]
            if not alternativas or len(restam) == len(alternativas):
                continue
            if not restam:
                ini, fim = linha_inteira(f, b.start_byte, b.end_byte)
                saida.append((a, ini, fim, ''))
                cobertos.append(b)
            else:
                padrao = b.child_by_field_name('pattern').named_children[0]
                saida.append((a, padrao.start_byte, padrao.end_byte, ' | '.join(texto(x, f) for x in restam)))
        abre = corpo.start_byte + 1
        saida.append((a, abre, abre, '\n            Self::Html(e) => e.mensagem,'))
    return saida


def edicoes_de_enum(plano):
    saida = []
    for nome, migra in plano.migra.items():
        e = plano.enums[nome]
        f = plano.fontes[e.arquivo][0]
        if nome in plano.some:
            inicio = min([x.start_byte for x in atributos_antes(e.no)] + [e.no.start_byte])
            ini, fim = linha_inteira(f, inicio, e.no.end_byte)
            saida.append((e.arquivo, ini, fim, ''))
            continue
        for v in migra:
            no = e.variantes[v][0]
            inicio = min([x.start_byte for x in atributos_antes(no)] + [no.start_byte])
            ini, fim = linha_inteira(f, inicio, no.end_byte)
            saida.append((e.arquivo, ini, fim, ''))
        corpo = e.no.child_by_field_name('body')
        fecha = corpo.end_byte - 1
        saida.append((e.arquivo, fecha, fecha, f'    #[causa(transparente)]\n    Html({TIPO}),\n'))
    return saida


def edicoes_de_eliminacao(plano):
    """Tipo, `use` e `impl` de cada `enum` que some."""
    saida = []
    if not plano.some:
        return saida
    for a, (f, arv) in plano.fontes.items():
        res = plano.res
        for imp in nos(arv.root_node, 'impl_item'):
            if imp.child_by_field_name('trait') is None and \
                    res.nome(a, imp, texto(imp.child_by_field_name('type'), f)) in plano.some:
                inicio = min([x.start_byte for x in atributos_antes(imp)] + [imp.start_byte])
                ini, fim = linha_inteira(f, inicio, imp.end_byte)
                saida.append((a, ini, fim, ''))
        for u in nos(arv.root_node, 'use_declaration'):
            saida.extend(use_sem(plano, a, f, u))
        for t in nos(arv.root_node, 'type_identifier', 'scoped_type_identifier'):
            if t.parent is not None and t.parent.type == 'scoped_type_identifier':
                continue
            nome_t = texto(t, f).split('::')[-1]
            if t.parent is not None and t.parent.type in ('enum_item', 'impl_item', 'type_item') \
                    and mesmo(t.parent.child_by_field_name('name' if t.parent.type != 'impl_item' else 'type'), t):
                continue
            nome = res.nome(a, t, nome_t) if '::' not in texto(t, f) else (nome_t if nome_t in plano.enums else None)
            if nome in plano.some:
                if '::' not in texto(t, f) and res.apelido(a, t, nome_t):
                    continue
                saida.append((a, t.start_byte, t.end_byte, TIPO))
    return saida


def use_sem(plano, a, f, u):
    """`use` que importa um `enum` que some: tira o nome, ou troca o apelido."""
    arg = u.child_by_field_name('argument')
    saida = []

    def alvo(caminho):
        n = caminho.split('::')[-1]
        return n if n in plano.some else None

    if arg.type in ('scoped_identifier', 'identifier') and alvo(texto(arg, f)):
        ini, fim = linha_inteira(f, u.start_byte, u.end_byte)
        return [(a, ini, fim, '')]
    if arg.type == 'use_as_clause' and alvo(texto(arg.child_by_field_name('path'), f)):
        p = arg.child_by_field_name('path')
        return [(a, p.start_byte, p.end_byte, TIPO)]
    if arg.type != 'scoped_use_list' or arg.child_by_field_name('list') is None:
        return saida
    lista = arg.child_by_field_name('list')
    restam, apelidos_ = [], []
    for c in lista.named_children:
        if c.type in ('identifier', 'scoped_identifier') and alvo(texto(c, f)):
            continue
        if c.type == 'use_as_clause' and alvo(texto(c.child_by_field_name('path'), f)):
            apelidos_.append(texto(c.child_by_field_name('alias'), f))
            continue
        restam.append(texto(c, f))
    if len(restam) == len(lista.named_children):
        return saida
    linha = f.rfind(b'\n', 0, u.start_byte) + 1
    recuo = f[linha:u.start_byte].decode()
    recuo = recuo if not recuo.strip() else ''
    visib = texto(u, f).split('use ')[0]
    novos = ''.join(f'\n{recuo}{visib}use {TIPO} as {x};' for x in apelidos_)
    if not restam:
        ini, fim = linha_inteira(f, u.start_byte, u.end_byte)
        return [(a, ini, fim, novos.lstrip('\n') + '\n' if novos else '')]
    saida.append((a, lista.start_byte, lista.end_byte, '{' + ', '.join(restam) + '}'))
    if novos:
        saida.append((a, u.end_byte, u.end_byte, novos))
    return saida


def edicoes_de_map_err(plano):
    """`map_err(|_| E::V)` com origem `ErroAtributoHtml`: a causa sobe."""
    saida, contadas = [], 0
    for a, (f, arv) in plano.fontes.items():
        for c in nos(arv.root_node, 'call_expression'):
            fn = c.child_by_field_name('function')
            if fn.type != 'field_expression' or texto(fn.child_by_field_name('field'), f) != 'map_err':
                continue
            args = c.child_by_field_name('arguments').named_children
            if len(args) != 1 or args[0].type != 'closure_expression':
                continue
            clo = args[0]
            corpo = clo.child_by_field_name('body')
            if texto(clo.child_by_field_name('parameters'), f) != '|_|':
                continue
            if corpo.type == 'identifier' and (a, corpo.start_byte) in APELIDOS_DE_VARIANTE:
                nome, v, prefixo, apel = APELIDOS_DE_VARIANTE[(a, corpo.start_byte)]
            elif corpo.type == 'scoped_identifier' and corpo.child_by_field_name('path') is not None:
                cam = corpo.child_by_field_name('path')
                prefixo, v = texto(cam, f), texto(corpo.child_by_field_name('name'), f)
                nome = plano.res.nome(a, corpo, prefixo)
                apel = plano.res.apelido(a, corpo, prefixo)
            else:
                continue
            if nome is None or v not in plano.migra.get(nome, {}):
                continue
            linha = clo.start_point[0] + 1
            tipo = plano.origens.get((str(a), linha)) or plano.origens.get((str(a), c.start_point[0] + 1))
            if tipo is None or tipo.split('::')[-1] != 'ErroAtributoHtml':
                continue
            novo = plano.construcao(nome, v, prefixo, apel, causa='e', curta=forma_curta(plano, a, f, corpo, nome),
                                     teste=em_teste(corpo, f, a))
            saida.append((a, clo.start_byte, clo.end_byte, f'|e| {novo}'))
            contadas += 1
    return saida, contadas


def aplicar_edicoes(fontes, edicoes):
    """Aplica as edições; a contida noutra maior (braço, `impl` removido) cai.

    Devolve {arquivo: [(ini, fim, delta)]} para traduzir posições antigas.
    """
    por_arquivo = {}
    for a, ini, fim, novo in edicoes:
        por_arquivo.setdefault(a, set()).add((ini, fim, novo))
    deslocamentos = {}
    for a, conjunto in por_arquivo.items():
        mantidas, fim_atual = [], -1
        for ini, fim, novo in sorted(conjunto, key=lambda x: (x[0], x[0] != x[1], -x[1])):
            if ini < fim_atual:
                continue
            mantidas.append((ini, fim, novo))
            fim_atual = max(fim_atual, fim)
        f = bytearray(fontes[a][0])
        for ini, fim, novo in reversed(mantidas):
            f[ini:fim] = novo.encode()
        a.write_bytes(bytes(f))
        deslocamentos[a] = [(ini, fim, len(novo.encode()) - (fim - ini)) for ini, fim, novo in mantidas]
    return deslocamentos


def nova_posicao(deslocamentos, a, pos):
    return pos + sum(d for ini, fim, d in deslocamentos.get(a, []) if fim <= pos)


def transformar(plano, fase_b):
    APELIDOS_DE_VARIANTE.clear()
    ocs = ocorrencias(plano, fase_b)
    mapa, contadas = edicoes_de_map_err(plano)
    sobrepostas = {(a, i, j) for a, i, j, _ in mapa}
    edicoes = [(a, ini, fim, novo) for a, ini, fim, novo, _, deixada in ocs if not deixada]
    # a closure do map_err já carrega a construção
    edicoes = [e for e in edicoes if not any(e[0] == a and i <= e[1] < j for a, i, j in sobrepostas)]
    edicoes += mapa + edicoes_de_mensagem(plano) + edicoes_de_enum(plano) + edicoes_de_eliminacao(plano)
    edicoes = nome_curto(plano, edicoes)
    desl = aplicar_edicoes(plano.fontes, edicoes)
    deixadas = [(a, nova_posicao(desl, a, ini), chave_) for a, ini, fim, _, chave_, d in ocs if d]
    return deixadas, contadas
