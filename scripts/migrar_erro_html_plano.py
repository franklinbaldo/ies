"""Plano do codemod `migrar_erro_html`: candidatas, nomes e mensagens.

Lê os fontes com tree-sitter, acha as variantes de forma dos `enum` que
derivam `Causa`, resolve os nomes (`use`, `type`, `Self`) e monta o
`onde` e a mensagem de cada uma.
"""

import re
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsr

RAIZ = Path(__file__).resolve().parents[1]
PARSER = ts.Parser(ts.Language(tsr.language()))
TIPO = 'crate::erro_html::ErroHtml'
PADROES = [
    ('ausente', ('Ausente', 'NaoEncontrad', 'SemValor')),
    ('repetido', ('Repetid', 'Duplicad', 'MaisDeUm', 'Varios', 'Varias')),
    ('html', ('Html', 'NaoInterpretad', 'Malformad')),
    ('valor', ('NaoReconhecid', 'ForaDa', 'Invalid', 'Inesperad', 'NaoPrevist', 'Desconhecid', 'NaoUnic',
               'Excessiv')),
]
GENERICA = {
    'ausente': '"a tela não tem o elemento observado"',
    'repetido': '"o elemento aparece mais de uma vez"',
    'valor': '"o valor não está na forma observada"',
    'html': '"o HTML não pôde ser interpretado"',
}
# Não são leitores de tela: o próprio tipo de destino e a causa que ele carrega.
FORA = {'Problema', 'ErroAtributoHtml'}
TIPOS_DE_PADRAO = {'or_pattern', 'tuple_struct_pattern', 'tuple_pattern', 'slice_pattern', 'reference_pattern',
                   'ref_pattern', 'captured_pattern', 'field_pattern', 'mut_pattern', 'struct_pattern'}


def problema(nome):
    for p, partes in PADROES:
        if any(s in nome for s in partes):
            return p
    return None


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


def nos(raiz, *tipos):
    pilha = [raiz]
    while pilha:
        no = pilha.pop()
        if no.type in tipos:
            yield no
        pilha.extend(reversed(no.children))


def mesmo(a, b):
    return a is not None and b is not None and (a.start_byte, a.end_byte, a.type) == (b.start_byte, b.end_byte, b.type)


def arquivos():
    return sorted((RAIZ / 'src').rglob('*.rs'))


def ler_fontes():
    saida = {}
    for a in arquivos():
        f = a.read_bytes()
        saida[a] = (f, PARSER.parse(f))
    return saida


def em_teste(no, fonte, arquivo):
    if arquivo.stem.startswith(('test', 'acervo')) or 'tests' in arquivo.parts:
        return True
    p = no.parent
    while p is not None:
        if p.type in ('mod_item', 'function_item'):
            irmao = p.prev_named_sibling
            while irmao is not None and irmao.type == 'attribute_item':
                t = texto(irmao, fonte)
                if t in ('#[cfg(test)]', '#[test]'):
                    return True
                irmao = irmao.prev_named_sibling
        p = p.parent
    return False


def atributos_antes(no):
    """Atributos e comentários colados antes do item, do mais próximo ao primeiro."""
    saida = []
    irmao = no.prev_sibling
    while irmao is not None and irmao.type in ('attribute_item', 'line_comment', 'block_comment'):
        saida.append(irmao)
        irmao = irmao.prev_sibling
    return saida


def linha_inteira(fonte, inicio, fim):
    """Estende [inicio, fim) às linhas inteiras, com a vírgula e o fim de linha."""
    ini = fonte.rfind(b'\n', 0, inicio) + 1
    nl = fonte.find(b'\n', fim)
    return ini, (len(fonte) if nl < 0 else nl + 1)


class Enum:
    def __init__(self, arquivo, no, fonte):
        self.arquivo = arquivo
        self.no = no
        self.nome = texto(no.child_by_field_name('name'), fonte)
        self.variantes = {}   # nome -> (nó, público, unidade)
        self.tipos = {}       # nome -> tipo do campo, em tupla de um campo
        self.internas = {}    # nome -> tipo, em #[causa(interna)]
        for v in no.child_by_field_name('body').named_children:
            if v.type != 'enum_variant':
                continue
            nome = texto(v.child_by_field_name('name'), fonte)
            publico = snake(nome)
            interna = False
            for a in atributos_antes(v):
                t = texto(a, fonte)
                r = re.search(r'#\[causa\((?:.*,\s*)?nome = "([^"]+)"', t)
                if r:
                    publico = r.group(1)
                interna = interna or ('#[causa(' in t and 'interna' in t)
            corpo = v.child_by_field_name('body')
            if corpo is not None and corpo.type == 'ordered_field_declaration_list':
                campos = [c for c in corpo.named_children if c.type not in ('attribute_item', 'visibility_modifier')]
                if len(campos) == 1:
                    self.tipos[nome] = texto(campos[0], fonte)
                    if interna:
                        self.internas[nome] = self.tipos[nome]
            self.variantes[nome] = (v, publico, corpo is None)


