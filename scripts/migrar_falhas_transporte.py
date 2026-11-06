"""Migração delimitada: erros de fronteira do transporte deixam de ser ErroHtml.

Uso:
    uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2 \
        python scripts/migrar_falhas_transporte.py --aplicar

Tree-sitter identifica as chamadas Rust; expressões não reconhecidas ficam
intactas. Com --check, a ferramenta falha se ainda houver mudanças previstas.
Não altera erros de DOM nem outras famílias de falhas.
"""
import argparse
import re
from pathlib import Path

from tree_sitter import Language, Parser
import tree_sitter_rust

ROOT = Path(__file__).resolve().parents[1]
PARSER = Parser(Language(tree_sitter_rust.language()))
ARQUIVOS = (
    "src/transporte.rs",
    "src/transporte/cliente_https.rs",
    "src/transporte/tests.rs",
    "src/transporte/tests_resposta.rs",
    "src/transporte/web_mock_tests.rs",
    "src/transporte/sessao.rs",
    "src/formulario_web.rs",
    "src/formulario_web/ajax.rs",
    "src/formulario_web/login.rs",
    "src/formulario_web/leitura.rs",
    "src/formulario_web/guardada.rs",
    "src/formulario_web/tests.rs",
    "src/sessao_login/fluxo.rs",
    "src/relogin_cli/tests.rs",
    "src/unidade_categoria/pagina_web.rs",
)
NOMES = {
    "url_invalida": "UrlInvalida",
    "cookie_invalido": "CookieInvalido",
    "tipo_de_conteudo_nao_html": "TipoDeConteudoNaoHtml",
}
WRAPPER = re.compile(
    r'FalhaTransporte::Html\s*\(\s*'
    r'(?:crate::erro_html::)?ErroHtml::(?:valor|html)\s*\(\s*'
    r'"(url_invalida|cookie_invalido|tipo_de_conteudo_nao_html)"\s*\)'
    r'(?:\s*\.com_mensagem\s*\(\s*"[^"]*"\s*\))?\s*,?\s*\)',
    re.S,
)
DIRETO = re.compile(
    r'ErroHtml::(?:valor|html)\s*\(\s*'
    r'"(url_invalida|cookie_invalido|tipo_de_conteudo_nao_html)"\s*\)'
    r'(?:\s*\.com_mensagem\s*\(\s*"[^"]*"\s*\))?',
    re.S,
)
PADRAO = re.compile(
    r'FalhaTransporte::Html\s*\(\s*ErroHtml\s*\{\s*'
    r'onde:\s*"url_invalida"\s*,\s*\.\.\s*\}\s*\)', re.S
)
DIRETOS = {"src/transporte.rs", "src/transporte/cliente_https.rs"}


def nos(raiz):
    pilha = [raiz]
    while pilha:
        atual = pilha.pop()
        yield atual
        pilha.extend(reversed(atual.children))


def aplicar_fonte(caminho: str, fonte: bytes):
    arvore = PARSER.parse(fonte)
    if arvore.root_node.has_error:
        raise ValueError(f"{caminho}: fonte Rust não reconhecida")
    candidatos = []
    for no in nos(arvore.root_node):
        if no.type != "call_expression":
            continue
        texto = fonte[no.start_byte:no.end_byte].decode()
        novo = None
        if casado := WRAPPER.fullmatch(texto):
            novo = "FalhaTransporte::" + NOMES[casado[1]]
        elif casado := PADRAO.fullmatch(texto):
            novo = "FalhaTransporte::UrlInvalida"
        elif caminho in DIRETOS and (casado := DIRETO.fullmatch(texto)):
            novo = "FalhaTransporte::" + NOMES[casado[1]]
        if novo is not None:
            candidatos.append((no.start_byte, no.end_byte, novo.encode()))
    # Chamadas externas prevalecem sobre chamadas aninhadas.
    selecionados = []
    for inicio, fim, novo in sorted(candidatos, key=lambda x: (x[0], -(x[1]-x[0]))):
        if not selecionados or inicio >= selecionados[-1][1]:
            selecionados.append((inicio, fim, novo))
    for inicio, fim, novo in reversed(selecionados):
        fonte = fonte[:inicio] + novo + fonte[fim:]
    if b"ErroHtml" not in fonte.replace(b"use crate::erro_html::ErroHtml;", b""):
        fonte = fonte.replace(b"use crate::erro_html::ErroHtml;\n", b"")
    # .into() aplicado ao mesmo tipo concreto não adiciona informação.
    if caminho in DIRETOS:
        fonte = re.sub(
            rb"(FalhaTransporte::(?:UrlInvalida|CookieInvalido|TipoDeConteudoNaoHtml))\s*\.into\(\)",
            rb"\1", fonte,
        )
    return fonte, len(selecionados)


