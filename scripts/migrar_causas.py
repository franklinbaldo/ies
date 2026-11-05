"""Codemod: troca `fn subcodigo()` manuais por `#[derive(Causa)]`.

Uso (ferramenta de desenvolvimento, fora do gate):

    uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \
        python scripts/migrar_causas.py [--aplicar]

Para cada `impl T { fn subcodigo(&self) -> &'static str { match self {..} } }`
cujos braços são só `Self::V.. => "literal"` ou `Self::V(x) => x.subcodigo()`:
- acrescenta `crate::erro_publico::Causa` ao `#[derive]` do `enum T`;
- marca `#[causa(nome = "...")]` onde o literal difere do `snake_case` da
  variante e `#[causa(interna)]` onde o braço delega;
- troca o corpo por `crate::erro_publico::folha(self)`.

Um tipo só migra depois dos tipos internos para os quais delega (ponto
fixo). Braço com guarda, padrão aninhado ou `_` deixa o tipo como está.
Os nomes públicos não mudam: a suíte de contrato é a prova.
"""

import sys
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsr

RAIZ = Path(__file__).resolve().parents[1]
PARSER = ts.Parser(ts.Language(tsr.language()))
JA_MIGRADOS = {'FalhaTransporte', 'FalhaDoProxy'}


def snake(ident):
    saida = []
    for i, atual in enumerate(ident):
        if i:
            antes = ident[i - 1]
            depois = ident[i + 1] if i + 1 < len(ident) else ''
            maiuscula = atual.isupper() and (not antes.isupper() or depois.islower())
            numero = atual.isdigit() and not antes.isdigit()
            if maiuscula or numero:
                saida.append('_')
        saida.append(atual.lower())
    return ''.join(saida)


def texto(no, fonte):
    return fonte[no.start_byte:no.end_byte].decode()


def filhos(no, tipo):
    return [c for c in no.named_children if c.type == tipo]


class Enum:
    def __init__(self, arquivo, no, fonte):
        self.arquivo = arquivo
        self.no = no
        self.nome = texto(no.child_by_field_name('name'), fonte)
        self.generico = no.child_by_field_name('type_parameters') is not None
        corpo = no.child_by_field_name('body')
        self.variantes = {}
        self.ok = not self.generico
        anterior_cfg = False
        for c in corpo.named_children:
            if c.type == 'attribute_item':
                anterior_cfg = anterior_cfg or texto(c, fonte).startswith('#[cfg(')
                continue
            if c.type != 'enum_variant':
                continue
            if anterior_cfg:
                self.ok = False
            anterior_cfg = False
            nome = texto(c.child_by_field_name('name'), fonte)
            corpo_v = c.child_by_field_name('body')
            if corpo_v is None:
                forma = ('unidade', None)
            elif corpo_v.type == 'ordered_field_declaration_list':
                tipos = [t for t in corpo_v.named_children if t.type not in ('attribute_item', 'visibility_modifier')]
                forma = ('tupla', [texto(t, fonte) for t in tipos])
            else:
                forma = ('registro', None)
            self.variantes[nome] = (c, forma)
        # #[derive(...)] imediatamente antes do enum
        self.derive = None
        irmao = no.prev_named_sibling
        while irmao is not None and irmao.type == 'attribute_item':
            if texto(irmao, fonte).startswith('#[derive('):
                self.derive = irmao
            irmao = irmao.prev_named_sibling


