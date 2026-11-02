"""Autoteste offline do saneador de HAR; somente dados SINTÉTICOS, sem rede."""
import copy
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parent / 'sanear_har_formulario.py'
BASE = 'https://sei.treinamento.exemplo.invalid/sei'
HASH = 'ab' * 32
NUP = '99999.000001/2099-00'
COOKIE = 'PHPSESSID=COOKIE_FICTICIO_123'
ROTULO = 'UNIDADE_FICTICIA'
FORM = f'''<html><body><a id="lnkInfraUnidade">{ROTULO}</a>
<form id="frmPesquisa" method="post" accept-charset="ISO-8859-1"
 action="controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_hash={HASH}"
 onsubmit="return validar();">
<input type="hidden" name="hdnAcao" value="pesquisar">
<input type="text" name="txtProtocoloPesquisa" value="">
<input type="checkbox" name="chkSinProcessos" value="P">
<select name="selTipoProcedimento"><option value=""></option></select>
<input type="submit" name="sbmPesquisar" value="Pesquisar">
</form></body></html>'''


def entrada(metodo, caminho, status=200, html=None, post=None, headers=()):
    req = {'method': metodo, 'url': f'{BASE}/{caminho}',
           'headers': [{'name': 'Cookie', 'value': COOKIE}, *headers],
           'cookies': [{'name': 'PHPSESSID', 'value': 'COOKIE_FICTICIO_123'}]}
    if post is not None:
        req['postData'] = post
    resp = {'status': status, 'content': {'mimeType': 'text/html; charset=iso-8859-1'}}
    if html:
        resp['content']['text'] = html
    return {'request': req, 'response': resp}


def har(acao='protocolo_pesquisar', post=None):
    qs = f'acao={acao}&acao_origem={acao}&infra_sistema=100000100&infra_hash={HASH}'
    post = post or {'mimeType': 'application/x-www-form-urlencoded; charset=ISO-8859-1',
                    'params': [{'name': 'hdnAcao', 'value': 'pesquisar'},
                               {'name': 'txtProtocoloPesquisa', 'value': NUP},
                               {'name': 'selTipoProcedimento', 'value': ''},
                               {'name': 'hdnCampoJs', 'value': 'VALOR_FICTICIO_JS'},
                               {'name': 'sbmPesquisar', 'value': 'Pesquisar'}]}
    cab = ({'name': 'Origin', 'value': 'https://sei.treinamento.exemplo.invalid'},
           {'name': 'Referer', 'value': f'{BASE}/controlador.php?{qs}'},
           {'name': 'Content-Type', 'value': post['mimeType']})
    return {'log': {'entries': [
        entrada('GET', f'controlador.php?{qs}', html=FORM.replace('protocolo_pesquisar', acao)),
        entrada('GET', 'infra_css/sei.css'),
        entrada('POST', f'controlador.php?{qs}', post=post, headers=cab)]}}


class SaneadorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = pathlib.Path(self.tmp.name)
        self.escrever('origem', f'{BASE}\nversao=4.0.9\n')
        self.escrever('allow', f'# instalações autorizadas\n{BASE}\n')
        self.escrever('har', json.dumps(har()))

    def tearDown(self):
        self.tmp.cleanup()

    def escrever(self, nome, texto, modo=0o600):
        p = self.d / nome
        p.write_text(texto, encoding='utf-8')
        os.chmod(p, modo)
        return p

    def rodar(self, *extra, operacao='processo.buscar', orcamento='3', saida='saida'):
        args = [sys.executable, '-I', str(SCRIPT), '--har', str(self.d / 'har'),
                '--origem-file', str(self.d / 'origem'), '--allowlist-file', str(self.d / 'allow'),
                '--instalacao', 'unb', '--operacao', operacao, '--orcamento', orcamento,
                '--efeito', 'filtro_lembrado_na_sessao', '--saida', str(self.d / saida), *extra]
        r = subprocess.run(args, capture_output=True, text=True, timeout=60, check=False)
        return r.returncode, r.stdout + r.stderr

    def falha(self, codigo, **kw):
        rc, out = self.rodar(**kw)
        self.assertEqual(rc, 1, out)
        self.assertIn(f'codigo={codigo}', out)
        self.assertFalse((self.d / kw.get('saida', 'saida')).exists())

    def test_formato_saneado_sem_valores_secretos(self):
        rc, out = self.rodar()
        self.assertEqual(rc, 0, out)
        self.assertTrue(out.startswith('PASS |'))
        saida = self.d / 'saida'
        self.assertEqual(saida.stat().st_mode & 0o777, 0o700)
        textos = []
        for nome in ('formato.json', 'manifesto.json'):
            self.assertEqual((saida / nome).stat().st_mode & 0o777, 0o600)
            textos.append((saida / nome).read_text(encoding='utf-8'))
        for proibido in (HASH, NUP, 'COOKIE_FICTICIO', ROTULO, 'VALOR_FICTICIO_JS', '100000100'):
            self.assertFalse(any(proibido in t for t in textos), proibido)
        formato, manifesto = map(json.loads, textos)
        envio = formato['envios'][0]
        campos = [(c['nome'], c['tipo'], c['classe'], c.get('valor')) for c in envio['corpo']['campos']]
        self.assertEqual(campos, [
            ('hdnAcao', 'oculto', 'literal', 'pesquisar'),
            ('txtProtocoloPesquisa', 'usuario', 'redigido', None),
            ('selTipoProcedimento', 'usuario', 'vazio', None),
            ('hdnCampoJs', 'desconhecido', 'redigido', None),
            ('sbmPesquisar', 'botao', 'literal', 'Pesquisar')])
        self.assertEqual(envio['formulario']['declarados_nao_enviados'], ['chkSinProcessos'])
        self.assertTrue(envio['formulario']['onsubmit_presente'])
        self.assertEqual(envio['formulario']['accept_charset_declarado'], 'iso-8859-1')
        cab = envio['cabecalhos']
        self.assertEqual(cab['nomes'], ['cookie', 'origin', 'referer', 'content-type'])
        self.assertTrue(cab['origin_igual_a_instalacao'] and cab['referer_mesma_instalacao'])
        self.assertEqual(cab['content_type'], {'mime': 'application/x-www-form-urlencoded',
                                               'charset': 'iso-8859-1'})
        params = {p['nome']: p['classe'] for p in envio['url']['parametros']}
        self.assertEqual(params, {'acao': 'literal', 'acao_origem': 'literal',
                                  'infra_sistema': 'redigido', 'infra_hash': 'redigido'})
        self.assertEqual([s['alvo'] for s in formato['sequencia']], [False, True])
        esperado = hashlib.sha256(b'ies-unidade-v1\0' + ROTULO.encode()).hexdigest()
        self.assertEqual(manifesto['unidade_rotulo_sha256'], esperado)
        self.assertEqual((manifesto['requisicoes_aplicacao'], manifesto['requisicoes_estaticas']), (2, 1))
        self.assertEqual(manifesto['versao_sei_declarada'], '4.0.9')
        self.assertEqual(manifesto['efeitos_observados'], ['filtro_lembrado_na_sessao'])

    def test_corpo_textual_e_paginacao_do_controle(self):
        post = {'mimeType': 'application/x-www-form-urlencoded',
                'text': 'hdnAcao=paginar&hdnInfraPaginaAtual=1&sbmPesquisar=Pesquisar'}
        self.escrever('har', json.dumps(har('procedimento_controlar', post)))
        self.falha('post_alvo_ausente')
        rc, out = self.rodar(operacao='unidade.listar_processos', saida='s2')
        self.assertEqual(rc, 0, out)
        f = json.loads((self.d / 's2' / 'formato.json').read_text(encoding='utf-8'))
        self.assertEqual([c['classe'] for c in f['envios'][0]['corpo']['campos']],
                         ['literal', 'redigido', 'literal'])

    def test_literal_com_digitos_continua_redigido(self):
        rc, out = self.rodar('--literal-campo', 'txtProtocoloPesquisa')
        self.assertEqual(rc, 0, out)
        texto = (self.d / 'saida' / 'formato.json').read_text(encoding='utf-8')
        self.assertNotIn(NUP, texto)

    def test_orcamento_excedido(self):
        self.falha('orcamento_excedido', orcamento='1')

    def test_permissoes_e_links(self):
        os.chmod(self.d / 'har', 0o644)
        self.falha('arquivo_privado_permissao_insegura')
        os.chmod(self.d / 'har', 0o600)
        os.rename(self.d / 'har', self.d / 'har-real')
        os.symlink(self.d / 'har-real', self.d / 'har')
        self.falha('arquivo_privado_link_simbolico')

    def test_origem_fora_da_allowlist(self):
        self.escrever('allow', 'https://outro.treinamento.exemplo.invalid/sei\n')
        self.falha('origem_fora_da_allowlist')

    def test_host_sem_marcador_exige_declaracao(self):
        base = 'https://sei.exemplo.invalid/sei'
        self.escrever('origem', base + '\n')
        self.escrever('har', json.dumps(har()).replace(BASE, base).replace(
            'sei.treinamento.exemplo.invalid', 'sei.exemplo.invalid'))
        self.escrever('allow', base + '\n')
        self.falha('host_sem_marcador_de_treinamento')
        self.escrever('allow', base + ' treinamento-declarado\n')
        rc, out = self.rodar()
        self.assertEqual(rc, 0, out)

    def test_entrada_fora_da_origem(self):
        h = har()
        h['log']['entries'].insert(0, {'request': {'method': 'GET', 'url': 'data:image/png;base64,AA=='}})
        self.escrever('har', json.dumps(h))
        self.assertEqual(self.rodar(saida='s4')[0], 0)
        h['log']['entries'][1]['request']['url'] = 'https://cdn.exemplo.invalid/a.css'
        self.escrever('har', json.dumps(h))
        self.falha('entrada_fora_da_origem')

    def test_credencial_ou_login_no_har(self):
        h = har()
        alvo = h['log']['entries'][2]['request']['postData']['params']
        alvo.append({'name': 'pwdSenha', 'value': 'SENHA_FICTICIA'})
        self.escrever('har', json.dumps(h))
        self.falha('credencial_capturada_no_har')
        h = har()
        h['log']['entries'][0]['request']['url'] = f'{BASE}/sip/login.php'
        self.escrever('har', json.dumps(h))
        self.falha('login_capturado_no_har')

    def test_multipart_recusado(self):
        h = har()
        h['log']['entries'][2]['request']['postData'] = {
            'mimeType': 'multipart/form-data; boundary=x', 'text': '--x--'}
        self.escrever('har', json.dumps(h))
        self.falha('corpo_multipart_nao_suportado')

    def test_saida_existente_e_efeitos_contraditorios(self):
        (self.d / 'saida').mkdir()
        rc, out = self.rodar()
        self.assertEqual(rc, 1)
        self.assertIn('codigo=saida_existente', out)
        rc, out = self.rodar('--efeito', 'nenhum_observado', saida='s3')
        self.assertIn('codigo=efeitos_contraditorios', out)

    def test_har_ilegivel(self):
        self.escrever('har', '{"log": {}}')
        self.falha('har_json_ilegivel')

    def test_vazamento_por_valor_de_rotulo_fail_closed(self):
        h = har()
        form = copy.deepcopy(h['log']['entries'][0])
        form['response']['content']['text'] = FORM.replace(ROTULO, 'pesquisar')
        h['log']['entries'][0] = form
        h['log']['entries'][2]['request']['postData']['params'][0]['name'] = 'hdnOutro'
        self.escrever('har', json.dumps(h))
        self.falha('vazamento_detectado')


if __name__ == '__main__':
    unittest.main()