def corrigir_enum(fonte: bytes):
    """Adiciona variantes do transporte sem criar categorias de erro HTML."""
    alvo = "    OrigemNaoAutorizada,\n".encode()
    insercao = (
        "    OrigemNaoAutorizada,\n"
        "    /// URL assinada inválida: falha da validação do transporte.\n"
        "    UrlInvalida,\n"
        "    /// Cookie local fora do formato permitido.\n"
        "    CookieInvalido,\n"
        "    /// A resposta não declarou o Content-Type HTML esperado.\n"
        "    TipoDeConteudoNaoHtml,\n"
    ).encode()
    if b"    UrlInvalida,\n" not in fonte:
        if fonte.count(alvo) != 1:
            raise ValueError("FalhaTransporte: âncora OrigemNaoAutorizada não única")
        fonte = fonte.replace(alvo, insercao)
    fonte = fonte.replace(b"    #[causa(transparente)]\n    Html(ErroHtml),\n", b"")
    fonte = fonte.replace(b"use crate::erro_html::ErroHtml;\n", b"")
    ancora = (
        '            Self::OrigemNaoAutorizada => "destino HTTPS diferente da origem autorizada da sessão",\n'
    ).encode()
    novas = ancora + (
        '            Self::UrlInvalida => "URL assinada da caixa inválida",\n'
        '            Self::CookieInvalido => "formato de cookie de sessão inválido",\n'
        '            Self::TipoDeConteudoNaoHtml => "a resposta não declara Content-Type text/html",\n'
    ).encode()
    if b"Self::UrlInvalida =>" not in fonte:
        if fonte.count(ancora) != 1:
            raise ValueError("FalhaTransporte: braço da mensagem não encontrado")
        fonte = fonte.replace(ancora, novas)
    fonte = fonte.replace(b"            Self::Html(e) => e.mensagem,\n", b"")
    return fonte

def main():
    cli = argparse.ArgumentParser()
    cli.add_argument("--aplicar", action="store_true")
    cli.add_argument("--check", action="store_true")
    args = cli.parse_args()
    alterados = 0
    for caminho in ARQUIVOS:
        arquivo = ROOT / caminho
        original = arquivo.read_bytes()
        novo, quantidade = aplicar_fonte(caminho, original)
        if novo != original:
            alterados += 1
            print(f"{caminho}: {quantidade} construções substituídas")
            if args.aplicar:
                arquivo.write_bytes(novo)
    arquivo = ROOT / "src/transporte/falha.rs"
    original = arquivo.read_bytes()
    novo = corrigir_enum(original)
    if novo != original:
        alterados += 1
        print("src/transporte/falha.rs: variantes específicas")
        if args.aplicar:
            arquivo.write_bytes(novo)
    if args.check and alterados:
        raise SystemExit("codemod ainda tem alterações pendentes")
    if not args.aplicar and not args.check:
        print(f"prévia: {alterados} arquivos; use --aplicar para escrever")


if __name__ == "__main__":
    main()