def ler_braco(braco, fonte):
    """[(variante, binding|None)], valor ('lit', s) | ('delega', binding)."""
    padrao = braco.child_by_field_name('pattern')
    if padrao is None or padrao.child_by_field_name('condition') is not None:
        return None
    if any(c.type == 'match_pattern' for c in [padrao]) and len(padrao.named_children) > 1:
        return None  # guarda
    alternativas = []

    def coletar(p):
        if p.type == 'match_pattern':
            if len(p.named_children) != 1:
                return False
            return coletar(p.named_children[0])
        if p.type == 'or_pattern':
            return all(coletar(c) for c in p.named_children)
        if p.type == 'scoped_identifier':
            caminho = texto(p, fonte)
            if not caminho.startswith('Self::'):
                return False
            alternativas.append((caminho[6:], None))
            return True
        if p.type == 'tuple_struct_pattern':
            tipo = texto(p.child_by_field_name('type'), fonte)
            if not tipo.startswith('Self::'):
                return False
            internos = [c for c in p.named_children if c != p.child_by_field_name('type')]
            if len(internos) == 1 and internos[0].type == 'identifier':
                alternativas.append((tipo[6:], texto(internos[0], fonte)))
                return True
            if all(c.type in ('remaining_field_pattern',) or texto(c, fonte) == '_' for c in internos):
                alternativas.append((tipo[6:], None))
                return True
            return False
        if p.type == 'struct_pattern':
            tipo = texto(p.child_by_field_name('type'), fonte)
            if not tipo.startswith('Self::'):
                return False
            alternativas.append((tipo[6:], None))
            return True
        return False

    if not coletar(padrao):
        return None
    valor = braco.child_by_field_name('value')
    if valor.type == 'block' and len(valor.named_children) == 1:
        valor = valor.named_children[0]
        if valor.type == 'expression_statement' and len(valor.named_children) == 1:
            valor = valor.named_children[0]
    if valor.type == 'string_literal':
        conteudo = texto(valor, fonte)[1:-1]
        if '\\' in conteudo or not conteudo:
            return None
        return alternativas, ('lit', conteudo)
    if valor.type == 'call_expression':
        funcao = valor.child_by_field_name('function')
        args = valor.child_by_field_name('arguments')
        if funcao.type == 'field_expression' and not args.named_children:
            alvo = texto(funcao.child_by_field_name('value'), fonte)
            campo = texto(funcao.child_by_field_name('field'), fonte)
            if campo == 'subcodigo' and all(b == alvo for _, b in alternativas):
                return alternativas, ('delega', alvo)
    return None


def candidatos():
    enums, impls = {}, []
    fontes = {}
    for arquivo in sorted((RAIZ / 'src').rglob('*.rs')):
        if 'test' in arquivo.name:
            continue
        fonte = arquivo.read_bytes()
        fontes[arquivo] = fonte
        arvore = PARSER.parse(fonte)
        pilha = [arvore.root_node]
        while pilha:
            no = pilha.pop()
            pilha.extend(no.named_children)
            if no.type == 'enum_item':
                e = Enum(arquivo, no, fonte)
                enums.setdefault(e.nome, []).append(e)
            if no.type == 'impl_item' and no.child_by_field_name('trait') is None:
                tipo = texto(no.child_by_field_name('type'), fonte)
                for f in filhos(no.child_by_field_name('body'), 'function_item'):
                    if texto(f.child_by_field_name('name'), fonte) != 'subcodigo':
                        continue
                    params = texto(f.child_by_field_name('parameters'), fonte)
                    retorno = f.child_by_field_name('return_type')
                    if params not in ('(&self)', '(self)') or retorno is None or texto(retorno, fonte) != "&'static str":
                        continue
                    corpo = f.child_by_field_name('body')
                    exprs = corpo.named_children
                    if len(exprs) == 1 and exprs[0].type == 'expression_statement':
                        exprs = exprs[0].named_children
                    if len(exprs) != 1 or exprs[0].type != 'match_expression':
                        continue
                    m = exprs[0]
                    if texto(m.child_by_field_name('value'), fonte) != 'self':
                        continue
                    bracos = filhos(m.child_by_field_name('body'), 'match_arm')
                    impls.append((tipo, arquivo, corpo, params, bracos))
    return enums, impls, fontes


