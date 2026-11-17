"""Tira as variantes de enum que o rustc aponta como nunca construídas
(lint `dead_code`, só no binário, sem os testes): a variante inteira, com
os campos, os atributos e a documentação logo acima. Depois, a alternativa
`| Tipo::Variante…` que a nomeava num padrão `|` de `match` (o erro E0599
na linha que só tem essa alternativa). Uma variante que só um teste
constrói também sai; o teste acusa no compilador e é revisto à mão.
Repete até não sobrar nenhuma (sair uma variante pode deixar morta outra).

Antes, `#[from] ErroAtributoHtml` vira `#[source]`: o `From` gerado conta
como construção e esconderia a variante morta. Onde o `?` ainda converte um
`ErroAtributoHtml` (E0277 no arquivo), o `#[from]` volta.

uso: python3 -I scripts/codemod/html5ever/variantes_mortas.py  (na raiz)
"""

import json
import re
import subprocess
import sys

# Uma linha que é só uma alternativa de padrão: `| Self::X`, `| E::X(_)`,
# `| E::X { .. }`; a primeira alternativa vem sem a barra. BRACO: a última
# alternativa, com o `=>` do braço.
ALTERNATIVA = re.compile(rb"^\s*\|?\s*[A-Za-z_:]+::[A-Za-z_]+(\(_\)|\(\.\.\)|\s*\{\s*\.\.\s*\})?\s*$")
BRACO = re.compile(rb"^\s*\|\s*[A-Za-z_:]+::[A-Za-z_]+(?:\(_\)|\(\.\.\)|\s*\{\s*\.\.\s*\})?\s*=>(.*)$")


