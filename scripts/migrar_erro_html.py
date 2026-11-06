"""Codemod: variantes de "forma não observada" viram `ErroHtml`.

Uso (ferramenta de desenvolvimento, fora do gate; precisa de cargo):

    uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \
        python scripts/migrar_erro_html.py [--aplicar]

Candidata: variante unitária de um `enum` que deriva `Causa`, cujo nome
diz a forma (`...Ausente`, `...Repetida`, `HtmlIlegivel`, `...Invalido`).
`onde` é o nome público atual (`#[causa(nome)]` ou o `snake_case`); a
mensagem é o literal do braço da `fn mensagem()` do `enum`, ou uma
genérica pelo problema.

- `enum` misto: ganha `#[causa(transparente)] Html(ErroHtml)`, e `E::V`
  vira `E::Html(ErroHtml::<problema>("onde"))`; `.com_mensagem("...")` só
  quando a mensagem não é a genérica do problema.
- `enum` só de forma: some, e o tipo passa a ser `ErroHtml`.
- `map_err(|_| E::V)` cuja origem é `ErroAtributoHtml` (pelo compilador,
  com o truque `|_: ()|`) passa a carregar a causa.

Segurança: a variante casada fora de teste (padrão, `matches!`, `==`, ou
outro `match` além da `fn mensagem`) sustenta uma decisão e fica. O
compilador decide: a fase A não reescreve padrões, macros nem `==`, e cada
erro de `cargo build` (código que não é teste) devolve a variante à
exclusão; refaz-se do estado limpo. A fase B reescreve também os testes.
"""


import json
import os
import re
import subprocess
import sys
from pathlib import Path

from migrar_erro_html_edicoes import transformar
from migrar_erro_html_plano import RAIZ, Plano, arquivos, ler_fontes

ENV = dict(os.environ, CARGO_TARGET_DIR=os.environ.get('CARGO_TARGET_DIR', str(Path.home() / '.cargo-target-ies')))


# --- compilador -----------------------------------------------------------

AVISOS = []


def cargo(*args):
    r = subprocess.run(['cargo', *args, '-q', '--offline', '--message-format=json'], cwd=RAIZ, env=ENV,
                       capture_output=True, text=True, check=False)
    erros = []
    for linha in r.stdout.splitlines():
        try:
            m = json.loads(linha)
        except ValueError:
            continue
        if m.get('reason') == 'compiler-message' and m['message']['level'] == 'error':
            erros.append(m['message'])
        elif m.get('reason') == 'compiler-message' and m['message']['level'] == 'warning':
            AVISOS.append(m['message'])
    if r.returncode and not erros:
        print(r.stderr[-3000:])
        raise SystemExit('cargo falhou sem diagnóstico')
    return erros


def tipos_das_origens():
    """{(arquivo, linha): tipo da origem} dos `map_err(|_| ...)`."""
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
            tipos.setdefault((str(RAIZ / sp['file_name']), sp['line_start']), r.group(1).strip())
    return tipos


# --- correções pelo compilador --------------------------------------------

def linha_col(fonte, byte):
    linha = fonte.count(b'\n', 0, byte) + 1
    return linha


def modulo(arquivo):
    partes = list(arquivo.relative_to(RAIZ / 'src').with_suffix('').parts)
    if partes[-1] in ('mod', 'main'):
        partes.pop()
    return '::'.join(['crate', *partes])


def corrigir_interrogacao(plano, erros):
    """`?` sem `From` (tipo interno repetido): `.map_err(Externo::Caso)?`."""
    mexeu = False
    por_arquivo = {}
    for m in erros:
        r = re.search(r"`\?` couldn't convert the error to `([\w:]+)`", m['message'])
        if not r:
            continue
        externo = r.group(1).split('::')[-1]
        e = plano.enums.get(externo)
        if e is None:
            continue
        casos = [v for v, t in e.internas.items() if t.split('::')[-1] in plano.some]
        if len(casos) != 1:
            print(f'  ? ambíguo em {externo}: {casos}')
            continue
        for sp in m['spans']:
            if sp['is_primary']:
                por_arquivo.setdefault(RAIZ / sp['file_name'], []).append(
                    (sp['line_start'], sp['column_start'], f'{modulo(e.arquivo)}::{externo}::{casos[0]}'))
    for a, lista in por_arquivo.items():
        linhas = a.read_text(encoding='utf-8').split('\n')
        for linha, coluna, caso in sorted(lista, reverse=True):
            s = linhas[linha - 1]
            i = coluna - 1
            if s[i:i + 1] == '?':
                linhas[linha - 1] = s[:i] + f'.map_err({caso})' + s[i:]
                mexeu = True
        a.write_text('\n'.join(linhas), encoding='utf-8')
    return mexeu


def remover_usos_mortos():
    """`use E as X;` que só servia às variantes migradas (agora `ErroHtml`)."""
    linhas_mortas = {}
    for m in AVISOS:
        if not m['message'].startswith('unused import'):
            continue
        for sp in m['spans']:
            if sp['is_primary']:
                linhas_mortas.setdefault(RAIZ / sp['file_name'], set()).add(sp['line_start'])
    for a, numeros in linhas_mortas.items():
        linhas = a.read_text(encoding='utf-8').split('\n')
        for n in sorted(numeros, reverse=True):
            if re.fullmatch(r'\s*use \w+ as \w+;', linhas[n - 1]):
                del linhas[n - 1]
        a.write_text('\n'.join(linhas), encoding='utf-8')