def enums_com_causa(fontes):
    todos, causa = {}, {}
    for a, (f, arv) in fontes.items():
        for e in nos(arv.root_node, 'enum_item'):
            nome = texto(e.child_by_field_name('name'), f)
            todos[nome] = todos.get(nome, 0) + 1
            deriva = any(texto(x, f).startswith('#[derive(') and 'Causa' in texto(x, f) for x in atributos_antes(e))
            if not deriva or em_teste(e, f, a) or a.name == 'erro_html.rs':
                continue
            if e.child_by_field_name('type_parameters') is not None:
                continue
            causa.setdefault(nome, []).append(Enum(a, e, f))
    return {n: l[0] for n, l in causa.items() if len(l) == 1 and todos[n] == 1 and n not in FORA}


# --- resolução de nomes ---------------------------------------------------

def apelidos(arv, f):
    """{chave do escopo: {apelido: nome}} dos `use` do arquivo."""
    escopos = {}

    def registrar(escopo, caminho, apelido):
        partes = caminho.split('::')
        escopos.setdefault(chave(escopo), {})[apelido or partes[-1]] = (partes[-1], partes)

    def percorrer(escopo, prefixo, no):
        if no.type in ('identifier', 'scoped_identifier', 'self', 'crate', 'super'):
            registrar(escopo, prefixo + texto(no, f), None)
        elif no.type == 'use_as_clause':
            registrar(escopo, prefixo + texto(no.child_by_field_name('path'), f),
                      texto(no.child_by_field_name('alias'), f))
        elif no.type == 'scoped_use_list':
            caminho = no.child_by_field_name('path')
            novo = prefixo + (texto(caminho, f) + '::' if caminho is not None else '')
            for c in no.child_by_field_name('list').named_children:
                percorrer(escopo, novo, c)
        elif no.type == 'use_list':
            for c in no.named_children:
                percorrer(escopo, prefixo, c)

    for u in nos(arv.root_node, 'use_declaration'):
        percorrer(u.parent, '', u.child_by_field_name('argument'))
    for t in nos(arv.root_node, 'type_item'):
        valor = t.child_by_field_name('type')
        if valor.type in ('type_identifier', 'scoped_type_identifier'):
            registrar(t.parent, texto(valor, f), texto(t.child_by_field_name('name'), f))
    return escopos


def chave(no):
    return (no.start_byte, no.end_byte, no.type)


class Resolvedor:
    def __init__(self, fontes, enums):
        self.fontes = fontes
        self.enums = enums
        self.escopos = {a: apelidos(arv, f) for a, (f, arv) in fontes.items()}

    def nome(self, arquivo, no, caminho):
        """O `enum` que `caminho` (texto antes de `::V`) nomeia em `no`."""
        f = self.fontes[arquivo][0]
        partes = caminho.split('::')
        if partes == ['Self']:
            p = no.parent
            while p is not None and p.type != 'impl_item':
                p = p.parent
            if p is None or p.child_by_field_name('trait') is not None:
                return None
            tipo = p.child_by_field_name('type')
            return self.nome(arquivo, p, texto(tipo, f))
        ultimo = partes[-1]
        if len(partes) == 1:
            p = no
            while p is not None:
                apel = self.escopos[arquivo].get(chave(p), {})
                if ultimo in apel:
                    alvo, cam = apel[ultimo]
                    if alvo == ultimo:
                        break
                    if len(cam) > 1 and cam[-2] in self.enums:
                        return None  # `use E::V as A`: variante importada
                    return alvo if alvo in self.enums else None
                p = p.parent
        return ultimo if ultimo in self.enums else None

    def apelido(self, arquivo, no, caminho):
        """O texto é apelido local (`use E as X`)?"""
        if '::' in caminho or caminho == 'Self':
            return False
        p = no
        while p is not None:
            apel = self.escopos[arquivo].get(chave(p), {})
            if caminho in apel:
                return apel[caminho][0] != caminho
            p = p.parent
        return False


# --- mensagens ------------------------------------------------------------

