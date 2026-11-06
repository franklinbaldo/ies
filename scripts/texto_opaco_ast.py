# Copyright 2026 Franklin Baldo
"""Audit Rust text restrictions and remove trim only in reviewed scopes.

Use the pinned Tree-sitter dependencies from dry_ast.py. Findings are review
candidates, not proof of defects. Macros are not expanded. Files with partial
ASTs are reported and never rewritten. No guard is removed automatically.

Run with uvx --with tree-sitter==0.24.0 --with tree-sitter-rust==0.23.2
python scripts/texto_opaco_ast.py --audit. After reviewing one function, use
--apply --escopo src/arquivo.rs::funcao::parametro. Repeat that scope with
--check to verify idempotence. The default mode never writes files.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from enum import StrEnum
from pathlib import Path
from typing import TYPE_CHECKING, Self

from dry_ast import PARSER, ROOT, code, in_test, nodes

if TYPE_CHECKING:
    from tree_sitter import Node, Tree


class Padrao(StrEnum):
    """Identify structural candidates without classifying business rules."""

    TRIM = "normalizacao_trim"
    CONTROLE = "recusa_controle"
    LIMITE = "limite_literal"
    CHARSET = "charset_fixo"


@dataclass(frozen=True)
class Ocorrencia:
    """Describe one source expression for manual review."""

    arquivo: str
    funcao: str
    linha: int
    padrao: Padrao
    expressao: str


@dataclass(frozen=True)
class Escopo:
    """Authorize one parameter in one explicitly reviewed function."""

    arquivo: str
    funcao: str
    parametro: str

    def aprova(self, arquivo: str, funcao: str, parametro: str) -> bool:
        """Match the full reviewed scope rather than an identifier globally."""
        return (self.arquivo, self.funcao, self.parametro) == (
            arquivo,
            funcao,
            parametro,
        )


@dataclass(frozen=True)
class Chamada:
    """Expose parsed method-call fields without reparsing Rust tokens."""

    nome: bytes
    receptor: Node
    argumentos: Node


@dataclass(frozen=True)
class Edicao:
    """Replace a byte range selected from a complete AST."""

    inicio: int
    fim: int
    conteudo: bytes


@dataclass(frozen=True)
class Reescrita:
    """Return proposed bytes and the number of approved expressions."""

    conteudo: bytes
    quantidade: int


def funcao_de(no: Node) -> Node | None:
    """Find the nearest function, including calls inside its closures."""
    atual = no.parent
    while atual is not None:
        if atual.type == "function_item":
            return atual
        atual = atual.parent
    return None


def sombreia(funcao: Node, parametro: bytes) -> bool:
    """Reject local bindings that can hide the reviewed parameter."""
    for no in nodes(funcao):
        if no.type in {"let_declaration", "let_condition", "for_expression"}:
            padrao = no.child_by_field_name("pattern")
        elif no.type == "closure_expression":
            padrao = no.child_by_field_name("parameters")
        else:
            continue
        if padrao is not None and any(
            item.type == "identifier" and item.text == parametro
            for item in nodes(padrao)
        ):
            return True
    return False


@dataclass(frozen=True)
class Fonte:
    """Own Rust source bytes and the existing parser's AST."""

    arquivo: str
    conteudo: bytes
    arvore: Tree

    @classmethod
    def de_bytes(cls, arquivo: str, conteudo: bytes) -> Self:
        """Parse bytes once, preserving offsets and line endings."""
        return cls(arquivo, conteudo, PARSER.parse(conteudo))

    @property
    def ast_parcial(self) -> bool:
        """Report unsupported or malformed syntax without guessing repairs."""
        return self.arvore.root_node.has_error

    def trecho(self, no: Node | None) -> bytes:
        """Read one AST byte range."""
        return b"" if no is None else code(no, self.conteudo)

    def chamada(self, no: Node | None) -> Chamada | None:
        """Read a method call's receiver, name and arguments."""
        if no is None or no.type != "call_expression":
            return None
        funcao = no.child_by_field_name("function")
        argumentos = no.child_by_field_name("arguments")
        if funcao is None or funcao.type != "field_expression" or argumentos is None:
            return None
        receptor = funcao.child_by_field_name("value")
        if receptor is None:
            return None
        return Chamada(
            self.trecho(funcao.child_by_field_name("field")), receptor, argumentos
        )

    def padrao(self, no: Node) -> Padrao | None:
        """Recognize audit candidates without deciding whether they are defects."""
        if no.type == "scoped_identifier" and self.trecho(no) == b"str::trim":
            return Padrao.TRIM
        chamada = self.chamada(no)
        if chamada is not None:
            if chamada.nome == b"trim" and not chamada.argumentos.named_children:
                return Padrao.TRIM
            if chamada.nome in {b"any", b"all"}:
                argumentos = tuple(nodes(chamada.argumentos))
                if any(self.trecho(item) == b"char::is_control" for item in argumentos):
                    return Padrao.CONTROLE
                if any(
                    self.trecho(item) == b"u32::from" for item in argumentos
                ) and any(
                    item.type == "integer_literal"
                    and self.trecho(item).lower() in {b"255", b"0xff"}
                    for item in argumentos
                ):
                    return Padrao.CHARSET
        return self.limite(no)

    def limite(self, no: Node) -> Padrao | None:
        """Find literal comparisons against len or character/byte counts."""
        if no.type != "binary_expression":
            return None
        esquerda = self.chamada(no.child_by_field_name("left"))
        direita = no.child_by_field_name("right")
        operador = self.trecho(no.child_by_field_name("operator"))
        if (
            direita is None
            or direita.type != "integer_literal"
            or operador not in {b">", b">=", b"<", b"<="}
        ):
            return None
        if esquerda is not None and esquerda.nome in {b"len", b"count"}:
            return Padrao.LIMITE
        return None

    def auditar(self) -> tuple[Ocorrencia, ...]:
        """List production candidates and omit partial ASTs and test code."""
        if self.ast_parcial:
            return ()
        achados: list[Ocorrencia] = []
        for no in nodes(self.arvore.root_node):
            if in_test(Path(self.arquivo), no, self.conteudo):
                continue
            padrao = self.padrao(no)
            if padrao is not None:
                funcao = funcao_de(no)
                nome = self.trecho(
                    None if funcao is None else funcao.child_by_field_name("name")
                )
                achados.append(
                    Ocorrencia(
                        self.arquivo,
                        nome.decode(),
                        no.start_point.row + 1,
                        padrao,
                        self.trecho(no).decode(),
                    )
                )
        return tuple(achados)

    def parametro_str(self, funcao: Node, nome: bytes) -> bool:
        """Require the receiver to be a direct str reference parameter."""
        parametros = funcao.child_by_field_name("parameters")
        if parametros is None:
            return False
        for parametro in parametros.named_children:
            if self.trecho(parametro.child_by_field_name("pattern")) != nome:
                continue
            tipo = parametro.child_by_field_name("type")
            return (
                tipo is not None
                and tipo.type == "reference_type"
                and self.trecho(tipo.child_by_field_name("type")) == b"str"
                and not any(item.type == "mutable_specifier" for item in tipo.children)
            )
        return False

    def base_trim(self, no: Node) -> Node | None:
        """Unwrap only pure zero-argument trim chains on an identifier."""
        chamada = self.chamada(no)
        if (
            chamada is None
            or chamada.nome != b"trim"
            or chamada.argumentos.named_children
        ):
            return None
        receptor = chamada.receptor
        if receptor.type == "identifier":
            return receptor
        return self.base_trim(receptor)

    def edicao_aprovada(self, no: Node, escopos: tuple[Escopo, ...]) -> Edicao | None:
        """Select only reviewed str parameters with unambiguous bindings."""
        if in_test(Path(self.arquivo), no, self.conteudo):
            return None
        base = self.base_trim(no)
        funcao = funcao_de(no)
        if base is None or funcao is None:
            return None
        parametro = self.trecho(base)
        nome = self.trecho(funcao.child_by_field_name("name"))
        if not any(
            escopo.aprova(self.arquivo, nome.decode(), parametro.decode())
            for escopo in escopos
        ):
            return None
        homonimas = [
            item
            for item in nodes(self.arvore.root_node)
            if item.type == "function_item"
            and self.trecho(item.child_by_field_name("name")) == nome
        ]
        if (
            len(homonimas) != 1
            or not self.parametro_str(funcao, parametro)
            or sombreia(funcao, parametro)
        ):
            return None
        return Edicao(no.start_byte, no.end_byte, parametro)

    def reescrever(self, escopos: tuple[Escopo, ...]) -> Reescrita:
        """Propose scoped byte edits without mutating files or guard conditions."""
        if self.ast_parcial or not escopos:
            return Reescrita(self.conteudo, 0)
        candidatas = [
            edicao
            for no in nodes(self.arvore.root_node)
            if (edicao := self.edicao_aprovada(no, escopos)) is not None
        ]
        selecionadas: list[Edicao] = []
        for edicao in sorted(candidatas, key=lambda item: (item.inicio, -item.fim)):
            if not selecionadas or edicao.inicio >= selecionadas[-1].fim:
                selecionadas.append(edicao)
        novo = self.conteudo
        for edicao in reversed(selecionadas):
            novo = novo[: edicao.inicio] + edicao.conteudo + novo[edicao.fim :]
        if PARSER.parse(novo).root_node.has_error:
            mensagem = "o codemod produziu um AST inválido"
            raise RuntimeError(mensagem)
        return Reescrita(novo, len(selecionadas))

    def aplicar_em_arquivo(self, raiz: Path, plano: Reescrita) -> None:
        """Write only unchanged, reviewed source within the selected root."""
        caminho = (raiz / self.arquivo).resolve()
        if (
            not caminho.is_relative_to(raiz.resolve())
            or caminho.read_bytes() != self.conteudo
        ):
            mensagem = (
                f"fonte fora da raiz ou alterada durante a análise: {self.arquivo}"
            )
            raise RuntimeError(mensagem)
        caminho.write_bytes(plano.conteudo)


