"""Regressões da auditoria OSV; Cargo.lock e respostas sintéticos, sem rede."""
import json
import unittest
from osv_audit import (RedeIndisponivel, avisos_por_pacote, carregar_allowlist,
                       executar, filtrar, pacotes_do_lock)

LOCK = '''version = 4

[[package]]
name = "ies"
version = "0.1.0"

[[package]]
name = "zip"
version = "4.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00"

[[package]]
name = "base64"
version = "0.23.1"
source = "sparse+https://index.crates.io/"

[[package]]
name = "fork"
version = "1.0.0"
source = "git+https://example.invalid/fork#abc"

[[package]]
name = "zip"
version = "4.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
'''
VAZIA = '{"accepted": []}'


def allowlist(*entradas):
    return json.dumps({'accepted': list(entradas)})


def respostas(*lotes):
    enviados, fila = [], list(lotes)

    def enviar(corpo):
        enviados.append(corpo)
        return {'results': fila.pop(0)}
    return enviar, enviados


class LockTests(unittest.TestCase):
    def test_only_crates_io_unique_sorted(self):
        self.assertEqual(pacotes_do_lock(LOCK),
                         [('base64', '0.23.1'), ('zip', '4.6.1')])


class AllowlistTests(unittest.TestCase):
    def test_entry_requires_written_justification(self):
        base = dict(id='RUSTSEC-1', package='zip', justification='não chama extract()')
        self.assertEqual(carregar_allowlist(allowlist(base)),
                         {('zip', 'RUSTSEC-1'): 'não chama extract()'})
        for campo, valor in [('justification', ''), ('justification', '   '),
                             ('justification', None), ('id', ''), ('package', 7)]:
            with self.subTest(campo=campo, valor=valor), self.assertRaises(ValueError):
                carregar_allowlist(allowlist({**base, campo: valor}))

    def test_malformed_and_duplicate_rejected(self):
        entrada = dict(id='R', package='p', justification='j')
        for texto in ['[]', '{}', '{"accepted": {}}', '{"accepted": ["R"]}',
                      allowlist(entrada, entrada)]:
            with self.subTest(texto=texto), self.assertRaises(ValueError):
                carregar_allowlist(texto)

    def test_filter_by_package_and_id_reports_unused(self):
        aceitos = {('zip', 'R1'): 'j', ('sha1', 'R9'): 'j'}
        achados = {('zip', '4.6.1'): ['G1', 'R1'], ('base64', '0.23.1'): ['R1']}
        abertos, sem_uso = filtrar(achados, aceitos)
        self.assertEqual(abertos, {('zip', '4.6.1'): ['G1'],
                                   ('base64', '0.23.1'): ['R1']})
        self.assertEqual(sem_uso, [('sha1', 'R9')])


class ConsultaTests(unittest.TestCase):
    def test_one_batch_then_pages_only_for_tokens(self):
        enviar, enviados = respostas(
            [{}, {'vulns': [{'id': 'R1'}], 'next_page_token': 't'}],
            [{'vulns': [{'id': 'R2'}, {'id': 'R1'}]}])
        achados = avisos_por_pacote([('a', '1'), ('b', '2')], enviar)
        self.assertEqual(achados, {('b', '2'): ['R1', 'R2']})
        self.assertEqual(len(enviados[0]['queries']), 2)
        self.assertEqual(enviados[0]['queries'][0],
                         {'package': {'ecosystem': 'crates.io', 'name': 'a'},
                          'version': '1'})
        self.assertEqual(enviados[1]['queries'][0]['page_token'], 't')

    def test_unexpected_shape_is_unavailable(self):
        for resultados in [[], [{'vulns': [{}]}], ['x']]:
            enviar, _ = respostas(resultados)
            with self.subTest(r=resultados), self.assertRaises(RedeIndisponivel):
                avisos_por_pacote([('a', '1')], enviar)


class ExecutarTests(unittest.TestCase):
    def rodar(self, enviar, ambiente, texto_allowlist=VAZIA):
        linhas = []
        codigo = executar(LOCK, texto_allowlist, ambiente, enviar, linhas.append)
        return codigo, '\n'.join(linhas)

    def test_findings_fail_listing_package_version_ids(self):
        enviar, _ = respostas([{}, {'vulns': [{'id': 'R2'}, {'id': 'G1'}]}])
        codigo, texto = self.rodar(enviar, {})
        self.assertEqual(codigo, 1)
        self.assertIn('zip 4.6.1: G1, R2', texto)

    def test_accepted_findings_pass(self):
        enviar, _ = respostas([{}, {'vulns': [{'id': 'R2'}]}])
        aceito = allowlist(dict(id='R2', package='zip', justification='j'))
        self.assertEqual(self.rodar(enviar, {}, aceito)[0], 0)

    def test_network_down_fails_in_ci_warns_locally(self):
        def sem_rede(_):
            raise RedeIndisponivel('URLError: offline')
        codigo, texto = self.rodar(sem_rede, {'CI': 'true'})
        self.assertEqual(codigo, 2)
        self.assertIn('FAIL', texto)
        codigo, texto = self.rodar(sem_rede, {})
        self.assertEqual(codigo, 0)
        self.assertTrue(texto.startswith('NOT_RUN'))


if __name__ == '__main__':
    unittest.main()
