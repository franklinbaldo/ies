"""Gate dos testes pendentes de fatias WIP (`#[ignore = "wip #N: ..."]`).

Fatias incompletas entram na `main` com os testes novos que ainda não passam
marcados como pendentes. Este gate impede que a marcação esconda regressão:

1. `#[ignore]` sem motivo é proibido.
2. Motivo que começa com `wip` segue `wip #<issue>: <o que falta>`.
3. Teste que já existia sem `#[ignore]` na base não pode passar a ser ignorado,
   com motivo nenhum.

A base é o primeiro pai quando HEAD é commit de merge (o checkout de
`pull_request` e o push de merge na `main`); senão, `merge-base HEAD
origin/main`. Sem base: com `CI` definido o gate falha, sem ele a regra 3
fica `NOT_RUN`. Imprime os pendentes por issue. Só biblioteca padrão.
"""

import os
import subprocess
import sys
from pathlib import Path

PASTAS = ('src', 'tests', 'crates', 'xtask')


def _motivo(linha):
    """Motivo de `#[ignore = "..."]`, `''` para `#[ignore]`, None se não é ignore."""
    texto = linha.strip()
    if not texto.startswith('#[ignore'):
        return None
    resto = texto[len('#[ignore'):].strip()
    if resto.startswith(']'):
        return ''
    if not resto.startswith('='):
        return ''
    resto = resto[1:].strip()
    if not resto.startswith('"'):
        return ''
    fim = resto.find('"', 1)
    return resto[1:fim] if fim > 0 else ''


def _nome_da_fn(linha):
    texto = linha.strip()
    for prefixo in ('pub(crate) ', 'pub ', 'async '):
        if texto.startswith(prefixo):
            texto = texto[len(prefixo):]
    if not texto.startswith('fn '):
        return None
    nome = texto[3:]
    corte = min((i for i in (nome.find('('), nome.find('<')) if i >= 0), default=-1)
    return nome[:corte].strip() if corte > 0 else None


def testes(fonte):
    """{nome: motivo do ignore ou None} para cada fn com `#[test]` ou `#[ignore]`."""
    achados = {}
    teste = False
    motivo = None
    for linha in fonte.splitlines():
        texto = linha.strip()
        m = _motivo(linha)
        if m is not None:
            motivo = m
            continue
        if texto.startswith('#[test') or texto.startswith('#[tokio::test'):
            teste = True
            continue
        if texto.startswith('#[') or texto.startswith('//') or not texto:
            continue
        nome = _nome_da_fn(linha)
        if nome and (teste or motivo is not None):
            achados[nome] = motivo
        teste = False
        motivo = None
    return achados


def _git(raiz, *args):
    r = subprocess.run(['git', *args], cwd=raiz, capture_output=True, text=True, check=False)
    return r.stdout.strip() if r.returncode == 0 else None


def base(raiz):
    if os.environ.get('IES_BASE_REF'):
        return os.environ['IES_BASE_REF']
    pais = (_git(raiz, 'rev-list', '--parents', '-n', '1', 'HEAD') or '').split()
    if len(pais) >= 3 and _git(raiz, 'cat-file', '-e', pais[1] + '^{commit}') is not None:
        return pais[1]
    return _git(raiz, 'merge-base', 'HEAD', 'origin/main')


def verificar(raiz, imprimir=print, ler_base=None):
    erros = []
    pendentes = {}
    ref = base(raiz) if ler_base is None else 'teste'
    if ler_base is None and ref is not None:
        def ler_base(caminho):
            return _git(raiz, 'show', f'{ref}:{caminho}')
    for pasta in PASTAS:
        for arquivo in sorted((raiz / pasta).rglob('*.rs')) if (raiz / pasta).exists() else []:
            caminho = str(arquivo.relative_to(raiz))
            atuais = testes(arquivo.read_text(encoding='utf-8'))
            antigos = None
            for nome, motivo in atuais.items():
                if motivo is None:
                    continue
                if motivo == '':
                    erros.append(f'{caminho}: {nome}: #[ignore] sem motivo')
                    continue
                if motivo.startswith('wip'):
                    cabeca, _, resto = motivo.partition(':')
                    numero = cabeca[len('wip'):].strip()
                    if not (numero.startswith('#') and numero[1:].isdigit() and resto.strip()):
                        erros.append(f'{caminho}: {nome}: use "wip #<issue>: <o que falta>"')
                        continue
                    pendentes.setdefault(numero, []).append(f'{caminho}::{nome}')
                if ler_base is not None:
                    if antigos is None:
                        fonte = ler_base(caminho)
                        antigos = testes(fonte) if fonte else {}
                    if nome in antigos and antigos[nome] is None:
                        erros.append(f'{caminho}: {nome}: teste que já passava na base foi ignorado')
    if ler_base is None:
        if os.environ.get('CI'):
            erros.append('base não encontrada (busque o primeiro pai: checkout com fetch-depth 2)')
        else:
            imprimir('NOT_RUN regra 3: sem base (origin/main ausente)')
    for issue in sorted(pendentes, key=lambda n: int(n[1:])):
        imprimir(f'PENDENTE {issue}: {len(pendentes[issue])} teste(s)')
        for teste in pendentes[issue]:
            imprimir(f'  {teste}')
    for erro in erros:
        imprimir(f'ERRO {erro}')
    imprimir(f'{sum(map(len, pendentes.values()))} pendentes wip; {len(erros)} erros')
    return int(bool(erros))


if __name__ == '__main__':
    sys.exit(verificar(Path(__file__).resolve().parents[1]))