@dataclass(frozen=True)
class Alteracao:
    """Summarize proposed or applied edits for one file."""

    arquivo: str
    quantidade: int


@dataclass(frozen=True)
class Relatorio:
    """Serialize audit results at the CLI boundary."""

    arquivos: int
    arquivos_ast_parcial: tuple[str, ...]
    ocorrencias: tuple[Ocorrencia, ...]
    alteracoes: tuple[Alteracao, ...]


def main() -> int:
    """Audit by default and require scopes for apply or idempotence checks."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    modo = parser.add_mutually_exclusive_group()
    modo.add_argument("--audit", action="store_true")
    modo.add_argument("--apply", action="store_true")
    modo.add_argument("--check", action="store_true")
    parser.add_argument(
        "--escopo", action="append", default=[], metavar="ARQUIVO::FUNCAO::PARAMETRO"
    )
    args = parser.parse_args()
    escopos: list[Escopo] = []
    for valor in args.escopo:
        partes = valor.rsplit("::", maxsplit=2)
        if len(partes) != len(Escopo.__dataclass_fields__) or not all(partes):
            parser.error("escopo deve ser ARQUIVO::FUNCAO::PARAMETRO")
        escopos.append(Escopo(partes[0].replace("\\", "/"), partes[1], partes[2]))
    if (args.apply or args.check) and not escopos:
        parser.error("apply e check exigem um escopo revisado")
    raiz = args.root.resolve()
    fontes = [
        Fonte.de_bytes(caminho.relative_to(raiz).as_posix(), caminho.read_bytes())
        for caminho in sorted((raiz / "src").rglob("*.rs"))
        if caminho.resolve().is_relative_to(raiz)
    ]
    alteracoes: list[Alteracao] = []
    for fonte in fontes:
        plano = fonte.reescrever(tuple(escopos))
        if plano.quantidade:
            alteracoes.append(Alteracao(fonte.arquivo, plano.quantidade))
            if args.apply:
                fonte.aplicar_em_arquivo(raiz, plano)
    relatorio = Relatorio(
        len(fontes),
        tuple(fonte.arquivo for fonte in fontes if fonte.ast_parcial),
        tuple(item for fonte in fontes for item in fonte.auditar()),
        tuple(alteracoes),
    )
    sys.stdout.buffer.write(
        json.dumps(asdict(relatorio), ensure_ascii=False).encode() + b"\n"
    )
    return int(bool(args.check and alteracoes))


if __name__ == "__main__":
    raise SystemExit(main())
