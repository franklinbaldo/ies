#!/usr/bin/env python3
"""Saneia offline o FORMATO de um POST de formulário do SEI a partir de um HAR.

Somente biblioteca padrão; executar com `python3 -I`. Não faz rede, não faz
login e não reproduz o envio. Registra nomes, ordem, tipos e CLASSES de valor;
nunca valores de cookie, infra_hash, protocolo, nomes ou rótulos. Falha
fechada: qualquer condição inesperada encerra sem criar a saída.
"""
import argparse
import hashlib
import json
import os
import re
import stat
import sys
from urllib.parse import parse_qsl, unquote_plus, urlsplit

# `-I` não põe o diretório do script no sys.path: inserir só o do próprio arquivo.
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from har_formulario_html import TOKEN_MIME, acao_de, charset_de, localizar_formulario  # noqa: E402

VERSAO = 1
OPERACOES = {
    'processo.buscar': 'protocolo_pesquisar',
    'documento.buscar': 'protocolo_pesquisar',
    'unidade.listar_processos': 'procedimento_controlar',
}
INSTALACOES = ('unb', 'ufrj', 'go', 'pen1', 'pen2', 'pen3', 'pen4')
EFEITOS = ('nenhum_observado', 'filtro_lembrado_na_sessao',
           'pagina_lembrada_na_sessao', 'recebimento_registrado',
           'nao_verificado')
LITERAIS_PADRAO = ('hdnAcao', 'sbmPesquisar')
PARAMETROS_LITERAIS = ('acao', 'acao_origem', 'acao_retorno')
MARCADOR_TREINAMENTO = re.compile(
    r'(^|[.-])(treinamento|treina|homologacao|homolog|hml|teste|testes)([.-]|$)')
ACAO = re.compile(r'^[a-z][a-z0-9_]{0,63}$')
LITERAL = re.compile(r'^[A-Za-zÀ-ÖØ-öø-ÿ_ ]{1,32}$')
NOME_CAMPO = re.compile(r'^[A-Za-z0-9_\[\].:{}-]{1,80}$')
VERSAO_SEI = re.compile(r'^\d{1,2}(\.\d{1,3}){1,3}$')
ESTATICOS = ('.js', '.css', '.png', '.gif', '.jpg', '.jpeg', '.svg', '.ico',
             '.woff', '.woff2', '.ttf', '.eot', '.map')
CREDENCIAL = re.compile(r'(senha|pwd|passw|otp)', re.I)
LIMITE_PEQUENO = 64 * 1024


class Falha(Exception):
    """Falha tipada: `codigo` é estável; a mensagem nunca classifica."""

    def __init__(self, codigo):
        super().__init__(codigo)
        self.codigo = codigo


def ler_privado(caminho, limite):
    if not os.path.isabs(caminho):
        raise Falha('caminho_nao_absoluto')
    try:
        st = os.lstat(caminho)
    except FileNotFoundError:
        raise Falha('arquivo_privado_ausente') from None
    if stat.S_ISLNK(st.st_mode):
        raise Falha('arquivo_privado_link_simbolico')
    if not stat.S_ISREG(st.st_mode):
        raise Falha('arquivo_privado_nao_regular')
    if st.st_mode & 0o077:
        raise Falha('arquivo_privado_permissao_insegura')
    if st.st_size == 0:
        raise Falha('arquivo_privado_vazio')
    if st.st_size > limite:
        raise Falha('arquivo_privado_excede_limite')
    fd = os.open(caminho, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, 'rb') as f:
        aberto = os.fstat(f.fileno())
        if (aberto.st_ino, aberto.st_dev) != (st.st_ino, st.st_dev):
            raise Falha('arquivo_privado_trocado_durante_leitura')
        dados = f.read(limite + 1)
    if len(dados) > limite:
        raise Falha('arquivo_privado_excede_limite')
    return dados


def base_https(texto):
    partes = urlsplit(texto.strip())
    if partes.scheme != 'https' or not partes.hostname:
        raise Falha('origem_nao_https')
    if partes.query or partes.fragment or partes.username or partes.password:
        raise Falha('origem_formato_desconhecido')
    return partes.netloc.lower(), partes.path.rstrip('/')


def ler_origem(caminho):
    linhas = ler_privado(caminho, LIMITE_PEQUENO).decode('utf-8').splitlines()
    if not 1 <= len(linhas) <= 2:
        raise Falha('origem_formato_desconhecido')
    versao = None
    if len(linhas) == 2:
        chave, _, valor = linhas[1].partition('=')
        if chave != 'versao' or not VERSAO_SEI.match(valor):
            raise Falha('origem_formato_desconhecido')
        versao = valor
    return base_https(linhas[0]), versao