def mortas():
    saida = subprocess.run(
        ["cargo", "check", "--bin", "ies", "--message-format=json", "-q"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    achados = set()
    for linha in saida.splitlines():
        try:
            item = json.loads(linha)
        except json.JSONDecodeError:
            continue
        mensagem = item.get("message") or {}
        if (mensagem.get("code") or {}).get("code") != "dead_code":
            continue
        texto = mensagem.get("message", "")
        if not texto.startswith("variant") or "never constructed" not in texto:
            continue
        for trecho in mensagem.get("spans", []):
            if trecho.get("is_primary"):
                achados.add((trecho["file_name"], trecho["byte_start"]))
    return achados


FROM = re.compile(rb"#\[from\](\s+(?:crate::html_atributos::)?ErroAtributoHtml\b)")


def from_ainda_usado():
    """Enums para os quais um `?` ainda converte `ErroAtributoHtml` (E0277)."""
    saida = subprocess.run(
        ["cargo", "check", "--bin", "ies", "--tests", "--message-format=json", "-q"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    enums = set()
    for linha in saida.splitlines():
        try:
            item = json.loads(linha)
        except json.JSONDecodeError:
            continue
        mensagem = item.get("message") or {}
        texto = mensagem.get("message", "")
        if (mensagem.get("code") or {}).get("code") != "E0277":
            continue
        if "ErroAtributoHtml" not in (mensagem.get("rendered") or ""):
            continue
        achado = re.search(r"convert the error to `(?:[\w:]*::)?(\w+)`", texto)
        if achado:
            enums.add(achado.group(1))
    return enums


def from_sem_uso():
    """`#[from] ErroAtributoHtml` vira `#[source]` onde nenhum `?` converte."""
    trocados = {}
    for arquivo in subprocess.run(
        ["git", "grep", "-l", "-E", r"#\[from\] (crate::html_atributos::)?ErroAtributoHtml", "--", "src"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout.split():
        with open(arquivo, "rb") as f:
            fonte = f.read()
        trocados[arquivo] = fonte
        with open(arquivo, "wb") as f:
            f.write(FROM.sub(rb"#[source]\1", fonte))
    for _ in range(len(trocados) + 1):
        enums = from_ainda_usado()
        if not enums:
            return
        # O `#[from]` volta no arquivo que declara o enum do `?`.
        for arquivo, fonte in list(trocados.items()):
            if any(re.search(rb"enum " + nome.encode() + rb"\b", fonte) for nome in enums):
                with open(arquivo, "wb") as f:
                    f.write(fonte)
                del trocados[arquivo]
    print("variantes_mortas: #[from] não converge", file=sys.stderr)


def fim_da_variante(fonte, inicio):
    """Byte logo depois da vírgula que fecha a variante (profundidade 0)."""
    profundidade = 0
    i = inicio
    while i < len(fonte):
        c = fonte[i : i + 1]
        if c in (b"(", b"{", b"["):
            profundidade += 1
        elif c in (b")", b"}", b"]"):
            if profundidade == 0:
                return i
            profundidade -= 1
        elif c == b"," and profundidade == 0:
            return i + 1
        i += 1
    return i


def inicio_com_documentacao(fonte, inicio):
    """Início da linha da variante, recuado sobre `///` e `#[...]` acima."""
    k = fonte.rfind(b"\n", 0, inicio) + 1
    while k > 0:
        a = fonte.rfind(b"\n", 0, k - 1) + 1
        linha = fonte[a:k].strip()
        if linha.startswith(b"///") or linha.startswith(b"#["):
            k = a
        else:
            break
    return k


def alternativas_orfas():
    """Linhas de padrão `|` que nomeiam variante que não existe mais."""
    saida = subprocess.run(
        ["cargo", "check", "--bin", "ies", "--tests", "--message-format=json", "-q"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    achados = set()
    for linha in saida.splitlines():
        try:
            item = json.loads(linha)
        except json.JSONDecodeError:
            continue
        mensagem = item.get("message") or {}
        if (mensagem.get("code") or {}).get("code") != "E0599":
            continue
        if not mensagem.get("message", "").startswith("no variant"):
            continue
        for trecho in mensagem.get("spans", []):
            if trecho.get("is_primary"):
                achados.add((trecho["file_name"], trecho["line_start"]))
    return achados


def sem_alternativas_orfas():
    por_arquivo = {}
    for arquivo, linha in alternativas_orfas():
        por_arquivo.setdefault(arquivo, []).append(linha)
    for arquivo, linhas in por_arquivo.items():
        with open(arquivo, "rb") as f:
            fonte = f.read().split(b"\n")
        for numero in sorted(linhas, reverse=True):
            atual = fonte[numero - 1]
            seguinte = fonte[numero] if numero < len(fonte) else b""
            com_barra = atual.lstrip().startswith(b"|")
            braco = BRACO.match(atual)
            if com_barra and braco:
                # `| X => corpo`: o corpo passa à alternativa de cima.
                del fonte[numero - 1]
                fonte[numero - 2] = fonte[numero - 2].rstrip() + b" =>" + braco.group(1)
            elif ALTERNATIVA.match(atual) and (com_barra or seguinte.lstrip().startswith(b"|")):
                del fonte[numero - 1]
                if not com_barra:
                    # A primeira alternativa saiu: a seguinte perde a barra.
                    fonte[numero - 1] = re.sub(rb"^(\s*)\|\s*", rb"\1", fonte[numero - 1], count=1)
        with open(arquivo, "wb") as f:
            f.write(b"\n".join(fonte))


def main():
    from_sem_uso()
    for _ in range(5):
        achados = mortas()
        if not achados:
            sem_alternativas_orfas()
            return 0
        por_arquivo = {}
        for arquivo, inicio in achados:
            por_arquivo.setdefault(arquivo, []).append(inicio)
        for arquivo, lista in por_arquivo.items():
            with open(arquivo, "rb") as f:
                fonte = f.read()
            for inicio in sorted(lista, reverse=True):
                fim = fim_da_variante(fonte, inicio)
                if fonte[fim : fim + 1] == b"\n":
                    fim += 1
                comeco = inicio_com_documentacao(fonte, inicio)
                fonte = fonte[:comeco] + fonte[fim:]
            with open(arquivo, "wb") as f:
                f.write(fonte)
    print("variantes_mortas: não converge", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