def corrigir_metodos(erros):
    """No tipo único, `mensagem` é campo e `subcodigo` é a folha."""
    mexeu = False
    for m in erros:
        if 'ErroHtml' not in m['message']:
            continue
        for sp in m['spans']:
            if not sp['is_primary']:
                continue
            a = RAIZ / sp['file_name']
            linhas = a.read_text(encoding='utf-8').split('\n')
            s = linhas[sp['line_start'] - 1]
            if 'mensagem' in m['message'] and '.mensagem()' in s:
                linhas[sp['line_start'] - 1] = s.replace('.mensagem()', '.mensagem')
            elif 'subcodigo' in m['message'] and '.subcodigo()' in s:
                linhas[sp['line_start'] - 1] = re.sub(r'([\w.]+)\.subcodigo\(\)', r'\1.onde', s)
            else:
                continue
            a.write_text('\n'.join(linhas), encoding='utf-8')
            mexeu = True
    return mexeu


def culpadas(plano, erros, deixadas):
    """(enum, variante) que cada erro aponta; None se algum erro não se explica."""
    saida = set()
    linhas = {}
    atuais = {}
    for a, pos, chave_ in deixadas:
        f = atuais.setdefault(a, a.read_bytes())
        linhas.setdefault((str(a.relative_to(RAIZ)), linha_col(f, pos)), set()).add(chave_)
    soltos = []
    for m in erros:
        achou = set()
        for sp in m['spans']:
            achou |= linhas.get((sp['file_name'], sp['line_start']), set())
        r = re.search(r'no variant[\w ,]*? named `(\w+)` found for enum `(\w+)`', m['message'])
        if r and not achou:
            achou = {(r.group(2), r.group(1))} & {(n, v) for n, vs in plano.migra.items() for v in vs}
        if not achou:
            # `use E::V as A`, ou erro sem o nome do enum: as variantes migradas citadas na linha.
            for sp in m['spans']:
                if not sp['is_primary']:
                    continue
                for t in (sp.get('text') or []):
                    for q, v in re.findall(r'(\w+)::(\w+)', t['text']):
                        donos = [n for n, vs in plano.migra.items() if v in vs]
                        achou |= {(n, v) for n in donos if n == q or len(donos) == 1}
                        if not any(n == q for n in donos):
                            achou |= {(n, v) for n in donos if n.endswith(q) or q in ('E', 'Self')}
            r = re.search(r'no `(\w+)` in `[\w:]*?(\w+)`', m['rendered'])
            if r:
                achou |= {(r.group(2), r.group(1))} & {(n, v) for n, vs in plano.migra.items() for v in vs}
        r = re.search(r'no (?:associated item|variant) named `(\w+)` found for struct `ErroHtml`', m['message'])
        if r and not achou:
            achou = {(n, r.group(1)) for n in plano.some if r.group(1) in plano.migra[n]}
        if not achou:
            soltos.append(m)
        saida |= achou
    return saida, soltos


def main():
    aplicar = '--aplicar' in sys.argv
    excluidos = set()
    if Path(RAIZ / 'scripts/.erro_html_excluidos.json').exists():
        excluidos = {tuple(x) for x in json.loads((RAIZ / 'scripts/.erro_html_excluidos.json').read_text())}
    print('lendo os tipos das origens pelo compilador...', flush=True)
    origens = tipos_das_origens()
    plano = Plano(ler_fontes(), excluidos, origens)
    print(f'{plano.encontradas} variantes de forma; {sum(map(len, plano.migra.values()))} candidatas; '
          f'{len(plano.some)} enum somem', flush=True)
    if not aplicar:
        for n in sorted(plano.migra):
            print(f'  {n}: {", ".join(sorted(plano.migra[n]))}{" (some)" if n in plano.some else ""}')
        return
    # Fase A: só construções fora de padrão, macro e `==`. Erro em `cargo build` é decisão.
    for rodada in range(40):
        plano = Plano(ler_fontes(), excluidos, origens)
        deixadas, _ = transformar(plano, fase_b=False)
        for _ in range(6):
            erros = cargo('build')
            if not (corrigir_interrogacao(plano, erros) or corrigir_metodos(erros)):
                break
        if not erros:
            print(f'fase A limpa na rodada {rodada}', flush=True)
            break
        novas, soltos = culpadas(plano, erros, deixadas)
        print(f'rodada {rodada}: {len(erros)} erros; exclui {sorted(novas)}', flush=True)
        for m in soltos:
            print('  sem dono:', m['rendered'][:600])
        subprocess.run(['git', 'checkout', '-q', '--', 'src'], cwd=RAIZ, check=True)
        if not novas:
            raise SystemExit('erro sem variante: corrigir o codemod')
        excluidos |= novas
        (RAIZ / 'scripts/.erro_html_excluidos.json').write_text(json.dumps(sorted(excluidos), indent=1))
    subprocess.run(['git', 'checkout', '-q', '--', 'src'], cwd=RAIZ, check=True)
    # Fase B: tudo, inclusive testes.
    plano = Plano(ler_fontes(), excluidos, origens)
    _, contadas = transformar(plano, fase_b=True)
    for _ in range(6):
        erros = cargo('build') + cargo('test', '--no-run')
        if not (corrigir_interrogacao(plano, erros) or corrigir_metodos(erros)):
            break
    AVISOS.clear()
    cargo('build')
    cargo('test', '--no-run')
    remover_usos_mortos()
    migradas = sum(map(len, plano.migra.values()))
    print(f'migradas {migradas} variantes; {len(plano.some)} enum somem ({sorted(plano.some)}); '
          f'{contadas} map_err com ErroAtributoHtml; {len(excluidos)} excluídas')
    for n, v in sorted(excluidos):
        print(f'  excluída: {n}::{v}')
    for m in erros:
        print('  resta:', m['rendered'][:800])


if __name__ == '__main__':
    main()