def conferir_allowlist(caminho, origem):
    texto = ler_privado(caminho, LIMITE_PEQUENO).decode('utf-8')
    for linha in texto.splitlines():
        linha = linha.strip()
        if not linha or linha.startswith('#'):
            continue
        campos = linha.split()
        if len(campos) > 2 or (len(campos) == 2 and campos[1] != 'treinamento-declarado'):
            raise Falha('allowlist_formato_desconhecido')
        if base_https(campos[0]) == origem:
            host = origem[0].split(':')[0]
            if not MARCADOR_TREINAMENTO.search(host) and len(campos) == 1:
                raise Falha('host_sem_marcador_de_treinamento')
            return
    raise Falha('origem_fora_da_allowlist')


def literal_acao(valor):
    return valor if valor is not None and ACAO.match(valor) else 'redigido'


def nome_seguro(nome):
    mascarado = re.sub(r'\d{5,}', '{n}', nome)
    if not NOME_CAMPO.match(mascarado):
        raise Falha('nome_de_campo_inesperado')
    return mascarado


def campos_do_corpo(post, charset):
    mime = charset_de(post.get('mimeType'))[0]
    if mime == 'multipart/form-data':
        raise Falha('corpo_multipart_nao_suportado')
    if mime != 'application/x-www-form-urlencoded':
        raise Falha('content_type_inesperado')
    params = post.get('params')
    if params:
        pares = []
        for p in params:
            if 'fileName' in p:
                raise Falha('arquivo_no_corpo')
            pares.append((unquote_plus(p.get('name', ''), encoding=charset or 'latin-1'),
                          p.get('value', ''), True))
    elif post.get('text') is not None:
        try:
            pares = [(k, v, False) for k, v in parse_qsl(
                post['text'], keep_blank_values=True,
                encoding=charset or 'latin-1', errors='strict')]
        except (UnicodeDecodeError, ValueError):
            raise Falha('corpo_ilegivel') from None
    else:
        raise Falha('corpo_ausente')
    saida = []
    for nome, valor, codificado in pares:
        if CREDENCIAL.search(nome):
            raise Falha('credencial_capturada_no_har')
        decod = unquote_plus(valor, encoding=charset or 'latin-1') if codificado else valor
        saida.append((nome, valor, decod))
    return saida


def classe(nome, valor, literais):
    if valor == '':
        return {'classe': 'vazio'}
    if nome in literais and LITERAL.match(valor):
        return {'classe': 'literal', 'valor': valor}
    return {'classe': 'redigido'}


def descrever_envio(entradas, i, origem, literais, segredos, publicados):
    e = entradas[i]
    req = e['request']
    parametros = []
    for k, v in parse_qsl(urlsplit(req['url']).query, keep_blank_values=True):
        if k in PARAMETROS_LITERAIS and ACAO.match(v):
            parametros.append({'nome': nome_seguro(k), 'classe': 'literal', 'valor': v})
            publicados.add(v)
        else:
            segredos.add(v)
            parametros.append({'nome': nome_seguro(k), **classe(k, v, ())})
    headers = req.get('headers') or []
    nomes = [h.get('name', '').lower() for h in headers]
    valores = {h.get('name', '').lower(): h.get('value', '') for h in headers}
    for h in headers:
        if h.get('name', '').lower() in ('cookie', 'authorization'):
            segredos.add(h.get('value', ''))
    if any(not re.match(r'^:?[a-z0-9-]{1,64}$', n) for n in nomes):
        raise Falha('cabecalho_inesperado')
    base = 'https://' + origem[0]
    post = req.get('postData') or {}
    mime_ct = post.get('mimeType') or valores.get('content-type', '')
    mime, charset = charset_de(mime_ct)
    campos, descritos = campos_do_corpo({**post, 'mimeType': mime_ct}, charset), []
    for n, (k, bruto, v) in enumerate(campos):
        c = classe(k, v, literais)
        if c['classe'] == 'literal':
            publicados.add(v)
        else:
            segredos.update((bruto, v))
        descritos.append({'ordem': n, 'nome': nome_seguro(k), 'tipo': None, **c})
    candidatos, unidade = localizar_formulario(entradas, i, acao_de(req['url']), segredos)
    tipos, form = {}, {'formulario_html': 'ausente_no_har'}
    if len(candidatos) > 1:
        form = {'formulario_html': 'multiplos_candidatos'}
    elif candidatos:
        f = candidatos[0]
        for n, t in f['campos']:
            tipos[n] = t if tipos.get(n, t) == t else 'misto'
        enviados = {n for n, _, _ in campos}
        form = {'formulario_html': 'encontrado',
                'metodo_declarado': f['method'].lower() if f['method'].lower() in ('get', 'post', '') else 'redigido',
                'enctype_declarado': charset_de(f['enctype'])[0] if f['enctype'] else '',
                'accept_charset_declarado': f['charset'].lower() if TOKEN_MIME.match(f['charset'].lower() or '-') else 'redigido',
                'onsubmit_presente': f['onsubmit'],
                'declarados_nao_enviados': [nome_seguro(n) for n in dict.fromkeys(n for n, _ in f['campos']) if n not in enviados]}
    referer = valores.get('referer')
    resp = e.get('response') or {}
    return {
        'indice_na_sequencia': i,
        'url': {'parametros': parametros},
        'cabecalhos': {
            'nomes': nomes,
            'origin_presente': 'origin' in valores,
            'origin_igual_a_instalacao': valores.get('origin', '').lower() == base,
            'referer_presente': referer is not None,
            'referer_mesma_instalacao': bool(referer) and dentro_da_origem(referer, origem),
            'referer_acao': literal_acao(acao_de(referer)) if referer else None,
            'content_type': {'mime': mime, 'charset': charset}},
        'corpo': {'enctype': mime, 'charset': charset, 'campos': [
            {**d, 'tipo': tipos.get(k, 'desconhecido')} for d, (k, _, _) in zip(descritos, campos)]},
        'formulario': form,
        'resposta': {'status': int(resp.get('status', 0)),
                     'mime': charset_de((resp.get('content') or {}).get('mimeType'))[0],
                     'redirect_presente': bool(resp.get('redirectURL'))},
    }, unidade