def bracos_de_mensagem(fontes, enums, res):
    """{enum: (nó do corpo do match, [(braço, [(alternativa, variante|None)], valor)])}."""
    saida = {}
    for a, (f, arv) in fontes.items():
        for imp in nos(arv.root_node, 'impl_item'):
            if imp.child_by_field_name('trait') is not None:
                continue
            nome = res.nome(a, imp, texto(imp.child_by_field_name('type'), f))
            if nome is None:
                continue
            for fn in imp.child_by_field_name('body').named_children:
                if fn.type != 'function_item' or texto(fn.child_by_field_name('name'), f) != 'mensagem':
                    continue
                corpo = fn.child_by_field_name('body').named_children
                if len(corpo) == 1 and corpo[0].type == 'expression_statement':
                    corpo = corpo[0].named_children
                if len(corpo) != 1 or corpo[0].type != 'match_expression':
                    continue
                m = corpo[0]
                if texto(m.child_by_field_name('value'), f) not in ('self', '*self'):
                    continue
                bracos = []
                for b in m.child_by_field_name('body').named_children:
                    if b.type != 'match_arm':
                        continue
                    padrao = b.child_by_field_name('pattern')
                    alternativas = []
                    if padrao.child_by_field_name('condition') is None and len(padrao.named_children) == 1:
                        raiz = padrao.named_children[0]
                        pilha = [raiz]
                        while pilha:
                            p = pilha.pop(0)
                            if p.type == 'or_pattern':
                                pilha[:0] = list(p.named_children)
                            elif p.type == 'scoped_identifier':
                                cam = p.child_by_field_name('path')
                                dono = res.nome(a, p, texto(cam, f)) if cam is not None else None
                                alternativas.append((p, texto(p.child_by_field_name('name'), f) if dono == nome else None))
                            else:
                                alternativas.append((p, '_' if texto(p, f) == '_' else None))
                    bracos.append((b, alternativas, b.child_by_field_name('value')))
                saida[nome] = (a, fn, m.child_by_field_name('body'), bracos)
    return saida


def mensagem_de(variante, prob, bracos, f):
    if bracos is None:
        return GENERICA[prob]
    for _, alternativas, valor in bracos:
        for _, v in alternativas:
            if v in (variante, '_'):
                # `V => { "texto" }`: o rustfmt põe o literal longo num bloco.
                if valor.type == 'block' and len(valor.named_children) == 1:
                    valor = valor.named_children[0]
                if valor.type == 'string_literal':
                    return texto(valor, f)
                return GENERICA[prob]
    return GENERICA[prob]


# --- plano ----------------------------------------------------------------

class Plano:
    def __init__(self, fontes, excluidos, origens):
        self.fontes = fontes
        self.enums = enums_com_causa(fontes)
        self.res = Resolvedor(fontes, self.enums)
        self.mensagens = bracos_de_mensagem(fontes, self.enums, self.res)
        self.migra = {}  # enum -> {variante: (problema, onde, mensagem)}
        self.encontradas = 0
        for nome, e in self.enums.items():
            f = self.fontes[e.arquivo][0]
            bracos = self.mensagens.get(nome, (None, None, None, None))[3]
            for v, (_, publico, unidade) in e.variantes.items():
                p = problema(v) if unidade else None
                if p is None:
                    continue
                self.encontradas += 1
                if (nome, v) in excluidos:
                    continue
                fm = self.fontes[self.mensagens[nome][0]][0] if nome in self.mensagens else f
                self.migra.setdefault(nome, {})[v] = (p, publico, mensagem_de(v, p, bracos, fm))
        self.some = {n for n, vs in self.migra.items()
                     if set(vs) == set(self.enums[n].variantes) and self.eliminavel(n)}
        self.origens = origens

    def eliminavel(self, nome):
        """Só de forma, e sem métodos além de `mensagem`/`subcodigo`."""
        for a, (f, arv) in self.fontes.items():
            for imp in nos(arv.root_node, 'impl_item'):
                if self.res.nome(a, imp, texto(imp.child_by_field_name('type'), f)) != nome:
                    continue
                if imp.child_by_field_name('trait') is not None:
                    return False
                for fn in imp.child_by_field_name('body').named_children:
                    if fn.type == 'function_item' and texto(fn.child_by_field_name('name'), f) not in (
                            'mensagem', 'subcodigo'):
                        return False
        return True

    def construcao(self, nome, v, prefixo, apelido, causa=None):
        """`ErroHtml::<problema>("onde")`; a mensagem só vai se não é a genérica."""
        p, onde, msg = self.migra[nome][v]
        ctor = p if causa is None else 'atributo'
        args = f'"{onde}"' + (f', {causa}' if causa else '')
        propria = '' if msg == GENERICA[p] else f'.com_mensagem({msg})'
        if nome in self.some:
            base = prefixo if apelido else TIPO
            return f'{base}::{ctor}({args}){propria}'
        return f'{prefixo}::Html({TIPO}::{ctor}({args}){propria})'

    def padrao(self, nome, v, prefixo, apelido):
        _, onde, _ = self.migra[nome][v]
        if nome in self.some:
            base = prefixo if apelido else TIPO
            return f'{base} {{ onde: "{onde}", .. }}'
        return f'{prefixo}::Html({TIPO} {{ onde: "{onde}", .. }})'
