"""Casos sintéticos do inventário; nenhuma rede ou dado de sessão."""
import json
from pathlib import Path
import tempfile
import unittest
from subcodigos import extrair_rust, extrair_documentacao, carregar_allowlist, verificar


class ExtracaoTests(unittest.TestCase):
    def test_delimitadores_comentarios_strings_e_chars(self):
        fonte = r'''// fn subcodigo() { "falso" }
        const FORA: &str = "fora";
        fn mensagem() { "mensagem"; }
        fn subcodigo(&self) -> &'static str {
            /* "comentario" /* } */ */
            let c = '}'; let escape = '\'';
            match self { A => "real", B => { r##"raw_codigo"## },
                C => "com\u{5f}escape", D => outro.subcodigo() }
        }
        fn subcodigo_das_pastas(self) -> &'static str { "pastas" }
        fn subcodigos_estaveis() { "teste" }
        fn subcodigo(&self) -> &'static str;
        fn depois() { "fora_tambem" }
        '''
        self.assertEqual(extrair_rust(fonte), {'real', 'raw_codigo', 'com_escape', 'pastas'})

    def test_json_literal_e_delegacao(self):
        self.assertEqual(extrair_rust('''fn subcodigo(&self) -> &str {
            self.erro.subcodigo() }
            fn saida() { json!({"subcodigo": "direto"}); }
            #[cfg(test)] mod tests { fn exemplo() {
                json!({"subcodigo": "somente_teste"}); } }
        '''), {'direto'})

    def test_onde_de_erro_html_e_subcodigo(self):
        self.assertEqual(extrair_rust('''fn ler() {
            E::Html(crate::erro_html::ErroHtml::ausente("tabela_ausente", "msg"));
            ErroHtml::atributo(
                "tag_ilegivel", "msg", e);
            ErroHtml::novo(onde, p, "não é código");
            #[cfg(test)] mod tests { fn t() { ErroHtml::valor("so_teste", "m"); } }
        }'''), {'tabela_ausente', 'tag_ilegivel'})

    def test_rust_incompleto_falha(self):
        for fonte in ['fn subcodigo() { "x"', '/* aberto', '"aberta', 'r#"aberta',
                      'fn subcodigo() { (] }']:
            with self.subTest(fonte=fonte), self.assertRaises(ValueError):
                extrair_rust(fonte)

    def test_markdown_so_coluna_de_subcodigo_ou_secao_tipadas(self):
        md = '''# Exemplo
`fora` e `aposentado`
| Fato | Subcódigo |
| --- | --- |
| `nao_codigo` | `ativo` / `outro` |
## Falhas tipadas
| Subcódigo | Fato |
| --- | --- |
| `terceiro` | `dado` |
## Histórico
`retirado`
'''
        self.assertEqual(extrair_documentacao(md), {'ativo', 'outro', 'terceiro'})

    def test_documento_guia_exige_tabela_tambem(self):
        self.assertEqual(extrair_documentacao('`exemplo`\n| Subcódigo | Fato |\n|---|---|\n| `certo` | x |'), {'certo'})

    def test_blocos_de_exemplo_nao_documentam(self):
        md = "```markdown\n| Subcódigo | Fato |\n|---|---|\n| `falso` | x |\n```"
        self.assertEqual(extrair_documentacao(md), set())

    def test_strings_fora_do_corpo_nao_emitem(self):
        fonte = 'const S: &str = r#"fn subcodigo() { fake }"#; fn subcodigo() -> &str { "real" }'
        self.assertEqual(extrair_rust(fonte), {'real'})


class GateTests(unittest.TestCase):
    def test_allowlist_justificada_sem_duplicatas(self):
        self.assertEqual(carregar_allowlist('[]'), {})
        self.assertEqual(carregar_allowlist('[{"subcodigo":"x", "justificativa":"motivo"}]'), {'x': 'motivo'})
        for dado in [{}, ['x'], [{'subcodigo': 'x', 'justificativa': ' '}],
                     [{'subcodigo': 'x', 'justificativa': 'a'}] * 2]:
            with self.subTest(dado=dado), self.assertRaises(ValueError):
                carregar_allowlist(json.dumps(dado))

    def test_estado_atual_ausente_falha_historico_so_avisa(self):
        with tempfile.TemporaryDirectory() as tmp:
            raiz = Path(tmp)
            (raiz / 'src').mkdir()
            (raiz / 'docs').mkdir()
            (raiz / 'contracts/cli').mkdir(parents=True)
            (raiz / 'scripts').mkdir()
            rust = raiz / 'src/erro.rs'
            rust.write_text('fn subcodigo(self) -> &str { "novo" }')
            (raiz / 'docs/erros-tipados.md').write_text('| Subcódigo | Fato |\n|---|---|\n| `velho` | x |')
            allow = raiz / 'scripts/subcodigos_permitidos.json'
            allow.write_text('[]')
            saida = []
            self.assertEqual(verificar(raiz, saida.append), 1)
            self.assertTrue(any('novo' in s and 'ERRO' in s for s in saida))
            self.assertTrue(any('velho' in s and 'AVISO' in s for s in saida))
            rust.write_text('fn subcodigo(self) -> &str { "velho" }')
            self.assertEqual(verificar(raiz, lambda _: None), 0)
            rust.write_text('fn subcodigo(self) -> &str { "novo" }')
            allow.write_text('[{"subcodigo":"novo","justificativa":"teste"}]')
            self.assertEqual(verificar(raiz, lambda _: None), 0)


if __name__ == '__main__':
    unittest.main()