def dentro_da_origem(url, origem):
    p = urlsplit(url)
    return (p.scheme == 'https' and p.netloc.lower() == origem[0]
            and (p.path == origem[1] or p.path.startswith(origem[1] + '/')))


def sanear(har_bytes, origem, operacao, orcamento, literais):
    try:
        entradas = json.loads(har_bytes.decode('utf-8'))['log']['entries']
        if not isinstance(entradas, list):
            raise TypeError
    except (UnicodeDecodeError, ValueError, KeyError, TypeError):
        raise Falha('har_json_ilegivel') from None
    acao_alvo, segredos, sequencia, estaticos = OPERACOES[operacao], set(), [], 0
    publicados, locais = set(), 0
    for e in entradas:
        req = e.get('request') or {}
        url, metodo = req.get('url', ''), req.get('method', '')
        if urlsplit(url).scheme in ('data', 'blob'):
            locais += 1  # conteúdo embutido; nenhuma requisição ao servidor
            continue
        if urlsplit(url).scheme != 'https':
            raise Falha('entrada_nao_https')
        if not dentro_da_origem(url, origem):
            raise Falha('entrada_fora_da_origem')
        caminho = urlsplit(url).path.lower()
        if '/sip/' in caminho or 'login' in caminho.rsplit('/', 1)[-1]:
            raise Falha('login_capturado_no_har')
        for c in req.get('cookies') or []:
            segredos.add(c.get('value', ''))
        if metodo not in ('GET', 'POST'):
            raise Falha('metodo_inesperado')
        if caminho.endswith(ESTATICOS) and metodo == 'GET':
            estaticos += 1
            continue
        if not caminho.endswith('.php'):
            raise Falha('recurso_inesperado')
        sequencia.append(e)
    if len(sequencia) > orcamento:
        raise Falha('orcamento_excedido')
    resumo, envios, unidade = [], [], None
    for i, e in enumerate(sequencia):
        req = e['request']
        acao = acao_de(req['url'])
        alvo = req['method'] == 'POST' and acao == acao_alvo
        resumo.append({'metodo': req['method'], 'acao': literal_acao(acao), 'alvo': alvo,
                       'status': int((e.get('response') or {}).get('status', 0))})
        if acao is not None and ACAO.match(acao):
            publicados.add(acao)
        if req['method'] == 'POST' and any(CREDENCIAL.search(n) for n in nomes_postados(req)):
            raise Falha('credencial_capturada_no_har')
        if alvo:
            envio, rotulo = descrever_envio(sequencia, i, origem, literais, segredos, publicados)
            envios.append(envio)
            unidade = unidade or rotulo
    if not envios:
        raise Falha('post_alvo_ausente')
    formato = {'schema_version': VERSAO, 'operacao': operacao, 'acao_sei': acao_alvo,
               'sequencia': resumo, 'envios': envios}
    return formato, unidade, (len(sequencia), estaticos, locais), segredos - publicados


