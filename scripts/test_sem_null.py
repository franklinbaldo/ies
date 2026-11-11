import tempfile
import unittest
from pathlib import Path

from sem_null import producao, violacoes


class SemNull(unittest.TestCase):
    def raiz(self, arquivos: dict[str, str]) -> Path:
        pasta = Path(tempfile.mkdtemp())
        for nome, texto in arquivos.items():
            caminho = pasta / nome
            caminho.parent.mkdir(parents=True, exist_ok=True)
            caminho.write_text(texto)
        return pasta

    def test_recusa_null_emitido_em_producao(self):
        raiz = self.raiz({"a.rs": 'fn f() -> V { json!({"x": null}) }\nfn g() { Value::Null; }\n'})
        self.assertEqual(len(violacoes(raiz)), 2)

    def test_ignora_modulo_de_teste_comentario_e_literal(self):
        texto = 'fn f() { if v == "null" {} } // "x": null\n#[cfg(test)]\nmod tests {\n  fn t() { json!({"x": null}); }\n}\n'
        self.assertEqual(violacoes(self.raiz({"a.rs": texto})), [])

    def test_ignora_arquivos_de_teste_acervo_e_cassette(self):
        raiz = self.raiz({
            "x/tests.rs": "Value::Null",
            "x/tests_fluxo.rs": "Value::Null",
            "x/acervo.rs": "Value::Null",
            "cassette/y.rs": "Value::Null",
        })
        self.assertEqual(violacoes(raiz), [])

    def test_producao_corta_no_modulo_de_teste(self):
        self.assertEqual(producao("a\n#[cfg(test)]\nmod t {}\n"), "a\n")


if __name__ == "__main__":
    unittest.main()
