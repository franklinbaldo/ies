"""Leitura offline dos formulários HTML de um HAR do SEI (somente stdlib).

Extrai apenas nomes e tipos de campos, atributos de envio e o rótulo de
`lnkInfraUnidade` (devolvido para hash, nunca para publicação).
"""
import base64
import html.parser
import re
from urllib.parse import parse_qsl, urlsplit

TOKEN_MIME = re.compile(r'^[a-z0-9/._+-]{1,64}$')


class LeitorFormularios(html.parser.HTMLParser):
    """Coleta formulários (nomes e tipos) e o texto de `lnkInfraUnidade`."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.formularios, self.atual, self.unidade = [], None, None
        self._unidade, self._buffer = 0, []

    def handle_starttag(self, tag, attrs):
        a = {k: (v or '') for k, v in attrs}
        if a.get('id') == 'lnkInfraUnidade':
            self._unidade, self._buffer = 1, []
        elif self._unidade:
            self._unidade += tag == 'a'
        if tag == 'form':
            self.atual = {'action': a.get('action', ''), 'method': a.get('method', ''),
                          'enctype': a.get('enctype', ''),
                          'charset': a.get('accept-charset', ''),
                          'onsubmit': 'onsubmit' in a, 'campos': []}
            self.formularios.append(self.atual)
        elif self.atual is not None and a.get('name'):
            tipo = None
            if tag == 'input':
                t = a.get('type', 'text').lower()
                tipo = 'oculto' if t == 'hidden' else (
                    'botao' if t in ('submit', 'button', 'image', 'reset') else 'usuario')
            elif tag == 'button':
                tipo = 'botao'
            elif tag in ('select', 'textarea'):
                tipo = 'usuario'
            if tipo:
                self.atual['campos'].append((a['name'], tipo))

    def handle_endtag(self, tag):
        if tag == 'form':
            self.atual = None
        if self._unidade and tag == 'a':
            self._unidade -= 1
            if not self._unidade and self.unidade is None:
                self.unidade = ''.join(self._buffer).strip()

    def handle_data(self, data):
        if self._unidade:
            self._buffer.append(data)


def charset_de(mime):
    tipo, _, resto = (mime or '').partition(';')
    charset = ''
    for p in resto.split(';'):
        k, _, v = p.strip().partition('=')
        if k.lower() == 'charset':
            charset = v.strip().strip('"').lower()
    tipo = tipo.strip().lower()
    return (tipo if TOKEN_MIME.match(tipo or '-') else 'redigido',
            charset if TOKEN_MIME.match(charset or '-') else ('redigido' if charset else ''))


def acao_de(url):
    for k, v in parse_qsl(urlsplit(url).query, keep_blank_values=True):
        if k == 'acao':
            return v
    return None


def conteudo_html(entrada):
    resp = entrada.get('response') or {}
    cont = resp.get('content') or {}
    mime, charset = charset_de(cont.get('mimeType'))
    if mime != 'text/html' or not cont.get('text'):
        return None
    if cont.get('encoding') == 'base64':
        bruto = base64.b64decode(cont['text'], validate=True)
        return bruto.decode(charset or 'latin-1', errors='replace')
    return cont['text']


def localizar_formulario(entradas, ate, acao, segredos):
    for anterior in reversed(entradas[:ate]):
        texto = conteudo_html(anterior)
        if texto is None:
            continue
        leitor = LeitorFormularios()
        leitor.feed(texto)
        if leitor.unidade:
            segredos.add(leitor.unidade)
        url = anterior['request']['url']
        candidatos = [f for f in leitor.formularios
                      if acao_de(f['action'] or url) == acao]
        if candidatos:
            return candidatos, leitor.unidade
    return [], None