def nomes_postados(req):
    post = req.get('postData') or {}
    nomes = [p.get('name', '') for p in post.get('params') or []]
    nomes += [k for k, _ in parse_qsl(post.get('text') or '', keep_blank_values=True)]
    return [unquote_plus(n) for n in nomes]


def conferir_vazamento(textos, segredos, permitidos):
    for texto in textos:
        for s in segredos:
            if len(s) >= 4 and s in texto:
                raise Falha('vazamento_detectado')
        for m in re.findall(r'[0-9a-fA-F]{32,}|\d{6,}', texto):
            if m not in permitidos:
                raise Falha('vazamento_detectado')


def escrever_privado(caminho, texto):
    fd = os.open(caminho, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'w', encoding='utf-8') as f:
        f.write(texto)


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument('--har', required=True)
    p.add_argument('--origem-file', required=True)
    p.add_argument('--allowlist-file', required=True)
    p.add_argument('--instalacao', required=True, choices=INSTALACOES)
    p.add_argument('--operacao', required=True, choices=sorted(OPERACOES))
    p.add_argument('--orcamento', required=True, type=int, choices=range(1, 51), metavar='1..50')
    p.add_argument('--efeito', required=True, action='append', choices=EFEITOS)
    p.add_argument('--literal-campo', action='append', default=[])
    p.add_argument('--saida', required=True)
    p.add_argument('--max-har-bytes', type=int, default=64 * 1024 * 1024)
    a = p.parse_args(argv)
    rotulo = f'instalação={a.instalacao} operação={a.operacao}'
    try:
        if not os.path.isabs(a.saida):
            raise Falha('saida_nao_absoluta')
        if os.path.lexists(a.saida):
            raise Falha('saida_existente')
        if 'nenhum_observado' in a.efeito and len(set(a.efeito)) > 1:
            raise Falha('efeitos_contraditorios')
        if any(not NOME_CAMPO.match(n) for n in a.literal_campo):
            raise Falha('literal_campo_invalido')
        origem, versao = ler_origem(a.origem_file)
        conferir_allowlist(a.allowlist_file, origem)
        har = ler_privado(a.har, a.max_har_bytes)
        formato, unidade, (n_app, n_estaticos, n_locais), segredos = sanear(
            har, origem, a.operacao, a.orcamento, set(LITERAIS_PADRAO) | set(a.literal_campo))
        texto_formato = json.dumps(formato, indent=2, ensure_ascii=False) + '\n'
        h_unidade = hashlib.sha256(b'ies-unidade-v1\0' + unidade.encode('utf-8')).hexdigest() if unidade else None
        h_formato = hashlib.sha256(texto_formato.encode('utf-8')).hexdigest()
        h_har = hashlib.sha256(har).hexdigest()
        manifesto = {
            'schema_version': VERSAO, 'estado': 'formato_saneado_para_revisao',
            'publicacao': 'revisao_humana_obrigatoria', 'instalacao': a.instalacao,
            'origem': origem[0] + origem[1], 'versao_sei_declarada': versao,
            'operacao': a.operacao, 'acao_sei': OPERACOES[a.operacao],
            'unidade_rotulo_sha256': h_unidade,
            'unidade_rotulo_estado': 'observado_em_lnkInfraUnidade' if unidade else 'nao_observado',
            'orcamento_declarado': a.orcamento, 'requisicoes_aplicacao': n_app,
            'requisicoes_estaticas': n_estaticos, 'entradas_locais_ignoradas': n_locais,
            'envios_alvo': len(formato['envios']),
            'efeitos_observados': sorted(set(a.efeito)),
            'formato_sha256': h_formato, 'har_privado_sha256': h_har,
        }
        texto_manifesto = json.dumps(manifesto, indent=2, ensure_ascii=False) + '\n'
        segredos.discard('')
        conferir_vazamento((texto_formato, texto_manifesto), segredos,
                           {h for h in (h_unidade, h_formato, h_har) if h})
        os.mkdir(a.saida, 0o700)
        escrever_privado(os.path.join(a.saida, 'formato.json'), texto_formato)
        escrever_privado(os.path.join(a.saida, 'manifesto.json'), texto_manifesto)
    except Falha as f:
        print(f'FAIL | {rotulo} | codigo={f.codigo}')
        return 1
    except (OSError, UnicodeDecodeError, ValueError):
        print(f'FAIL | {rotulo} | codigo=erro_de_entrada_ou_saida')
        return 1
    print(f'PASS | {rotulo} | formato saneado; revisão humana antes de publicar')
    return 0


if __name__ == '__main__':
    sys.exit(main())
