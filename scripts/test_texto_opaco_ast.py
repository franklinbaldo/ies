# Copyright 2026 Franklin Baldo
# ruff: noqa: PT009, PT027  # The canonical gate uses unittest, not pytest.
"""Test the reviewed-scope codemod, including its pinned AST environment."""

from __future__ import annotations

import importlib.util
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

AST_DISPONIVEL = importlib.util.find_spec("tree_sitter") is not None
SCRIPTS = Path(__file__).resolve().parent

if AST_DISPONIVEL:
    from texto_opaco_ast import Escopo, Fonte, Padrao


def executar(argumentos: list[str]) -> subprocess.CompletedProcess[str]:
    """Run argv constructed only by these tests, without a shell."""
    return subprocess.run(  # noqa: S603 -- argv contains only test-owned executables, flags and paths.
        argumentos,
        capture_output=True,
        encoding="utf-8",
        check=True,
    )


class AmbienteAstTest(unittest.TestCase):
    """Run the same tests with pinned dependencies when the gate lacks them."""

    @unittest.skipIf(AST_DISPONIVEL, "o ambiente AST já está ativo")
    def test_executar_com_dependencias_fixadas(self) -> None:
        """Keep the canonical Python gate from silently skipping AST tests."""
        uvx = shutil.which("uvx")
        if uvx is None:
            self.fail("o gate AST exige uvx no PATH")
        executar(
            [
                uvx,
                "--with",
                "tree-sitter==0.24.0",
                "--with",
                "tree-sitter-rust==0.23.2",
                "python",
                "-B",
                "-m",
                "unittest",
                "discover",
                "-s",
                str(SCRIPTS),
                "-p",
                Path(__file__).name,
            ],
        )