def plano():
    enums, impls, fontes = candidatos()
    analisados = {}
    for tipo, arquivo, corpo, params, bracos in impls:
        lista = enums.get(tipo, [])
        if len(lista) != 1 or not lista[0].ok:
            continue
        e = lista[0]
        fonte = fontes[arquivo]
        por_variante = {}
        valido = True
        for b in bracos:
            lido = ler_braco(b, fonte)
            if lido is None:
                valido = False
                break
            alternativas, valor = lido
            for variante, _ in alternativas:
                if variante in por_variante or variante not in e.variantes:
                    valido = False
                por_variante[variante] = valor
        if not valido or set(por_variante) != set(e.variantes):
            continue
        analisados[tipo] = (e, arquivo, corpo, params, por_variante)
    # ponto fixo: delega só para tipo já migrado
    migrados = set(JA_MIGRADOS)
    escolhidos = {}
    mudou = True
    while mudou:
        mudou = False
        for tipo, (e, arquivo, corpo, params, por_variante) in analisados.items():
            if tipo in escolhidos:
                continue
            pronto = True
            for variante, valor in por_variante.items():
                if valor[0] != 'delega':
                    continue
                forma, tipos = e.variantes[variante][1]
                if forma != 'tupla' or len(tipos) != 1:
                    pronto = False
                    break
                interno = tipos[0].split('::')[-1]
                if interno not in migrados:
                    pronto = False
                    break
            if pronto:
                escolhidos[tipo] = analisados[tipo]
                migrados.add(tipo)
                mudou = True
    return escolhidos, analisados, fontes


def aplicar(escolhidos, fontes):
    edicoes = {}
    for tipo, (e, arquivo, corpo, params, por_variante) in escolhidos.items():
        fonte = fontes[arquivo]
        lista = edicoes.setdefault(arquivo, [])
        # derive
        if e.derive is not None:
            fim = e.derive.end_byte - 2  # antes de ")]"
            lista.append((fim, fim, ', crate::erro_publico::Causa'))
        else:
            inicio = e.no.start_byte
            linha = fonte.rfind(b'\n', 0, inicio) + 1
            recuo = fonte[linha:inicio].decode()
            lista.append((linha, linha, f'{recuo}#[derive(crate::erro_publico::Causa)]\n'))
        for variante, valor in por_variante.items():
            no, _ = e.variantes[variante]
            partes = []
            if valor[0] == 'delega':
                partes.append('interna')
            elif valor[1] != snake(variante):
                partes.append(f'nome = "{valor[1]}"')
            if not partes:
                continue
            linha = fonte.rfind(b'\n', 0, no.start_byte) + 1
            recuo = fonte[linha:no.start_byte].decode()
            lista.append((linha, linha, f'{recuo}#[causa({", ".join(partes)})]\n'))
        alvo = 'self' if params == '(&self)' else '&self'
        recuo_fn = ' ' * (corpo.start_point[1])
        linha_fn = fonte.rfind(b'\n', 0, corpo.start_byte) + 1
        base = len(fonte[linha_fn:corpo.start_byte]) - len(fonte[linha_fn:corpo.start_byte].lstrip())
        recuo = ' ' * base
        lista.append((corpo.start_byte, corpo.end_byte,
                      f'{{\n{recuo}    crate::erro_publico::folha({alvo})\n{recuo}}}'))
        del recuo_fn
    for arquivo, lista in edicoes.items():
        fonte = bytearray(fontes[arquivo])
        for inicio, fim, novo in sorted(lista, key=lambda x: (x[0], x[1]), reverse=True):
            fonte[inicio:fim] = novo.encode()
        arquivo.write_bytes(bytes(fonte))
    return edicoes


def main():
    escolhidos, analisados, fontes = plano()
    _, impls, _ = candidatos()
    print(f'{len(impls)} fn subcodigo candidatas; {len(analisados)} legíveis; {len(escolhidos)} migráveis')
    for tipo in sorted(set(analisados) - set(escolhidos)):
        print(f'  aguardando internos: {tipo}')
    if '--aplicar' in sys.argv:
        edicoes = aplicar(escolhidos, fontes)
        print(f'{len(edicoes)} arquivos alterados')


if __name__ == '__main__':
    main()