@unittest.skipUnless(AST_DISPONIVEL, "executado pelo launcher com dependências fixadas")
class TextoOpacoAstTest(unittest.TestCase):
    """Verify byte edits and refusal of ambiguous transformations."""

    def test_auditar_padroes_sem_declara_los_defeitos(self) -> None:
        """Find structural candidates rather than comments or text searches."""
        fonte = Fonte.de_bytes(
            "src/exemplo.rs",
            b"""
fn validar(texto: &str) {
    if texto.chars().any(char::is_control) {}
    if texto.chars().count() > 100 {}
    if texto.chars().any(|c| u32::from(c) > 0xff) {}
    let _ = texto.trim();
}
""",
        )
        self.assertEqual(
            {item.padrao for item in fonte.auditar()},
            {Padrao.TRIM, Padrao.CONTROLE, Padrao.LIMITE, Padrao.CHARSET},
        )

    def test_comentarios_strings_e_testes_nao_sao_producao(self) -> None:
        """Exclude test bodies, comments and string contents."""
        fonte = Fonte.de_bytes(
            "src/exemplo.rs",
            b"""
// texto.trim()
fn normal() { let _ = "texto.trim()"; }
#[cfg(test)] mod tests { fn teste(texto: &str) { texto.trim(); } }
""",
        )
        self.assertEqual(fonte.auditar(), ())

    def test_reescrever_apenas_funcao_e_parametro_aprovados(self) -> None:
        """Preserve Unicode offsets and unrelated validation."""
        original = """// ação 🚀
fn salvar(texto: &str) {
    guardar(texto.trim());
    let _ = Resultado { texto: texto.trim().to_owned() };
}
fn regra(texto: &str) -> bool { texto.trim().is_empty() }
""".encode()
        fonte = Fonte.de_bytes("src/exemplo.rs", original)
        escopo = Escopo("src/exemplo.rs", "salvar", "texto")
        plano = fonte.reescrever((escopo,))
        esperado = original.replace(b"guardar(texto.trim())", b"guardar(texto)")
        esperado = esperado.replace(
            b"texto: texto.trim().to_owned()", b"texto: texto.to_owned()"
        )
        self.assertEqual(plano.conteudo, esperado)
        self.assertEqual(plano.quantidade, 2)
        repetido = Fonte.de_bytes("src/exemplo.rs", plano.conteudo).reescrever(
            (escopo,)
        )
        self.assertEqual(repetido.quantidade, 0)
        self.assertEqual(repetido.conteudo, esperado)

    def test_sem_escopo_nao_aplica(self) -> None:
        """Require explicit review even for a simple trim call."""
        fonte = Fonte.de_bytes(
            "src/exemplo.rs", b"fn salvar(texto: &str) { texto.trim(); }"
        )
        plano = fonte.reescrever(())
        self.assertEqual(plano.quantidade, 0)
        self.assertEqual(plano.conteudo, fonte.conteudo)

    def test_tipo_customizado_e_sombreamento_impedem_reescrita(self) -> None:
        """Reject methods whose receiver is not the original str parameter."""
        for codigo in (
            b"fn salvar(texto: Texto) { texto.trim(); }",
            b"fn salvar(texto: &str) { let texto = outro(); texto.trim(); }",
            b"fn salvar(texto: &str) { let f = |texto: &str| texto.trim(); }",
        ):
            fonte = Fonte.de_bytes("src/exemplo.rs", codigo)
            plano = fonte.reescrever((Escopo("src/exemplo.rs", "salvar", "texto"),))
            self.assertEqual(plano.quantidade, 0)
            self.assertEqual(plano.conteudo, codigo)

    def test_ast_parcial_nao_e_reescrita(self) -> None:
        """Leave source with parser errors untouched."""
        fonte = Fonte.de_bytes(
            "src/exemplo.rs", b"fn salvar(texto: &str) { texto.trim( ; }"
        )
        self.assertTrue(fonte.ast_parcial)
        plano = fonte.reescrever((Escopo("src/exemplo.rs", "salvar", "texto"),))
        self.assertEqual(plano.quantidade, 0)

    def test_trim_encadeado_e_idempotente(self) -> None:
        """Collapse only pure trim chains without leaving a second edit."""
        original = b"fn salvar(texto: &str) { guardar(texto.trim().trim()); }"
        fonte = Fonte.de_bytes("src/exemplo.rs", original)
        escopo = (Escopo("src/exemplo.rs", "salvar", "texto"),)
        plano = fonte.reescrever(escopo)
        self.assertEqual(plano.quantidade, 1)
        self.assertEqual(plano.conteudo, b"fn salvar(texto: &str) { guardar(texto); }")
        self.assertEqual(
            Fonte.de_bytes("src/exemplo.rs", plano.conteudo)
            .reescrever(escopo)
            .quantidade,
            0,
        )

    def test_funcoes_homonimas_nao_sao_reescritas(self) -> None:
        """Require a unique function instead of guessing between modules."""
        original = b"""
mod a { fn salvar(texto: &str) { texto.trim(); } }
mod b { fn salvar(texto: &str) { texto.trim(); } }
"""
        fonte = Fonte.de_bytes("src/exemplo.rs", original)
        plano = fonte.reescrever((Escopo("src/exemplo.rs", "salvar", "texto"),))
        self.assertEqual(plano.quantidade, 0)

    def test_concorrencia_e_caminho_fora_da_raiz_impedem_gravacao(self) -> None:
        """Refuse stale plans and paths outside the selected source root."""
        with tempfile.TemporaryDirectory() as pasta:
            raiz = Path(pasta)
            caminho = raiz / "exemplo.rs"
            original = b"fn salvar(texto: &str) { texto.trim(); }"
            fonte = Fonte.de_bytes("exemplo.rs", original)
            plano = fonte.reescrever((Escopo("exemplo.rs", "salvar", "texto"),))
            caminho.write_bytes(b"// alterado por outra sessao")
            with self.assertRaises(RuntimeError):
                fonte.aplicar_em_arquivo(raiz, plano)
            self.assertEqual(caminho.read_bytes(), b"// alterado por outra sessao")
            fora = Fonte.de_bytes("../fora.rs", original)
            with self.assertRaises(RuntimeError):
                fora.aplicar_em_arquivo(raiz, plano)

    def test_cli_padrao_so_audita_e_apply_e_idempotente(self) -> None:
        """Exercise real filesystem writes only after explicit selection."""
        with tempfile.TemporaryDirectory() as pasta:
            raiz = Path(pasta)
            caminho = raiz / "src" / "exemplo.rs"
            caminho.parent.mkdir()
            original = b"fn salvar(texto: &str) { guardar(texto.trim()); }"
            caminho.write_bytes(original)
            comando = [
                sys.executable,
                "-B",
                str(SCRIPTS / "texto_opaco_ast.py"),
                "--root",
                pasta,
            ]
            executar(comando)
            self.assertEqual(caminho.read_bytes(), original)
            escopo = ["--escopo", "src/exemplo.rs::salvar::texto"]
            executar([*comando, "--apply", *escopo])
            self.assertEqual(
                caminho.read_bytes(), original.replace(b"texto.trim()", b"texto")
            )
            executar([*comando, "--check", *escopo])


if __name__ == "__main__":
    unittest.main()
